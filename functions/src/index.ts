import { onRequest } from "firebase-functions/v2/https";
import * as admin from "firebase-admin";
import { defineSecret } from "firebase-functions/params";

// Define secret parameter for Telegram bot token
const telegramBotToken = defineSecret("TELEGRAM_BOT_TOKEN");

// Initialize Firebase Admin
admin.initializeApp();
const db = admin.firestore();

interface TelegramUpdate {
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
 * Edit message to add confirmation text (currently unused)
 */
// async function editMessage(
//   chatId: number,
//   messageId: number,
//   friendName: string,
//   days: number,
//   botToken: string
// ): Promise<void> {
//
//   const url = `https://api.telegram.org/bot${botToken}/editMessageReplyMarkup`;
//
//   // Remove the buttons by setting empty inline keyboard
//   await fetch(url, {
//     method: "POST",
//     headers: { "Content-Type": "application/json" },
//     body: JSON.stringify({
//       chat_id: chatId,
//       message_id: messageId,
//       reply_markup: { inline_keyboard: [] },
//     }),
//   });
//
//   // Note: We're just removing buttons for simplicity
//   // Could also edit the message text to add "✅ Snoozed X for Y days"
// }

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

        // Optionally edit message to show confirmation
        if (message) {
          // For now, just log - could enhance to edit message text
          console.log(`Could edit message ${message.message_id} in chat ${message.chat.id}`);
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
