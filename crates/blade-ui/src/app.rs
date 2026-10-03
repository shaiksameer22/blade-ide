use crate::explorer::FileExplorer;
// use crate::command::CommandPalette;
use blade_core::document::Document;
use std::path::PathBuf;

/// Main application state
pub enum Focus {
    Editor,
    Explorer,
    Palette,
    Terminal,
    Github,
    TestRunner,
}

#[derive(Default)]
pub struct GithubState {
    pub prs: Vec<crate::github::PullRequest>,
    pub issues: Vec<crate::github::Issue>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum TestStatus {
    Running,
    Passed,
    Failed,
}

pub struct TestRunnerState {
    pub output: Vec<String>,
    pub status: TestStatus,
}

impl Default for TestRunnerState {
    fn default() -> Self {
        Self {
            output: Vec::new(),
            status: TestStatus::Running,
        }
    }
}

pub struct App {
    pub documents: Vec<Document>,
    pub active_doc: usize,
    pub explorer: FileExplorer,
    pub show_explorer: bool,
    pub focus: Focus,
    pub palette: Option<crate::palette::PaletteState>,
    pub find_state: Option<FindState>,
    pub should_quit: bool,
    pub status_message: Option<(String, std::time::Instant)>,
    pub cwd: PathBuf,
    pub dialog: Option<Dialog>,
    pub autocomplete: Option<crate::autocomplete::AutocompleteState>,
    pub terminal: Option<blade_terminal::TerminalEmulator>,
    pub show_terminal: bool,
    pub github_state: Option<GithubState>,
    pub github_rx: Option<std::sync::mpsc::Receiver<GithubState>>,
    pub test_runner: Option<TestRunnerState>,
    pub test_rx: Option<std::sync::mpsc::Receiver<String>>,
    pub show_test_runner: bool,
    pub is_recording_macro: bool,
    pub macro_events: Vec<crossterm::event::KeyEvent>,
}

pub struct FindState {
    pub query: String,
    pub case_sensitive: bool,
    pub regex: bool,
    pub replace: Option<String>,
    pub matches: Vec<(usize, usize)>, // (line, col) pairs
    pub current_match: usize,
}

pub enum Dialog {
    SaveConfirm { doc_index: usize },
    QuitConfirm,
}

impl App {
    pub fn new(cwd: PathBuf) -> Self {
        Self {
            documents: vec![Document::new()],
            active_doc: 0,
            explorer: FileExplorer::new(&cwd),
            show_explorer: true,
            focus: Focus::Editor,
            palette: None,
            find_state: None,
            should_quit: false,
            status_message: None,
            cwd,
            dialog: None,
            autocomplete: None,
            terminal: None,
            show_terminal: false,
            github_state: None,
            github_rx: None,
            test_runner: None,
            test_rx: None,
            show_test_runner: false,
            is_recording_macro: false,
            macro_events: Vec::new(),
        }
    }

    pub fn active_document(&self) -> &Document {
        &self.documents[self.active_doc]
    }

    pub fn active_document_mut(&mut self) -> &mut Document {
        &mut self.documents[self.active_doc]
    }

    pub fn open_file(&mut self, path: &std::path::Path) -> anyhow::Result<()> {
        // Check if already open
        for (i, doc) in self.documents.iter().enumerate() {
            if doc.buffer.path() == Some(&path.to_path_buf()) {
                self.active_doc = i;
                return Ok(());
            }
        }

        let buffer = blade_core::buffer::Buffer::from_file(path)?;
        let mut doc = Document::from_buffer(buffer);

        // Setup tree-sitter based on language
        if let Some(lang_name) = blade_syntax::languages::detect_language(path) {
            if let Some((language, query)) = blade_syntax::languages::get_language(lang_name) {
                if !query.is_empty() {
                    let _ = doc.highlighter.set_language(language, lang_name, query);
                    doc.highlighter.parse(doc.buffer.text(), doc.huge_file);
                }
            }
        }

        self.documents.push(doc);
        self.active_doc = self.documents.len() - 1;
        Ok(())
    }

    pub fn close_document(&mut self, index: usize) -> bool {
        if self.documents[index].is_modified() {
            self.dialog = Some(Dialog::SaveConfirm { doc_index: index });
            return false;
        }
        self.documents.remove(index);
        if self.documents.is_empty() {
            self.documents.push(Document::new());
            self.active_doc = 0;
        } else if self.active_doc >= self.documents.len() {
            self.active_doc = self.documents.len() - 1;
        }
        true
    }

    pub fn set_status(&mut self, msg: impl Into<String>) {
        self.status_message = Some((msg.into(), std::time::Instant::now()));
    }
}
