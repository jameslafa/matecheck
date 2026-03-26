import { friendLink, formatStatusReport, findFriend, FriendStatus, StatusReport, FriendConfig } from "./formatter";

const friends: FriendConfig[] = [
  { id: "alice", name: "Alice", telegram_username: "alice_tg" },
  { id: "bob", name: "Bob", whatsapp_phone: "+49 157 3463 0875" },
  { id: "charlie", name: "Charlie" },
  { id: "dave", name: "Dave", telegram_username: "", whatsapp_phone: "+1234567890" },
];

function makeStatus(overrides: Partial<FriendStatus> & Pick<FriendStatus, "friend_id" | "friend_name" | "status">): FriendStatus {
  return {
    frequency_days: 30,
    days_overdue: 0,
    snoozed: false,
    ...overrides,
  };
}

function makeReport(friends_: FriendStatus[]): StatusReport {
  return { updated_at: "2026-03-25T08:00:00Z", friends: friends_ };
}

const friendsWithAliases: FriendConfig[] = [
  { id: "alice", name: "Alice", aliases: ["Al", "Alicia"] },
  { id: "bob", name: "Bob" },
];

// --- findFriend ---

test("findFriend: match by id", () => {
  expect(findFriend("alice", friendsWithAliases)?.id).toBe("alice");
});

test("findFriend: match by name case-insensitive", () => {
  expect(findFriend("ALICE", friendsWithAliases)?.id).toBe("alice");
});

test("findFriend: match by alias case-insensitive", () => {
  expect(findFriend("al", friendsWithAliases)?.id).toBe("alice");
  expect(findFriend("Alicia", friendsWithAliases)?.id).toBe("alice");
});

test("findFriend: not found returns null", () => {
  expect(findFriend("dave", friendsWithAliases)).toBeNull();
});

test("findFriend: no aliases field does not crash", () => {
  expect(findFriend("bob", friendsWithAliases)?.id).toBe("bob");
});

// --- friendLink ---

test("friendLink: telegram username", () => {
  expect(friendLink("Alice", "alice_tg")).toBe('<a href="https://t.me/alice_tg">Alice</a>');
});

test("friendLink: whatsapp phone", () => {
  expect(friendLink("Bob", undefined, "+49 157 3463 0875")).toBe('<a href="https://wa.me/4915734630875">Bob</a>');
});

test("friendLink: strips + and spaces from phone", () => {
  expect(friendLink("Bob", undefined, "+1 234 567 8900")).toBe('<a href="https://wa.me/12345678900">Bob</a>');
});

test("friendLink: empty telegram falls back to whatsapp", () => {
  expect(friendLink("Dave", "", "+1234567890")).toBe('<a href="https://wa.me/1234567890">Dave</a>');
});

test("friendLink: no contact info returns plain name", () => {
  expect(friendLink("Charlie")).toBe("Charlie");
});

test("friendLink: escapes HTML special chars in name", () => {
  expect(friendLink("A&B")).toBe("A&amp;B");
});

// --- formatStatusReport ---

test("formatStatusReport: never_met appears under Need to catch up", () => {
  const report = makeReport([makeStatus({ friend_id: "charlie", friend_name: "Charlie", status: "never_met" })]);
  const text = formatStatusReport(report, friends);
  expect(text).toContain("<b>🔴 Need to catch up</b>");
  expect(text).toContain("<b>Charlie</b>: never met");
});

test("formatStatusReport: overdue shows days ago and late count", () => {
  const report = makeReport([makeStatus({ friend_id: "alice", friend_name: "Alice", status: "overdue", days_since_last_seen: 45, days_overdue: 15 })]);
  const text = formatStatusReport(report, friends);
  expect(text).toContain('<b><a href="https://t.me/alice_tg">Alice</a></b>: 45d ago · 15d late');
});

test("formatStatusReport: on_track no overdue suffix", () => {
  const report = makeReport([makeStatus({ friend_id: "alice", friend_name: "Alice", status: "on_track", days_since_last_seen: 10, days_overdue: -5 })]);
  const text = formatStatusReport(report, friends);
  expect(text).toContain('<b><a href="https://t.me/alice_tg">Alice</a></b>: 10d ago');
  expect(text).not.toContain("d overdue");
});

test("formatStatusReport: friend with next_planned_date goes to Already planned bucket", () => {
  const futureDate = new Date(Date.now() + 3 * 864e5).toISOString();
  const report = makeReport([makeStatus({
    friend_id: "alice", friend_name: "Alice", status: "on_track",
    days_since_last_seen: 10, days_overdue: -5,
    next_planned_event: "Coffee", next_planned_date: futureDate,
  })]);
  const text = formatStatusReport(report, friends);
  expect(text).toContain("<b>📅 Already planned</b>");
  expect(text).toContain('<b><a href="https://t.me/alice_tg">Alice</a></b>: in 3d');
  expect(text).not.toContain("<b>🟢 On track</b>");
});

test("formatStatusReport: snoozed with expiry appears in Snoozed section with until date", () => {
  const until = "2026-04-02T12:00:00Z";
  const report = makeReport([makeStatus({ friend_id: "alice", friend_name: "Alice", status: "overdue", days_since_last_seen: 40, days_overdue: 10, snoozed: true, snoozed_until: until })]);
  const text = formatStatusReport(report, friends);
  expect(text).toContain("<b>💤 Snoozed</b>");
  expect(text).toContain("💤 until 2 Apr");
  expect(text).not.toContain("<b>🔴 Need to catch up</b>");
});

test("formatStatusReport: snoozed without expiry appears in Snoozed section with plain 💤", () => {
  const report = makeReport([makeStatus({ friend_id: "alice", friend_name: "Alice", status: "overdue", days_since_last_seen: 40, days_overdue: 10, snoozed: true })]);
  const text = formatStatusReport(report, friends);
  expect(text).toContain("<b>💤 Snoozed</b>");
  expect(text).toContain(" 💤");
  expect(text).not.toContain("until");
  expect(text).not.toContain("<b>🔴 Need to catch up</b>");
});

test("formatStatusReport: bucket order is planned, catch-up, soon, on-track, snoozed", () => {
  const futureDate = new Date(Date.now() + 5 * 864e5).toISOString();
  const statuses: FriendStatus[] = [
    makeStatus({ friend_id: "charlie", friend_name: "Charlie", status: "on_track", days_since_last_seen: 10 }),
    makeStatus({ friend_id: "alice", friend_name: "Alice", status: "overdue", days_since_last_seen: 50, days_overdue: 20 }),
    makeStatus({ friend_id: "bob", friend_name: "Bob", status: "due_soon", days_since_last_seen: 25, next_planned_date: futureDate, next_planned_event: "Dinner" }),
    makeStatus({ friend_id: "dave", friend_name: "Dave", status: "overdue", days_since_last_seen: 40, days_overdue: 10, snoozed: true }),
  ];
  const text = formatStatusReport(makeReport(statuses), friends);
  const plannedPos = text.indexOf("Already planned");
  const catchUpPos = text.indexOf("Need to catch up");
  const onTrackPos = text.indexOf("On track");
  const snoozedPos = text.indexOf("Snoozed");
  expect(plannedPos).toBeLessThan(catchUpPos);
  expect(catchUpPos).toBeLessThan(onTrackPos);
  expect(onTrackPos).toBeLessThan(snoozedPos);
});

test("formatStatusReport: frequency-based sort within bucket", () => {
  const statuses: FriendStatus[] = [
    makeStatus({ friend_id: "alice", friend_name: "Alice", status: "overdue", days_since_last_seen: 50, days_overdue: 20, frequency_days: 60 }),
    makeStatus({ friend_id: "bob", friend_name: "Bob", status: "overdue", days_since_last_seen: 50, days_overdue: 20, frequency_days: 14 }),
  ];
  const text = formatStatusReport(makeReport(statuses), friends);
  expect(text.indexOf("Bob")).toBeLessThan(text.indexOf("Alice"));
});

test("formatStatusReport: header has Berlin timestamp", () => {
  const report = makeReport([makeStatus({ friend_id: "alice", friend_name: "Alice", status: "on_track", days_since_last_seen: 5 })]);
  const text = formatStatusReport(report, friends);
  expect(text).toContain("📊 <b>Friend Status Report</b>");
  expect(text).toContain("🕐 ");
  // updated_at is 2026-03-25T08:00:00Z = 09:00 Berlin (CET, UTC+1)
  expect(text).toContain("25 Mar 2026 at 09:00");
});

test("formatStatusReport: due_soon without plan goes to Schedule soon bucket", () => {
  const report = makeReport([makeStatus({ friend_id: "bob", friend_name: "Bob", status: "due_soon", days_since_last_seen: 20 })]);
  const text = formatStatusReport(report, friends);
  expect(text).toContain("<b>🟡 Schedule soon</b>");
  expect(text).toContain('<b><a href="https://wa.me/4915734630875">Bob</a></b>: 20d ago');
});

test("formatStatusReport: planned shows 'today' when daysUntil <= 0", () => {
  const pastDate = new Date(Date.now() - 1 * 864e5).toISOString();
  const report = makeReport([makeStatus({ friend_id: "alice", friend_name: "Alice", status: "on_track", next_planned_date: pastDate })]);
  const text = formatStatusReport(report, friends);
  expect(text).toContain('<b><a href="https://t.me/alice_tg">Alice</a></b>: today');
});

test("formatStatusReport: planned shows 'tomorrow' when daysUntil is 1", () => {
  const tomorrow = new Date(Date.now() + 1.2 * 864e5).toISOString();
  const report = makeReport([makeStatus({ friend_id: "alice", friend_name: "Alice", status: "on_track", next_planned_date: tomorrow })]);
  const text = formatStatusReport(report, friends);
  expect(text).toContain('<b><a href="https://t.me/alice_tg">Alice</a></b>: tomorrow');
});

test("formatStatusReport: no data when days_since_last_seen absent on non-never_met", () => {
  const report = makeReport([makeStatus({ friend_id: "alice", friend_name: "Alice", status: "overdue", days_overdue: 5 })]);
  const text = formatStatusReport(report, friends);
  expect(text).toContain('<b><a href="https://t.me/alice_tg">Alice</a></b>: no data');
});

