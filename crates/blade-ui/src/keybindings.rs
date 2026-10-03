use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

/// All actions that can be triggered by keybindings
#[derive(Debug, Clone, PartialEq)]
pub enum Action {
    // File operations
    Save,
    SaveAs,
    NewFile,
    OpenFile,
    CloseTab,
    Quit,
    FileFinder,
    CommandPalette,

    // Editing
    Undo,
    Redo,
    Cut,
    Copy,
    Paste,
    SelectAll,
    DeleteLine,
    DuplicateLine,
    MoveLineUp,
    MoveLineDown,
    IndentLine,
    OutdentLine,
    ToggleComment,

    // Navigation
    MoveUp,
    MoveDown,
    MoveLeft,
    MoveRight,
    MoveWordLeft,
    MoveWordRight,
    MoveLineStart,
    MoveLineEnd,
    MoveDocStart,
    MoveDocEnd,
    PageUp,
    PageDown,
    GoToLine,

    // Selection
    SelectUp,
    SelectDown,
    SelectLeft,
    SelectRight,
    SelectWordLeft,
    SelectWordRight,
    SelectLineStart,
    SelectLineEnd,

    // Search
    Find,
    FindReplace,
    FindNext,
    FindPrev,

    // UI
    ToggleExplorer,
    NextTab,
    PrevTab,
    SplitVertical,
    SplitHorizontal,
    FocusNextPane,
    ToggleTerminal,

    // Text input
    InsertChar(char),
    InsertNewline,
    InsertTab,
    Backspace,
    Delete,

    // No-op
    None,
}

/// Map a key event to an action
pub fn map_key(event: KeyEvent) -> Action {
    let ctrl = event.modifiers.contains(KeyModifiers::CONTROL);
    let shift = event.modifiers.contains(KeyModifiers::SHIFT);
    let alt = event.modifiers.contains(KeyModifiers::ALT);

    match event.code {
        // === File Operations ===
        KeyCode::Char('s') if ctrl && shift => Action::SaveAs,
        KeyCode::Char('s') if ctrl => Action::Save,
        KeyCode::Char('n') if ctrl => Action::NewFile,
        KeyCode::Char('o') if ctrl => Action::OpenFile,
        KeyCode::Char('w') if ctrl => Action::CloseTab,
        KeyCode::Char('q') if ctrl => Action::Quit,
        KeyCode::Char('p') if ctrl && shift => Action::CommandPalette,
        KeyCode::Char('p') if ctrl => Action::FileFinder,

        // === Editing ===
        KeyCode::Char('z') if ctrl && shift => Action::Redo,
        KeyCode::Char('z') if ctrl => Action::Undo,
        KeyCode::Char('x') if ctrl => Action::Cut,
        KeyCode::Char('c') if ctrl => Action::Copy,
        KeyCode::Char('v') if ctrl => Action::Paste,
        KeyCode::Char('a') if ctrl => Action::SelectAll,
        KeyCode::Char('d') if ctrl && shift => Action::DuplicateLine,
        KeyCode::Char('k') if ctrl && shift => Action::DeleteLine,
        KeyCode::Char('/') if ctrl => Action::ToggleComment,

        // === Line operations (Alt) ===
        KeyCode::Up if alt => Action::MoveLineUp,
        KeyCode::Down if alt => Action::MoveLineDown,

        // === Selection (Shift + navigation) ===
        KeyCode::Up if shift => Action::SelectUp,
        KeyCode::Down if shift => Action::SelectDown,
        KeyCode::Left if shift && ctrl => Action::SelectWordLeft,
        KeyCode::Right if shift && ctrl => Action::SelectWordRight,
        KeyCode::Left if shift => Action::SelectLeft,
        KeyCode::Right if shift => Action::SelectRight,
        KeyCode::Home if shift => Action::SelectLineStart,
        KeyCode::End if shift => Action::SelectLineEnd,

        // === Navigation ===
        KeyCode::Up if ctrl => Action::MoveDocStart,
        KeyCode::Down if ctrl => Action::MoveDocEnd,
        KeyCode::Left if ctrl => Action::MoveWordLeft,
        KeyCode::Right if ctrl => Action::MoveWordRight,
        KeyCode::Home => Action::MoveLineStart,
        KeyCode::End => Action::MoveLineEnd,
        KeyCode::Up => Action::MoveUp,
        KeyCode::Down => Action::MoveDown,
        KeyCode::Left => Action::MoveLeft,
        KeyCode::Right => Action::MoveRight,
        KeyCode::PageUp => Action::PageUp,
        KeyCode::PageDown => Action::PageDown,
        KeyCode::Char('g') if ctrl => Action::GoToLine,

        // === Search ===
        KeyCode::Char('f') if ctrl => Action::Find,
        KeyCode::Char('h') if ctrl => Action::FindReplace,
        KeyCode::F(3) if shift => Action::FindPrev,
        KeyCode::F(3) => Action::FindNext,

        // === UI ===
        KeyCode::Char('b') if ctrl => Action::ToggleExplorer,
        KeyCode::Char('p') if ctrl && shift => Action::CommandPalette,
        KeyCode::Char('p') if ctrl => Action::FileFinder,
        KeyCode::Tab if ctrl => Action::NextTab,
        KeyCode::BackTab if ctrl => Action::PrevTab,
        KeyCode::Char('`') if ctrl => Action::ToggleTerminal,
        KeyCode::Char('\\') if ctrl => Action::SplitVertical,

        // === Text Input ===
        KeyCode::Char(c) => Action::InsertChar(c),
        KeyCode::Enter => Action::InsertNewline,
        KeyCode::Tab => Action::InsertTab,
        KeyCode::Backspace => Action::Backspace,
        KeyCode::Delete => Action::Delete,

        _ => Action::None,
    }
}
