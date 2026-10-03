use crate::app::App;
use crate::theme::Theme;
use ratatui::{
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

pub fn render(frame: &mut Frame, app: &mut App, area: Rect, theme: &Theme) {
    let mut spans = Vec::new();

    for (i, doc) in app.documents.iter().enumerate() {
        let is_active = i == app.active_doc;
        let style = if is_active {
            Style::default()
                .bg(theme.tab_active_bg)
                .fg(theme.tab_active_fg)
        } else {
            Style::default()
                .bg(theme.tab_inactive_bg)
                .fg(theme.tab_inactive_fg)
        };

        let modified_indicator = if doc.is_modified() { " •" } else { "" };
        let title = format!(" {}{} ", doc.title(), modified_indicator);

        spans.push(Span::styled(title, style));
        spans.push(Span::styled(
            "│",
            Style::default().fg(theme.tab_inactive_fg),
        )); // separator
    }

    let p = Paragraph::new(Line::from(spans));
    frame.render_widget(p, area);
}
