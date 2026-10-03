use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use crate::theme::Theme;

pub struct MergeToolState {
    pub ours: String,
    pub theirs: String,
    pub result: String,
}

impl Default for MergeToolState {
    fn default() -> Self {
        Self::new()
    }
}

impl MergeToolState {
    pub fn new() -> Self {
        Self {
            ours: String::new(),
            theirs: String::new(),
            result: String::new(),
        }
    }

    pub fn parse(content: &str) -> Self {
        let mut ours = String::new();
        let mut theirs = String::new();
        let mut result = String::new();

        let mut state = 0; // 0: normal, 1: ours, 2: theirs

        for line in content.lines() {
            if line.starts_with("<<<<<<<") {
                state = 1;
                continue;
            } else if line.starts_with("=======") {
                state = 2;
                continue;
            } else if line.starts_with(">>>>>>>") {
                state = 0;
                continue;
            }

            match state {
                1 => {
                    ours.push_str(line);
                    ours.push('\n');
                }
                2 => {
                    theirs.push_str(line);
                    theirs.push('\n');
                }
                _ => {
                    ours.push_str(line);
                    ours.push('\n');
                    theirs.push_str(line);
                    theirs.push('\n');
                    result.push_str(line);
                    result.push('\n');
                }
            }
        }

        Self {
            ours,
            theirs,
            result,
        }
    }
}

pub fn render(frame: &mut Frame, state: &MergeToolState, area: Rect, theme: &Theme) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Ratio(1, 3),
            Constraint::Ratio(1, 3),
            Constraint::Ratio(1, 3),
        ])
        .split(area);

    let ours_block = Block::default()
        .title(" Ours ('a' to accept) ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.selection));
    
    let result_block = Block::default()
        .title(" Result ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray));

    let theirs_block = Block::default()
        .title(" Theirs ('t' to accept) ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.selection));

    let ours_p = Paragraph::new(state.ours.as_str()).block(ours_block);
    let result_p = Paragraph::new(state.result.as_str()).block(result_block);
    let theirs_p = Paragraph::new(state.theirs.as_str()).block(theirs_block);

    frame.render_widget(ours_p, chunks[0]);
    frame.render_widget(result_p, chunks[1]);
    frame.render_widget(theirs_p, chunks[2]);
}
