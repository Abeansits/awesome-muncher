use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};

use crate::models::ProjectStatus;
use crate::ui::theme::Theme;

/// Filter bar component for search and filtering
pub struct FilterBar {
    pub search_text: String,
    pub selected_category: Option<String>,
    pub selected_status: Option<ProjectStatus>,
    pub categories: Vec<String>,
    pub is_searching: bool,
}

impl FilterBar {
    pub fn new(categories: Vec<String>) -> Self {
        Self {
            search_text: String::new(),
            selected_category: None,
            selected_status: None,
            categories,
            is_searching: false,
        }
    }

    pub fn render(&self, frame: &mut Frame, area: Rect) {
        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(if self.is_searching {
                Theme::border_focused()
            } else {
                Theme::border()
            });

        let inner = block.inner(area);
        frame.render_widget(block, area);

        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Length(30), // Search
                Constraint::Length(2),  // Spacer
                Constraint::Length(25), // Category
                Constraint::Length(2),  // Spacer
                Constraint::Length(20), // Status
                Constraint::Fill(1),    // Rest
            ])
            .split(inner);

        // Search box
        let search_display = if self.search_text.is_empty() && !self.is_searching {
            Span::styled("Type / to search...", Theme::dim())
        } else {
            Span::styled(&self.search_text, Theme::normal())
        };

        let search_line = Line::from(vec![
            Span::styled("🔍 ", Theme::normal()),
            search_display,
            if self.is_searching {
                Span::styled("▌", Style::default().add_modifier(Modifier::SLOW_BLINK))
            } else {
                Span::raw("")
            },
        ]);
        frame.render_widget(Paragraph::new(search_line), chunks[0]);

        // Category dropdown
        let category_text = self
            .selected_category
            .as_deref()
            .unwrap_or("All Categories");
        let category_line = Line::from(vec![
            Span::styled("📁 ", Theme::normal()),
            Span::styled(truncate_str(category_text, 18), Theme::normal()),
            Span::styled(" ▼", Theme::dim()),
        ]);
        frame.render_widget(Paragraph::new(category_line), chunks[2]);

        // Status filter
        let status_text = match self.selected_status {
            Some(ProjectStatus::Active) => "● Active",
            Some(ProjectStatus::Stale) => "◐ Stale",
            Some(ProjectStatus::Abandoned) => "👻 Ghosts",
            Some(ProjectStatus::Archived) => "⚰️  RIP",
            Some(ProjectStatus::Unknown) => "○ Unknown",
            None => "All Status",
        };
        let status_line = Line::from(vec![
            Span::styled("👁 ", Theme::normal()),
            Span::styled(status_text, Theme::normal()),
            Span::styled(" ▼", Theme::dim()),
        ]);
        frame.render_widget(Paragraph::new(status_line), chunks[4]);
    }

    pub fn clear_search(&mut self) {
        self.search_text.clear();
        self.is_searching = false;
    }

    pub fn cycle_status(&mut self) {
        self.selected_status = match self.selected_status {
            None => Some(ProjectStatus::Active),
            Some(ProjectStatus::Active) => Some(ProjectStatus::Stale),
            Some(ProjectStatus::Stale) => Some(ProjectStatus::Abandoned),
            Some(ProjectStatus::Abandoned) => Some(ProjectStatus::Archived),
            Some(ProjectStatus::Archived) => Some(ProjectStatus::Unknown),
            Some(ProjectStatus::Unknown) => None,
        };
    }

    pub fn cycle_category(&mut self) {
        if self.categories.is_empty() {
            return;
        }

        self.selected_category = match &self.selected_category {
            None => Some(self.categories[0].clone()),
            Some(current) => {
                let idx = self.categories.iter().position(|c| c == current);
                match idx {
                    Some(i) if i + 1 < self.categories.len() => {
                        Some(self.categories[i + 1].clone())
                    }
                    _ => None,
                }
            }
        };
    }
}

fn truncate_str(s: &str, max_len: usize) -> String {
    if s.chars().count() <= max_len {
        s.to_string()
    } else {
        format!("{}...", s.chars().take(max_len - 3).collect::<String>())
    }
}
