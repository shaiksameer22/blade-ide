use crate::app::App;
use crate::theme::Theme;
use ratatui::{
    layout::Rect,
    style::{Style, Stylize},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
    Frame,
};

pub fn render(frame: &mut Frame, app: &mut App, area: Rect, theme: &Theme) {
    if let Some(code) = &app.peek_definition {
        let width = (area.width * 3) / 4;
        let height = (area.height * 3) / 4;

        let x = area.x + (area.width.saturating_sub(width)) / 2;
        let y = area.y + (area.height.saturating_sub(height)) / 2;

        let rect = Rect::new(x, y, width, height);

        let block = Block::default()
            .title(" Peek Definition (Esc to close) ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme.selection))
            .bg(theme.editor_bg);

        let paragraph = Paragraph::new(code.as_str())
            .block(block)
            .wrap(Wrap { trim: false })
            .style(Style::default().fg(theme.editor_fg).bg(theme.editor_bg));

        frame.render_widget(Clear, rect);
        frame.render_widget(paragraph, rect);
    }
}
