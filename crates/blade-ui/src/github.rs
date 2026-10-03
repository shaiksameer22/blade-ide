use serde::Deserialize;
use tokio::process::Command;

#[derive(Debug, Deserialize, Clone)]
pub struct PullRequest {
    pub number: u64,
    pub title: String,
    pub state: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Issue {
    pub number: u64,
    pub title: String,
    pub state: String,
}

pub async fn fetch_pull_requests() -> anyhow::Result<Vec<PullRequest>> {
    let output = Command::new("gh")
        .args(["pr", "list", "--json", "number,title,state"])
        .output()
        .await?;

    if !output.status.success() {
        anyhow::bail!("Failed to run gh pr list");
    }

    let prs: Vec<PullRequest> = serde_json::from_slice(&output.stdout)?;
    Ok(prs)
}

pub async fn fetch_issues() -> anyhow::Result<Vec<Issue>> {
    let output = Command::new("gh")
        .args(["issue", "list", "--json", "number,title,state"])
        .output()
        .await?;

    if !output.status.success() {
        anyhow::bail!("Failed to run gh issue list");
    }

    let issues: Vec<Issue> = serde_json::from_slice(&output.stdout)?;
    Ok(issues)
}
