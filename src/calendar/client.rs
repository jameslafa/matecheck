use anyhow::{Context, Result};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use google_calendar3::{
    api::Event as GoogleEvent,
    hyper_rustls, hyper_util,
    yup_oauth2::{InstalledFlowAuthenticator, InstalledFlowReturnMethod},
    CalendarHub,
};
use hyper_util::client::legacy::Client;

use super::types::Event;

#[cfg(test)]
#[path = "client_test.rs"]
mod client_test;

/// Trait defining the interface for a calendar client
///
/// Allows abstraction over different calendar providers and enables testing with mocks.
#[async_trait]
pub trait CalendarClient {
    /// Fetches calendar events within a time range
    async fn fetch_events(&self, start: DateTime<Utc>, end: DateTime<Utc>) -> Result<Vec<Event>>;
}

/// Google Calendar API client implementation
pub struct GoogleCalendarClient {
    hub: CalendarHub<
        hyper_rustls::HttpsConnector<hyper_util::client::legacy::connect::HttpConnector>,
    >,
}

impl GoogleCalendarClient {
    /// Creates a new GoogleCalendarClient with OAuth authentication
    ///
    /// On first run, opens browser for OAuth authorization and saves token to token.json.
    /// Subsequent runs use the cached token.
    pub async fn new() -> Result<Self> {
        let auth = authenticate().await?;

        // Create HTTPS-only connector with HTTP/2 support
        let https = hyper_rustls::HttpsConnectorBuilder::new()
            .with_native_roots()?
            .https_only()
            .enable_http1()
            .enable_http2()
            .build();

        let client = Client::builder(hyper_util::rt::TokioExecutor::new()).build(https);
        let hub = CalendarHub::new(client, auth);

        Ok(GoogleCalendarClient { hub })
    }

    /// Converts a Google Calendar API event to our simplified Event type
    pub(crate) fn convert_event(google_event: &GoogleEvent) -> Result<Event> {
        let start = google_event
            .start
            .as_ref()
            .and_then(|dt| dt.date_time)
            .ok_or_else(|| anyhow::anyhow!("Event has no start"))?;

        let end = google_event.end.as_ref().and_then(|dt| dt.date_time);

        let attendees: Vec<String> = google_event
            .attendees
            .as_ref()
            .unwrap_or(&vec![])
            .iter()
            .filter_map(|attendee| attendee.email.clone())
            .collect();

        Ok(Event {
            title: google_event
                .summary
                .clone()
                .unwrap_or_else(|| "Untitled event".to_string()),
            attendees,
            start,
            end,
        })
    }
}

/// Sets up OAuth 2.0 authentication for Google Calendar API
///
/// Reads credentials from credentials.json and persists tokens to token.json.
async fn authenticate() -> Result<
    google_calendar3::yup_oauth2::authenticator::Authenticator<
        hyper_rustls::HttpsConnector<hyper_util::client::legacy::connect::HttpConnector>,
    >,
> {
    let app_secret = google_calendar3::yup_oauth2::read_application_secret("credentials.json")
        .await
        .context("failed to read credentials.json")?;

    let auth =
        InstalledFlowAuthenticator::builder(app_secret, InstalledFlowReturnMethod::HTTPRedirect)
            .persist_tokens_to_disk("token.json")
            .build()
            .await?;

    Ok(auth)
}

#[async_trait]
impl CalendarClient for GoogleCalendarClient {
    async fn fetch_events(&self, start: DateTime<Utc>, end: DateTime<Utc>) -> Result<Vec<Event>> {
        let (_response, events) = self
            .hub
            .events()
            .list("primary")
            .time_min(start)
            .time_max(end)
            .single_events(true)
            .order_by("startTime")
            .doit()
            .await?;

        let items = events.items.unwrap_or_default();

        let converted: Vec<Event> = items
            .iter()
            .filter_map(|e| Self::convert_event(e).ok())
            .collect();

        Ok(converted)
    }
}
