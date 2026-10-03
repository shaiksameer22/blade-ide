use crate::buffer::Buffer;
use crate::cursor::CursorManager;
use crate::history::History;
use blade_syntax::highlighter::SyntaxHighlighter;
use crate::crdt::SharedDocument;

/// A Document ties together a Buffer, Cursor state, and Edit history
pub struct Document {
    pub buffer: Buffer,
    pub cursors: CursorManager,
    pub history: History,
    pub highlighter: SyntaxHighlighter,
    pub huge_file: bool,
    /// Viewport scroll offset
    pub scroll_offset: usize,
    /// Horizontal scroll offset
    pub h_scroll_offset: usize,
    pub semantic_tokens: std::collections::HashMap<(usize, usize), lsp_types::SemanticTokenType>,
    pub crdt: Option<SharedDocument>,
}

impl Document {
    pub fn new() -> Self {
        Self {
            buffer: Buffer::new(),
            cursors: CursorManager::new(),
            history: History::new(1000),
            highlighter: SyntaxHighlighter::new(),
            huge_file: false,
            scroll_offset: 0,
            h_scroll_offset: 0,
            semantic_tokens: std::collections::HashMap::new(),
            crdt: None,
        }
    }

    pub fn from_buffer(buffer: Buffer) -> Self {
        let huge_file = buffer.text().len_bytes() > 5_000_000;
        Self {
            buffer,
            cursors: CursorManager::new(),
            history: History::new(1000),
            highlighter: SyntaxHighlighter::new(),
            huge_file,
            scroll_offset: 0,
            h_scroll_offset: 0,
            semantic_tokens: std::collections::HashMap::new(),
            crdt: None,
        }
    }


    /// Get the title for display (filename or "Untitled")
    pub fn title(&self) -> String {
        self.buffer
            .path()
            .and_then(|p| p.file_name())
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "Untitled".to_string())
    }

    /// Whether this document has unsaved modifications
    pub fn is_modified(&self) -> bool {
        self.buffer.is_modified()
    }
}

impl Default for Document {
    fn default() -> Self {
        Self::new()
    }
}
