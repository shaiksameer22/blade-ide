use ratatui::{
    layout::Rect,
    style::{Color, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};

use crate::app::{TestRunnerState, TestStatus};
use crate::theme::Theme;

pub fn render(frame: &mut Frame, state: &TestRunnerState, area: Rect, theme: &Theme, is_focused: bool) {
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
        .border_style(Style::default().fg(border_color))
        .bg(theme.editor_bg);

    let items: Vec<ListItem> = state
        .output
        .iter()
        .map(|line| {
            // Very simple color mapping based on common cargo test output
            let style = if line.contains("ok") || line.contains("passed") {
                Style::default().fg(Color::Green)
            } else if line.contains("FAILED") || line.contains("error") {
                Style::default().fg(Color::Red)
            } else if line.contains("running") || line.contains("test") {
                Style::default().fg(Color::Cyan)
            } else {
                Style::default().fg(theme.editor_fg)
            };
            ListItem::new(Line::from(vec![Span::styled(line, style)]))
        })
        .collect();

    let list = List::new(items).block(block);
    frame.render_widget(list, area);
}
