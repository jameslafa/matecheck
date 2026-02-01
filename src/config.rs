use anyhow::{Context, Result};
use serde::Deserialize;
use std::collections::HashSet;
use std::fs;

/// Represents a single friend to track meetings with.
#[derive(Deserialize, Debug, Clone)]
pub struct Friend {
    /// Unique identifier for this friend (must be unique across all friends)
    pub id: String,

    /// Friend's display name
    pub name: String,

    /// Email address to match in calendar events (optional)
    pub email: Option<String>,

    /// Telegram username (without @) for sending reminders (optional)
    pub telegram_username: Option<String>,

    /// How often (in days) you want to meet this friend
    pub frequency_days: u32,
}

/// Main configuration structure that holds all friends.
#[derive(Deserialize, Debug)]
pub struct Config {
    pub friends: Vec<Friend>,
}

impl Config {
    /// Loads configuration from a YAML file and validates it.
    pub fn load(path: &str) -> Result<Config> {
        let contents =
            fs::read_to_string(path).context(format!("Failed to read config file: {}", path))?;

        let config: Config =
            serde_yaml::from_str(&contents).context("Failed to parse YAML config")?;

        // Validate that all friend IDs are unique
        config.validate_unique_ids()?;

        Ok(config)
    }

    /// Validates that all friend IDs are unique.
    fn validate_unique_ids(&self) -> Result<()> {
        let mut seen = HashSet::new();
        for friend in &self.friends {
            if !seen.insert(&friend.id) {
                return Err(anyhow::anyhow!("Duplicate friend ID: {}", &friend.id));
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_unique_ids_success() {
        let config = Config {
            friends: vec![
                Friend {
                    id: "alice".to_string(),
                    name: "Alice".to_string(),
                    email: Some("alice@example.com".to_string()),
                    telegram_username: Some("alice_s".to_string()),
                    frequency_days: 14,
                },
                Friend {
                    id: "bob".to_string(),
                    name: "Bob".to_string(),
                    email: None,
                    telegram_username: None,
                    frequency_days: 30,
                },
            ],
        };

        assert!(config.validate_unique_ids().is_ok());
    }

    #[test]
    fn test_validate_unique_ids_duplicate() {
        let config = Config {
            friends: vec![
                Friend {
                    id: "alice".to_string(),
                    name: "Alice".to_string(),
                    email: Some("alice@example.com".to_string()),
                    telegram_username: Some("alice_s".to_string()),
                    frequency_days: 14,
                },
                Friend {
                    id: "alice".to_string(), // Duplicate!
                    name: "Alice Smith".to_string(),
                    email: Some("alice.smith@example.com".to_string()),
                    telegram_username: Some("alice_smith".to_string()),
                    frequency_days: 30,
                },
            ],
        };

        let result = config.validate_unique_ids();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("alice"));
    }
}
