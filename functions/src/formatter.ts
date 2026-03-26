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
  aliases?: string[];
  telegram_username?: string;
  whatsapp_phone?: string;
}

/**
 * Find a friend by id, name, or alias (case-insensitive)
 */
export function findFriend(query: string, friends: FriendConfig[]): FriendConfig | null {
  const q = query.toLowerCase();
  return friends.find((f) =>
    f.id.toLowerCase() === q ||
    f.name.toLowerCase() === q ||
    f.aliases?.some((a) => a.toLowerCase() === q)
  ) ?? null;
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

function formatBerlinTime(isoString: string): string {
  const date = new Date(isoString);
  const parts = new Intl.DateTimeFormat("en-GB", {
    timeZone: "Europe/Berlin",
    weekday: "short",
    day: "numeric",
    month: "short",
    year: "numeric",
    hour: "2-digit",
    minute: "2-digit",
    hour12: false,
  }).formatToParts(date);
  const get = (type: string) => parts.find((p) => p.type === type)?.value ?? "";
  return `${get("weekday")} ${get("day")} ${get("month")} ${get("year")} at ${get("hour")}:${get("minute")}`;
}

function truncate(s: string, max = 50): string {
  return s.length > max ? s.slice(0, max) + "…" : s;
}

/**
 * Format status report for Telegram, with friend names as clickable links.
 */
export function formatStatusReport(report: StatusReport, friends: FriendConfig[]): string {
  const friendMap = new Map(friends.map((f) => [f.id, f]));

  const getName = (f: FriendStatus) => {
    const fc = friendMap.get(f.friend_id);
    return fc ? friendLink(f.friend_name, fc.telegram_username, fc.whatsapp_phone) : f.friend_name;
  };

  const byFreq = (a: FriendStatus, b: FriendStatus) => a.frequency_days - b.frequency_days;

  const planned = report.friends.filter((f) => f.next_planned_date).sort(byFreq);
  const catchUp = report.friends.filter((f) => !f.next_planned_date && (f.status === "overdue" || f.status === "never_met")).sort(byFreq);
  const soon = report.friends.filter((f) => !f.next_planned_date && f.status === "due_soon").sort(byFreq);
  const onTrack = report.friends.filter((f) => !f.next_planned_date && f.status === "on_track").sort(byFreq);

  const renderPlanned = (f: FriendStatus) => {
    const name = getName(f);
    const snoozed = f.snoozed ? " 💤" : "";
    const daysUntil = Math.round((new Date(f.next_planned_date!).getTime() - Date.now()) / 864e5);
    const when = daysUntil <= 0 ? "today" : daysUntil === 1 ? "tomorrow" : `in ${daysUntil}d`;
    const eventPart = f.next_planned_event ? ` · ${truncate(f.next_planned_event)}` : "";
    return `📅 ${name}: ${when}${eventPart}${snoozed}`;
  };

  const renderOther = (f: FriendStatus, emoji: string) => {
    const name = getName(f);
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
    const eventPart = f.last_seen_event ? ` · ${truncate(f.last_seen_event)}` : "";
    return `${emoji} ${name}: ${detail}${eventPart}${snoozed}`;
  };

  const sections: string[] = [];

  if (planned.length > 0) {
    sections.push("*📅 Already planned*\n" + planned.map(renderPlanned).join("\n"));
  }
  if (catchUp.length > 0) {
    sections.push("*🔴 Need to catch up*\n" + catchUp.map((f) => renderOther(f, "🔴")).join("\n"));
  }
  if (soon.length > 0) {
    sections.push("*🟡 Schedule soon*\n" + soon.map((f) => renderOther(f, "🟡")).join("\n"));
  }
  if (onTrack.length > 0) {
    sections.push("*🟢 On track*\n" + onTrack.map((f) => renderOther(f, "🟢")).join("\n"));
  }

  const header = `📊 *Friend Status Report*\n🕐 ${formatBerlinTime(report.updated_at)}`;

  return header + "\n\n" + sections.join("\n\n") + "\n\n[Dashboard](https://jameslafa.github.io/matecheck/)";
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
        !f.next_planned_date &&
        (f.status === "overdue" || f.status === "due_soon" || f.status === "never_met")
    )
    .map((f) => [
      { text: `${f.friend_name}: 3d`, callback_data: `snooze_${f.friend_id}_3` },
      { text: `${f.friend_name}: 1w`, callback_data: `snooze_${f.friend_id}_7` },
      { text: `${f.friend_name}: 2w`, callback_data: `snooze_${f.friend_id}_14` },
    ]);
}
