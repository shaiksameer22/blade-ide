use crate::cursor::Position;

/// Represents a single atomic edit operation
#[derive(Debug, Clone)]
pub enum EditOp {
    Insert {
        pos: usize,       // char index
        text: String,
    },
    Delete {
        pos: usize,       // char index
        text: String,      // deleted text (for redo)
    },
}

/// A group of operations that should be undone/redone together
#[derive(Debug, Clone)]
pub struct EditGroup {
    pub ops: Vec<EditOp>,
    pub cursor_before: Position,
    pub cursor_after: Position,
}

/// Undo/redo history manager
pub struct History {
    undo_stack: Vec<EditGroup>,
    redo_stack: Vec<EditGroup>,
    /// Maximum number of undo groups to keep
    max_history: usize,
}

impl History {
    pub fn new(max_history: usize) -> Self {
        Self {
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            max_history,
        }
    }

    pub fn push(&mut self, group: EditGroup) {
        self.undo_stack.push(group);
        self.redo_stack.clear(); // New edit invalidates redo
        if self.undo_stack.len() > self.max_history {
            self.undo_stack.remove(0);
        }
    }

    pub fn undo(&mut self) -> Option<EditGroup> {
        let group = self.undo_stack.pop()?;
        self.redo_stack.push(group.clone());
        Some(group)
    }

    pub fn redo(&mut self) -> Option<EditGroup> {
        let group = self.redo_stack.pop()?;
        self.undo_stack.push(group.clone());
        Some(group)
    }

    pub fn can_undo(&self) -> bool { !self.undo_stack.is_empty() }
    pub fn can_redo(&self) -> bool { !self.redo_stack.is_empty() }
}
