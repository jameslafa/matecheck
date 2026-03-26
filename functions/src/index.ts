import { onRequest } from "firebase-functions/v2/https";
import { onDocumentWritten } from "firebase-functions/v2/firestore";
import * as admin from "firebase-admin";
import { defineSecret } from "firebase-functions/params";
import { StatusReport, FriendConfig, formatStatusReport, buildSnoozeButtons, findFriend } from "./formatter";

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

interface SnoozeData {
  friendId: string;
  days: number;
}

/**
 * Parse snooze callback data
 * Format: "snooze_alice_7" -> { friendId: "alice", days: 7 }
 */
function parseSnoozeData(data: string): SnoozeData | null {
  const parts = data.split("_");
  if (parts.length !== 3 || parts[0] !== "snooze") {
    return null;
  }

  const friendId = parts[1];
  const days = parseInt(parts[2], 10);

  if (isNaN(days) || days <= 0) {
    return null;
  }

  return { friendId, days };
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
 * Get IDs of all currently snoozed friends
 */
async function getActiveSnoozedFriendIds(): Promise<Set<string>> {
  const now = admin.firestore.Timestamp.now();
  const snapshot = await db.collection("snoozes")
    .where("snoozed_until", ">", now)
    .get();
  const ids = new Set<string>();
  snapshot.forEach((doc) => ids.add(doc.data().friend_id));
  return ids;
}

/**
 * Load all friends from Firestore
 */
async function getAllFriends(): Promise<FriendConfig[]> {
  const snapshot = await db.collection("friends").get();
  return snapshot.docs.map((doc) => doc.data() as FriendConfig);
}

/**
 * Answer Telegram callback query (dismisses loading spinner)
 */
async function answerCallbackQuery(
  callbackQueryId: string,
  text: string,
  botToken: string
): Promise<void> {
  const url = `https://api.telegram.org/bot${botToken}/answerCallbackQuery`;

  const response = await fetch(url, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({
      callback_query_id: callbackQueryId,
      text,
      show_alert: false,
    }),
  });

  if (!response.ok) {
    const error = await response.text();
    console.error("Failed to answer callback query:", error);
  }
}

/**
 * Send a Telegram message
 */
async function sendTelegramMessage(
  chatId: number,
  text: string,
  botToken: string
): Promise<void> {
  const url = `https://api.telegram.org/bot${botToken}/sendMessage`;

  const response = await fetch(url, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({
      chat_id: chatId,
      text,
      parse_mode: "Markdown",
    }),
  });

  if (!response.ok) {
    const error = await response.text();
    console.error("Failed to send message:", error);
  }
}

/**
 * Send a Telegram message with inline keyboard buttons
 */
async function sendTelegramMessageWithButtons(
  chatId: number,
  text: string,
  buttons: Array<Array<{ text: string; callback_data: string }>>,
  botToken: string
): Promise<void> {
  const url = `https://api.telegram.org/bot${botToken}/sendMessage`;

  const response = await fetch(url, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({
      chat_id: chatId,
      text,
      parse_mode: "Markdown",
      link_preview_options: { is_disabled: true },
      reply_markup: { inline_keyboard: buttons },
    }),
  });

  if (!response.ok) {
    const error = await response.text();
    console.error("Failed to send message with buttons:", error);
  }
}

/**
 * Rebuild the morning message from live Firestore state after a snooze.
 * Reads the current status report, active snoozes, and friend configs,
 * then edits the message with accurate content and the updated keyboard.
 */
async function rebuildMessageAfterSnooze(
  chatId: number,
  messageId: number,
  snoozedFriendId: string,
  currentKeyboard: Array<Array<{ text: string; callback_data: string }>>,
  botToken: string
): Promise<void> {
  const [statusDoc, snoozedIds, friends] = await Promise.all([
    db.collection("status").doc("latest").get(),
    getActiveSnoozedFriendIds(),
    getAllFriends(),
  ]);

  if (!statusDoc.exists) return;

  const report = statusDoc.data() as StatusReport;

  // Update snoozed flags from current live snooze state
  report.friends = report.friends.map((f) => ({
    ...f,
    snoozed: snoozedIds.has(f.friend_id),
  }));

  const updatedText = formatStatusReport(report, friends);

  // Remove the snoozed friend's button row
  const updatedKeyboard = currentKeyboard.filter(
    (row) => !row.some((btn) => btn.callback_data.startsWith(`snooze_${snoozedFriendId}_`))
  );

  const url = `https://api.telegram.org/bot${botToken}/editMessageText`;
  await fetch(url, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({
      chat_id: chatId,
      message_id: messageId,
      text: updatedText,
      parse_mode: "Markdown",
      reply_markup: { inline_keyboard: updatedKeyboard },
    }),
  });
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

    // Handle /report command
    if (update.message?.text === "/report" && update.message.chat) {
      try {
        const [statusDoc, friends] = await Promise.all([
          db.collection("status").doc("latest").get(),
          getAllFriends(),
        ]);

        if (!statusDoc.exists) {
          await sendTelegramMessage(
            update.message.chat.id,
            "No status report available yet. Run the cron job first.",
            botToken
          );
        } else {
          const report = statusDoc.data() as StatusReport;
          const snoozedIds = await getActiveSnoozedFriendIds();
          report.friends = report.friends.map((f) => ({
            ...f,
            snoozed: snoozedIds.has(f.friend_id),
          }));
          const formatted = formatStatusReport(report, friends);
          await sendTelegramMessage(update.message.chat.id, formatted, botToken);
        }
      } catch (error) {
        console.error("Error handling /report:", error);
        await sendTelegramMessage(
          update.message.chat.id,
          "❌ Failed to load status report.",
          botToken
        );
      }
      res.status(200).send("OK");
      return;
    }

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
        await sendTelegramMessage(chatId, `✅ Snooze removed for ${friend.name}`, botToken);
      } catch (error) {
        console.error("Error handling /unsnooze:", error);
        await sendTelegramMessage(chatId, "❌ Failed to remove snooze.", botToken);
      }

      res.status(200).send("OK");
      return;
    }

    // Handle callback queries (button clicks)
    if (update.callback_query) {
      const { id, data, from, message } = update.callback_query;

      if (!data) {
        res.status(200).send("OK");
        return;
      }

      console.log(`Callback from ${from.first_name}: ${data}`);

      // Parse snooze data
      const snoozeData = parseSnoozeData(data);
      if (!snoozeData) {
        console.error(`Invalid callback data: ${data}`);
        await answerCallbackQuery(id, "❌ Invalid action", botToken);
        res.status(200).send("OK");
        return;
      }

      const { friendId, days } = snoozeData;

      try {
        // Update Firestore
        await snoozeFriend(friendId, days);
        console.log(`Snoozed ${friendId} for ${days} days`);

        // Answer callback query (dismisses spinner)
        const daysText = days === 3 ? "3 days" : days === 7 ? "1 week" : "2 weeks";
        await answerCallbackQuery(id, `✅ Snoozed for ${daysText}`, botToken);

        // Rebuild message from live Firestore state
        if (message?.reply_markup?.inline_keyboard) {
          await rebuildMessageAfterSnooze(
            message.chat.id,
            message.message_id,
            friendId,
            message.reply_markup.inline_keyboard,
            botToken
          );
        }

        res.status(200).send("OK");
      } catch (error) {
        console.error("Error processing snooze:", error);
        await answerCallbackQuery(id, "❌ Error - try again", botToken);
        res.status(500).send("Error");
      }
    } else {
      // Not a callback query, just acknowledge
      res.status(200).send("OK");
    }
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
    const [snoozedIds, friends] = await Promise.all([
      getActiveSnoozedFriendIds(),
      getAllFriends(),
    ]);
    report.friends = report.friends.map((f) => ({
      ...f,
      snoozed: snoozedIds.has(f.friend_id),
    }));

    const text = formatStatusReport(report, friends);
    const buttons = buildSnoozeButtons(report.friends);
    await sendTelegramMessageWithButtons(
      Number(telegramChatId.value()),
      text,
      buttons,
      telegramBotToken.value()
    );
  }
);
