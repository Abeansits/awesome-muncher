use chrono::{DateTime, Utc};
use color_eyre::Result;
use rusqlite::{Connection, OptionalExtension, params};

use crate::models::source::SourceType;
use crate::models::{Project, ProjectStatus, Source};

/// Database wrapper with query methods
pub struct Database {
    conn: Connection,
}

impl Database {
    /// Open or create database at the given path
    pub fn open(path: &std::path::Path) -> Result<Self> {
        let conn = Connection::open(path)?;
        super::schema::init_db(&conn)?;
        Ok(Self { conn })
    }

    /// Open an in-memory database (for testing)
    #[cfg(test)]
    pub fn open_in_memory() -> Result<Self> {
        let conn = Connection::open_in_memory()?;
        super::schema::init_db(&conn)?;
        Ok(Self { conn })
    }

    // ==================== Source queries ====================

    /// Insert or update a source
    pub fn upsert_source(&self, source: &Source) -> Result<i64> {
        self.conn.execute(
            r#"
            INSERT INTO sources (name, url, source_type, last_synced, enabled)
            VALUES (?1, ?2, ?3, ?4, ?5)
            ON CONFLICT(url) DO UPDATE SET
                name = excluded.name,
                source_type = excluded.source_type,
                enabled = excluded.enabled
            "#,
            params![
                source.name,
                source.url,
                format!("{:?}", source.source_type).to_lowercase(),
                source.last_synced.map(|d| d.to_rfc3339()),
                source.enabled,
            ],
        )?;

        Ok(self.conn.last_insert_rowid())
    }

    /// Get all sources
    pub fn get_sources(&self) -> Result<Vec<Source>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name, url, source_type, last_synced, enabled FROM sources ORDER BY name",
        )?;

        let sources = stmt
            .query_map([], |row| {
                let source_type_str: String = row.get(3)?;
                let source_type = match source_type_str.as_str() {
                    "csv" => SourceType::Csv,
                    _ => SourceType::Markdown,
                };

                let last_synced: Option<String> = row.get(4)?;

                Ok(Source {
                    id: Some(row.get(0)?),
                    name: row.get(1)?,
                    url: row.get(2)?,
                    source_type,
                    last_synced: last_synced
                        .and_then(|s| DateTime::parse_from_rfc3339(&s).ok())
                        .map(|d| d.with_timezone(&Utc)),
                    enabled: row.get(5)?,
                })
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;

        Ok(sources)
    }

    /// Get a source by ID
    pub fn get_source(&self, id: i64) -> Result<Option<Source>> {
        let source = self
            .conn
            .query_row(
                "SELECT id, name, url, source_type, last_synced, enabled FROM sources WHERE id = ?1",
                params![id],
                |row| {
                    let source_type_str: String = row.get(3)?;
                    let source_type = match source_type_str.as_str() {
                        "csv" => SourceType::Csv,
                        _ => SourceType::Markdown,
                    };

                    let last_synced: Option<String> = row.get(4)?;

                    Ok(Source {
                        id: Some(row.get(0)?),
                        name: row.get(1)?,
                        url: row.get(2)?,
                        source_type,
                        last_synced: last_synced
                            .and_then(|s| DateTime::parse_from_rfc3339(&s).ok())
                            .map(|d| d.with_timezone(&Utc)),
                        enabled: row.get(5)?,
                    })
                },
            )
            .optional()?;

        Ok(source)
    }

    /// Update source last_synced timestamp
    pub fn mark_source_synced(&self, source_id: i64) -> Result<()> {
        self.conn.execute(
            "UPDATE sources SET last_synced = ?1 WHERE id = ?2",
            params![Utc::now().to_rfc3339(), source_id],
        )?;
        Ok(())
    }

    /// Delete a source and all its projects
    pub fn delete_source(&self, source_id: i64) -> Result<()> {
        self.conn.execute(
            "DELETE FROM projects WHERE source_id = ?1",
            params![source_id],
        )?;
        self.conn
            .execute("DELETE FROM sources WHERE id = ?1", params![source_id])?;
        Ok(())
    }

    // ==================== Project queries ====================

    /// Insert or update a project
    pub fn upsert_project(&self, project: &Project) -> Result<i64> {
        self.conn.execute(
            r#"
            INSERT INTO projects (
                source_id, name, url, description, category,
                github_owner, github_repo, stars, open_issues, open_prs,
                contributors, last_commit, is_archived, fetched_at
            )
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)
            ON CONFLICT(source_id, url) DO UPDATE SET
                name = excluded.name,
                description = excluded.description,
                category = excluded.category,
                github_owner = excluded.github_owner,
                github_repo = excluded.github_repo,
                stars = COALESCE(excluded.stars, projects.stars),
                open_issues = COALESCE(excluded.open_issues, projects.open_issues),
                open_prs = COALESCE(excluded.open_prs, projects.open_prs),
                contributors = COALESCE(excluded.contributors, projects.contributors),
                last_commit = COALESCE(excluded.last_commit, projects.last_commit),
                is_archived = excluded.is_archived,
                fetched_at = COALESCE(excluded.fetched_at, projects.fetched_at)
            "#,
            params![
                project.source_id,
                project.name,
                project.url,
                project.description,
                project.category,
                project.github_owner,
                project.github_repo,
                project.stars,
                project.open_issues,
                project.open_prs,
                project.contributors,
                project.last_commit.map(|d| d.to_rfc3339()),
                project.is_archived,
                project.fetched_at.map(|d| d.to_rfc3339()),
            ],
        )?;

        Ok(self.conn.last_insert_rowid())
    }

    /// Bulk insert projects (more efficient for initial sync)
    pub fn bulk_insert_projects(&mut self, projects: &[Project]) -> Result<usize> {
        let tx = self.conn.transaction()?;
        let mut count = 0;

        {
            let mut stmt = tx.prepare(
                r#"
                INSERT INTO projects (
                    source_id, name, url, description, category,
                    github_owner, github_repo
                )
                VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
                ON CONFLICT(source_id, url) DO UPDATE SET
                    name = excluded.name,
                    description = excluded.description,
                    category = excluded.category,
                    github_owner = excluded.github_owner,
                    github_repo = excluded.github_repo
                "#,
            )?;

            for project in projects {
                stmt.execute(params![
                    project.source_id,
                    project.name,
                    project.url,
                    project.description,
                    project.category,
                    project.github_owner,
                    project.github_repo,
                ])?;
                count += 1;
            }
        }

        tx.commit()?;
        Ok(count)
    }

    /// Get all projects, optionally filtered
    pub fn get_projects(
        &self,
        source_id: Option<i64>,
        category: Option<&str>,
        status_filter: Option<ProjectStatus>,
        search: Option<&str>,
    ) -> Result<Vec<Project>> {
        let mut sql = String::from(
            r#"
            SELECT 
                id, source_id, name, url, description, category,
                github_owner, github_repo, stars, open_issues, open_prs,
                contributors, last_commit, is_archived, fetched_at
            FROM projects
            WHERE 1=1
            "#,
        );

        let mut params_vec: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

        if let Some(sid) = source_id {
            sql.push_str(" AND source_id = ?");
            params_vec.push(Box::new(sid));
        }

        if let Some(cat) = category {
            sql.push_str(" AND category = ?");
            params_vec.push(Box::new(cat.to_string()));
        }

        if let Some(search_term) = search {
            sql.push_str(" AND (name LIKE ? OR description LIKE ?)");
            let pattern = format!("%{}%", search_term);
            params_vec.push(Box::new(pattern.clone()));
            params_vec.push(Box::new(pattern));
        }

        // Status filter applied post-query since it's calculated
        sql.push_str(" ORDER BY stars DESC NULLS LAST, name ASC");

        let mut stmt = self.conn.prepare(&sql)?;

        let params_refs: Vec<&dyn rusqlite::ToSql> =
            params_vec.iter().map(|p| p.as_ref()).collect();

        let projects = stmt
            .query_map(params_refs.as_slice(), |row| {
                let last_commit: Option<String> = row.get(12)?;
                let fetched_at: Option<String> = row.get(14)?;

                Ok(Project {
                    id: Some(row.get(0)?),
                    source_id: row.get(1)?,
                    name: row.get(2)?,
                    url: row.get(3)?,
                    description: row.get(4)?,
                    category: row.get(5)?,
                    github_owner: row.get(6)?,
                    github_repo: row.get(7)?,
                    stars: row.get(8)?,
                    open_issues: row.get(9)?,
                    open_prs: row.get(10)?,
                    contributors: row.get(11)?,
                    last_commit: last_commit
                        .and_then(|s| DateTime::parse_from_rfc3339(&s).ok())
                        .map(|d| d.with_timezone(&Utc)),
                    is_archived: row.get(13)?,
                    fetched_at: fetched_at
                        .and_then(|s| DateTime::parse_from_rfc3339(&s).ok())
                        .map(|d| d.with_timezone(&Utc)),
                })
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;

        // Apply status filter if specified
        let projects = match status_filter {
            Some(status) => projects
                .into_iter()
                .filter(|p| p.status() == status)
                .collect(),
            None => projects,
        };

        Ok(projects)
    }

    /// Get all unique categories
    pub fn get_categories(&self) -> Result<Vec<String>> {
        let mut stmt = self.conn.prepare(
            "SELECT DISTINCT category FROM projects WHERE category != '' ORDER BY category",
        )?;

        let categories = stmt
            .query_map([], |row| row.get(0))?
            .collect::<std::result::Result<Vec<_>, _>>()?;

        Ok(categories)
    }

    /// Get projects that need GitHub metadata fetched
    pub fn get_unfetched_projects(&self, limit: usize) -> Result<Vec<Project>> {
        let mut stmt = self.conn.prepare(
            r#"
            SELECT 
                id, source_id, name, url, description, category,
                github_owner, github_repo, stars, open_issues, open_prs,
                contributors, last_commit, is_archived, fetched_at
            FROM projects
            WHERE github_owner IS NOT NULL 
              AND github_repo IS NOT NULL
              AND (fetched_at IS NULL OR fetched_at < datetime('now', '-1 day'))
            ORDER BY fetched_at ASC NULLS FIRST
            LIMIT ?1
            "#,
        )?;

        let projects = stmt
            .query_map(params![limit as i64], |row| {
                let last_commit: Option<String> = row.get(12)?;
                let fetched_at: Option<String> = row.get(14)?;

                Ok(Project {
                    id: Some(row.get(0)?),
                    source_id: row.get(1)?,
                    name: row.get(2)?,
                    url: row.get(3)?,
                    description: row.get(4)?,
                    category: row.get(5)?,
                    github_owner: row.get(6)?,
                    github_repo: row.get(7)?,
                    stars: row.get(8)?,
                    open_issues: row.get(9)?,
                    open_prs: row.get(10)?,
                    contributors: row.get(11)?,
                    last_commit: last_commit
                        .and_then(|s| DateTime::parse_from_rfc3339(&s).ok())
                        .map(|d| d.with_timezone(&Utc)),
                    is_archived: row.get(13)?,
                    fetched_at: fetched_at
                        .and_then(|s| DateTime::parse_from_rfc3339(&s).ok())
                        .map(|d| d.with_timezone(&Utc)),
                })
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;

        Ok(projects)
    }

    /// Update GitHub metadata for a project
    pub fn update_github_metadata(
        &self,
        project_id: i64,
        stars: i64,
        open_issues: i64,
        open_prs: i64,
        contributors: i64,
        last_commit: Option<DateTime<Utc>>,
        is_archived: bool,
    ) -> Result<()> {
        self.conn.execute(
            r#"
            UPDATE projects SET
                stars = ?2,
                open_issues = ?3,
                open_prs = ?4,
                contributors = ?5,
                last_commit = ?6,
                is_archived = ?7,
                fetched_at = ?8
            WHERE id = ?1
            "#,
            params![
                project_id,
                stars,
                open_issues,
                open_prs,
                contributors,
                last_commit.map(|d| d.to_rfc3339()),
                is_archived,
                Utc::now().to_rfc3339(),
            ],
        )?;
        Ok(())
    }

    /// Get project statistics
    pub fn get_stats(&self) -> Result<ProjectStats> {
        let total: i64 = self
            .conn
            .query_row("SELECT COUNT(*) FROM projects", [], |row| row.get(0))?;

        // We need to calculate status in Rust since it's derived
        let projects = self.get_projects(None, None, None, None)?;

        let active = projects
            .iter()
            .filter(|p| p.status() == ProjectStatus::Active)
            .count() as i64;
        let stale = projects
            .iter()
            .filter(|p| p.status() == ProjectStatus::Stale)
            .count() as i64;
        let abandoned = projects
            .iter()
            .filter(|p| p.status() == ProjectStatus::Abandoned)
            .count() as i64;
        let archived = projects
            .iter()
            .filter(|p| p.status() == ProjectStatus::Archived)
            .count() as i64;
        let unknown = projects
            .iter()
            .filter(|p| p.status() == ProjectStatus::Unknown)
            .count() as i64;

        Ok(ProjectStats {
            total,
            active,
            stale,
            abandoned,
            archived,
            unknown,
        })
    }

    /// Get total project count
    pub fn get_project_count(&self) -> Result<i64> {
        let count: i64 = self
            .conn
            .query_row("SELECT COUNT(*) FROM projects", [], |row| row.get(0))?;
        Ok(count)
    }

    /// Get count of projects with GitHub metadata
    pub fn get_fetched_count(&self) -> Result<i64> {
        let count: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM projects WHERE fetched_at IS NOT NULL",
            [],
            |row| row.get(0),
        )?;
        Ok(count)
    }
}

/// Statistics about projects
#[derive(Debug, Clone)]
pub struct ProjectStats {
    pub total: i64,
    pub active: i64,
    pub stale: i64,
    pub abandoned: i64,
    pub archived: i64,
    pub unknown: i64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_source_crud() {
        let db = Database::open_in_memory().unwrap();

        let source = Source::default_cli_apps();
        let id = db.upsert_source(&source).unwrap();
        assert!(id > 0);

        let sources = db.get_sources().unwrap();
        assert_eq!(sources.len(), 1);
        assert_eq!(sources[0].name, "Awesome CLI Apps");
    }

    #[test]
    fn test_project_crud() {
        let mut db = Database::open_in_memory().unwrap();

        let source = Source::default_cli_apps();
        let source_id = db.upsert_source(&source).unwrap();

        let projects = vec![
            Project::new(
                source_id,
                "ratatui".to_string(),
                "https://github.com/ratatui-org/ratatui".to_string(),
                "A Rust TUI library".to_string(),
                "TUI".to_string(),
            ),
            Project::new(
                source_id,
                "gitui".to_string(),
                "https://github.com/extrawurst/gitui".to_string(),
                "A Git TUI".to_string(),
                "Git".to_string(),
            ),
        ];

        let count = db.bulk_insert_projects(&projects).unwrap();
        assert_eq!(count, 2);

        let fetched = db.get_projects(None, None, None, None).unwrap();
        assert_eq!(fetched.len(), 2);

        // Test category filter
        let tui_only = db.get_projects(None, Some("TUI"), None, None).unwrap();
        assert_eq!(tui_only.len(), 1);

        // Test search
        let search = db.get_projects(None, None, None, Some("git")).unwrap();
        assert_eq!(search.len(), 1);
    }
}
