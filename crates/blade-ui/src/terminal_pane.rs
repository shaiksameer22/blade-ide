use blade_terminal::TerminalEmulator;
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::Span,
    widgets::{Block, Borders, Widget},
};

pub struct TerminalPane<'a> {
    emulator: &'a mut TerminalEmulator,
    is_focused: bool,
}

impl<'a> TerminalPane<'a> {
    pub fn new(emulator: &'a mut TerminalEmulator, is_focused: bool) -> Self {
        Self {
            emulator,
            is_focused,
        }
    }
}

impl<'a> Widget for TerminalPane<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let block = Block::default()
            .borders(Borders::ALL)
            .border_type(ratatui::widgets::BorderType::Rounded)
            .title(if self.is_focused {
                "Terminal (Focused)"
            } else {
                "Terminal"
            })
            .border_style(Style::default().fg(if self.is_focused {
                Color::Yellow
            } else {
                Color::White
            }));

        let inner_area = block.inner(area);
        block.render(area, buf);

        // Check if resize is needed
        {
            let parser = self.emulator.screen();
            let screen = parser.screen();
            let (rows, cols) = screen.size();
            if rows != inner_area.height || cols != inner_area.width {
                // Drop the lock before resizing
                drop(parser);
                let _ = self.emulator.resize(inner_area.width, inner_area.height);
            }
        }

        let parser = self.emulator.screen();
        let screen = parser.screen();
        let (rows, cols) = screen.size();

        for r in 0..rows.min(inner_area.height) {
            let mut line_spans = Vec::new();
            for c in 0..cols.min(inner_area.width) {
                let cell = screen.cell(r, c);
                if let Some(cell) = cell {
                    let fg = convert_color(cell.fgcolor());
                    let bg = convert_color(cell.bgcolor());

                    let mut style = Style::default().fg(fg).bg(bg);
                    if cell.bold() {
                        style = style.add_modifier(Modifier::BOLD);
                    }
                    if cell.italic() {
                        style = style.add_modifier(Modifier::ITALIC);
                    }
                    if cell.underline() {
                        style = style.add_modifier(Modifier::UNDERLINED);
                    }

                    line_spans.push(Span::styled(cell.contents().to_string(), style));
                }
            }

            let tui_line = ratatui::text::Line::from(line_spans);
            buf.set_line(inner_area.x, inner_area.y + r, &tui_line, inner_area.width);
        }

        if !screen.hide_cursor() {
            let (cr, cc) = screen.cursor_position();
            if cr < inner_area.height && cc < inner_area.width {
                if let Some(cell) = buf.cell_mut((inner_area.x + cc, inner_area.y + cr)) {
                    cell.set_style(cell.style().add_modifier(Modifier::REVERSED));
                }
            }
        }
    }
}

fn convert_color(color: vt100::Color) -> Color {
    match color {
        vt100::Color::Default => Color::Reset,
        vt100::Color::Idx(i) => Color::Indexed(i),
        vt100::Color::Rgb(r, g, b) => Color::Rgb(r, g, b),
    }
}
