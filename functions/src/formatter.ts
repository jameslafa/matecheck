export interface FriendStatus {
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

export interface StatusReport {
  updated_at: string;
  friends: FriendStatus[];
  should_notify?: boolean;
}

export interface FriendConfig {
  id: string;
  name: string;
  telegram_username?: string;
  whatsapp_phone?: string;
}

/**
 * Returns a clickable Telegram/WhatsApp link, or plain name as fallback
 */
export function friendLink(name: string, telegramUsername?: string, whatsappPhone?: string): string {
  if (telegramUsername) {
    return `[${name}](https://t.me/${telegramUsername})`;
  }
  if (whatsappPhone) {
    const clean = whatsappPhone.replace(/\+/g, "").replace(/\s/g, "");
    return `[${name}](https://wa.me/${clean})`;
  }
  return name;
}

/**
 * Format status report for Telegram, with friend names as clickable links.
 */
export function formatStatusReport(report: StatusReport, friends: FriendConfig[]): string {
  const friendMap = new Map(friends.map((f) => [f.id, f]));

  const statusEmoji: Record<string, string> = {
    overdue: "🔴",
    due_soon: "🟡",
    on_track: "🟢",
    never_met: "⚪",
  };

  const order = ["overdue", "due_soon", "on_track", "never_met"];
  const sorted = [...report.friends].sort(
    (a, b) => order.indexOf(a.status) - order.indexOf(b.status)
  );

  const lines = sorted.map((f) => {
    const emoji = statusEmoji[f.status] || "⚪";
    const snoozed = f.snoozed ? " 💤" : "";

    const fc = friendMap.get(f.friend_id);
    const name = fc
      ? friendLink(f.friend_name, fc.telegram_username, fc.whatsapp_phone)
      : f.friend_name;

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

    return `${emoji} ${name}: ${detail}${snoozed}`;
  });

  const overdue = report.friends.filter((f) => f.status === "overdue").length;
  const dueSoon = report.friends.filter((f) => f.status === "due_soon").length;
  const onTrack = report.friends.filter((f) => f.status === "on_track").length;

  const header = `📊 *Friend Status Report*\n🔴 ${overdue} overdue · 🟡 ${dueSoon} due soon · 🟢 ${onTrack} on track\n`;

  return header + "\n" + lines.join("\n") + "\n\n[Dashboard](https://jameslafa.github.io/matecheck/)";
}

/**
 * Build snooze button rows for friends needing reminders who aren't snoozed.
 */
export function buildSnoozeButtons(
  friends: FriendStatus[]
): Array<Array<{ text: string; callback_data: string }>> {
  return friends
    .filter(
      (f) =>
        !f.snoozed &&
        (f.status === "overdue" || f.status === "due_soon" || f.status === "never_met")
    )
    .map((f) => [
      { text: `${f.friend_name}: 3d`, callback_data: `snooze_${f.friend_id}_3` },
      { text: `${f.friend_name}: 1w`, callback_data: `snooze_${f.friend_id}_7` },
      { text: `${f.friend_name}: 2w`, callback_data: `snooze_${f.friend_id}_14` },
    ]);
}
