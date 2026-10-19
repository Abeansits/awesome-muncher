use ratatui::{
    Frame,
    layout::{Alignment, Rect},
    style::Modifier,
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
};

use crate::ui::theme::Theme;

/// Pac-Man loading animation
pub struct LoadingAnimation {
    pub message: String,
    pub progress: Option<(usize, usize)>,
    frame: usize,
}

impl LoadingAnimation {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            progress: None,
            frame: 0,
        }
    }

    pub fn tick(&mut self) {
        self.frame = self.frame.wrapping_add(1);
    }

    pub fn render(&self, frame: &mut Frame, area: Rect) {
        // Center the popup
        let popup_width: u16 = 55;
        let popup_height: u16 = 8;
        let x = area.width.saturating_sub(popup_width) / 2;
        let y = area.height.saturating_sub(popup_height) / 2;
        let popup_area = Rect::new(
            area.x + x,
            area.y + y,
            popup_width.min(area.width),
            popup_height.min(area.height),
        );

        // Clear the area behind the popup
        frame.render_widget(Clear, popup_area);

        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Theme::border_focused())
            .title(Span::styled(" 🟡 Loading ", Theme::title()));

        let inner = block.inner(popup_area);
        frame.render_widget(block, popup_area);

        // Pac-Man animation - mouth open/closed
        let pacman = if self.frame % 4 < 2 { "ᗧ" } else { "ᗤ" };

        let bar_width = 35;

        // Progress bar with Pac-Man eating through it
        let (bar_line, progress_text) = if let Some((current, total)) = self.progress {
            let pct = current as f32 / total.max(1) as f32;
            let pacman_pos = ((pct * bar_width as f32) as usize).min(bar_width - 1);

            // Eaten part (empty), Pac-Man, remaining dots to eat
            let eaten = " ".repeat(pacman_pos);
            let remaining_dots = (bar_width - pacman_pos - 1).min(bar_width);
            let to_eat = "·".repeat(remaining_dots);

            let line = Line::from(vec![
                Span::raw("["),
                Span::styled(eaten, Theme::dim()),
                Span::styled(pacman, Theme::title().add_modifier(Modifier::BOLD)),
                Span::styled(to_eat, Theme::status_active()),
                Span::raw("]"),
            ]);

            let text = format!("{}/{} ({}%)", current, total, (pct * 100.0) as usize);
            (line, text)
        } else {
            // Indeterminate animation - Pac-Man bounces back and forth
            let pos = self.frame % (bar_width * 2);
            let pacman_pos = if pos < bar_width {
                pos
            } else {
                bar_width * 2 - pos - 1
            };

            let before = "·".repeat(pacman_pos);
            let after = "·".repeat(bar_width.saturating_sub(pacman_pos + 1));

            let line = Line::from(vec![
                Span::raw("["),
                Span::styled(before, Theme::dim()),
                Span::styled(pacman, Theme::title().add_modifier(Modifier::BOLD)),
                Span::styled(after, Theme::dim()),
                Span::raw("]"),
            ]);

            (line, String::new())
        };

        // Build content
        let mut lines = vec![
            Line::from(""),
            Line::from(Span::styled(self.message.clone(), Theme::normal())),
            Line::from(""),
            bar_line,
        ];

        if !progress_text.is_empty() {
            lines.push(Line::from(Span::styled(progress_text, Theme::dim())));
        }

        let paragraph = Paragraph::new(lines).alignment(Alignment::Center);
        frame.render_widget(paragraph, inner);
    }
}
