use anyhow::{Result, Context};
use portable_pty::{CommandBuilder, NativePtySystem, PtySize, PtySystem, MasterPty, Child};
use std::io::{Read, Write};
use std::sync::{Arc, Mutex, MutexGuard};
use vt100::Parser;

pub struct TerminalEmulator {
    parser: Arc<Mutex<Parser>>,
    writer: Box<dyn Write + Send>,
    master: Box<dyn MasterPty + Send>,
    _child: Box<dyn Child + Send + Sync>,
}

impl TerminalEmulator {
    pub fn new(cols: u16, rows: u16) -> Result<Self> {
        let parser = Arc::new(Mutex::new(Parser::new(rows, cols, 0)));
        
        let pty_system = NativePtySystem::default();
        let pair = pty_system.openpty(PtySize {
            rows,
            cols,
            pixel_width: 0,
            pixel_height: 0,
        }).context("Failed to open PTY")?;

        let shell = std::env::var("SHELL").unwrap_or_else(|_| "bash".to_string());
        let cmd = CommandBuilder::new(shell);
        let child = pair.slave.spawn_command(cmd).context("Failed to spawn command")?;

        let mut reader = pair.master.try_clone_reader().context("Failed to clone reader")?;
        let writer = pair.master.take_writer().context("Failed to take writer")?;

        let parser_clone = Arc::clone(&parser);
        
        std::thread::spawn(move || {
            let mut buf = [0u8; 4096];
            loop {
                match reader.read(&mut buf) {
                    Ok(0) => break, // EOF
                    Ok(n) => {
                        if let Ok(mut parser_lock) = parser_clone.lock() {
                            parser_lock.process(&buf[..n]);
                        }
                    }
                    Err(_) => break, // Error or closed
                }
            }
        });

        Ok(TerminalEmulator {
            parser,
            writer,
            master: pair.master,
            _child: child,
        })
    }

    pub fn write(&mut self, data: &[u8]) -> std::io::Result<()> {
        self.writer.write_all(data)?;
        self.writer.flush()
    }

    pub fn screen(&self) -> MutexGuard<'_, vt100::Parser> {
        self.parser.lock().unwrap()
    }

    pub fn resize(&mut self, cols: u16, rows: u16) -> Result<()> {
        self.master.resize(PtySize {
            rows,
            cols,
            pixel_width: 0,
            pixel_height: 0,
        }).context("Failed to resize PTY")?;
        
        if let Ok(mut parser) = self.parser.lock() {
            parser.set_size(rows, cols);
        }
        
        Ok(())
    }
}
