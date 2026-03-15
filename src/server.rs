use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Server {
    pub alias: String,
    pub host: String,
    pub username: String,
    pub port: u16,
    pub tags: String,
}

impl Server {
    pub fn display_connection(&self) -> String {
        let user = if self.username.is_empty() {
            whoami()
        } else {
            self.username.clone()
        };
        format!("{}@{}:{}", user, self.host, self.port)
    }

    pub fn ssh_args(&self) -> Vec<String> {
        let user = if self.username.is_empty() {
            whoami()
        } else {
            self.username.clone()
        };
        vec![
            "-p".to_string(),
            self.port.to_string(),
            format!("{}@{}", user, self.host),
        ]
    }
}

fn whoami() -> String {
    std::env::var("USER")
        .or_else(|_| std::env::var("USERNAME"))
        .unwrap_or_else(|_| "root".to_string())
}

pub fn config_path(custom: Option<&str>) -> PathBuf {
    if let Some(p) = custom {
        PathBuf::from(p)
    } else {
        let proj = directories::ProjectDirs::from("com", "term-ssh", "term-ssh-manager")
            .expect("Cannot determine config directory");
        let dir = proj.config_dir().to_path_buf();
        fs::create_dir_all(&dir).ok();
        dir.join("servers.json")
    }
}

pub fn load_servers(path: &PathBuf) -> Vec<Server> {
    match fs::read_to_string(path) {
        Ok(content) => serde_json::from_str(&content).unwrap_or_default(),
        Err(_) => Vec::new(),
    }
}

pub fn save_servers(path: &PathBuf, servers: &[Server]) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let json = serde_json::to_string_pretty(servers)?;
    fs::write(path, json)
}
