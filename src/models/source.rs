use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Type of source (how to parse it)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SourceType {
    /// Parse as markdown awesome-list format
    Markdown,
    /// Parse as CSV (future support)
    Csv,
}

impl std::fmt::Display for SourceType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SourceType::Markdown => write!(f, "Markdown"),
            SourceType::Csv => write!(f, "CSV"),
        }
    }
}

/// A source for awesome list data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Source {
    /// Unique ID
    pub id: Option<i64>,
    /// Display name
    pub name: String,
    /// URL to fetch the raw content
    pub url: String,
    /// Type of source
    pub source_type: SourceType,
    /// When this source was last synced
    pub last_synced: Option<DateTime<Utc>>,
    /// Whether this source is enabled
    pub enabled: bool,
}

impl Source {
    /// Create a new source
    pub fn new(name: String, url: String, source_type: SourceType) -> Self {
        Self {
            id: None,
            name,
            url,
            source_type,
            last_synced: None,
            enabled: true,
        }
    }

    /// Create the default awesome-cli-apps source
    pub fn default_cli_apps() -> Self {
        Self::new(
            "Awesome CLI Apps".to_string(),
            "https://raw.githubusercontent.com/toolleeo/awesome-cli-apps-in-a-csv/master/README.md"
                .to_string(),
            SourceType::Markdown,
        )
    }

    /// Get a human-readable sync status
    pub fn sync_status(&self) -> String {
        match self.last_synced {
            Some(date) => {
                let hours = (Utc::now() - date).num_hours();
                if hours < 1 {
                    "Just now".to_string()
                } else if hours < 24 {
                    format!("{}h ago", hours)
                } else {
                    format!("{}d ago", hours / 24)
                }
            }
            None => "Never".to_string(),
        }
    }
}
