use ratatui::{
    layout::{Constraint, Rect},
    style::Modifier,
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Row, Table, TableState},
    Frame,
};

use crate::models::{Project, ProjectStatus};
use crate::ui::theme::Theme;

/// Sortable columns
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortColumn {
    Status,
    Name,
    Stars,
    Activity,
    Category,
}

impl SortColumn {
    pub fn next(self) -> Self {
        match self {
            SortColumn::Status => SortColumn::Name,
            SortColumn::Name => SortColumn::Stars,
            SortColumn::Stars => SortColumn::Activity,
            SortColumn::Activity => SortColumn::Category,
            SortColumn::Category => SortColumn::Status,
        }
    }
}

/// Project table component
pub struct ProjectTable {
    pub projects: Vec<Project>,
    pub state: TableState,
    pub sort_column: SortColumn,
    pub sort_ascending: bool,
}

impl ProjectTable {
    pub fn new(projects: Vec<Project>) -> Self {
        let mut state = TableState::default();
        if !projects.is_empty() {
            state.select(Some(0));
        }

        Self {
            projects,
            state,
            sort_column: SortColumn::Stars,
            sort_ascending: false,
        }
    }

    pub fn set_projects(&mut self, projects: Vec<Project>) {
        self.projects = projects;
        self.sort_projects();

        // Preserve selection if possible
        let selected = self.state.selected().unwrap_or(0);
        if selected >= self.projects.len() && !self.projects.is_empty() {
            self.state.select(Some(self.projects.len() - 1));
        } else if self.projects.is_empty() {
            self.state.select(None);
        }
    }

    pub fn sort_by(&mut self, column: SortColumn) {
        if self.sort_column == column {
            self.sort_ascending = !self.sort_ascending;
        } else {
            self.sort_column = column;
            self.sort_ascending = false;
        }
        self.sort_projects();
    }

    fn sort_projects(&mut self) {
        let asc = self.sort_ascending;

        self.projects.sort_by(|a, b| {
            let cmp = match self.sort_column {
                SortColumn::Status => {
                    let status_order = |s: ProjectStatus| match s {
                        ProjectStatus::Active => 0,
                        ProjectStatus::Stale => 1,
                        ProjectStatus::Unknown => 2,
                        ProjectStatus::Abandoned => 3,
                        ProjectStatus::Archived => 4,
                    };
                    status_order(a.status()).cmp(&status_order(b.status()))
                }
                SortColumn::Name => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
                SortColumn::Stars => b.stars.unwrap_or(0).cmp(&a.stars.unwrap_or(0)),
                SortColumn::Activity => {
                    // More recent = "greater"
                    b.last_commit.cmp(&a.last_commit)
                }
                SortColumn::Category => a.category.to_lowercase().cmp(&b.category.to_lowercase()),
            };

            if asc {
                cmp.reverse()
            } else {
                cmp
            }
        });
    }

    pub fn selected_project(&self) -> Option<&Project> {
        self.state.selected().and_then(|i| self.projects.get(i))
    }

    pub fn next(&mut self) {
        if self.projects.is_empty() {
            return;
        }
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.projects.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    pub fn previous(&mut self) {
        if self.projects.is_empty() {
            return;
        }
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.projects.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    pub fn page_down(&mut self, page_size: usize) {
        if self.projects.is_empty() {
            return;
        }
        let i = self.state.selected().unwrap_or(0);
        let new_i = (i + page_size).min(self.projects.len() - 1);
        self.state.select(Some(new_i));
    }

    pub fn page_up(&mut self, page_size: usize) {
        if self.projects.is_empty() {
            return;
        }
        let i = self.state.selected().unwrap_or(0);
        let new_i = i.saturating_sub(page_size);
        self.state.select(Some(new_i));
    }

    pub fn first(&mut self) {
        if !self.projects.is_empty() {
            self.state.select(Some(0));
        }
    }

    pub fn last(&mut self) {
        if !self.projects.is_empty() {
            self.state.select(Some(self.projects.len() - 1));
        }
    }

    pub fn render(&mut self, frame: &mut Frame, area: Rect) {
        let header_cells = [
            ("St", SortColumn::Status, 4),
            ("Name", SortColumn::Name, 0), // 0 = fill
            ("★", SortColumn::Stars, 8),
            ("Activity", SortColumn::Activity, 12),
            ("Category", SortColumn::Category, 15),
        ]
        .iter()
        .map(|(name, col, _)| {
            let is_sorted = self.sort_column == *col;
            let arrow = if is_sorted {
                if self.sort_ascending {
                    "↑"
                } else {
                    "↓"
                }
            } else {
                ""
            };
            let style = if is_sorted {
                Theme::header().add_modifier(Modifier::UNDERLINED)
            } else {
                Theme::header()
            };
            Cell::from(format!("{}{}", name, arrow)).style(style)
        });

        let header = Row::new(header_cells).style(Theme::header()).height(1);

        let rows = self.projects.iter().map(|project| {
            let status = project.status();
            let status_style = match status {
                ProjectStatus::Active => Theme::status_active(),
                ProjectStatus::Stale => Theme::status_stale(),
                ProjectStatus::Abandoned => Theme::status_abandoned(),
                ProjectStatus::Archived => Theme::status_archived(),
                ProjectStatus::Unknown => Theme::status_unknown(),
            };

            // Name with description in gray
            let name_with_desc = if project.description.is_empty() {
                Line::from(Span::styled(project.name.clone(), Theme::normal()))
            } else {
                let desc = clean_display_string(&project.description);
                let truncated_desc = if desc.chars().count() > 60 {
                    format!("{}...", desc.chars().take(57).collect::<String>())
                } else {
                    desc
                };
                Line::from(vec![
                    Span::styled(project.name.clone(), Theme::normal()),
                    Span::styled(" - ", Theme::dim()),
                    Span::styled(truncated_desc, Theme::dim()),
                ])
            };

            let cells = vec![
                Cell::from(status.icon()).style(status_style),
                Cell::from(name_with_desc),
                Cell::from(project.stars_display()).style(Theme::normal()),
                Cell::from(project.activity_display()).style(status_style),
                Cell::from(truncate_str(&project.category, 14)).style(Theme::dim()),
            ];

            Row::new(cells).height(1)
        });

        let widths = [
            Constraint::Length(4),
            Constraint::Fill(1),
            Constraint::Length(8),
            Constraint::Length(12),
            Constraint::Length(15),
        ];

        let table = Table::new(rows, widths)
            .header(header)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Theme::border())
                    .title(Span::styled(
                        format!(" Projects ({}) ", self.projects.len()),
                        Theme::title(),
                    )),
            )
            .row_highlight_style(Theme::selected());

        frame.render_stateful_widget(table, area, &mut self.state);
    }
}

fn truncate_str(s: &str, max_len: usize) -> String {
    // Clean any HTML tags that might have slipped through
    let clean = clean_display_string(s);
    if clean.chars().count() <= max_len {
        clean
    } else {
        format!("{}...", clean.chars().take(max_len - 3).collect::<String>())
    }
}

fn clean_display_string(s: &str) -> String {
    // Remove any HTML tags like <a name="..."> or </a>
    let mut result = s.to_string();

    // Simple HTML tag removal
    while let Some(start) = result.find('<') {
        if let Some(end) = result[start..].find('>') {
            result = format!("{}{}", &result[..start], &result[start + end + 1..]);
        } else {
            break;
        }
    }

    // Clean up multiple spaces
    while result.contains("  ") {
        result = result.replace("  ", " ");
    }

    result.trim().to_string()
}
