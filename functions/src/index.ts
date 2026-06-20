import { onRequest } from "firebase-functions/v2/https";
import { onDocumentWritten } from "firebase-functions/v2/firestore";
import * as admin from "firebase-admin";
import { defineSecret } from "firebase-functions/params";
import { StatusReport, FriendConfig, formatStatusReport, findFriend } from "./formatter";

const telegramBotToken = defineSecret("TELEGRAM_BOT_TOKEN");
const telegramChatId = defineSecret("TELEGRAM_CHAT_ID");
const githubToken = defineSecret("GITHUB_TOKEN");

// Initialize Firebase Admin
admin.initializeApp();
const db = admin.firestore();

interface TelegramUpdate {
  message?: {
    message_id: number;
    chat: {
      id: number;
    };
    text?: string;
    from?: {
      id: number;
      first_name: string;
    };
  };
  callback_query?: {
    id: string;
    from: {
      id: number;
      first_name: string;
    };
    message?: {
      message_id: number;
      chat: {
        id: number;
      };
      text?: string;
      reply_markup?: {
        inline_keyboard: Array<Array<{ text: string; callback_data: string }>>;
      };
    };
    data?: string;
  };
}

/**
 * Update Firestore with snooze
 */
async function snoozeFriend(friendId: string, days: number): Promise<void> {
  const now = admin.firestore.Timestamp.now();
  const until = admin.firestore.Timestamp.fromMillis(
    now.toMillis() + days * 24 * 60 * 60 * 1000
  );

  await db.collection("snoozes").doc(friendId).set({
    friend_id: friendId,
    snoozed_until: until,
    snoozed_at: now,
    reason: null,
  });
}

/**
 * Remove a snooze for a friend
 */
async function unsnoozeFriend(friendId: string): Promise<void> {
  await db.collection("snoozes").doc(friendId).delete();
}

/**
 * Get active snoozes as a map of friendId → snoozed_until ISO string
 */
async function getActiveSnoozedFriends(): Promise<Map<string, string>> {
  const now = admin.firestore.Timestamp.now();
  const snapshot = await db.collection("snoozes")
    .where("snoozed_until", ">", now)
    .get();
  const map = new Map<string, string>();
  snapshot.forEach((doc) => {
    const data = doc.data();
    map.set(data.friend_id, (data.snoozed_until as admin.firestore.Timestamp).toDate().toISOString());
  });
  return map;
}

/**
 * Load all active friends from Firestore
 */
async function getAllFriends(): Promise<FriendConfig[]> {
  const snapshot = await db.collection("friends").get();
  return snapshot.docs
    .map((doc) => doc.data() as FriendConfig)
    .filter((f) => f.active !== false);
}

/**
 * Set a friend's active flag in Firestore
 */
async function setFriendActive(friendId: string, active: boolean): Promise<void> {
  await db.collection("friends").doc(friendId).update({ active });
}

/**
 * Send a Telegram message, returns the message_id on success
 */
async function sendTelegramMessage(
  chatId: number,
  text: string,
  botToken: string
): Promise<number | null> {
  const url = `https://api.telegram.org/bot${botToken}/sendMessage`;

  const response = await fetch(url, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({
      chat_id: chatId,
      text,
      parse_mode: "HTML",
      link_preview_options: { is_disabled: true },
    }),
  });

  if (!response.ok) {
    const error = await response.text();
    console.error("Failed to send message:", error);
    return null;
  }

  const json = await response.json();
  return json.result?.message_id ?? null;
}

/**
 * Edit an existing Telegram message
 */
async function editTelegramMessage(
  chatId: number,
  messageId: number,
  text: string,
  botToken: string
): Promise<void> {
  const url = `https://api.telegram.org/bot${botToken}/editMessageText`;

  const response = await fetch(url, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({
      chat_id: chatId,
      message_id: messageId,
      text,
      parse_mode: "HTML",
      link_preview_options: { is_disabled: true },
    }),
  });

  if (!response.ok) {
    const error = await response.text();
    console.error("Failed to edit message:", error);
  }
}

/**
 * Rebuild and edit the last morning notification from current Firestore state.
 * Silently does nothing if no message_id is stored.
 */
async function editNotification(chatId: number, botToken: string): Promise<void> {
  const [statusDoc, snoozedFriends, friends] = await Promise.all([
    db.collection("status").doc("latest").get(),
    getActiveSnoozedFriends(),
    getAllFriends(),
  ]);

  if (!statusDoc.exists) return;

  const messageId = statusDoc.data()?.last_notification_message_id;
  if (!messageId) return;

  const report = statusDoc.data() as StatusReport;
  report.friends = report.friends.map((f) => ({
    ...f,
    snoozed: snoozedFriends.has(f.friend_id),
    snoozed_until: snoozedFriends.get(f.friend_id),
  }));

  const text = formatStatusReport(report, friends);
  await editTelegramMessage(chatId, messageId, text, botToken);
}

/**
 * Main webhook handler
 */
export const webhook = onRequest(
  { secrets: [telegramBotToken, telegramChatId, githubToken] },
  async (req, res) => {
    const botToken = telegramBotToken.value();

    // Only accept POST requests
    if (req.method !== "POST") {
      res.status(405).send("Method Not Allowed");
      return;
    }

    const update: TelegramUpdate = req.body;

    // Handle /update command — triggers the GitHub Actions workflow
    if (update.message?.text === "/update" && update.message.chat) {
      const chatId = update.message.chat.id;
      try {
        const response = await fetch(
          "https://api.github.com/repos/jameslafa/matecheck/actions/workflows/daily-check.yml/dispatches",
          {
            method: "POST",
            headers: {
              Authorization: `Bearer ${githubToken.value()}`,
              Accept: "application/vnd.github+json",
              "Content-Type": "application/json",
            },
            body: JSON.stringify({ ref: "master" }),
          }
        );

        if (response.status === 204) {
          await sendTelegramMessage(chatId, "🔄 Update triggered! The check will run in a minute.", botToken);
        } else {
          const body = await response.text();
          console.error("GitHub API error:", response.status, body);
          await sendTelegramMessage(chatId, "❌ Failed to trigger update.", botToken);
        }
      } catch (error) {
        console.error("Error triggering workflow:", error);
        await sendTelegramMessage(chatId, "❌ Failed to trigger update.", botToken);
      }
      res.status(200).send("OK");
      return;
    }

    // Handle /refresh command
    if (update.message?.text === "/refresh" && update.message.chat) {
      const chatId = update.message.chat.id;
      try {
        await editNotification(Number(telegramChatId.value()), botToken);
        await sendTelegramMessage(chatId, "✅ Notification refreshed.", botToken);
      } catch (error) {
        console.error("Error handling /refresh:", error);
        await sendTelegramMessage(chatId, "❌ Failed to refresh notification.", botToken);
      }
      res.status(200).send("OK");
      return;
    }

    // Handle /unsnooze command
    if (update.message?.text?.startsWith("/unsnooze") && update.message.chat) {
      const chatId = update.message.chat.id;
      const arg = update.message.text.replace("/unsnooze", "").trim();

      if (!arg) {
        await sendTelegramMessage(chatId, "Usage: /unsnooze <name or id>", botToken);
        res.status(200).send("OK");
        return;
      }

      try {
        const friends = await getAllFriends();
        const friend = findFriend(arg, friends);

        if (!friend) {
          await sendTelegramMessage(chatId, `❌ Friend not found: "${arg}"`, botToken);
          res.status(200).send("OK");
          return;
        }

        await unsnoozeFriend(friend.id);
        await Promise.all([
          sendTelegramMessage(chatId, `✅ Snooze removed for ${friend.name}`, botToken),
          editNotification(Number(telegramChatId.value()), botToken),
        ]);
      } catch (error) {
        console.error("Error handling /unsnooze:", error);
        await sendTelegramMessage(chatId, "❌ Failed to remove snooze.", botToken);
      }

      res.status(200).send("OK");
      return;
    }

    // Handle /snooze command
    if (update.message?.text?.startsWith("/snooze") && update.message.chat) {
      const chatId = update.message.chat.id;
      const parts = update.message.text.replace("/snooze", "").trim().split(/\s+/);

      if (parts.length < 2 || !parts[0]) {
        await sendTelegramMessage(chatId, "Usage: /snooze <name> <days>", botToken);
        res.status(200).send("OK");
        return;
      }

      const days = parseInt(parts[parts.length - 1], 10);
      const nameQuery = parts.slice(0, -1).join(" ");

      if (isNaN(days) || days <= 0) {
        await sendTelegramMessage(chatId, "Usage: /snooze <name> <days>", botToken);
        res.status(200).send("OK");
        return;
      }

      try {
        const friends = await getAllFriends();
        const friend = findFriend(nameQuery, friends);

        if (!friend) {
          await sendTelegramMessage(chatId, `❌ Friend not found: "${nameQuery}"`, botToken);
          res.status(200).send("OK");
          return;
        }

        await snoozeFriend(friend.id, days);
        await Promise.all([
          sendTelegramMessage(chatId, `✅ ${friend.name} snoozed for ${days} days`, botToken),
          editNotification(Number(telegramChatId.value()), botToken),
        ]);
      } catch (error) {
        console.error("Error handling /snooze:", error);
        await sendTelegramMessage(chatId, "❌ Failed to snooze.", botToken);
      }

      res.status(200).send("OK");
      return;
    }

    // Handle /deactivate command
    if (update.message?.text?.startsWith("/deactivate") && update.message.chat) {
      const chatId = update.message.chat.id;
      const arg = update.message.text.replace("/deactivate", "").trim();

      if (!arg) {
        await sendTelegramMessage(chatId, "Usage: /deactivate <name or id>", botToken);
        res.status(200).send("OK");
        return;
      }

      try {
        const friends = await db.collection("friends").get().then(s => s.docs.map(d => d.data() as FriendConfig));
        const friend = findFriend(arg, friends);

        if (!friend) {
          await sendTelegramMessage(chatId, `❌ Friend not found: "${arg}"`, botToken);
          res.status(200).send("OK");
          return;
        }

        await setFriendActive(friend.id, false);
        await sendTelegramMessage(chatId, `✅ ${friend.name} deactivated and will be excluded from notifications`, botToken);
      } catch (error) {
        console.error("Error handling /deactivate:", error);
        await sendTelegramMessage(chatId, "❌ Failed to deactivate.", botToken);
      }

      res.status(200).send("OK");
      return;
    }

    // Handle /activate command
    if (update.message?.text?.startsWith("/activate") && update.message.chat) {
      const chatId = update.message.chat.id;
      const arg = update.message.text.replace("/activate", "").trim();

      if (!arg) {
        await sendTelegramMessage(chatId, "Usage: /activate <name or id>", botToken);
        res.status(200).send("OK");
        return;
      }

      try {
        const friends = await db.collection("friends").get().then(s => s.docs.map(d => d.data() as FriendConfig));
        const friend = findFriend(arg, friends);

        if (!friend) {
          await sendTelegramMessage(chatId, `❌ Friend not found: "${arg}"`, botToken);
          res.status(200).send("OK");
          return;
        }

        await setFriendActive(friend.id, true);
        await sendTelegramMessage(chatId, `✅ ${friend.name} activated and will appear in notifications`, botToken);
      } catch (error) {
        console.error("Error handling /activate:", error);
        await sendTelegramMessage(chatId, "❌ Failed to activate.", botToken);
      }

      res.status(200).send("OK");
      return;
    }

    res.status(200).send("OK");
  }
);

/**
 * Firestore trigger: sends morning Telegram notification when Rust writes should_notify=true
 */
export const morningNotification = onDocumentWritten(
  { document: "status/latest", secrets: [telegramBotToken, telegramChatId] },
  async (event) => {
    const data = event.data?.after?.data();
    if (!data?.should_notify) return;

    // Reset flag immediately to prevent re-triggering
    await db.collection("status").doc("latest").update({ should_notify: false });

    const report = data as StatusReport;
    const [snoozedFriends, friends] = await Promise.all([
      getActiveSnoozedFriends(),
      getAllFriends(),
    ]);
    report.friends = report.friends.map((f) => ({
      ...f,
      snoozed: snoozedFriends.has(f.friend_id),
      snoozed_until: snoozedFriends.get(f.friend_id),
    }));

    const text = formatStatusReport(report, friends);
    const messageId = await sendTelegramMessage(
      Number(telegramChatId.value()),
      text,
      telegramBotToken.value()
    );

    if (messageId) {
      await db.collection("status").doc("latest").update({ last_notification_message_id: messageId });
    }
  }
);
