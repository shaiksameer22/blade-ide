use std::path::Path;

#[derive(Clone, Debug)]
pub struct Task {
    pub name: String,
    pub command: String,
}

pub fn get_tasks(cwd: &Path) -> Vec<Task> {
    let mut tasks = Vec::new();

    if cwd.join("Cargo.toml").exists() {
        tasks.push(Task {
            name: "Cargo: Build".into(),
            command: "cargo build".into(),
        });
        tasks.push(Task {
            name: "Cargo: Run".into(),
            command: "cargo run".into(),
        });
        tasks.push(Task {
            name: "Cargo: Test".into(),
            command: "cargo test".into(),
        });
        tasks.push(Task {
            name: "Cargo: Check".into(),
            command: "cargo check".into(),
        });
    }

    if cwd.join("package.json").exists() {
        tasks.push(Task {
            name: "NPM: Start".into(),
            command: "npm start".into(),
        });
        tasks.push(Task {
            name: "NPM: Build".into(),
            command: "npm run build".into(),
        });
        tasks.push(Task {
            name: "NPM: Test".into(),
            command: "npm test".into(),
        });
    }

    if cwd.join("Makefile").exists() {
        tasks.push(Task {
            name: "Make: Default".into(),
            command: "make".into(),
        });
        tasks.push(Task {
            name: "Make: Build".into(),
            command: "make build".into(),
        });
    }

    tasks
}
