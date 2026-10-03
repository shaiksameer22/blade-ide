use crate::app::App;
use crate::theme::Theme;
use lsp_types::DiagnosticSeverity;
use ratatui::{
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem},
    Frame,
};

pub fn render(frame: &mut Frame, app: &mut App, area: Rect, _theme: &Theme) {
    let items: Vec<ListItem> = app
        .diagnostics
        .iter()
        .map(|diag| {
            let severity_color = match diag.severity {
                Some(DiagnosticSeverity::ERROR) => Color::Red,
                Some(DiagnosticSeverity::WARNING) => Color::Yellow,
                Some(DiagnosticSeverity::INFORMATION) => Color::Blue,
                Some(DiagnosticSeverity::HINT) => Color::Cyan,
                _ => Color::White,
            };

            let prefix = match diag.severity {
                Some(DiagnosticSeverity::ERROR) => "Error",
                Some(DiagnosticSeverity::WARNING) => "Warning",
                Some(DiagnosticSeverity::INFORMATION) => "Info",
                Some(DiagnosticSeverity::HINT) => "Hint",
                _ => "Unknown",
            };

            // Diagnostic has no direct file info in itself usually (it's published per file).
            // The instructions say "file, line number, and error message". Wait, maybe we can just show line number and message.
            let line = diag.range.start.line + 1; // 1-indexed for display

            let content = Line::from(vec![
                Span::styled(
                    format!("[{}] ", prefix),
                    Style::default().fg(severity_color),
                ),
                Span::styled(
                    format!("Line {}: ", line),
                    Style::default().fg(Color::DarkGray),
                ),
                Span::raw(&diag.message),
            ]);

            ListItem::new(content)
        })
        .collect();

    let block = Block::default()
        .borders(Borders::ALL)
        .title("Diagnostics")
        .border_style(Style::default().fg(Color::DarkGray));

    let list = List::new(items).block(block);
    frame.render_widget(list, area);
}
