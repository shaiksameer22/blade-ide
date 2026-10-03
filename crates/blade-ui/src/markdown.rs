use crate::app::App;
use crate::theme::Theme;
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

pub fn render(frame: &mut Frame, app: &mut App, area: Rect, theme: &Theme) {
    let doc = app.active_document();
    let text = doc.buffer.text();

    let mut lines = Vec::new();

    for line in text.lines() {
        let line_str = line.to_string();
        if line_str.starts_with("# ") {
            lines.push(Line::from(vec![Span::styled(
                line_str,
                Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
            )]));
        } else if line_str.starts_with("## ") {
            lines.push(Line::from(vec![Span::styled(
                line_str,
                Style::default().fg(Color::LightCyan).add_modifier(Modifier::BOLD),
            )]));
        } else if line_str.starts_with("### ") {
            lines.push(Line::from(vec![Span::styled(
                line_str,
                Style::default().fg(Color::Blue).add_modifier(Modifier::BOLD),
            )]));
        } else {
            // Basic bold parsing for **text**
            let mut spans = Vec::new();
            let mut current = String::new();
            let mut is_bold = false;
            let mut chars = line_str.chars().peekable();
            
            while let Some(c) = chars.next() {
                if c == '*' && chars.peek() == Some(&'*') {
                    chars.next(); // consume second *
                    if !current.is_empty() {
                        spans.push(Span::styled(
                            current.clone(),
                            if is_bold {
                                Style::default().add_modifier(Modifier::BOLD)
                            } else {
                                Style::default().fg(theme.editor_fg)
                            },
                        ));
                        current.clear();
                    }
                    is_bold = !is_bold;
                } else {
                    current.push(c);
                }
            }
            if !current.is_empty() {
                spans.push(Span::styled(
                    current,
                    if is_bold {
                        Style::default().add_modifier(Modifier::BOLD)
                    } else {
                        Style::default().fg(theme.editor_fg)
                    },
                ));
            }
            lines.push(Line::from(spans));
        }
    }

    let p = Paragraph::new(lines).block(Block::default().borders(Borders::ALL).title("Markdown Preview"));
    frame.render_widget(p, area);
}
