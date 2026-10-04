use anyhow::Result;
use clap::Parser;
use crossterm::{
    event::{self, Event},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::prelude::*;
use std::io;
use std::path::PathBuf;
use std::time::Duration;

/// Blade — A lightweight CLI IDE
#[derive(Parser)]
#[command(name = "blade", about = "A lightweight CLI IDE with VS Code-like features")]
struct Cli {
    /// File or directory to open
    #[arg(default_value = ".")]
    path: PathBuf,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Initialize terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(
        stdout, 
        EnterAlternateScreen, 
        crossterm::cursor::SetCursorStyle::SteadyBar
    )?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app state
    let cwd = if cli.path.is_dir() {
        cli.path.canonicalize()?
    } else {
        cli.path
            .parent()
            .unwrap_or(std::path::Path::new("."))
            .canonicalize()?
    };

    let mut app = blade_ui::app::App::new(cwd);

    // Open file if specified
    if cli.path.is_file() {
        app.open_file(&cli.path.canonicalize()?)?;
    }

    let lsp_client = match blade_lsp::LspClient::new("rust-analyzer").await {
        Ok(client) => Some(std::sync::Arc::new(client)),
        Err(e) => {
            app.status_message = Some((format!("LSP start failed: {}", e), std::time::Instant::now()));
            None
        }
    };

    let (completion_tx, mut completion_rx) = tokio::sync::mpsc::unbounded_channel();

    if let Some(ref lsp) = lsp_client {
        let lsp_clone = lsp.clone();
        tokio::spawn(async move {
            if lsp_clone.initialize(None).await.is_ok() {
                // Background initialization finished
            }
        });
    }

    // Main event loop
    loop {
        // Handle incoming completions
        while let Ok(state) = completion_rx.try_recv() {
            app.autocomplete = Some(state);
        }

        // Render
        let render_start = std::time::Instant::now();
        terminal.draw(|frame| {
            blade_ui::render(frame, &mut app);
        })?;
        app.last_render_time = render_start.elapsed();

        // Handle input
        if event::poll(Duration::from_millis(16))? {
            if let Event::Key(key) = event::read()? {
                blade_ui::handle_key(key, &mut app);

                // Auto-trigger completion on typing (very basic demo)
                if let crossterm::event::KeyCode::Char(c) = key.code {
                    if c.is_alphabetic() || c == '.' || c == ':' {
                        if let Some(ref lsp) = lsp_client {
                            if let Some(doc) = app.documents.get(app.active_doc) {
                                if let Some(path) = doc.buffer.path() {
                                    if let Ok(url) = url::Url::from_file_path(path) {
                                        if let Ok(uri) = url.as_str().parse::<lsp_types::Uri>() {
                                            let pos = doc.cursors.primary().head;
                                            let position = lsp_types::Position {
                                                line: pos.line as u32,
                                                character: pos.col as u32,
                                            };
                                            let params = lsp_types::CompletionParams {
                                                text_document_position: lsp_types::TextDocumentPositionParams {
                                                    text_document: lsp_types::TextDocumentIdentifier { uri },
                                                    position,
                                                },
                                                work_done_progress_params: Default::default(),
                                                partial_result_params: Default::default(),
                                                context: None,
                                            };
                                            let lsp_clone = lsp.clone();
                                            let tx_clone = completion_tx.clone();
                                            
                                            // Mock position for UI for now
                                            let popup_x = 30;
                                            let popup_y = 5;
                                            
                                            tokio::spawn(async move {
                                                if let Ok(resp) = lsp_clone.text_document_completion(params).await {
                                                    let items = match resp {
                                                        lsp_types::CompletionResponse::Array(arr) => arr,
                                                        lsp_types::CompletionResponse::List(list) => list.items,
                                                    };
                                                    
                                                    let mut completions = Vec::new();
                                                    for item in items {
                                                        completions.push(blade_ui::autocomplete::CompletionItem {
                                                            label: item.label,
                                                            detail: item.detail,
                                                        });
                                                    }
                                                    
                                                    if !completions.is_empty() {
                                                        let state = blade_ui::autocomplete::AutocompleteState::new(completions, (popup_x, popup_y));
                                                        let _ = tx_clone.send(state);
                                                    }
                                                }
                                            });
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        if app.should_quit {
            break;
        }
    }

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(), 
        LeaveAlternateScreen,
        crossterm::cursor::SetCursorStyle::DefaultUserShape
    )?;
    terminal.show_cursor()?;

    Ok(())
}
