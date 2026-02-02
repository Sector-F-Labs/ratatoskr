use crate::config::{UserEntry, UsersConfig};
use anyhow::Result;
use std::path::PathBuf;

pub struct AuthService {
    config: UsersConfig,
    config_path: PathBuf,
}

impl AuthService {
    pub fn new(config: UsersConfig, config_path: PathBuf) -> Self {
        Self {
            config,
            config_path,
        }
    }

    /// Check if a telegram user is authorized. Returns the matching user entry index.
    pub fn check(&self, telegram_user_id: u64, telegram_username: Option<&str>) -> Option<usize> {
        // First: match by telegram_user_id
        for (i, entry) in self.config.users.iter().enumerate() {
            if !entry.enabled {
                continue;
            }
            if entry.telegram_user_id == Some(telegram_user_id) {
                return Some(i);
            }
        }

        // Second: match by allowed_usernames if promote_on_first_auth is set
        if let Some(username) = telegram_username {
            let username_lower = username.to_lowercase();
            for (i, entry) in self.config.users.iter().enumerate() {
                if !entry.enabled || !entry.promote_on_first_auth {
                    continue;
                }
                if entry
                    .allowed_usernames
                    .iter()
                    .any(|u| u.to_lowercase() == username_lower)
                {
                    return Some(i);
                }
            }
        }

        None
    }

    /// Whether the user list is empty (no auth configured).
    pub fn is_empty(&self) -> bool {
        self.config.users.is_empty()
    }

    /// Get user entry by index.
    pub fn get_user(&self, index: usize) -> Option<&UserEntry> {
        self.config.users.get(index)
    }

    /// Promote a user: capture their telegram_user_id, clear allowed_usernames, persist.
    pub fn promote(&mut self, index: usize, telegram_user_id: u64) -> Result<()> {
        if let Some(entry) = self.config.users.get_mut(index) {
            entry.telegram_user_id = Some(telegram_user_id);
            entry.allowed_usernames.clear();
            entry.promote_on_first_auth = false;
            tracing::info!(
                system_user = %entry.system_user,
                telegram_user_id = telegram_user_id,
                "Promoted user — captured telegram_user_id"
            );
            self.config.save(&self.config_path)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::UserEntry;
    use tempfile::NamedTempFile;

    fn make_entry(
        system_user: &str,
        tg_id: Option<u64>,
        promote: bool,
        usernames: Vec<&str>,
    ) -> UserEntry {
        UserEntry {
            system_user: system_user.to_string(),
            enabled: true,
            telegram_user_id: tg_id,
            promote_on_first_auth: promote,
            socket_path: "/tmp/test.sock".to_string(),
            allowed_usernames: usernames.into_iter().map(String::from).collect(),
            first_seen_at: None,
            last_seen_at: None,
        }
    }

    #[test]
    fn check_by_telegram_id() {
        let config = UsersConfig {
            users: vec![make_entry("alice", Some(111), false, vec![])],
        };
        let svc = AuthService::new(config, PathBuf::from("/tmp/test.toml"));
        assert!(svc.check(111, None).is_some());
        assert!(svc.check(999, None).is_none());
    }

    #[test]
    fn check_by_username_with_promote() {
        let config = UsersConfig {
            users: vec![make_entry("bob", None, true, vec!["BobTG"])],
        };
        let svc = AuthService::new(config, PathBuf::from("/tmp/test.toml"));
        // Case-insensitive match
        assert!(svc.check(555, Some("bobtg")).is_some());
        assert!(svc.check(555, Some("unknown")).is_none());
        // Without promote flag, username match should fail
        assert!(svc.check(555, None).is_none());
    }

    #[test]
    fn check_disabled_user_rejected() {
        let mut entry = make_entry("carol", Some(222), false, vec![]);
        entry.enabled = false;
        let config = UsersConfig {
            users: vec![entry],
        };
        let svc = AuthService::new(config, PathBuf::from("/tmp/test.toml"));
        assert!(svc.check(222, None).is_none());
    }

    #[test]
    fn promote_captures_id_and_clears_usernames() {
        let tmp = NamedTempFile::new().unwrap();
        let config = UsersConfig {
            users: vec![make_entry("dave", None, true, vec!["DaveTG"])],
        };
        config.save(tmp.path()).unwrap();

        let mut svc = AuthService::new(config, tmp.path().to_path_buf());
        svc.promote(0, 777).unwrap();

        assert_eq!(svc.config.users[0].telegram_user_id, Some(777));
        assert!(svc.config.users[0].allowed_usernames.is_empty());
        assert!(!svc.config.users[0].promote_on_first_auth);

        // Verify persisted
        let reloaded = UsersConfig::load(tmp.path()).unwrap();
        assert_eq!(reloaded.users[0].telegram_user_id, Some(777));
    }
}
