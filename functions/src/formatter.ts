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
  snoozed_until?: string;
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

function escapeHtml(s: string): string {
  return s.replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;");
}

/**
 * Returns an HTML link, or plain escaped name as fallback
 */
export function friendLink(name: string, telegramUsername?: string, whatsappPhone?: string): string {
  const safe = escapeHtml(name);
  if (telegramUsername) {
    return `<a href="https://t.me/${telegramUsername}">${safe}</a>`;
  }
  if (whatsappPhone) {
    const clean = whatsappPhone.replace(/\+/g, "").replace(/\s/g, "");
    return `<a href="https://wa.me/${clean}">${safe}</a>`;
  }
  return safe;
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

function formatShortDate(isoString: string): string {
  return new Intl.DateTimeFormat("en-GB", {
    day: "numeric",
    month: "short",
    timeZone: "Europe/Berlin",
  }).format(new Date(isoString));
}

function snoozeLabel(f: FriendStatus): string {
  if (!f.snoozed) return "";
  return f.snoozed_until ? ` 💤 until ${formatShortDate(f.snoozed_until)}` : " 💤";
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
    const daysUntil = Math.round((new Date(f.next_planned_date!).getTime() - Date.now()) / 864e5);
    const when = daysUntil <= 0 ? "today" : daysUntil === 1 ? "tomorrow" : `in ${daysUntil}d`;
    return `<b>${name}</b>: ${when}${snoozeLabel(f)}`;
  };

  const renderOther = (f: FriendStatus) => {
    const name = getName(f);
    let detail: string;
    if (f.status === "never_met") {
      detail = "never met";
    } else if (f.days_since_last_seen != null) {
      detail = `${f.days_since_last_seen}d ago`;
      if (f.days_overdue > 0) {
        detail += ` · ${f.days_overdue}d late`;
      }
    } else {
      detail = "no data";
    }
    return `<b>${name}</b>: ${detail}${snoozeLabel(f)}`;
  };

  const sections: string[] = [];

  if (planned.length > 0) {
    sections.push("<b>📅 Already planned</b>\n" + planned.map(renderPlanned).join("\n"));
  }
  if (catchUp.length > 0) {
    sections.push("<b>🔴 Need to catch up</b>\n" + catchUp.map(renderOther).join("\n"));
  }
  if (soon.length > 0) {
    sections.push("<b>🟡 Schedule soon</b>\n" + soon.map(renderOther).join("\n"));
  }
  if (onTrack.length > 0) {
    sections.push("<b>🟢 On track</b>\n" + onTrack.map(renderOther).join("\n"));
  }

  const header = `📊 <b>Friend Status Report</b>\n🕐 ${formatBerlinTime(report.updated_at)}`;

  return header + "\n\n" + sections.join("\n\n") + "\n\n<a href=\"https://jameslafa.github.io/matecheck/\">Dashboard</a>";
}
