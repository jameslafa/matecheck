import { onRequest } from "firebase-functions/v2/https";
import * as admin from "firebase-admin";
import { defineSecret } from "firebase-functions/params";

// Define secret parameter for Telegram bot token
const telegramBotToken = defineSecret("TELEGRAM_BOT_TOKEN");

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
      reply_markup?: {
        inline_keyboard: Array<Array<{ text: string; callback_data: string }>>;
      };
    };
    data?: string;
  };
}

interface FriendStatus {
  friend_id: string;
  friend_name: string;
  last_seen_date?: string;
  last_seen_event?: string;
  next_planned_date?: string;
  next_planned_event?: string;
  days_since_last_seen?: number;
  frequency_days: number;
  days_overdue: number;
  status: "on_track" | "due_soon" | "overdue" | "never_met";
  snoozed: boolean;
}

interface StatusReport {
  updated_at: string;
  friends: FriendStatus[];
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
 * Edit the inline keyboard of an existing message, removing rows for a specific friend
 */
async function removeSnoozeButtonsForFriend(
  chatId: number,
  messageId: number,
  snoozedFriendId: string,
  currentKeyboard: Array<Array<{ text: string; callback_data: string }>>,
  botToken: string
): Promise<void> {
  const updatedKeyboard = currentKeyboard.filter(
    (row) => !row.some((btn) => btn.callback_data.startsWith(`snooze_${snoozedFriendId}_`))
  );

  const url = `https://api.telegram.org/bot${botToken}/editMessageReplyMarkup`;
  await fetch(url, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({
      chat_id: chatId,
      message_id: messageId,
      reply_markup: { inline_keyboard: updatedKeyboard },
    }),
  });
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
 * Format status report for Telegram
 */
function formatStatusReport(report: StatusReport): string {
  const statusEmoji: Record<string, string> = {
    overdue: "🔴",
    due_soon: "🟡",
    on_track: "🟢",
    never_met: "⚪",
  };

  // Sort: overdue first, then due_soon, on_track, never_met
  const order = ["overdue", "due_soon", "on_track", "never_met"];
  const sorted = [...report.friends].sort(
    (a, b) => order.indexOf(a.status) - order.indexOf(b.status)
  );

  const lines = sorted.map((f) => {
    const emoji = statusEmoji[f.status] || "⚪";
    const snoozed = f.snoozed ? " 💤" : "";
    let detail: string;

    if (f.status === "never_met") {
      detail = "never met";
    } else if (f.days_since_last_seen != null) {
      detail = `${f.days_since_last_seen}d ago`;
      if (f.days_overdue > 0) {
        detail += ` (${f.days_overdue}d overdue)`;
      }
    } else {
      detail = "no data";
    }

    if (f.next_planned_event) {
      const daysUntil = Math.round((new Date(f.next_planned_date!).getTime() - Date.now()) / 864e5);
      const when = daysUntil <= 0 ? "today" : daysUntil === 1 ? "tomorrow" : `in ${daysUntil}d`;
      detail += ` · 📅 ${when}`;
    }

    return `${emoji} *${f.friend_name}*: ${detail}${snoozed}`;
  });

  const overdue = report.friends.filter((f) => f.status === "overdue").length;
  const dueSoon = report.friends.filter((f) => f.status === "due_soon").length;
  const onTrack = report.friends.filter((f) => f.status === "on_track").length;

  const header = `📊 *Friend Status Report*\n🔴 ${overdue} overdue · 🟡 ${dueSoon} due soon · 🟢 ${onTrack} on track\n`;

  return header + "\n" + lines.join("\n");
}

/**
 * Main webhook handler
 */
export const webhook = onRequest(
  { secrets: [telegramBotToken] },
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
        const statusDoc = await db.collection("status").doc("latest").get();
        if (!statusDoc.exists) {
          await sendTelegramMessage(
            update.message.chat.id,
            "No status report available yet. Run the cron job first.",
            botToken
          );
        } else {
          const report = statusDoc.data() as StatusReport;
          const formatted = formatStatusReport(report);
          await sendTelegramMessage(
            update.message.chat.id,
            formatted,
            botToken
          );
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

        // Remove the snoozed friend's buttons from the message
        if (message?.reply_markup?.inline_keyboard) {
          await removeSnoozeButtonsForFriend(
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
