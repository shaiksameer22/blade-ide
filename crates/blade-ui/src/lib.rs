pub mod github;
pub mod github_pane;
pub mod app;
pub mod autocomplete;
pub mod editor;
pub mod explorer;
pub mod keybindings;
pub mod layout;
pub mod palette;
pub mod statusbar;
pub mod tabs;
pub mod terminal_pane;
pub mod test_runner;
pub mod theme;

use ratatui::{
    style::{Color, Style, Stylize},
    widgets::{Block, Borders, List, ListItem},
    Frame,
};

pub fn render(frame: &mut Frame, app: &mut app::App) {
    let theme = theme::Theme::dark();
    let area = frame.area();

    let layout = layout::calculate_layout(area, app.show_explorer, app.show_terminal, app.show_test_runner);

    if app.show_explorer {
        // Draw explorer background
        frame.render_widget(Block::default().bg(theme.sidebar_bg), layout.explorer);

        let mut items = Vec::new();
        for (i, entry) in app.explorer.entries_flat.iter().enumerate() {
            let indent = "  ".repeat(entry.depth);
            let prefix = if entry.is_dir {
                if entry.expanded {
                    "▼ "
                } else {
                    "▶ "
                }
            } else {
                "  "
            };
            let text = format!("{}{}{}", indent, prefix, entry.name);
            let style = if i == app.explorer.selected {
                if matches!(app.focus, app::Focus::Explorer) {
                    Style::default().bg(theme.selection).fg(theme.sidebar_fg)
                } else {
                    Style::default()
                        .bg(Color::Rgb(60, 60, 60))
                        .fg(theme.sidebar_fg) // Dimmed selection
                }
            } else {
                Style::default().fg(theme.sidebar_fg)
            };
            items.push(ListItem::new(text).style(style));
        }

        let border_color = if matches!(app.focus, app::Focus::Explorer) {
            theme.statusbar_bg // active color
        } else {
            Color::DarkGray
        };

        let list = List::new(items).block(
            Block::default()
                .title(" EXPLORER ")
                .borders(Borders::RIGHT)
                .border_style(Style::default().fg(border_color)),
        );
        frame.render_widget(list, layout.explorer);
    }

    tabs::render(frame, app, layout.tab_bar, &theme);
    editor::render(frame, app, layout.editor_area, &theme);

    if app.show_terminal {
        if let Some(term) = &mut app.terminal {
            let is_focused = matches!(app.focus, app::Focus::Terminal);
            let pane = terminal_pane::TerminalPane::new(term, is_focused);
            frame.render_widget(pane, layout.terminal_area);
        }
    }

    if app.show_test_runner {
        if let Some(state) = &app.test_runner {
            let is_focused = matches!(app.focus, app::Focus::TestRunner);
            test_runner::render(frame, state, layout.test_runner_area, &theme, is_focused);
        }
    }

    statusbar::render(frame, app, layout.status_bar, &theme);

    if let Some(rx) = &app.github_rx {
        if let Ok(state) = rx.try_recv() {
            app.github_state = Some(state);
            app.github_rx = None;
        }
    }

    if let Some(rx) = &app.test_rx {
        while let Ok(line) = rx.try_recv() {
            if let Some(state) = &mut app.test_runner {
                if line == "__TEST_PASSED__" {
                    state.status = app::TestStatus::Passed;
                } else if line == "__TEST_FAILED__" {
                    state.status = app::TestStatus::Failed;
                } else {
                    state.output.push(line);
                }
            }
        }
    }

    if matches!(app.focus, app::Focus::Github) {
        if let Some(state) = &app.github_state {
            github_pane::render(frame, state, layout.editor_area, &theme);
        } else {
            // Loading state
            let p = ratatui::widgets::Paragraph::new("Loading GitHub PRs and Issues...")
                .alignment(ratatui::layout::Alignment::Center)
                .block(ratatui::widgets::Block::default().borders(ratatui::widgets::Borders::ALL));
            frame.render_widget(p, layout.editor_area);
        }
    }

    if let Some(palette_state) = &app.palette {
        palette::render(frame, palette_state, &theme);
    }

    if let Some(autocomplete_state) = &app.autocomplete {
        autocomplete::render(frame, autocomplete_state, &theme);
    }
}

pub fn handle_key(key: crossterm::event::KeyEvent, app: &mut app::App) {
    use crossterm::event::{KeyCode, KeyModifiers};

    if key.code == KeyCode::Char('q') && key.modifiers.contains(KeyModifiers::CONTROL) {
        app.is_recording_macro = !app.is_recording_macro;
        if app.is_recording_macro {
            app.macro_events.clear();
        }
        return;
    }

    if key.code == KeyCode::Char('e') && key.modifiers.contains(KeyModifiers::CONTROL) {
        if !app.is_recording_macro {
            let events = app.macro_events.clone();
            for e in events {
                handle_key(e, app);
            }
        }
        return;
    }

    if app.is_recording_macro {
        app.macro_events.push(key);
    }

    if key.code == KeyCode::Up && key.modifiers.contains(KeyModifiers::ALT) && matches!(app.focus, app::Focus::Editor) {
        let doc = app.active_document_mut();
        if let Some(tree) = &doc.highlighter.tree {
            let cursor = doc.cursors.primary();
            let anchor_char = doc.buffer.line_col_to_char(cursor.anchor.line, cursor.anchor.col);
            let head_char = doc.buffer.line_col_to_char(cursor.head.line, cursor.head.col);
            let start_char = anchor_char.min(head_char);
            let end_char = anchor_char.max(head_char);
            
            let start_byte = doc.buffer.text().char_to_byte(start_char);
            let end_byte = doc.buffer.text().char_to_byte(end_char);
            
            if let Some(range) = blade_syntax::expand_selection(tree, start_byte, end_byte) {
                let new_start_char = doc.buffer.text().byte_to_char(range.start);
                let new_end_char = doc.buffer.text().byte_to_char(range.end);
                
                let (anchor_line, anchor_col) = doc.buffer.char_to_line_col(new_start_char);
                let (head_line, head_col) = doc.buffer.char_to_line_col(new_end_char);
                
                let sel = doc.cursors.primary_mut();
                sel.anchor = blade_core::cursor::Position { line: anchor_line, col: anchor_col };
                sel.head = blade_core::cursor::Position { line: head_line, col: head_col };
            }
        }
        return;
    }

    if let Some(palette_state) = &mut app.palette {
        match key.code {
            KeyCode::Esc => {
                app.palette = None;
                app.focus = app::Focus::Editor;
            }
            KeyCode::Up => {
                palette_state.move_up();
            }
            KeyCode::Down => {
                palette_state.move_down();
            }
            KeyCode::Enter => {
                if palette_state.selected < palette_state.matches.len() {
                    let text = palette_state.matches[palette_state.selected].0.clone();
                    match palette_state.palette_type {
                        palette::PaletteType::FileFinder => {
                            let path = app.cwd.join(text);
                            let _ = app.open_file(&path);
                        }
                        palette::PaletteType::CommandPalette => match text.as_str() {
                            "File: Save" => {
                                let doc = app.active_document_mut();
                                let _ = doc.buffer.save();
                            }
                            "Editor: Toggle Explorer" => {
                                app.show_explorer = !app.show_explorer;
                            }
                            "Editor: Next Tab" => {
                                if !app.documents.is_empty() {
                                    app.active_doc = (app.active_doc + 1) % app.documents.len();
                                }
                            }
                            "Editor: Previous Tab" => {
                                if !app.documents.is_empty() {
                                    if app.active_doc == 0 {
                                        app.active_doc = app.documents.len() - 1;
                                    } else {
                                        app.active_doc -= 1;
                                    }
                                }
                            }
                            "App: Quit" => {
                                app.should_quit = true;
                            }
                            "GitHub: View PRs & Issues" => {
                                app.focus = app::Focus::Github;
                                app.github_state = None;
                                let (tx, rx) = std::sync::mpsc::channel();
                                app.github_rx = Some(rx);
                                tokio::spawn(async move {
                                    let prs = crate::github::fetch_pull_requests().await.unwrap_or_default();
                                    let issues = crate::github::fetch_issues().await.unwrap_or_default();
                                    let _ = tx.send(crate::app::GithubState { prs, issues });
                                });
                            }
                            "Test: Run All Tests" => {
                                app.show_test_runner = true;
                                app.focus = app::Focus::TestRunner;
                                app.test_runner = Some(app::TestRunnerState::default());
                                let (tx, rx) = std::sync::mpsc::channel();
                                app.test_rx = Some(rx);

                                let cwd = app.cwd.clone();
                                tokio::spawn(async move {
                                    use std::process::Stdio;
                                    use tokio::io::{AsyncBufReadExt, BufReader};
                                    use tokio::process::Command;

                                    let child = Command::new("cargo")
                                        .arg("test")
                                        .arg("--color")
                                        .arg("always")
                                        .current_dir(cwd)
                                        .stdout(Stdio::piped())
                                        .stderr(Stdio::piped())
                                        .spawn();
                                    
                                    if let Ok(mut child) = child {
                                        if let Some(stdout) = child.stdout.take() {
                                            let mut reader = BufReader::new(stdout).lines();
                                            while let Ok(Some(line)) = reader.next_line().await {
                                                let _ = tx.send(line);
                                            }
                                        }
                                        if let Some(stderr) = child.stderr.take() {
                                            let mut reader = BufReader::new(stderr).lines();
                                            while let Ok(Some(line)) = reader.next_line().await {
                                                let _ = tx.send(line);
                                            }
                                        }
                                        let status = child.wait().await;
                                        if let Ok(status) = status {
                                            if status.success() {
                                                let _ = tx.send("__TEST_PASSED__".to_string());
                                            } else {
                                                let _ = tx.send("__TEST_FAILED__".to_string());
                                            }
                                        } else {
                                            let _ = tx.send("__TEST_FAILED__".to_string());
                                        }
                                    } else {
                                        let _ = tx.send("Failed to spawn cargo test".to_string());
                                        let _ = tx.send("__TEST_FAILED__".to_string());
                                    }
                                });
                            }
                            _ => {}
                        },
                    }
                }
                app.palette = None;
                if !matches!(app.focus, app::Focus::Github | app::Focus::TestRunner) {
                    app.focus = app::Focus::Editor;
                }
            }
            KeyCode::Backspace => {
                if !palette_state.query.is_empty() {
                    palette_state.query.pop();
                    palette_state.update_matches();
                }
            }
            KeyCode::Char(c) => {
                palette_state.query.push(c);
                palette_state.update_matches();
            }
            _ => {}
        }
        return;
    }

    if let Some(autocomplete_state) = &mut app.autocomplete {
        use crossterm::event::KeyCode;
        match key.code {
            KeyCode::Esc => {
                app.autocomplete = None;
                return;
            }
            KeyCode::Up => {
                autocomplete_state.previous();
                return;
            }
            KeyCode::Down => {
                autocomplete_state.next();
                return;
            }
            KeyCode::Enter | KeyCode::Tab => {
                if !autocomplete_state.items.is_empty() {
                    let item = &autocomplete_state.items[autocomplete_state.selected];
                    let text_to_insert = item.label.clone();

                    let doc = app.active_document_mut();
                    let pos = doc.cursors.primary().head;
                    let char_idx = doc.buffer.line_col_to_char(pos.line, pos.col);
                    doc.buffer.insert(char_idx, &text_to_insert);

                    let mut new_pos = pos;
                    new_pos.col += text_to_insert.chars().count();
                    doc.cursors.set_position(new_pos);
                }
                app.autocomplete = None;
                return;
            }
            _ => {
                // If they type, dismiss for now and let the default logic handle the key
                app.autocomplete = None;
            }
        }
    }

    if matches!(app.focus, app::Focus::Terminal) {
        let action = keybindings::map_key(key);
        // Allow global shortcuts like ToggleTerminal and FocusNextPane
        if action != keybindings::Action::ToggleTerminal && action != keybindings::Action::FocusNextPane {
            if let Some(term) = &mut app.terminal {
                use crossterm::event::{KeyCode, KeyModifiers};
                let mut buf = Vec::new();
                match key.code {
                    KeyCode::Char(c) => {
                        if key.modifiers.contains(KeyModifiers::CONTROL) {
                            let b = c as u8;
                            if b.is_ascii_lowercase() {
                                buf.push(b - b'a' + 1);
                            }
                        } else {
                            let mut b = [0; 4];
                            let s = c.encode_utf8(&mut b);
                            buf.extend_from_slice(s.as_bytes());
                        }
                    }
                    KeyCode::Enter => buf.push(b'\r'),
                    KeyCode::Backspace => buf.push(0x7f),
                    KeyCode::Esc => buf.push(0x1b),
                    KeyCode::Tab => buf.push(b'\t'),
                    KeyCode::Up => buf.extend_from_slice(b"\x1b[A"),
                    KeyCode::Down => buf.extend_from_slice(b"\x1b[B"),
                    KeyCode::Right => buf.extend_from_slice(b"\x1b[C"),
                    KeyCode::Left => buf.extend_from_slice(b"\x1b[D"),
                    _ => {}
                }
                if !buf.is_empty() {
                    let _ = term.write(&buf);
                }
            }
            return;
        }
    }

    let action = keybindings::map_key(key);

    // Global actions
    match action {
        keybindings::Action::Quit => {
            app.should_quit = true;
            return;
        }
        keybindings::Action::ToggleExplorer => {
            app.show_explorer = !app.show_explorer;
            if !app.show_explorer {
                app.focus = app::Focus::Editor;
            }
            return;
        }
        keybindings::Action::FocusNextPane => {
            app.focus = match app.focus {
                app::Focus::Editor => {
                    if app.show_explorer {
                        app::Focus::Explorer
                    } else if app.show_terminal {
                        app::Focus::Terminal
                    } else if app.show_test_runner {
                        app::Focus::TestRunner
                    } else {
                        app::Focus::Editor
                    }
                }
                app::Focus::Explorer => {
                    if app.show_terminal {
                        app::Focus::Terminal
                    } else if app.show_test_runner {
                        app::Focus::TestRunner
                    } else {
                        app::Focus::Editor
                    }
                }
                app::Focus::Terminal => {
                    if app.show_test_runner {
                        app::Focus::TestRunner
                    } else {
                        app::Focus::Editor
                    }
                }
                app::Focus::Palette => app::Focus::Editor,
                app::Focus::TestRunner => app::Focus::Editor,
                app::Focus::Github => app::Focus::Editor,
            };
            return;
        }
        keybindings::Action::NextTab => {
            if !app.documents.is_empty() {
                app.active_doc = (app.active_doc + 1) % app.documents.len();
            }
            return;
        }
        keybindings::Action::PrevTab => {
            if !app.documents.is_empty() {
                if app.active_doc == 0 {
                    app.active_doc = app.documents.len() - 1;
                } else {
                    app.active_doc -= 1;
                }
            }
            return;
        }
        keybindings::Action::Save => {
            let doc = app.active_document_mut();
            let _ = doc.buffer.save();
            return;
        }
        keybindings::Action::FileFinder => {
            app.palette = Some(palette::PaletteState::new_file_finder(&app.cwd));
            app.focus = app::Focus::Palette;
            return;
        }
        keybindings::Action::CommandPalette => {
            app.palette = Some(palette::PaletteState::new_command_palette());
            app.focus = app::Focus::Palette;
            return;
        }
        keybindings::Action::ToggleTerminal => {
            app.show_terminal = !app.show_terminal;
            if app.show_terminal {
                if app.terminal.is_none() {
                    app.terminal = blade_terminal::TerminalEmulator::new(80, 24).ok();
                }
                app.focus = app::Focus::Terminal;
            } else {
                if matches!(app.focus, app::Focus::Terminal) {
                    app.focus = app::Focus::Editor;
                }
            }
            return;
        }
        _ => {}
    }

    match app.focus {
        app::Focus::Editor => handle_editor_key(action, app),
        app::Focus::Explorer => handle_explorer_key(action, app),
        app::Focus::Palette => {}  // Handled early
        app::Focus::Terminal => {} // Handled early
        app::Focus::Github => {
            if let crossterm::event::KeyCode::Esc = key.code {
                app.focus = app::Focus::Editor;
            }
        }
        app::Focus::TestRunner => {
            if let crossterm::event::KeyCode::Esc = key.code {
                app.show_test_runner = false;
                app.focus = app::Focus::Editor;
            }
        }
    }
}

fn handle_explorer_key(action: keybindings::Action, app: &mut app::App) {
    match action {
        keybindings::Action::MoveUp => app.explorer.move_up(),
        keybindings::Action::MoveDown => app.explorer.move_down(),
        keybindings::Action::InsertNewline => {
            if let Some(entry) = app
                .explorer
                .entries_flat
                .get(app.explorer.selected)
                .cloned()
            {
                if entry.is_dir {
                    app.explorer.toggle_expand();
                } else {
                    // Open file
                    if app.open_file(&entry.path).is_ok() {
                        app.focus = app::Focus::Editor;
                    }
                }
            }
        }
        _ => {}
    }
}

fn handle_editor_key(action: keybindings::Action, app: &mut app::App) {
    match action {
        keybindings::Action::MoveUp => {
            let doc = app.active_document_mut();
            let mut pos = doc.cursors.primary().head;
            if pos.line > 0 {
                pos.line -= 1;
                let line_len = doc.buffer.line(pos.line).len_chars().saturating_sub(1);
                pos.col = pos.col.min(line_len);
                doc.cursors.set_position(pos);
            }
        }
        keybindings::Action::MoveDown => {
            let doc = app.active_document_mut();
            let mut pos = doc.cursors.primary().head;
            if pos.line + 1 < doc.buffer.line_count() {
                pos.line += 1;
                let line_len = doc.buffer.line(pos.line).len_chars().saturating_sub(1);
                pos.col = pos.col.min(line_len);
                doc.cursors.set_position(pos);
            }
        }
        keybindings::Action::MoveLeft => {
            let doc = app.active_document_mut();
            let mut pos = doc.cursors.primary().head;
            if pos.col > 0 {
                pos.col -= 1;
            } else if pos.line > 0 {
                pos.line -= 1;
                pos.col = doc.buffer.line(pos.line).len_chars().saturating_sub(1);
            }
            doc.cursors.set_position(pos);
        }
        keybindings::Action::MoveRight => {
            let doc = app.active_document_mut();
            let mut pos = doc.cursors.primary().head;
            let line_len = doc.buffer.line(pos.line).len_chars().saturating_sub(1);
            if pos.col < line_len {
                pos.col += 1;
            } else if pos.line + 1 < doc.buffer.line_count() {
                pos.line += 1;
                pos.col = 0;
            }
            doc.cursors.set_position(pos);
        }
        keybindings::Action::InsertChar(c) => {
            let doc = app.active_document_mut();
            let pos = doc.cursors.primary().head;
            let char_idx = doc.buffer.line_col_to_char(pos.line, pos.col);
            doc.buffer.insert(char_idx, &c.to_string());

            let mut new_pos = pos;
            new_pos.col += 1;
            doc.cursors.set_position(new_pos);
        }
        keybindings::Action::InsertNewline => {
            let doc = app.active_document_mut();
            let pos = doc.cursors.primary().head;
            let char_idx = doc.buffer.line_col_to_char(pos.line, pos.col);
            let nl = match doc.buffer.line_ending {
                blade_core::buffer::LineEnding::LF => "\n",
                blade_core::buffer::LineEnding::CRLF => "\r\n",
            };
            doc.buffer.insert(char_idx, nl);

            let mut new_pos = pos;
            new_pos.line += 1;
            new_pos.col = 0;
            doc.cursors.set_position(new_pos);
        }
        keybindings::Action::Backspace => {
            let doc = app.active_document_mut();
            let pos = doc.cursors.primary().head;
            if pos.col > 0 || pos.line > 0 {
                let char_idx = doc.buffer.line_col_to_char(pos.line, pos.col);
                if char_idx > 0 {
                    doc.buffer.remove(char_idx - 1, char_idx);

                    let (new_line, new_col) = doc.buffer.char_to_line_col(char_idx - 1);
                    doc.cursors.set_position(blade_core::cursor::Position {
                        line: new_line,
                        col: new_col,
                    });
                }
            }
        }
        _ => {}
    }
}
