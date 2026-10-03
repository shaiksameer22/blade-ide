use crate::app::App;
use crate::theme::Theme;
use ratatui::{
    layout::Rect,
    style::Style,
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

pub fn render(frame: &mut Frame, app: &mut App, area: Rect, theme: &Theme) {
    let height = area.height as usize;
    let width = area.width as usize;

    // Adjust scroll offsets to keep cursor in view
    {
        let doc = app.active_document_mut();
        let total_lines = doc.buffer.line_count();
        let line_num_width = format!("{}", total_lines).len();
        let text_width = width.saturating_sub(line_num_width + 2);

        let primary_cursor = doc.cursors.primary().head;

        if primary_cursor.line < doc.scroll_offset {
            doc.scroll_offset = primary_cursor.line;
        } else if primary_cursor.line >= doc.scroll_offset + height {
            doc.scroll_offset = primary_cursor.line.saturating_sub(height).saturating_add(1);
        }

        if primary_cursor.col < doc.h_scroll_offset {
            doc.h_scroll_offset = primary_cursor.col;
        } else if primary_cursor.col >= doc.h_scroll_offset + text_width {
            doc.h_scroll_offset = primary_cursor
                .col
                .saturating_sub(text_width)
                .saturating_add(1);
        }
    }

    let doc = app.active_document();
    let text_buf = doc.buffer.text();

    let scroll_y = doc.scroll_offset;
    let scroll_x = doc.h_scroll_offset;

    let total_lines = doc.buffer.line_count();
    let line_num_width = format!("{}", total_lines).len();

    let primary_cursor = doc.cursors.primary().head;

    let mut lines = Vec::new();
    let start_line = scroll_y;
    let end_line = (scroll_y + height).min(total_lines);

    let start_byte = text_buf.line_to_byte(start_line);
    let end_byte = text_buf.line_to_byte(end_line.min(total_lines));
    let highlights = if doc.highlighter.has_tree() {
        doc.highlighter
            .highlights(start_byte, end_byte, doc.buffer.text())
    } else {
        Vec::new()
    };

    let mut hl_idx = 0;

    for i in start_line..end_line {
        let line_slice = text_buf.line(i);
        let line_byte_start = text_buf.line_to_byte(i);

        let mut line_str = line_slice.to_string();
        if line_str.ends_with('\n') {
            line_str.pop();
        }
        if line_str.ends_with('\r') {
            line_str.pop();
        }

        // Apply horizontal scroll (simple version)
        let chars_count = line_str.chars().count();
        let display_chars = if scroll_x < chars_count {
            line_str
                .chars()
                .skip(scroll_x)
                .take(width.saturating_sub(line_num_width + 2))
        } else {
            "".chars().skip(1).take(0)
        };

        // Gutter (line numbers)
        let is_current_line = i == primary_cursor.line;
        let num_style = if is_current_line {
            Style::default().fg(theme.line_number_active)
        } else {
            Style::default().fg(theme.line_number)
        };

        let mut spans = Vec::new();
        spans.push(Span::styled(
            format!("{:>width$} ", i + 1, width = line_num_width),
            num_style,
        ));

        // Skip bytes for horizontal scrolling
        let mut current_byte = line_byte_start;
        for c in line_str.chars().take(scroll_x) {
            current_byte += c.len_utf8();
        }

        let mut current_hl_type = blade_syntax::highlighter::HighlightType::None;

        let mut current_span_text = String::new();

        for c in display_chars {
            // Advance hl_idx to the token containing current_byte
            while hl_idx < highlights.len() && highlights[hl_idx].end_byte <= current_byte {
                hl_idx += 1;
            }

            let new_hl_type =
                if hl_idx < highlights.len() && highlights[hl_idx].start_byte <= current_byte {
                    highlights[hl_idx].highlight_type
                } else {
                    blade_syntax::highlighter::HighlightType::None
                };

            if new_hl_type != current_hl_type && !current_span_text.is_empty() {
                // push span
                let color = theme.highlight_color(current_hl_type);
                spans.push(Span::styled(
                    current_span_text.clone(),
                    Style::default().fg(color),
                ));
                current_span_text.clear();
            }

            current_hl_type = new_hl_type;
            current_span_text.push(c);
            current_byte += c.len_utf8();
        }

        if !current_span_text.is_empty() {
            let color = theme.highlight_color(current_hl_type);
            spans.push(Span::styled(current_span_text, Style::default().fg(color)));
        }

        let line_style = if is_current_line {
            Style::default().bg(theme.current_line)
        } else {
            Style::default()
        };

        lines.push(Line::from(spans).style(line_style));
    }

    let p = Paragraph::new(lines).style(Style::default().bg(theme.editor_bg));
    frame.render_widget(p, area);

    // Render cursor
    let rel_y = primary_cursor.line.saturating_sub(scroll_y);
    if rel_y < height {
        // Find screen x position
        let line_slice = text_buf.line(primary_cursor.line);
        let line_str = line_slice.to_string();
        let chars_before = line_str.chars().take(primary_cursor.col).count();
        if chars_before >= scroll_x {
            let rel_x = line_num_width + 1 + chars_before - scroll_x;
            if rel_x < width {
                frame.set_cursor_position((area.x + rel_x as u16, area.y + rel_y as u16));
            }
        }
    }
}
