use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use sled::Db;
use anyhow::Result;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct WorkspaceState {
    pub open_files: Vec<String>,
    pub active_file_index: usize,
}

pub struct DbManager {
    db: Db,
}

impl DbManager {
    pub fn new<P: AsRef<Path>>(workspace_root: P) -> Result<Self> {
        let db_path = workspace_root.as_ref().join(".blade").join("db");
        let db = sled::open(db_path)?;
        Ok(Self { db })
    }

    pub fn save_workspace_state(&self, state: &WorkspaceState) -> Result<()> {
        let encoded = bincode::serialize(state)?;
        self.db.insert("workspace_state", encoded)?;
        self.db.flush()?;
        Ok(())
    }

    pub fn load_workspace_state(&self) -> Result<Option<WorkspaceState>> {
        if let Some(data) = self.db.get("workspace_state")? {
            let state: WorkspaceState = bincode::deserialize(&data)?;
            Ok(Some(state))
        } else {
            Ok(None)
        }
    }
}
