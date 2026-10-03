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
    execute!(stdout, EnterAlternateScreen)?;
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

    // Try to start LSP client in background
    let lsp_client = match blade_lsp::LspClient::new("rust-analyzer").await {
        Ok(client) => Some(std::sync::Arc::new(client)),
        Err(e) => {
            app.status_message = Some((format!("LSP start failed: {}", e), std::time::Instant::now()));
            None
        }
    };

    if let Some(ref lsp) = lsp_client {
        // Initialize it
        if let Ok(_) = lsp.initialize(None).await {
            app.status_message = Some(("LSP connected: rust-analyzer".to_string(), std::time::Instant::now()));
        }
    }

    // Main event loop
    loop {
        // Render
        terminal.draw(|frame| {
            blade_ui::render(frame, &mut app);
        })?;

        // Handle input
        if event::poll(Duration::from_millis(16))? {
            if let Event::Key(key) = event::read()? {
                blade_ui::handle_key(key, &mut app);

                // Auto-trigger completion on typing (very basic demo)
                if let crossterm::event::KeyCode::Char(c) = key.code {
                    if c.is_alphabetic() || c == '.' || c == ':' {
                        if let Some(ref lsp) = lsp_client {
                            if let Some(doc) = app.documents.get(app.active_doc) {
                                // In a real app we'd trigger an async task here and update state when it returns.
                                // For now we'll just mock the completion popup manually if missing, to show the UI
                                if app.autocomplete.is_none() {
                                    app.autocomplete = Some(blade_ui::autocomplete::AutocompleteState::new(
                                        vec![
                                            blade_ui::autocomplete::CompletionItem { label: "print!".to_string(), detail: Some("macro".to_string()) },
                                            blade_ui::autocomplete::CompletionItem { label: "println!".to_string(), detail: Some("macro".to_string()) },
                                            blade_ui::autocomplete::CompletionItem { label: "String".to_string(), detail: Some("struct".to_string()) },
                                            blade_ui::autocomplete::CompletionItem { label: "Vec".to_string(), detail: Some("struct".to_string()) },
                                        ],
                                        (app.explorer.selected as u16 * 0 + 30, 5) // Mock position
                                    ));
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
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    Ok(())
}
