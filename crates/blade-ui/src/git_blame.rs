use anyhow::Result;
use git2::{BlameOptions, Repository};

#[derive(Clone, Debug)]
pub struct BlameInfo {
    pub author: String,
    pub date: String,
    pub summary: String,
}

use std::collections::HashMap;

pub fn get_blame_for_file(file_path: &std::path::Path) -> Result<HashMap<usize, BlameInfo>> {
    let repo = Repository::discover(file_path)?;
    let mut opts = BlameOptions::new();
    
    let blame = repo.blame_file(file_path, Some(&mut opts))?;
    
    let mut map = HashMap::new();
    for hunk in blame.iter() {
        let commit_id = hunk.final_commit_id();
        if let Ok(commit) = repo.find_commit(commit_id) {
            let author = commit.author().name().unwrap_or("Unknown").to_string();
            let summary = commit.summary().unwrap_or("").to_string();
            let time = commit.time();
            
            let datetime = chrono::DateTime::from_timestamp(time.seconds(), 0).unwrap_or_default();
            let date = datetime.format("%Y-%m-%d").to_string();

            let info = BlameInfo {
                author,
                date,
                summary,
            };

            let start = hunk.final_start_line(); // 1-indexed
            let lines = hunk.lines_in_hunk();
            for i in 0..lines {
                map.insert(start + i, info.clone());
            }
        }
    }
    
    Ok(map)
}
