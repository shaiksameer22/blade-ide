use ratatui::{
    layout::Rect,
    style::{Color, Style, Stylize},
    widgets::{Block, Borders, Clear, List, ListItem},
    Frame,
};

#[derive(Clone, Debug)]
pub struct CompletionItem {
    pub label: String,
    pub detail: Option<String>,
}

#[derive(Clone, Debug)]
pub struct AutocompleteState {
    pub items: Vec<CompletionItem>,
    pub selected: usize,
    pub position: (u16, u16),
}

impl AutocompleteState {
    pub fn new(items: Vec<CompletionItem>, position: (u16, u16)) -> Self {
        Self {
            items,
            selected: 0,
            position,
        }
    }

    pub fn next(&mut self) {
        if !self.items.is_empty() {
            self.selected = (self.selected + 1) % self.items.len();
        }
    }

    pub fn previous(&mut self) {
        if !self.items.is_empty() {
            if self.selected == 0 {
                self.selected = self.items.len() - 1;
            } else {
                self.selected -= 1;
            }
        }
    }
}

pub fn render(frame: &mut Frame, state: &AutocompleteState, theme: &crate::theme::Theme) {
    if state.items.is_empty() {
        return;
    }

    let width = 40;
    let height = state.items.len().min(10) as u16 + 2;

    let x = state.position.0;
    let y = state.position.1 + 1;

    let area = Rect::new(
        x.min(frame.area().width.saturating_sub(width)),
        y.min(frame.area().height.saturating_sub(height)),
        width,
        height,
    );

    let items: Vec<ListItem> = state
        .items
        .iter()
        .enumerate()
        .map(|(i, item)| {
            let style = if i == state.selected {
                Style::default().bg(theme.selection).fg(theme.editor_fg)
            } else {
                Style::default().fg(theme.editor_fg)
            };

            let mut content = item.label.clone();
            if let Some(ref detail) = item.detail {
                content.push_str("  ");
                content.push_str(detail);
            }

            ListItem::new(content).style(style)
        })
        .collect();

    let list = List::new(items).block(Block::default().borders(Borders::ALL).bg(theme.editor_bg));

    frame.render_widget(Clear, area);
    frame.render_widget(list, area);
}
