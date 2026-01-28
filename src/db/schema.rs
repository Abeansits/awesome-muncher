use color_eyre::Result;
use rusqlite::Connection;

/// Initialize the database schema
pub fn init_db(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        r#"
        -- Sources table: where we get awesome lists from
        CREATE TABLE IF NOT EXISTS sources (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL,
            url TEXT NOT NULL UNIQUE,
            source_type TEXT NOT NULL DEFAULT 'markdown',
            last_synced TEXT,
            enabled INTEGER NOT NULL DEFAULT 1
        );

        -- Projects table: individual projects from awesome lists
        CREATE TABLE IF NOT EXISTS projects (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            source_id INTEGER NOT NULL,
            name TEXT NOT NULL,
            url TEXT NOT NULL,
            description TEXT NOT NULL DEFAULT '',
            category TEXT NOT NULL DEFAULT '',
            
            -- GitHub metadata
            github_owner TEXT,
            github_repo TEXT,
            stars INTEGER,
            open_issues INTEGER,
            open_prs INTEGER,
            contributors INTEGER,
            last_commit TEXT,
            is_archived INTEGER NOT NULL DEFAULT 0,
            fetched_at TEXT,
            
            FOREIGN KEY (source_id) REFERENCES sources(id) ON DELETE CASCADE,
            UNIQUE(source_id, url)
        );

        -- Index for faster queries
        CREATE INDEX IF NOT EXISTS idx_projects_source ON projects(source_id);
        CREATE INDEX IF NOT EXISTS idx_projects_category ON projects(category);
        CREATE INDEX IF NOT EXISTS idx_projects_stars ON projects(stars);
        CREATE INDEX IF NOT EXISTS idx_projects_github ON projects(github_owner, github_repo);
        "#,
    )?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_init_db() {
        let conn = Connection::open_in_memory().unwrap();
        init_db(&conn).unwrap();

        // Verify tables exist
        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='projects'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(count, 1);
    }
}
