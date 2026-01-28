use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Status of a project based on activity and archive state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProjectStatus {
    /// Active: last commit < 30 days
    Active,
    /// Stale: last commit 30-60 days
    Stale,
    /// Abandoned: last commit > 60 days (not archived)
    Abandoned,
    /// Archived: GitHub archived flag is set
    Archived,
    /// Unknown: not yet fetched from GitHub
    Unknown,
}

impl ProjectStatus {
    /// Get the icon for this status (Pac-Man themed!)
    pub fn icon(&self) -> &'static str {
        match self {
            ProjectStatus::Active => "●",     // Power pellet - active!
            ProjectStatus::Stale => "◐",      // Half moon - getting stale
            ProjectStatus::Abandoned => "👻", // Ghost - abandoned
            ProjectStatus::Archived => "⚰️",  // Coffin - RIP
            ProjectStatus::Unknown => "○",    // Empty dot - unknown
        }
    }

    /// Get the display name
    pub fn label(&self) -> &'static str {
        match self {
            ProjectStatus::Active => "Active",
            ProjectStatus::Stale => "Stale",
            ProjectStatus::Abandoned => "Ghost",
            ProjectStatus::Archived => "RIP",
            ProjectStatus::Unknown => "???",
        }
    }

    /// Calculate status from last commit date and archived flag
    pub fn from_activity(last_commit: Option<DateTime<Utc>>, is_archived: bool) -> Self {
        if is_archived {
            return ProjectStatus::Archived;
        }

        match last_commit {
            Some(date) => {
                let days_ago = (Utc::now() - date).num_days();
                if days_ago < 30 {
                    ProjectStatus::Active
                } else if days_ago < 60 {
                    ProjectStatus::Stale
                } else {
                    ProjectStatus::Abandoned
                }
            }
            None => ProjectStatus::Unknown,
        }
    }
}

impl std::fmt::Display for ProjectStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", self.icon(), self.label())
    }
}

/// A project from an awesome list
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    /// Unique ID in our database
    pub id: Option<i64>,
    /// Source ID this project belongs to
    pub source_id: i64,
    /// Project name
    pub name: String,
    /// GitHub URL or other URL
    pub url: String,
    /// Short description from the awesome list
    pub description: String,
    /// Category from the awesome list
    pub category: String,

    // GitHub metadata (fetched separately)
    /// GitHub owner (e.g., "ratatui-org")
    pub github_owner: Option<String>,
    /// GitHub repo name (e.g., "ratatui")
    pub github_repo: Option<String>,
    /// Star count
    pub stars: Option<i64>,
    /// Open issue count
    pub open_issues: Option<i64>,
    /// Open PR count
    pub open_prs: Option<i64>,
    /// Number of contributors
    pub contributors: Option<i64>,
    /// Last commit date
    pub last_commit: Option<DateTime<Utc>>,
    /// Is the repo archived on GitHub?
    pub is_archived: bool,
    /// When we last fetched GitHub data
    pub fetched_at: Option<DateTime<Utc>>,
}

impl Project {
    /// Create a new project from parsed awesome list data
    pub fn new(
        source_id: i64,
        name: String,
        url: String,
        description: String,
        category: String,
    ) -> Self {
        let (github_owner, github_repo) = Self::parse_github_url(&url);

        Self {
            id: None,
            source_id,
            name,
            url,
            description,
            category,
            github_owner,
            github_repo,
            stars: None,
            open_issues: None,
            open_prs: None,
            contributors: None,
            last_commit: None,
            is_archived: false,
            fetched_at: None,
        }
    }

    /// Parse GitHub owner and repo from a URL
    fn parse_github_url(url: &str) -> (Option<String>, Option<String>) {
        // Match patterns like:
        // https://github.com/owner/repo
        // https://github.com/owner/repo/...
        // http://github.com/owner/repo
        let re = match regex::Regex::new(r"github\.com/([^/]+)/([^/\s#?]+)") {
            Ok(r) => r,
            Err(_) => return (None, None),
        };

        let caps = match re.captures(url) {
            Some(c) => c,
            None => return (None, None),
        };

        let owner = caps.get(1).map(|m| m.as_str().to_string());
        let repo = caps
            .get(2)
            .map(|m| m.as_str().trim_end_matches(".git").to_string());

        (owner, repo)
    }

    /// Get the calculated status
    pub fn status(&self) -> ProjectStatus {
        ProjectStatus::from_activity(self.last_commit, self.is_archived)
    }

    /// Check if this is a GitHub project
    pub fn is_github(&self) -> bool {
        self.github_owner.is_some() && self.github_repo.is_some()
    }

    /// Get the full GitHub path (owner/repo)
    pub fn github_path(&self) -> Option<String> {
        match (&self.github_owner, &self.github_repo) {
            (Some(owner), Some(repo)) => Some(format!("{}/{}", owner, repo)),
            _ => None,
        }
    }

    /// Format stars for display (e.g., "1.2k", "15.3k")
    pub fn stars_display(&self) -> String {
        match self.stars {
            Some(s) if s >= 1000 => format!("{:.1}k", s as f64 / 1000.0),
            Some(s) => s.to_string(),
            None => "-".to_string(),
        }
    }

    /// Format last activity for display
    pub fn activity_display(&self) -> String {
        match self.last_commit {
            Some(date) => {
                let days = (Utc::now() - date).num_days();
                if days == 0 {
                    "today".to_string()
                } else if days == 1 {
                    "yesterday".to_string()
                } else if days < 30 {
                    format!("{}d ago", days)
                } else if days < 365 {
                    format!("{}mo ago", days / 30)
                } else {
                    format!("{}y ago", days / 365)
                }
            }
            None => {
                if self.is_archived {
                    "Archived".to_string()
                } else {
                    "-".to_string()
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_github_url() {
        let project = Project::new(
            1,
            "ratatui".to_string(),
            "https://github.com/ratatui-org/ratatui".to_string(),
            "A Rust TUI library".to_string(),
            "TUI".to_string(),
        );

        assert_eq!(project.github_owner, Some("ratatui-org".to_string()));
        assert_eq!(project.github_repo, Some("ratatui".to_string()));
    }

    #[test]
    fn test_status_calculation() {
        // Active: recent commit
        let recent = Utc::now() - chrono::Duration::days(5);
        assert_eq!(
            ProjectStatus::from_activity(Some(recent), false),
            ProjectStatus::Active
        );

        // Stale: 30-60 days
        let stale = Utc::now() - chrono::Duration::days(45);
        assert_eq!(
            ProjectStatus::from_activity(Some(stale), false),
            ProjectStatus::Stale
        );

        // Abandoned: > 60 days
        let old = Utc::now() - chrono::Duration::days(100);
        assert_eq!(
            ProjectStatus::from_activity(Some(old), false),
            ProjectStatus::Abandoned
        );

        // Archived always wins
        assert_eq!(
            ProjectStatus::from_activity(Some(recent), true),
            ProjectStatus::Archived
        );
    }

    #[test]
    fn test_stars_display() {
        let mut project = Project::new(
            1,
            "test".to_string(),
            "https://github.com/test/test".to_string(),
            "Test".to_string(),
            "Test".to_string(),
        );

        project.stars = Some(500);
        assert_eq!(project.stars_display(), "500");

        project.stars = Some(1500);
        assert_eq!(project.stars_display(), "1.5k");

        project.stars = Some(15300);
        assert_eq!(project.stars_display(), "15.3k");

        project.stars = None;
        assert_eq!(project.stars_display(), "-");
    }
}
