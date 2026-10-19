use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout},
};

use crate::db::ProjectStats;
use crate::models::Project;
use crate::ui::components::{
    FilterBar, HelpPopup, LoadingAnimation, ProjectTable, StatsHeader, StatusBar,
};

/// Main screen state
pub struct MainScreen {
    pub stats_header: StatsHeader,
    pub filter_bar: FilterBar,
    pub project_table: ProjectTable,
    pub status_bar: StatusBar,
    pub show_help: bool,
    pub loading: Option<LoadingAnimation>,
}

impl MainScreen {
    pub fn new() -> Self {
        Self {
            stats_header: StatsHeader::default(),
            filter_bar: FilterBar::new(vec![]),
            project_table: ProjectTable::new(vec![]),
            status_bar: StatusBar::new(),
            show_help: false,
            loading: None,
        }
    }

    pub fn set_projects(&mut self, projects: Vec<Project>) {
        self.project_table.set_projects(projects);
    }

    pub fn set_categories(&mut self, categories: Vec<String>) {
        self.filter_bar.categories = categories;
    }

    pub fn set_stats(&mut self, stats: ProjectStats) {
        self.stats_header = StatsHeader::new(stats);
    }

    pub fn show_loading(&mut self, message: impl Into<String>) {
        self.loading = Some(LoadingAnimation::new(message));
    }

    pub fn hide_loading(&mut self) {
        self.loading = None;
    }

    pub fn tick_loading(&mut self) {
        if let Some(ref mut loading) = self.loading {
            loading.tick();
        }
    }

    pub fn set_loading_progress(&mut self, current: usize, total: usize) {
        if let Some(ref mut loading) = self.loading {
            loading.progress = Some((current, total));
        }
    }

    pub fn render(&mut self, frame: &mut Frame) {
        let area = frame.area();

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Stats header
                Constraint::Length(3), // Filter bar
                Constraint::Fill(1),   // Project table
                Constraint::Length(1), // Status bar
            ])
            .split(area);

        self.stats_header.render(frame, chunks[0]);
        self.filter_bar.render(frame, chunks[1]);
        self.project_table.render(frame, chunks[2]);
        self.status_bar.render(frame, chunks[3]);

        // Render overlays - loading should be on top of everything
        if let Some(ref loading) = self.loading {
            loading.render(frame, area);
        }

        // Help popup renders on top of loading
        if self.show_help {
            HelpPopup::render(frame, area);
        }
    }
}

impl Default for MainScreen {
    fn default() -> Self {
        Self::new()
    }
}
