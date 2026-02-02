use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct UsersConfig {
    #[serde(default)]
    pub users: Vec<UserEntry>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UserEntry {
    pub system_user: String,
    #[serde(default = "default_true")]
    pub enabled: bool,
    pub telegram_user_id: Option<u64>,
    #[serde(default)]
    pub promote_on_first_auth: bool,
    pub socket_path: String,
    #[serde(default)]
    pub allowed_usernames: Vec<String>,
    pub first_seen_at: Option<String>,
    pub last_seen_at: Option<String>,
}

fn default_true() -> bool {
    true
}

impl UsersConfig {
    pub fn load(path: &Path) -> Result<Self> {
        if !path.exists() {
            return Ok(Self::default());
        }
        let content = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read {}", path.display()))?;
        toml::from_str(&content)
            .with_context(|| format!("Failed to parse {}", path.display()))
    }

    pub fn save(&self, path: &Path) -> Result<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create directory {}", parent.display()))?;
        }
        let content = toml::to_string_pretty(self)
            .context("Failed to serialize users config")?;
        std::fs::write(path, content)
            .with_context(|| format!("Failed to write {}", path.display()))
    }
}

/// Derive default socket path from system username via uid lookup.
pub fn default_socket_path(system_user: &str) -> String {
    match nix::unistd::User::from_name(system_user) {
        Ok(Some(user)) => format!("/run/user/{}/ratatoskr.sock", user.uid),
        _ => format!("/tmp/ratatoskr-{}.sock", system_user),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn round_trip_serialization() {
        let config = UsersConfig {
            users: vec![UserEntry {
                system_user: "alice".to_string(),
                enabled: true,
                telegram_user_id: Some(12345),
                promote_on_first_auth: false,
                socket_path: "/run/user/1000/ratatoskr.sock".to_string(),
                allowed_usernames: vec!["alice_tg".to_string()],
                first_seen_at: None,
                last_seen_at: None,
            }],
        };

        let tmp = NamedTempFile::new().unwrap();
        config.save(tmp.path()).unwrap();
        let loaded = UsersConfig::load(tmp.path()).unwrap();

        assert_eq!(loaded.users.len(), 1);
        assert_eq!(loaded.users[0].system_user, "alice");
        assert_eq!(loaded.users[0].telegram_user_id, Some(12345));
        assert_eq!(loaded.users[0].allowed_usernames, vec!["alice_tg"]);
    }

    #[test]
    fn load_missing_file_returns_empty() {
        let config = UsersConfig::load(Path::new("/nonexistent/users.toml")).unwrap();
        assert!(config.users.is_empty());
    }

    #[test]
    fn defaults_enabled_true() {
        let toml_str = r#"
[[users]]
system_user = "bob"
socket_path = "/tmp/bob.sock"
"#;
        let config: UsersConfig = toml::from_str(toml_str).unwrap();
        assert!(config.users[0].enabled);
    }
}
