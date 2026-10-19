use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    text::{Line, Span},
    widgets::Paragraph,
};

use crate::ui::theme::Theme;

/// Witty status messages for different operations
pub const FETCH_MESSAGES: &[&str] = &[
    "Munching through the awesome list...",
    "Checking for signs of life...",
    "Poking repos with a stick...",
    "Separating fresh from fossilized...",
    "Hunting for abandoned treasures...",
    "Consulting the GitHub oracle...",
    "Waking up sleeping projects...",
    "Counting stars in the terminal sky...",
    "Feeding the Pac-Man...",
    "Chasing ghosts through the repos...",
];

pub const SYNC_MESSAGES: &[&str] = &[
    "Syncing with the awesome dimension...",
    "Downloading awesomeness...",
    "Parsing markdown magic...",
    "Building the project matrix...",
    "Initializing ghost detector...",
];

/// Status bar component
pub struct StatusBar {
    pub message: String,
    pub progress: Option<(usize, usize)>, // (current, total)
    pub sync_active: bool,
}

impl StatusBar {
    pub fn new() -> Self {
        Self {
            message: String::new(),
            progress: None,
            sync_active: false,
        }
    }

    pub fn set_message(&mut self, message: impl Into<String>) {
        self.message = message.into();
    }

    pub fn set_progress(&mut self, current: usize, total: usize) {
        self.progress = Some((current, total));
    }

    pub fn clear_progress(&mut self) {
        self.progress = None;
        self.sync_active = false;
    }

    pub fn render(&self, frame: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Fill(1),    // Message + progress
                Constraint::Length(50), // Key hints
            ])
            .split(area);

        // Left side: message and progress
        let mut left_spans = vec![];

        if let Some((current, total)) = self.progress {
            // Pac-Man animation
            let progress_pct = (current as f32 / total as f32 * 100.0) as usize;
            let bar_width = 20;
            let filled = (progress_pct * bar_width / 100).min(bar_width);
            let empty = bar_width - filled;

            // Pac-Man character at the end of filled portion
            let pacman = if self.sync_active {
                let frames = ["ᗧ", "ᗤ"]; // Open/closed mouth
                let frame_idx = (current / 3) % frames.len();
                frames[frame_idx]
            } else {
                "●"
            };

            left_spans.push(Span::styled("[", Theme::dim()));
            left_spans.push(Span::styled(
                "●".repeat(filled.saturating_sub(1)),
                Theme::status_active(),
            ));
            left_spans.push(Span::styled(pacman, Theme::title())); // Pac-Man yellow
            left_spans.push(Span::styled("·".repeat(empty), Theme::dim()));
            left_spans.push(Span::styled("] ", Theme::dim()));
            left_spans.push(Span::styled(
                format!("{}/{} ", current, total),
                Theme::normal(),
            ));
        }

        if !self.message.is_empty() {
            left_spans.push(Span::styled(&self.message, Theme::dim()));
        }

        let left = Paragraph::new(Line::from(left_spans));
        frame.render_widget(left, chunks[0]);

        // Right side: key hints
        let hints = Line::from(vec![
            Span::styled("j/k", Theme::key_hint()),
            Span::styled(":nav ", Theme::key_desc()),
            Span::styled("/", Theme::key_hint()),
            Span::styled(":search ", Theme::key_desc()),
            Span::styled("s", Theme::key_hint()),
            Span::styled(":sort ", Theme::key_desc()),
            Span::styled("Enter", Theme::key_hint()),
            Span::styled(":open ", Theme::key_desc()),
            Span::styled("?", Theme::key_hint()),
            Span::styled(":help ", Theme::key_desc()),
            Span::styled("q", Theme::key_hint()),
            Span::styled(":quit", Theme::key_desc()),
        ]);

        let right = Paragraph::new(hints);
        frame.render_widget(right, chunks[1]);
    }
}

impl Default for StatusBar {
    fn default() -> Self {
        Self::new()
    }
}

/// Get a random witty message
pub fn random_fetch_message() -> &'static str {
    use rand::seq::SliceRandom;
    FETCH_MESSAGES
        .choose(&mut rand::thread_rng())
        .unwrap_or(&FETCH_MESSAGES[0])
}

pub fn random_sync_message() -> &'static str {
    use rand::seq::SliceRandom;
    SYNC_MESSAGES
        .choose(&mut rand::thread_rng())
        .unwrap_or(&SYNC_MESSAGES[0])
}
