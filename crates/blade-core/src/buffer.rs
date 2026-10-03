use ropey::Rope;
use std::path::PathBuf;
use std::fs;
use anyhow::Result;

/// A text buffer backed by a Rope data structure.
/// Provides O(log N) insertion/deletion and efficient line indexing.
pub struct Buffer {
    /// The rope holding the text content
    text: Rope,
    /// File path associated with this buffer (None for untitled)
    path: Option<PathBuf>,
    /// Whether the buffer has unsaved changes
    modified: bool,
    /// Line ending style detected from the file
    pub line_ending: LineEnding,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LineEnding {
    LF,    // \n (Unix)
    CRLF,  // \r\n (Windows)
}

impl Buffer {
    pub fn new() -> Self {
        Self {
            text: Rope::new(),
            path: None,
            modified: false,
            line_ending: LineEnding::LF,
        }
    }

    pub fn from_file(path: &std::path::Path) -> Result<Self> {
        let content = fs::read_to_string(path)?;
        let line_ending = if content.contains("\r\n") {
            LineEnding::CRLF
        } else {
            LineEnding::LF
        };
        Ok(Self {
            text: Rope::from_str(&content),
            path: Some(path.to_path_buf()),
            modified: false,
            line_ending,
        })
    }

    pub fn save(&mut self) -> Result<()> {
        if let Some(path) = &self.path {
            let content: String = self.text.to_string();
            fs::write(path, &content)?;
            self.modified = false;
        }
        Ok(())
    }

    pub fn save_as(&mut self, path: &std::path::Path) -> Result<()> {
        self.path = Some(path.to_path_buf());
        self.save()
    }

    /// Insert text at a character index
    pub fn insert(&mut self, char_idx: usize, text: &str) {
        self.text.insert(char_idx, text);
        self.modified = true;
    }

    /// Remove a range of characters
    pub fn remove(&mut self, start: usize, end: usize) {
        self.text.remove(start..end);
        self.modified = true;
    }

    /// Get the text of a specific line (0-indexed)
    pub fn line(&self, line_idx: usize) -> ropey::RopeSlice<'_> {
        self.text.line(line_idx)
    }

    /// Total number of lines
    pub fn line_count(&self) -> usize {
        self.text.len_lines()
    }

    /// Total character count
    pub fn char_count(&self) -> usize {
        self.text.len_chars()
    }

    /// Convert line/column to char index
    pub fn line_col_to_char(&self, line: usize, col: usize) -> usize {
        let line_start = self.text.line_to_char(line);
        let line_len = self.text.line(line).len_chars();
        line_start + col.min(line_len.saturating_sub(1)) // -1 for newline if present
    }

    /// Convert char index to line/column
    pub fn char_to_line_col(&self, char_idx: usize) -> (usize, usize) {
        let line = self.text.char_to_line(char_idx);
        let line_start = self.text.line_to_char(line);
        (line, char_idx - line_start)
    }

    pub fn text(&self) -> &Rope { &self.text }
    pub fn path(&self) -> Option<&PathBuf> { self.path.as_ref() }
    pub fn is_modified(&self) -> bool { self.modified }
    pub fn set_modified(&mut self, modified: bool) { self.modified = modified; }
}

impl Default for Buffer {
    fn default() -> Self {
        Self::new()
    }
}
