use ratatui::style::{Color, Modifier, Style};

/// Application color theme (Pac-Man inspired!)
pub struct Theme;

impl Theme {
    // Main colors
    pub const BACKGROUND: Color = Color::Rgb(20, 20, 30);
    pub const FOREGROUND: Color = Color::Rgb(240, 240, 240);

    // Pac-Man palette
    pub const PACMAN_YELLOW: Color = Color::Rgb(255, 255, 0);
    pub const GHOST_RED: Color = Color::Rgb(255, 0, 0);
    pub const GHOST_PINK: Color = Color::Rgb(255, 184, 255);
    pub const GHOST_CYAN: Color = Color::Rgb(0, 255, 255);
    pub const GHOST_ORANGE: Color = Color::Rgb(255, 184, 82);

    // Status colors
    pub const ACTIVE: Color = Color::Rgb(0, 255, 136); // Bright green
    pub const STALE: Color = Color::Rgb(255, 200, 0); // Yellow/orange
    pub const ABANDONED: Color = Color::Rgb(128, 128, 128); // Gray
    pub const ARCHIVED: Color = Color::Rgb(255, 80, 80); // Red
    pub const UNKNOWN: Color = Color::Rgb(100, 100, 100); // Dark gray

    // UI elements
    pub const BORDER: Color = Color::Rgb(80, 80, 120);
    pub const BORDER_FOCUSED: Color = Color::Rgb(100, 100, 200);
    pub const SELECTION: Color = Color::Rgb(60, 60, 100);
    pub const HIGHLIGHT: Color = Color::Rgb(80, 80, 140);

    // Text colors
    pub const TEXT_DIM: Color = Color::Rgb(128, 128, 128);
    pub const TEXT_MUTED: Color = Color::Rgb(180, 180, 180);

    // Styles
    pub fn title() -> Style {
        Style::default()
            .fg(Self::PACMAN_YELLOW)
            .add_modifier(Modifier::BOLD)
    }

    pub fn header() -> Style {
        Style::default()
            .fg(Self::FOREGROUND)
            .add_modifier(Modifier::BOLD)
    }

    pub fn normal() -> Style {
        Style::default().fg(Self::FOREGROUND)
    }

    pub fn dim() -> Style {
        Style::default().fg(Self::TEXT_DIM)
    }

    pub fn selected() -> Style {
        Style::default()
            .bg(Self::SELECTION)
            .fg(Self::FOREGROUND)
            .add_modifier(Modifier::BOLD)
    }

    pub fn highlight() -> Style {
        Style::default().bg(Self::HIGHLIGHT).fg(Self::FOREGROUND)
    }

    pub fn border() -> Style {
        Style::default().fg(Self::BORDER)
    }

    pub fn border_focused() -> Style {
        Style::default().fg(Self::BORDER_FOCUSED)
    }

    pub fn status_active() -> Style {
        Style::default().fg(Self::ACTIVE)
    }

    pub fn status_stale() -> Style {
        Style::default().fg(Self::STALE)
    }

    pub fn status_abandoned() -> Style {
        Style::default().fg(Self::ABANDONED)
    }

    pub fn status_archived() -> Style {
        Style::default().fg(Self::ARCHIVED)
    }

    pub fn status_unknown() -> Style {
        Style::default().fg(Self::UNKNOWN)
    }

    pub fn key_hint() -> Style {
        Style::default().fg(Self::PACMAN_YELLOW)
    }

    pub fn key_desc() -> Style {
        Style::default().fg(Self::TEXT_MUTED)
    }
}
