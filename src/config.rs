use anyhow::{Context, Result};
use serde::Deserialize;
use std::fs;

/// Represents a single friend to track meetings with.
///
/// The `#[derive(Deserialize, Debug)]` automatically generates:
/// - Deserialize: Code to parse YAML into this struct
/// - Debug: Code to print this struct nicely (like fmt.Printf("%+v") in Go)
#[derive(Deserialize, Debug, Clone)]
pub struct Friend {
    /// Friend's display name
    pub name: String,

    /// Email address to match in calendar events
    pub email: String,

    /// Telegram username (without @) for generating chat links
    pub telegram_username: String,

    /// How often (in days) you want to meet this friend
    pub frequency_days: u32,
}

/// Main configuration structure that holds all friends.
#[derive(Deserialize, Debug)]
pub struct Config {
    pub friends: Vec<Friend>,
}

impl Config {
    /// Loads configuration from a YAML file.
    ///
    /// Returns Result<Config, anyhow::Error>
    /// - Ok(config) if successful
    /// - Err(error) if file not found or YAML is invalid
    ///
    /// This is like Go's: func Load(path string) (*Config, error)
    pub fn load(path: &str) -> Result<Config> {
        // Read file to string
        // The `?` operator is like: if err != nil { return err }
        // If there's an error, it returns early with that error
        let contents = fs::read_to_string(path)
            .context(format!("Failed to read config file: {}", path))?;

        // Parse YAML into Config struct
        // serde_yaml uses the Deserialize trait we derived!
        let config: Config = serde_yaml::from_str(&contents)
            .context("Failed to parse YAML config")?;

        // Return Ok(config) - like return config, nil in Go
        Ok(config)
    }
}
