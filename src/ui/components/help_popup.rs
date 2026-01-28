use ratatui::{
    layout::Rect,
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

use crate::ui::theme::Theme;

/// Help popup showing keyboard shortcuts
pub struct HelpPopup;

impl HelpPopup {
    pub fn render(frame: &mut Frame, area: Rect) {
        let popup_width = 45;
        let popup_height = 20;
        let x = (area.width.saturating_sub(popup_width)) / 2;
        let y = (area.height.saturating_sub(popup_height)) / 2;
        let popup_area = Rect::new(
            area.x + x,
            area.y + y,
            popup_width.min(area.width),
            popup_height.min(area.height),
        );

        frame.render_widget(Clear, popup_area);

        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Theme::border_focused())
            .title(Span::styled(" 🎮 Keyboard Shortcuts ", Theme::title()));

        let help_text = vec![
            Line::from(""),
            Line::from(vec![Span::styled("  Navigation", Theme::header())]),
            Line::from(vec![
                Span::styled("  j/↓       ", Theme::key_hint()),
                Span::styled("Move down", Theme::key_desc()),
            ]),
            Line::from(vec![
                Span::styled("  k/↑       ", Theme::key_hint()),
                Span::styled("Move up", Theme::key_desc()),
            ]),
            Line::from(vec![
                Span::styled("  g/Home    ", Theme::key_hint()),
                Span::styled("Go to first", Theme::key_desc()),
            ]),
            Line::from(vec![
                Span::styled("  G/End     ", Theme::key_hint()),
                Span::styled("Go to last", Theme::key_desc()),
            ]),
            Line::from(vec![
                Span::styled("  PgDn/PgUp ", Theme::key_hint()),
                Span::styled("Page down/up", Theme::key_desc()),
            ]),
            Line::from(""),
            Line::from(vec![Span::styled("  Actions", Theme::header())]),
            Line::from(vec![
                Span::styled("  Enter/o   ", Theme::key_hint()),
                Span::styled("Open in browser", Theme::key_desc()),
            ]),
            Line::from(vec![
                Span::styled("  /         ", Theme::key_hint()),
                Span::styled("Search", Theme::key_desc()),
            ]),
            Line::from(vec![
                Span::styled("  s         ", Theme::key_hint()),
                Span::styled("Cycle sort column", Theme::key_desc()),
            ]),
            Line::from(vec![
                Span::styled("  c         ", Theme::key_hint()),
                Span::styled("Cycle category filter", Theme::key_desc()),
            ]),
            Line::from(vec![
                Span::styled("  t         ", Theme::key_hint()),
                Span::styled("Cycle status filter", Theme::key_desc()),
            ]),
            Line::from(vec![
                Span::styled("  S         ", Theme::key_hint()),
                Span::styled("Sync/refresh data", Theme::key_desc()),
            ]),
            Line::from(vec![
                Span::styled("  Esc       ", Theme::key_hint()),
                Span::styled("Clear/cancel", Theme::key_desc()),
            ]),
            Line::from(vec![
                Span::styled("  q         ", Theme::key_hint()),
                Span::styled("Quit", Theme::key_desc()),
            ]),
            Line::from(""),
        ];

        let paragraph = Paragraph::new(help_text).block(block);
        frame.render_widget(paragraph, popup_area);
    }
}
