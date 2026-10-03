use crate::theme::Theme;
use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;
use ignore::WalkBuilder;
use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph},
    Frame,
};

pub enum PaletteType {
    FileFinder,
    CommandPalette,
}

pub struct PaletteState {
    pub palette_type: PaletteType,
    pub query: String,
    pub selected: usize,
    pub all_items: Vec<String>,
    pub matches: Vec<(String, i64, Vec<usize>)>, // text, score, matched indices
    pub matcher: SkimMatcherV2,
}

impl PaletteState {
    pub fn new_file_finder(cwd: &std::path::Path) -> Self {
        let mut all_items = Vec::new();

        let walker = WalkBuilder::new(cwd).hidden(true).git_ignore(true).build();

        for result in walker {
            if let Ok(entry) = result {
                if entry.file_type().map_or(false, |ft| ft.is_file()) {
                    if let Ok(path) = entry.path().strip_prefix(cwd) {
                        all_items.push(path.to_string_lossy().to_string());
                    }
                }
            }
        }

        Self {
            palette_type: PaletteType::FileFinder,
            query: String::new(),
            selected: 0,
            all_items,
            matches: Vec::new(),
            matcher: SkimMatcherV2::default(),
        }
    }

    pub fn new_command_palette() -> Self {
        let all_items = vec![
            "File: Save".to_string(),
            "File: Open".to_string(),
            "Editor: Toggle Explorer".to_string(),
            "Editor: Next Tab".to_string(),
            "Editor: Previous Tab".to_string(),
            "GitHub: View PRs & Issues".to_string(),
            "Test: Run All Tests".to_string(),
            "App: Quit".to_string(),
        ];

        let mut state = Self {
            palette_type: PaletteType::CommandPalette,
            query: String::new(),
            selected: 0,
            all_items,
            matches: Vec::new(),
            matcher: SkimMatcherV2::default(),
        };
        state.update_matches();
        state
    }

    pub fn update_matches(&mut self) {
        if self.query.is_empty() && matches!(self.palette_type, PaletteType::CommandPalette) {
            self.matches = self
                .all_items
                .iter()
                .map(|i| (i.clone(), 0, Vec::new()))
                .collect();
            self.selected = 0;
            return;
        }

        let mut matches = Vec::new();
        for item in &self.all_items {
            if let Some((score, indices)) = self.matcher.fuzzy_indices(item, &self.query) {
                matches.push((item.clone(), score, indices));
            }
        }

        matches.sort_by(|a, b| b.1.cmp(&a.1));
        self.matches = matches;
        self.selected = 0;
    }

    pub fn move_up(&mut self) {
        if self.selected > 0 {
            self.selected -= 1;
        }
    }

    pub fn move_down(&mut self) {
        if self.selected + 1 < self.matches.len() {
            self.selected += 1;
        }
    }
}

pub fn render(frame: &mut Frame, state: &PaletteState, theme: &Theme) {
    let area = frame.area();

    // Create a centered rect
    let popup_area = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(20),
            Constraint::Length(15),
            Constraint::Percentage(80),
        ])
        .split(area)[1];

    let popup_area = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(20),
            Constraint::Percentage(60),
            Constraint::Percentage(20),
        ])
        .split(popup_area)[1];

    frame.render_widget(Clear, popup_area);

    let title = match state.palette_type {
        PaletteType::FileFinder => "Search Files (Ctrl+P)",
        PaletteType::CommandPalette => "Command Palette (Ctrl+Shift+P)",
    };

    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.statusbar_bg))
        .bg(theme.editor_bg);

    frame.render_widget(block.clone(), popup_area);
    let inner_area = block.inner(popup_area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // input
            Constraint::Length(1), // separator
            Constraint::Min(0),    // list
        ])
        .split(inner_area);

    let input_text = format!("> {}", state.query);
    let p = Paragraph::new(input_text).style(Style::default().fg(theme.editor_fg));
    frame.render_widget(p, chunks[0]);

    // Separator
    let sep = Paragraph::new("─".repeat(chunks[1].width as usize))
        .style(Style::default().fg(Color::DarkGray));
    frame.render_widget(sep, chunks[1]);

    // Render list
    let mut list_items = Vec::new();
    for (i, (text, _score, indices)) in state.matches.iter().enumerate() {
        let mut spans = Vec::new();

        let mut char_idx = 0;
        let mut matched_idx_pos = 0;

        for (_byte_idx, c) in text.char_indices() {
            let is_matched =
                if matched_idx_pos < indices.len() && indices[matched_idx_pos] == char_idx {
                    matched_idx_pos += 1;
                    true
                } else {
                    false
                };

            if is_matched {
                spans.push(Span::styled(
                    c.to_string(),
                    Style::default().fg(theme.keyword),
                ));
            } else {
                spans.push(Span::raw(c.to_string()));
            }
            char_idx += 1;
        }

        let style = if i == state.selected {
            Style::default().bg(theme.selection).fg(theme.editor_fg)
        } else {
            Style::default().fg(theme.editor_fg)
        };

        list_items.push(ListItem::new(Line::from(spans)).style(style));
    }

    let list = List::new(list_items);
    frame.render_widget(list, chunks[2]);
}
