use color_eyre::Result;
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// Application configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// GitHub personal access token (optional, increases rate limit)
    #[serde(default)]
    pub github_token: Option<String>,

    /// Number of days without commits before a project is considered abandoned
    #[serde(default = "default_abandoned_days")]
    pub abandoned_days: u32,

    /// Number of days without commits before a project is considered stale
    #[serde(default = "default_stale_days")]
    pub stale_days: u32,

    /// How often to re-fetch GitHub metadata (in hours)
    #[serde(default = "default_refresh_hours")]
    pub refresh_hours: u32,

    /// Maximum concurrent GitHub API requests
    #[serde(default = "default_max_concurrent")]
    pub max_concurrent_requests: u32,

    /// Enable animations
    #[serde(default = "default_true")]
    pub animations_enabled: bool,

    /// Show witty messages
    #[serde(default = "default_true")]
    pub witty_messages: bool,
}

fn default_abandoned_days() -> u32 {
    60
}

fn default_stale_days() -> u32 {
    30
}

fn default_refresh_hours() -> u32 {
    24
}

fn default_max_concurrent() -> u32 {
    5
}

fn default_true() -> bool {
    true
}

impl Default for Config {
    fn default() -> Self {
        Self {
            github_token: None,
            abandoned_days: default_abandoned_days(),
            stale_days: default_stale_days(),
            refresh_hours: default_refresh_hours(),
            max_concurrent_requests: default_max_concurrent(),
            animations_enabled: true,
            witty_messages: true,
        }
    }
}

impl Config {
    /// Get the config directory path
    pub fn config_dir() -> Result<PathBuf> {
        let proj_dirs = ProjectDirs::from("com", "awesome-muncher", "awesome-muncher")
            .ok_or_else(|| color_eyre::eyre::eyre!("Could not determine config directory"))?;

        let config_dir = proj_dirs.config_dir();
        fs::create_dir_all(config_dir)?;

        Ok(config_dir.to_path_buf())
    }

    /// Get the data directory path (for database)
    pub fn data_dir() -> Result<PathBuf> {
        let proj_dirs = ProjectDirs::from("com", "awesome-muncher", "awesome-muncher")
            .ok_or_else(|| color_eyre::eyre::eyre!("Could not determine data directory"))?;

        let data_dir = proj_dirs.data_dir();
        fs::create_dir_all(data_dir)?;

        Ok(data_dir.to_path_buf())
    }

    /// Get the config file path
    pub fn config_path() -> Result<PathBuf> {
        Ok(Self::config_dir()?.join("config.toml"))
    }

    /// Get the database file path
    pub fn database_path() -> Result<PathBuf> {
        Ok(Self::data_dir()?.join("awesome-muncher.db"))
    }

    /// Load config from file, or create default
    pub fn load() -> Result<Self> {
        let path = Self::config_path()?;

        if path.exists() {
            let content = fs::read_to_string(&path)?;
            let config: Config = toml::from_str(&content)?;
            Ok(config)
        } else {
            let config = Config::default();
            config.save()?;
            Ok(config)
        }
    }

    /// Save config to file
    pub fn save(&self) -> Result<()> {
        let path = Self::config_path()?;
        let content = toml::to_string_pretty(self)?;
        fs::write(path, content)?;
        Ok(())
    }

    /// Check if we have a GitHub token configured
    pub fn has_github_token(&self) -> bool {
        self.github_token
            .as_ref()
            .is_some_and(|t| !t.trim().is_empty())
    }

    /// Get rate limit info based on token status
    pub fn rate_limit_info(&self) -> &'static str {
        if self.has_github_token() {
            "5,000 requests/hour (authenticated)"
        } else {
            "60 requests/hour (unauthenticated)"
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.abandoned_days, 60);
        assert_eq!(config.stale_days, 30);
        assert!(config.animations_enabled);
        assert!(config.witty_messages);
    }

    #[test]
    fn test_config_serialization() {
        let config = Config::default();
        let toml_str = toml::to_string_pretty(&config).unwrap();
        let parsed: Config = toml::from_str(&toml_str).unwrap();
        assert_eq!(config.abandoned_days, parsed.abandoned_days);
    }
}
