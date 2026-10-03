use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::Style,
    widgets::{Block, Borders, Row, Table},
};
use crate::app::GithubState;

pub fn render(frame: &mut ratatui::Frame, state: &GithubState, area: ratatui::layout::Rect, theme: &crate::theme::Theme) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    // Pull Requests
    let pr_header = Row::new(vec!["Number", "State", "Title"]).style(Style::default().fg(theme.keyword));
    let pr_rows: Vec<Row> = state.prs.iter().map(|pr| {
        Row::new(vec![
            pr.number.to_string(),
            pr.state.clone(),
            pr.title.clone(),
        ]).style(Style::default().fg(theme.editor_fg))
    }).collect();

    let pr_table = Table::new(pr_rows, [
        Constraint::Length(10),
        Constraint::Length(10),
        Constraint::Min(20),
    ])
    .header(pr_header)
    .block(Block::default().title(" Pull Requests (GH) ").borders(Borders::ALL).border_type(ratatui::widgets::BorderType::Rounded).border_style(Style::default().fg(theme.statusbar_bg)));

    frame.render_widget(pr_table, chunks[0]);

    // Issues
    let issue_header = Row::new(vec!["Number", "State", "Title"]).style(Style::default().fg(theme.keyword));
    let issue_rows: Vec<Row> = state.issues.iter().map(|issue| {
        Row::new(vec![
            issue.number.to_string(),
            issue.state.clone(),
            issue.title.clone(),
        ]).style(Style::default().fg(theme.editor_fg))
    }).collect();

    let issue_table = Table::new(issue_rows, [
        Constraint::Length(10),
        Constraint::Length(10),
        Constraint::Min(20),
    ])
    .header(issue_header)
    .block(Block::default().title(" Issues (GH) ").borders(Borders::ALL).border_type(ratatui::widgets::BorderType::Rounded).border_style(Style::default().fg(theme.statusbar_bg)));

    frame.render_widget(issue_table, chunks[1]);
}
