/// Represents a cursor position in the document
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Position {
    pub line: usize,   // 0-indexed line number
    pub col: usize,    // 0-indexed column (character offset)
}

/// A selection range with anchor and head
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Selection {
    pub anchor: Position,  // Where selection started
    pub head: Position,    // Where cursor currently is
}

impl Selection {
    pub fn new(pos: Position) -> Self {
        Self { anchor: pos, head: pos }
    }

    pub fn is_empty(&self) -> bool {
        self.anchor == self.head
    }

    pub fn start(&self) -> Position {
        if self.anchor.line < self.head.line
            || (self.anchor.line == self.head.line && self.anchor.col <= self.head.col)
        {
            self.anchor
        } else {
            self.head
        }
    }

    pub fn end(&self) -> Position {
        if self.anchor.line < self.head.line
            || (self.anchor.line == self.head.line && self.anchor.col <= self.head.col)
        {
            self.head
        } else {
            self.anchor
        }
    }
}

/// Manages one or more cursors (multi-cursor support)
pub struct CursorManager {
    pub selections: Vec<Selection>,
    /// Desired column when moving vertically (sticky column)
    pub desired_col: Option<usize>,
}

impl CursorManager {
    pub fn new() -> Self {
        Self {
            selections: vec![Selection::new(Position { line: 0, col: 0 })],
            desired_col: None,
        }
    }

    /// Get the primary cursor position
    pub fn primary(&self) -> &Selection {
        self.selections.last().expect("must have at least one cursor")
    }

    pub fn primary_mut(&mut self) -> &mut Selection {
        self.selections.last_mut().expect("must have at least one cursor")
    }

    /// Set a single cursor position (clears multi-cursors)
    pub fn set_position(&mut self, pos: Position) {
        self.selections = vec![Selection::new(pos)];
        self.desired_col = None;
    }

    /// Add an additional cursor
    pub fn add_cursor(&mut self, pos: Position) {
        self.selections.push(Selection::new(pos));
    }

    /// Start or extend selection from the primary cursor
    pub fn select_to(&mut self, pos: Position) {
        if let Some(sel) = self.selections.last_mut() {
            sel.head = pos;
        }
    }
}

impl Default for CursorManager {
    fn default() -> Self {
        Self::new()
    }
}
