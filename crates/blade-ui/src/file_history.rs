use crate::app::App;
use crate::theme::Theme;
use ratatui::{
    layout::{Alignment, Rect},
    style::Style,
    text::Line,
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph},
    Frame,
};

pub fn render(frame: &mut Frame, app: &mut App, theme: &Theme) {
    if let Some(history) = &app.file_history {
        let area = frame.area();
        // create a centered popup
        let width = 80.min(area.width.saturating_sub(4));
        let height = 20.min(area.height.saturating_sub(4));
        let x = (area.width - width) / 2;
        let y = (area.height - height) / 2;
        let popup_area = Rect::new(x, y, width, height);

        frame.render_widget(Clear, popup_area);

        let items: Vec<ListItem> = history
            .iter()
            .map(|s| ListItem::new(Line::from(s.clone())))
            .collect();

        let list = List::new(items)
            .block(
                Block::default()
                    .title(" File History ")
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(theme.selection)),
            )
            .highlight_style(Style::default().bg(theme.selection))
            .highlight_symbol("> ");

        let mut list_state = ratatui::widgets::ListState::default();
        list_state.select(Some(app.file_history_scroll));

        frame.render_stateful_widget(list, popup_area, &mut list_state);
    } else {
        // Loading state
        let area = frame.area();
        let p = Paragraph::new("Loading File History...")
            .alignment(Alignment::Center)
            .block(Block::default().title(" File History ").borders(Borders::ALL));
        
        let width = 40.min(area.width);
        let height = 3.min(area.height);
        let x = (area.width - width) / 2;
        let y = (area.height - height) / 2;
        let popup_area = Rect::new(x, y, width, height);
        
        frame.render_widget(Clear, popup_area);
        frame.render_widget(p, popup_area);
    }
}
