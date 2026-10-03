use crate::app::App;
use crate::theme::Theme;
use ratatui::{
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Paragraph},
    Frame,
};

pub fn render(frame: &mut Frame, app: &mut App, area: Rect, theme: &Theme) {
    let doc = app.active_document();
    let file_name = doc.title();

    let lang = doc
        .buffer
        .path()
        .and_then(|p| blade_syntax::languages::detect_language(p))
        .unwrap_or("Plain Text");

    let modified = if doc.is_modified() { "[+]" } else { "" };

    let pos = doc.cursors.primary().head;
    let pos_str = format!("Ln {}, Col {}", pos.line + 1, pos.col + 1);

    let left = format!(" {} {modified}", file_name);
    let right = format!("{} | {} ", lang, pos_str);

    // Naive padding
    let padding = area
        .width
        .saturating_sub(left.len() as u16 + right.len() as u16);
    let text = format!("{}{}{}", left, " ".repeat(padding as usize), right);

    let p = Paragraph::new(text).style(
        Style::default()
            .bg(theme.statusbar_bg)
            .fg(theme.statusbar_fg),
    );
    frame.render_widget(p, area);
}
