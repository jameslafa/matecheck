// Calendar module - handles Google Calendar API integration
//
// This module is organized into submodules:
// - types: Data structures for calendar events
// - client: API client for fetching events (we'll create this in Step 2.2)

pub mod types;
// pub mod client;  // Uncomment in Step 2.2

// Re-export commonly used types so users can do:
// use calendar::Event instead of calendar::types::Event
pub use types::Event;
