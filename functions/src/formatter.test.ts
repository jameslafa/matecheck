import { friendLink, formatStatusReport, buildSnoozeButtons, findFriend, FriendStatus, StatusReport, FriendConfig } from "./formatter";

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
  expect(friendLink("Alice", "alice_tg")).toBe("[Alice](https://t.me/alice_tg)");
});

test("friendLink: whatsapp phone", () => {
  expect(friendLink("Bob", undefined, "+49 157 3463 0875")).toBe("[Bob](https://wa.me/4915734630875)");
});

test("friendLink: strips + and spaces from phone", () => {
  expect(friendLink("Bob", undefined, "+1 234 567 8900")).toBe("[Bob](https://wa.me/12345678900)");
});

test("friendLink: empty telegram falls back to whatsapp", () => {
  expect(friendLink("Dave", "", "+1234567890")).toBe("[Dave](https://wa.me/1234567890)");
});

test("friendLink: no contact info returns plain name", () => {
  expect(friendLink("Charlie")).toBe("Charlie");
});

// --- formatStatusReport ---

test("formatStatusReport: never_met appears under Need to catch up", () => {
  const report = makeReport([makeStatus({ friend_id: "charlie", friend_name: "Charlie", status: "never_met" })]);
  const text = formatStatusReport(report, friends);
  expect(text).toContain("*🔴 Need to catch up*");
  expect(text).toContain("🔴 Charlie: never met");
});

test("formatStatusReport: overdue shows days and overdue count", () => {
  const report = makeReport([makeStatus({ friend_id: "alice", friend_name: "Alice", status: "overdue", days_since_last_seen: 45, days_overdue: 15 })]);
  const text = formatStatusReport(report, friends);
  expect(text).toContain("🔴 [Alice](https://t.me/alice_tg): 45d ago (15d overdue)");
});

test("formatStatusReport: on_track no overdue suffix", () => {
  const report = makeReport([makeStatus({ friend_id: "alice", friend_name: "Alice", status: "on_track", days_since_last_seen: 10, days_overdue: -5 })]);
  const text = formatStatusReport(report, friends);
  expect(text).toContain("🟢 [Alice](https://t.me/alice_tg): 10d ago");
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
  expect(text).toContain("*📅 Already planned*");
  expect(text).toContain("📅 [Alice](https://t.me/alice_tg): in 3d · Coffee");
  expect(text).not.toContain("*🟢 On track*");
});

test("formatStatusReport: snoozed shows 💤", () => {
  const report = makeReport([makeStatus({ friend_id: "alice", friend_name: "Alice", status: "overdue", days_since_last_seen: 40, days_overdue: 10, snoozed: true })]);
  const text = formatStatusReport(report, friends);
  expect(text).toContain("💤");
});

test("formatStatusReport: bucket order is planned, catch-up, soon, on-track", () => {
  const futureDate = new Date(Date.now() + 5 * 864e5).toISOString();
  const statuses: FriendStatus[] = [
    makeStatus({ friend_id: "charlie", friend_name: "Charlie", status: "on_track", days_since_last_seen: 10 }),
    makeStatus({ friend_id: "alice", friend_name: "Alice", status: "overdue", days_since_last_seen: 50, days_overdue: 20 }),
    makeStatus({ friend_id: "bob", friend_name: "Bob", status: "due_soon", days_since_last_seen: 25, next_planned_date: futureDate, next_planned_event: "Dinner" }),
  ];
  const text = formatStatusReport(makeReport(statuses), friends);
  const plannedPos = text.indexOf("Already planned");
  const catchUpPos = text.indexOf("Need to catch up");
  const onTrackPos = text.indexOf("On track");
  expect(plannedPos).toBeLessThan(catchUpPos);
  expect(catchUpPos).toBeLessThan(onTrackPos);
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
  expect(text).toContain("📊 *Friend Status Report*");
  expect(text).toContain("🕐 ");
  // updated_at is 2026-03-25T08:00:00Z = 09:00 Berlin (CET, UTC+1)
  expect(text).toContain("25 Mar 2026 at 09:00");
});

test("formatStatusReport: event name truncated to 50 chars", () => {
  const longEvent = "A".repeat(55);
  const report = makeReport([makeStatus({ friend_id: "alice", friend_name: "Alice", status: "overdue", days_since_last_seen: 40, days_overdue: 10, last_seen_event: longEvent })]);
  const text = formatStatusReport(report, friends);
  expect(text).toContain("A".repeat(50) + "…");
  expect(text).not.toContain("A".repeat(51));
});

test("formatStatusReport: due_soon without plan goes to Schedule soon bucket", () => {
  const report = makeReport([makeStatus({ friend_id: "bob", friend_name: "Bob", status: "due_soon", days_since_last_seen: 20 })]);
  const text = formatStatusReport(report, friends);
  expect(text).toContain("*🟡 Schedule soon*");
  expect(text).toContain("🟡 [Bob](https://wa.me/4915734630875): 20d ago");
});

test("formatStatusReport: planned shows 'today' when daysUntil <= 0", () => {
  const pastDate = new Date(Date.now() - 1 * 864e5).toISOString();
  const report = makeReport([makeStatus({ friend_id: "alice", friend_name: "Alice", status: "on_track", next_planned_date: pastDate, next_planned_event: "Coffee" })]);
  const text = formatStatusReport(report, friends);
  expect(text).toContain("📅 [Alice](https://t.me/alice_tg): today · Coffee");
});

test("formatStatusReport: planned shows 'tomorrow' when daysUntil is 1", () => {
  const tomorrow = new Date(Date.now() + 1.2 * 864e5).toISOString();
  const report = makeReport([makeStatus({ friend_id: "alice", friend_name: "Alice", status: "on_track", next_planned_date: tomorrow, next_planned_event: "Dinner" })]);
  const text = formatStatusReport(report, friends);
  expect(text).toContain("📅 [Alice](https://t.me/alice_tg): tomorrow · Dinner");
});

test("formatStatusReport: last_seen_event appended on non-planned line", () => {
  const report = makeReport([makeStatus({ friend_id: "alice", friend_name: "Alice", status: "overdue", days_since_last_seen: 45, days_overdue: 15, last_seen_event: "Coffee" })]);
  const text = formatStatusReport(report, friends);
  expect(text).toContain("🔴 [Alice](https://t.me/alice_tg): 45d ago (15d overdue) · Coffee");
});

test("formatStatusReport: no data when days_since_last_seen absent on non-never_met", () => {
  const report = makeReport([makeStatus({ friend_id: "alice", friend_name: "Alice", status: "overdue", days_overdue: 5 })]);
  const text = formatStatusReport(report, friends);
  expect(text).toContain("🔴 [Alice](https://t.me/alice_tg): no data");
});

// --- buildSnoozeButtons ---

test("buildSnoozeButtons: includes overdue non-snoozed friends", () => {
  const statuses = [makeStatus({ friend_id: "alice", friend_name: "Alice", status: "overdue" })];
  const buttons = buildSnoozeButtons(statuses);
  expect(buttons).toHaveLength(1);
  expect(buttons[0]).toHaveLength(3);
  expect(buttons[0][0]).toEqual({ text: "Alice: 3d", callback_data: "snooze_alice_3" });
  expect(buttons[0][1]).toEqual({ text: "Alice: 1w", callback_data: "snooze_alice_7" });
  expect(buttons[0][2]).toEqual({ text: "Alice: 2w", callback_data: "snooze_alice_14" });
});

test("buildSnoozeButtons: excludes on_track friends", () => {
  const statuses = [makeStatus({ friend_id: "alice", friend_name: "Alice", status: "on_track" })];
  expect(buildSnoozeButtons(statuses)).toHaveLength(0);
});

test("buildSnoozeButtons: excludes snoozed friends", () => {
  const statuses = [makeStatus({ friend_id: "alice", friend_name: "Alice", status: "overdue", snoozed: true })];
  expect(buildSnoozeButtons(statuses)).toHaveLength(0);
});

test("buildSnoozeButtons: includes never_met and due_soon", () => {
  const statuses = [
    makeStatus({ friend_id: "alice", friend_name: "Alice", status: "never_met" }),
    makeStatus({ friend_id: "bob", friend_name: "Bob", status: "due_soon" }),
  ];
  expect(buildSnoozeButtons(statuses)).toHaveLength(2);
});

test("buildSnoozeButtons: excludes friends with next_planned_date", () => {
  const futureDate = new Date(Date.now() + 5 * 864e5).toISOString();
  const statuses = [
    makeStatus({ friend_id: "alice", friend_name: "Alice", status: "overdue", next_planned_date: futureDate }),
    makeStatus({ friend_id: "bob", friend_name: "Bob", status: "overdue" }),
  ];
  const buttons = buildSnoozeButtons(statuses);
  expect(buttons).toHaveLength(1);
  expect(buttons[0][0].callback_data).toContain("bob");
});
