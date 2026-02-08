use chrono::{DateTime, Utc};
use super::types::Event;

/// Check if Do Not Disturb mode is currently active
///
/// Returns Some(event_title) if DND is active, None otherwise.
/// DND is detected by:
/// - All-day events only (not timed events)
/// - Event title contains 🔕 emoji OR [DND] text (case-insensitive)
/// - Current time falls within the event's date range
pub fn is_dnd_active(events: &[Event], now: DateTime<Utc>) -> Option<String> {
    events
        .iter()
        .filter(|e| is_dnd_event(e))
        .find(|e| is_time_in_range(now, e.start, e.end))
        .map(|e| e.title.clone())
}

/// Check if an event is a DND event based on markers
fn is_dnd_event(event: &Event) -> bool {
    if !event.is_all_day {
        return false;
    }

    let title_lower = event.title.to_lowercase();
    title_lower.contains("🔕") || title_lower.contains("[dnd]")
}

/// Check if current time falls within event's time range
fn is_time_in_range(now: DateTime<Utc>, start: DateTime<Utc>, end: Option<DateTime<Utc>>) -> bool {
    if let Some(end_time) = end {
        // Multi-day event: check if now is between start and end (exclusive)
        now >= start && now < end_time
    } else {
        // Single-day event: check if same day
        now.date_naive() == start.date_naive()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    fn make_all_day_event(title: &str, start_day: u32, end_day: Option<u32>) -> Event {
        let start = Utc.with_ymd_and_hms(2024, 1, start_day, 0, 0, 0).unwrap();
        let end = end_day.map(|d| Utc.with_ymd_and_hms(2024, 1, d, 0, 0, 0).unwrap());
        Event {
            title: title.to_string(),
            attendees: vec![],
            start,
            end,
            is_all_day: true,
        }
    }

    fn make_timed_event(title: &str, day: u32, hour: u32) -> Event {
        let start = Utc.with_ymd_and_hms(2024, 1, day, hour, 0, 0).unwrap();
        let end = Utc.with_ymd_and_hms(2024, 1, day, hour + 1, 0, 0).unwrap();
        Event {
            title: title.to_string(),
            attendees: vec![],
            start,
            end: Some(end),
            is_all_day: false,
        }
    }

    #[test]
    fn test_dnd_emoji_marker() {
        let event = make_all_day_event("🔕 Vacation in Paris", 15, Some(18));
        assert!(is_dnd_event(&event));
    }

    #[test]
    fn test_dnd_text_marker_uppercase() {
        let event = make_all_day_event("[DND] Focus Week", 15, Some(20));
        assert!(is_dnd_event(&event));
    }

    #[test]
    fn test_dnd_text_marker_lowercase() {
        let event = make_all_day_event("[dnd] Personal Time", 15, None);
        assert!(is_dnd_event(&event));
    }

    #[test]
    fn test_dnd_text_marker_mixed_case() {
        let event = make_all_day_event("[DnD] Mixed case", 15, None);
        assert!(is_dnd_event(&event));
    }

    #[test]
    fn test_dnd_both_markers() {
        let event = make_all_day_event("🔕 [DND] Both markers", 15, None);
        assert!(is_dnd_event(&event));
    }

    #[test]
    fn test_dnd_marker_in_middle() {
        let event = make_all_day_event("Working from home [DND] today", 15, None);
        assert!(is_dnd_event(&event));
    }

    #[test]
    fn test_non_dnd_event() {
        let event = make_all_day_event("Regular Vacation", 15, Some(20));
        assert!(!is_dnd_event(&event));
    }

    #[test]
    fn test_timed_event_with_dnd_marker_ignored() {
        let event = make_timed_event("🔕 Meeting", 15, 10);
        assert!(!is_dnd_event(&event));
    }

    #[test]
    fn test_timed_event_with_dnd_text_ignored() {
        let event = make_timed_event("[DND] Conference Call", 15, 14);
        assert!(!is_dnd_event(&event));
    }

    #[test]
    fn test_single_day_dnd_active_same_day() {
        let events = vec![make_all_day_event("🔕 Rest Day", 15, None)];
        let now = Utc.with_ymd_and_hms(2024, 1, 15, 12, 30, 0).unwrap();

        let result = is_dnd_active(&events, now);
        assert_eq!(result, Some("🔕 Rest Day".to_string()));
    }

    #[test]
    fn test_single_day_dnd_inactive_different_day() {
        let events = vec![make_all_day_event("🔕 Rest Day", 15, None)];
        let now = Utc.with_ymd_and_hms(2024, 1, 16, 12, 30, 0).unwrap();

        let result = is_dnd_active(&events, now);
        assert_eq!(result, None);
    }

    #[test]
    fn test_multi_day_dnd_active_first_day() {
        let events = vec![make_all_day_event("🔕 Vacation", 15, Some(18))];
        let now = Utc.with_ymd_and_hms(2024, 1, 15, 0, 0, 0).unwrap();

        let result = is_dnd_active(&events, now);
        assert_eq!(result, Some("🔕 Vacation".to_string()));
    }

    #[test]
    fn test_multi_day_dnd_active_middle_day() {
        let events = vec![make_all_day_event("🔕 Vacation", 15, Some(18))];
        let now = Utc.with_ymd_and_hms(2024, 1, 16, 12, 0, 0).unwrap();

        let result = is_dnd_active(&events, now);
        assert_eq!(result, Some("🔕 Vacation".to_string()));
    }

    #[test]
    fn test_multi_day_dnd_active_last_day_before_end() {
        let events = vec![make_all_day_event("🔕 Vacation", 15, Some(18))];
        let now = Utc.with_ymd_and_hms(2024, 1, 17, 23, 59, 59).unwrap();

        let result = is_dnd_active(&events, now);
        assert_eq!(result, Some("🔕 Vacation".to_string()));
    }

    #[test]
    fn test_multi_day_dnd_inactive_on_end_date() {
        // End date is exclusive in Google Calendar
        let events = vec![make_all_day_event("🔕 Vacation", 15, Some(18))];
        let now = Utc.with_ymd_and_hms(2024, 1, 18, 0, 0, 0).unwrap();

        let result = is_dnd_active(&events, now);
        assert_eq!(result, None);
    }

    #[test]
    fn test_dnd_inactive_before_start() {
        let events = vec![make_all_day_event("🔕 Future Event", 20, Some(25))];
        let now = Utc.with_ymd_and_hms(2024, 1, 19, 12, 0, 0).unwrap();

        let result = is_dnd_active(&events, now);
        assert_eq!(result, None);
    }

    #[test]
    fn test_dnd_inactive_after_end() {
        let events = vec![make_all_day_event("🔕 Past Event", 10, Some(13))];
        let now = Utc.with_ymd_and_hms(2024, 1, 15, 12, 0, 0).unwrap();

        let result = is_dnd_active(&events, now);
        assert_eq!(result, None);
    }

    #[test]
    fn test_multiple_dnd_events_returns_first_active() {
        let events = vec![
            make_all_day_event("🔕 First", 15, Some(18)),
            make_all_day_event("[DND] Second", 15, Some(20)),
        ];
        let now = Utc.with_ymd_and_hms(2024, 1, 15, 12, 0, 0).unwrap();

        let result = is_dnd_active(&events, now);
        assert_eq!(result, Some("🔕 First".to_string()));
    }

    #[test]
    fn test_multiple_events_only_one_active() {
        let events = vec![
            make_all_day_event("🔕 Past", 10, Some(13)),
            make_all_day_event("[DND] Active", 15, Some(18)),
            make_all_day_event("🔕 Future", 20, Some(25)),
        ];
        let now = Utc.with_ymd_and_hms(2024, 1, 16, 12, 0, 0).unwrap();

        let result = is_dnd_active(&events, now);
        assert_eq!(result, Some("[DND] Active".to_string()));
    }

    #[test]
    fn test_mixed_events_only_dnd_detected() {
        let events = vec![
            make_all_day_event("Regular Event", 15, None),
            make_timed_event("🔕 Meeting", 15, 10),
            make_all_day_event("[DND] Focus Day", 15, None),
        ];
        let now = Utc.with_ymd_and_hms(2024, 1, 15, 12, 0, 0).unwrap();

        let result = is_dnd_active(&events, now);
        assert_eq!(result, Some("[DND] Focus Day".to_string()));
    }

    #[test]
    fn test_empty_events_list() {
        let events: Vec<Event> = vec![];
        let now = Utc.with_ymd_and_hms(2024, 1, 15, 12, 0, 0).unwrap();

        let result = is_dnd_active(&events, now);
        assert_eq!(result, None);
    }

    #[test]
    fn test_no_dnd_events_in_list() {
        let events = vec![
            make_all_day_event("Regular Event 1", 15, None),
            make_all_day_event("Regular Event 2", 15, Some(18)),
            make_timed_event("Meeting", 15, 10),
        ];
        let now = Utc.with_ymd_and_hms(2024, 1, 15, 12, 0, 0).unwrap();

        let result = is_dnd_active(&events, now);
        assert_eq!(result, None);
    }
}
