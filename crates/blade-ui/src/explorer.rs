use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct FileEntry {
    pub path: PathBuf,
    pub name: String,
    pub is_dir: bool,
    pub depth: usize,
    pub expanded: bool,
    pub children: Vec<FileEntry>,
}

pub struct FileExplorer {
    pub root: FileEntry,
    pub selected: usize,
    pub entries_flat: Vec<FlatEntry>,
    pub scroll_offset: usize,
}

#[derive(Debug, Clone)]
pub struct FlatEntry {
    pub path: PathBuf,
    pub name: String,
    pub is_dir: bool,
    pub depth: usize,
    pub expanded: bool,
}

impl FileExplorer {
    pub fn new(root_path: &Path) -> Self {
        let root = Self::read_dir(root_path, 0);
        let mut explorer = Self {
            root,
            selected: 0,
            entries_flat: Vec::new(),
            scroll_offset: 0,
        };
        explorer.flatten();
        explorer
    }

    fn read_dir(path: &Path, depth: usize) -> FileEntry {
        let name = path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| path.display().to_string());

        let mut children = Vec::new();
        if path.is_dir() {
            if let Ok(entries) = fs::read_dir(path) {
                let mut items: Vec<_> = entries
                    .filter_map(|e| e.ok())
                    .filter(|e| {
                        // Hide hidden files and common ignore patterns
                        let name = e.file_name().to_string_lossy().to_string();
                        !name.starts_with('.')
                            && name != "node_modules"
                            && name != "target"
                            && name != "__pycache__"
                    })
                    .collect();
                items.sort_by(|a, b| {
                    let a_dir = a.path().is_dir();
                    let b_dir = b.path().is_dir();
                    b_dir.cmp(&a_dir).then_with(|| {
                        a.file_name()
                            .to_string_lossy()
                            .to_lowercase()
                            .cmp(&b.file_name().to_string_lossy().to_lowercase())
                    })
                });
                for item in items {
                    children.push(Self::read_dir(&item.path(), depth + 1));
                }
            }
        }

        FileEntry {
            path: path.to_path_buf(),
            name,
            is_dir: path.is_dir(),
            depth,
            expanded: depth == 0, // Root is expanded by default
            children,
        }
    }

    /// Flatten the tree for rendering
    pub fn flatten(&mut self) {
        self.entries_flat.clear();
        Self::flatten_entry(&self.root, &mut self.entries_flat);
    }

    fn flatten_entry(entry: &FileEntry, flat: &mut Vec<FlatEntry>) {
        flat.push(FlatEntry {
            path: entry.path.clone(),
            name: entry.name.clone(),
            is_dir: entry.is_dir,
            depth: entry.depth,
            expanded: entry.expanded,
        });
        if entry.is_dir && entry.expanded {
            for child in &entry.children {
                Self::flatten_entry(child, flat);
            }
        }
    }

    pub fn toggle_expand(&mut self) {
        if let Some(entry) = self.entries_flat.get(self.selected) {
            if entry.is_dir {
                let path = entry.path.clone();
                Self::toggle_dir(&mut self.root, &path);
                self.flatten();
            }
        }
    }

    fn toggle_dir(entry: &mut FileEntry, path: &Path) -> bool {
        if entry.path == path {
            entry.expanded = !entry.expanded;
            return true;
        }
        for child in &mut entry.children {
            if Self::toggle_dir(child, path) {
                return true;
            }
        }
        false
    }

    pub fn selected_path(&self) -> Option<&PathBuf> {
        self.entries_flat.get(self.selected).map(|e| &e.path)
    }

    pub fn move_up(&mut self) {
        if self.selected > 0 {
            self.selected -= 1;
        }
    }

    pub fn move_down(&mut self) {
        if self.selected < self.entries_flat.len().saturating_sub(1) {
            self.selected += 1;
        }
    }
}
