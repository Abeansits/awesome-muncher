use color_eyre::Result;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use std::time::Duration;
use tokio::sync::mpsc;

use crate::config::Config;
use crate::db::Database;
use crate::fetch::{GitHubClient, parse_awesome_list};
use crate::models::{Project, Source};
use crate::ui::components::status_bar::{random_fetch_message, random_sync_message};
use crate::ui::screens::MainScreen;

/// Message types for background task communication
#[derive(Debug)]
pub enum AppMessage {
    /// Source markdown fetched and parsed
    SourceParsed {
        projects: Vec<Project>,
        source_id: i64,
    },
    /// GitHub metadata fetched for a project
    ProjectMetadataFetched {
        project_id: i64,
        stars: i64,
        open_issues: i64,
        open_prs: i64,
        contributors: i64,
        last_commit: Option<chrono::DateTime<chrono::Utc>>,
        is_archived: bool,
    },
    /// Sync progress update
    SyncProgress { current: usize, total: usize },
    /// Sync completed
    SyncComplete,
    /// Rate limit hit - includes seconds until reset
    RateLimited { reset_in_seconds: i64 },
    /// Error occurred
    Error(String),
}

/// Application state
pub struct App {
    pub config: Config,
    pub db: Database,
    pub screen: MainScreen,
    pub should_quit: bool,
    pub message_rx: mpsc::Receiver<AppMessage>,
    pub message_tx: mpsc::Sender<AppMessage>,
    konami_buffer: Vec<KeyCode>,
}

// Konami code: Up Up Down Down Left Right Left Right B A
const KONAMI_CODE: &[KeyCode] = &[
    KeyCode::Up,
    KeyCode::Up,
    KeyCode::Down,
    KeyCode::Down,
    KeyCode::Left,
    KeyCode::Right,
    KeyCode::Left,
    KeyCode::Right,
    KeyCode::Char('b'),
    KeyCode::Char('a'),
];

impl App {
    /// Create a new application instance
    pub fn new(config: Config, db: Database) -> Self {
        let (tx, rx) = mpsc::channel(100);

        Self {
            config,
            db,
            screen: MainScreen::new(),
            should_quit: false,
            message_rx: rx,
            message_tx: tx,
            konami_buffer: Vec::with_capacity(KONAMI_CODE.len()),
        }
    }

    /// Initialize the app with data from database
    pub fn init(&mut self) -> Result<()> {
        self.refresh_data()?;
        Ok(())
    }

    /// Refresh UI data from database
    pub fn refresh_data(&mut self) -> Result<()> {
        let projects = self.db.get_projects(
            None,
            self.screen.filter_bar.selected_category.as_deref(),
            self.screen.filter_bar.selected_status,
            if self.screen.filter_bar.search_text.is_empty() {
                None
            } else {
                Some(&self.screen.filter_bar.search_text)
            },
        )?;

        self.screen.set_projects(projects);
        self.screen.set_categories(self.db.get_categories()?);
        self.screen.set_stats(self.db.get_stats()?);

        // Update status bar with sync info
        let total = self.db.get_project_count()?;
        let fetched = self.db.get_fetched_count()?;
        if total > 0 && fetched < total {
            self.screen
                .status_bar
                .set_message(format!("Metadata: {}/{} projects fetched", fetched, total));
        } else if total > 0 {
            self.screen.status_bar.set_message("All projects synced");
        }

        Ok(())
    }

    /// Handle keyboard input
    pub fn handle_key(&mut self, key: KeyEvent) -> Result<()> {
        // Check for Konami code
        self.check_konami(key.code);

        // Handle search mode
        if self.screen.filter_bar.is_searching {
            match key.code {
                KeyCode::Esc => {
                    self.screen.filter_bar.clear_search();
                    self.refresh_data()?;
                }
                KeyCode::Enter => {
                    self.screen.filter_bar.is_searching = false;
                    self.refresh_data()?;
                }
                KeyCode::Backspace => {
                    self.screen.filter_bar.search_text.pop();
                    self.refresh_data()?;
                }
                KeyCode::Char(c) => {
                    self.screen.filter_bar.search_text.push(c);
                    self.refresh_data()?;
                }
                _ => {}
            }
            return Ok(());
        }

        // Handle help overlay
        if self.screen.show_help {
            match key.code {
                KeyCode::Esc | KeyCode::Char('?') | KeyCode::Char('q') => {
                    self.screen.show_help = false;
                }
                _ => {}
            }
            return Ok(());
        }

        // Normal mode key handling
        match key.code {
            // Quit
            KeyCode::Char('q') => {
                self.should_quit = true;
            }
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.should_quit = true;
            }

            // Navigation
            KeyCode::Char('j') | KeyCode::Down => {
                self.screen.project_table.next();
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.screen.project_table.previous();
            }
            KeyCode::Char('g') | KeyCode::Home => {
                self.screen.project_table.first();
            }
            KeyCode::Char('G') | KeyCode::End => {
                self.screen.project_table.last();
            }
            KeyCode::PageDown => {
                self.screen.project_table.page_down(10);
            }
            KeyCode::PageUp => {
                self.screen.project_table.page_up(10);
            }

            // Search
            KeyCode::Char('/') => {
                self.screen.filter_bar.is_searching = true;
            }

            // Filters
            KeyCode::Char('c') => {
                self.screen.filter_bar.cycle_category();
                self.refresh_data()?;
            }
            KeyCode::Char('t') => {
                self.screen.filter_bar.cycle_status();
                self.refresh_data()?;
            }

            // Sort
            KeyCode::Char('s') => {
                let next_col = self.screen.project_table.sort_column.next();
                self.screen.project_table.sort_by(next_col);
            }

            // Open URL
            KeyCode::Enter | KeyCode::Char('o') => {
                if let Some(project) = self.screen.project_table.selected_project() {
                    let _ = open::that(&project.url);
                }
            }

            // Sync
            KeyCode::Char('S') => {
                self.start_sync()?;
            }

            // Help
            KeyCode::Char('?') => {
                self.screen.show_help = true;
            }

            // Clear filters
            KeyCode::Esc => {
                self.screen.filter_bar.clear_search();
                self.screen.filter_bar.selected_category = None;
                self.screen.filter_bar.selected_status = None;
                self.refresh_data()?;
            }

            _ => {}
        }

        Ok(())
    }

    /// Check for Konami code
    fn check_konami(&mut self, key: KeyCode) {
        self.konami_buffer.push(key);

        if self.konami_buffer.len() > KONAMI_CODE.len() {
            self.konami_buffer.remove(0);
        }

        if self.konami_buffer == KONAMI_CODE {
            // Easter egg activated!
            self.screen
                .status_bar
                .set_message("🎮 KONAMI CODE ACTIVATED! You found the secret! 🎮");
            self.konami_buffer.clear();
        }
    }

    /// Start syncing data from sources
    pub fn start_sync(&mut self) -> Result<()> {
        let msg = random_sync_message();
        self.screen.show_loading(msg);
        self.screen.status_bar.sync_active = true;

        // Get or create default source
        let sources = self.db.get_sources()?;
        let source = if sources.is_empty() {
            let default_source = Source::default_cli_apps();
            let id = self.db.upsert_source(&default_source)?;
            Source {
                id: Some(id),
                ..default_source
            }
        } else {
            sources.into_iter().next().unwrap()
        };

        let tx = self.message_tx.clone();
        let source_url = source.url.clone();
        let source_id = source.id.unwrap_or(1);

        // Spawn background task for fetching source
        tokio::spawn(async move {
            match fetch_source_content(&source_url).await {
                Ok(content) => match parse_awesome_list(&content, source_id) {
                    Ok(projects) => {
                        let _ = tx
                            .send(AppMessage::SourceParsed {
                                projects,
                                source_id,
                            })
                            .await;
                    }
                    Err(e) => {
                        let _ = tx
                            .send(AppMessage::Error(format!("Parse error: {}", e)))
                            .await;
                    }
                },
                Err(e) => {
                    let _ = tx
                        .send(AppMessage::Error(format!("Fetch error: {}", e)))
                        .await;
                }
            }
        });

        Ok(())
    }

    /// Start fetching GitHub metadata for projects
    pub fn start_github_fetch(&self) -> Result<()> {
        let projects = self.db.get_unfetched_projects(100)?;
        if projects.is_empty() {
            return Ok(());
        }

        let tx = self.message_tx.clone();
        let token = self.config.github_token.clone();
        let total = projects.len();

        tokio::spawn(async move {
            let mut client = GitHubClient::new(token);
            let mut fetched_count = 0;

            for (i, project) in projects.into_iter().enumerate() {
                let _ = tx
                    .send(AppMessage::SyncProgress {
                        current: i + 1,
                        total,
                    })
                    .await;

                if let (Some(owner), Some(repo)) =
                    (project.github_owner.as_ref(), project.github_repo.as_ref())
                {
                    let (metadata, was_rate_limited) =
                        client.fetch_project_metadata(owner, repo).await;

                    if was_rate_limited {
                        // Stop and notify about rate limit
                        let reset_in = client.rate_limit_reset_in().unwrap_or(3600);
                        let _ = tx
                            .send(AppMessage::RateLimited {
                                reset_in_seconds: reset_in,
                            })
                            .await;
                        break;
                    }

                    if let Some(metadata) = metadata {
                        fetched_count += 1;
                        let _ = tx
                            .send(AppMessage::ProjectMetadataFetched {
                                project_id: project.id.unwrap_or(0),
                                stars: metadata.stars,
                                open_issues: metadata.open_issues,
                                open_prs: metadata.open_prs,
                                contributors: metadata.contributors,
                                last_commit: metadata.last_commit,
                                is_archived: metadata.is_archived,
                            })
                            .await;
                    }
                }

                // Small delay to be nice to GitHub API
                tokio::time::sleep(Duration::from_millis(200)).await;
            }

            if fetched_count > 0 {
                let _ = tx.send(AppMessage::SyncComplete).await;
            }
        });

        Ok(())
    }

    /// Process pending messages from background tasks
    pub fn process_messages(&mut self) -> Result<()> {
        while let Ok(msg) = self.message_rx.try_recv() {
            match msg {
                AppMessage::SourceParsed {
                    projects,
                    source_id,
                } => {
                    let count = self.db.bulk_insert_projects(&projects)?;
                    self.db.mark_source_synced(source_id)?;
                    self.screen
                        .status_bar
                        .set_message(format!("Parsed {} projects", count));
                    self.screen.hide_loading();
                    self.refresh_data()?;

                    // Start fetching GitHub metadata
                    let msg = random_fetch_message();
                    self.screen.show_loading(msg);
                    self.start_github_fetch()?;
                }
                AppMessage::ProjectMetadataFetched {
                    project_id,
                    stars,
                    open_issues,
                    open_prs,
                    contributors,
                    last_commit,
                    is_archived,
                } => {
                    self.db.update_github_metadata(
                        project_id,
                        stars,
                        open_issues,
                        open_prs,
                        contributors,
                        last_commit,
                        is_archived,
                    )?;
                    // Don't refresh on every update - too slow
                }
                AppMessage::SyncProgress { current, total } => {
                    self.screen.set_loading_progress(current, total);
                    self.screen.status_bar.set_progress(current, total);
                    self.screen.status_bar.set_message(random_fetch_message());

                    // Refresh data periodically during sync
                    if current % 20 == 0 {
                        self.refresh_data()?;
                    }
                }
                AppMessage::SyncComplete => {
                    self.screen.hide_loading();
                    self.screen.status_bar.clear_progress();
                    self.screen.status_bar.set_message("Sync complete! 🎉");
                    self.refresh_data()?;
                }
                AppMessage::RateLimited { reset_in_seconds } => {
                    self.screen.hide_loading();
                    self.screen.status_bar.clear_progress();
                    let minutes = reset_in_seconds / 60;
                    self.screen.status_bar.set_message(format!(
                        "⚠️ GitHub rate limit hit! Resets in {}m. Add token to config for 5000 req/hr",
                        minutes
                    ));
                    self.refresh_data()?;
                }
                AppMessage::Error(err) => {
                    self.screen.hide_loading();
                    self.screen
                        .status_bar
                        .set_message(format!("Error: {}", err));
                }
            }
        }
        Ok(())
    }

    /// Tick for animations
    pub fn tick(&mut self) {
        self.screen.tick_loading();
    }
}

/// Fetch content from a URL
async fn fetch_source_content(url: &str) -> Result<String> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(30))
        .build()?;

    let response = client.get(url).send().await?;
    let content = response.text().await?;
    Ok(content)
}
