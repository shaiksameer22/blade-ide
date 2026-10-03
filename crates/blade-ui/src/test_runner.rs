use ratatui::{
    layout::Rect,
    style::{Color, Style, Stylize},
    widgets::{Block, Borders, List, ListItem},
    Frame,
};

use crate::app::{TestRunnerState, TestStatus};
use crate::theme::Theme;
use ansi_to_tui::IntoText;

pub fn render(
    frame: &mut Frame,
    state: &TestRunnerState,
    area: Rect,
    theme: &Theme,
    is_focused: bool,
) {
    let border_color = if is_focused {
        theme.statusbar_bg // active color
    } else {
        Color::DarkGray
    };

    let title = match state.status {
        TestStatus::Running => " TEST RUNNER [Running] ",
        TestStatus::Passed => " TEST RUNNER [Passed] ",
        TestStatus::Failed => " TEST RUNNER [Failed] ",
    };

    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_type(ratatui::widgets::BorderType::Rounded)
        .border_style(Style::default().fg(border_color))
        .bg(theme.editor_bg);

    let items: Vec<ListItem> = state
        .output
        .iter()
        .map(|line| {
            let parsed = line
                .into_text()
                .unwrap_or_else(|_| ratatui::text::Text::raw(line));
            ListItem::new(parsed)
        })
        .collect();

    let list = List::new(items).block(block);
    frame.render_widget(list, area);
}
