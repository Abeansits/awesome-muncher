use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::db::ProjectStats;
use crate::ui::theme::Theme;

/// Stats dashboard header showing project counts
pub struct StatsHeader {
    stats: ProjectStats,
}

impl StatsHeader {
    pub fn new(stats: ProjectStats) -> Self {
        Self { stats }
    }

    pub fn render(&self, frame: &mut Frame, area: Rect) {
        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Theme::border())
            .title(Span::styled(" 🟡 awesome-muncher ", Theme::title()));

        let inner = block.inner(area);
        frame.render_widget(block, area);

        // Create stats line
        let stats_line = Line::from(vec![
            Span::styled(
                format!("● {} ", self.stats.active),
                Theme::status_active().add_modifier(Modifier::BOLD),
            ),
            Span::styled("Active  ", Theme::dim()),
            Span::styled(
                format!("◐ {} ", self.stats.stale),
                Theme::status_stale().add_modifier(Modifier::BOLD),
            ),
            Span::styled("Stale  ", Theme::dim()),
            Span::styled(
                format!("👻 {} ", self.stats.abandoned),
                Theme::status_abandoned().add_modifier(Modifier::BOLD),
            ),
            Span::styled("Ghosts  ", Theme::dim()),
            Span::styled(
                format!("⚰️  {} ", self.stats.archived),
                Theme::status_archived().add_modifier(Modifier::BOLD),
            ),
            Span::styled("RIP  ", Theme::dim()),
            Span::styled(
                format!("○ {} ", self.stats.unknown),
                Theme::status_unknown().add_modifier(Modifier::BOLD),
            ),
            Span::styled("Unknown", Theme::dim()),
        ]);

        // Center the stats
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Fill(1),
                Constraint::Min(80),
                Constraint::Fill(1),
            ])
            .split(inner);

        let stats_widget = Paragraph::new(stats_line);
        frame.render_widget(stats_widget, chunks[1]);
    }
}

impl Default for StatsHeader {
    fn default() -> Self {
        Self {
            stats: ProjectStats {
                total: 0,
                active: 0,
                stale: 0,
                abandoned: 0,
                archived: 0,
                unknown: 0,
            },
        }
    }
}
