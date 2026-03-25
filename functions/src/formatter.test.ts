import { friendLink, formatStatusReport, buildSnoozeButtons, FriendStatus, StatusReport, FriendConfig } from "./formatter";

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

test("formatStatusReport: never_met status", () => {
  const report = makeReport([makeStatus({ friend_id: "charlie", friend_name: "Charlie", status: "never_met" })]);
  const text = formatStatusReport(report, friends);
  expect(text).toContain("⚪ Charlie: never met");
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

test("formatStatusReport: next planned event appended", () => {
  const futureDate = new Date(Date.now() + 3 * 864e5).toISOString();
  const report = makeReport([makeStatus({
    friend_id: "alice", friend_name: "Alice", status: "on_track",
    days_since_last_seen: 10, days_overdue: -5,
    next_planned_event: "Coffee", next_planned_date: futureDate,
  })]);
  const text = formatStatusReport(report, friends);
  expect(text).toContain("📅 in 3d");
});

test("formatStatusReport: snoozed shows 💤", () => {
  const report = makeReport([makeStatus({ friend_id: "alice", friend_name: "Alice", status: "overdue", days_since_last_seen: 40, days_overdue: 10, snoozed: true })]);
  const text = formatStatusReport(report, friends);
  expect(text).toContain("💤");
});

test("formatStatusReport: sort order is overdue, due_soon, on_track, never_met", () => {
  const statuses: FriendStatus[] = [
    makeStatus({ friend_id: "charlie", friend_name: "Charlie", status: "never_met" }),
    makeStatus({ friend_id: "alice", friend_name: "Alice", status: "on_track", days_since_last_seen: 10 }),
    makeStatus({ friend_id: "bob", friend_name: "Bob", status: "overdue", days_since_last_seen: 50, days_overdue: 20 }),
  ];
  const text = formatStatusReport(makeReport(statuses), friends);
  const bobPos = text.indexOf("Bob");
  const alicePos = text.indexOf("Alice");
  const charliePos = text.indexOf("Charlie");
  expect(bobPos).toBeLessThan(alicePos);
  expect(alicePos).toBeLessThan(charliePos);
});

test("formatStatusReport: header shows correct counts", () => {
  const statuses: FriendStatus[] = [
    makeStatus({ friend_id: "alice", friend_name: "Alice", status: "overdue", days_since_last_seen: 50, days_overdue: 20 }),
    makeStatus({ friend_id: "bob", friend_name: "Bob", status: "due_soon", days_since_last_seen: 25, days_overdue: -3 }),
    makeStatus({ friend_id: "charlie", friend_name: "Charlie", status: "on_track", days_since_last_seen: 5 }),
  ];
  const text = formatStatusReport(makeReport(statuses), friends);
  expect(text).toContain("🔴 1 overdue · 🟡 1 due soon · 🟢 1 on track");
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
