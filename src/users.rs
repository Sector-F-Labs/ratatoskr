use crate::config::{UserEntry, UsersConfig};
use anyhow::Result;
use std::path::Path;

pub fn add_user(
    users_file: &Path,
    system_user: &str,
    usernames: &[String],
    promote: bool,
) -> Result<()> {
    let mut config = UsersConfig::load(users_file)?;

    if config.users.iter().any(|u| u.system_user == system_user) {
        anyhow::bail!("User '{}' already exists", system_user);
    }

    config.users.push(UserEntry {
        system_user: system_user.to_string(),
        enabled: true,
        telegram_user_id: None,
        promote_on_first_auth: promote,
        allowed_usernames: usernames.to_vec(),
        first_seen_at: None,
        last_seen_at: None,
    });

    config.save(users_file)?;
    eprintln!("Added user '{}'", system_user);
    Ok(())
}

pub fn remove_user(users_file: &Path, system_user: &str) -> Result<()> {
    let mut config = UsersConfig::load(users_file)?;
    let before = config.users.len();
    config.users.retain(|u| u.system_user != system_user);

    if config.users.len() == before {
        anyhow::bail!("User '{}' not found", system_user);
    }

    config.save(users_file)?;
    eprintln!("Removed user '{}'", system_user);
    Ok(())
}

pub fn list_users(users_file: &Path) -> Result<()> {
    let config = UsersConfig::load(users_file)?;

    if config.users.is_empty() {
        println!("No users configured.");
        return Ok(());
    }

    println!(
        "{:<16} {:<8} {:<14} {:<8} USERNAMES",
        "SYSTEM_USER", "ENABLED", "TELEGRAM_ID", "PROMOTE"
    );
    for u in &config.users {
        println!(
            "{:<16} {:<8} {:<14} {:<8} {}",
            u.system_user,
            u.enabled,
            u.telegram_user_id
                .map(|id| id.to_string())
                .unwrap_or_else(|| "-".to_string()),
            u.promote_on_first_auth,
            if u.allowed_usernames.is_empty() {
                "-".to_string()
            } else {
                u.allowed_usernames.join(", ")
            },
        );
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::UsersConfig;
    use tempfile::NamedTempFile;

    fn empty_file() -> NamedTempFile {
        let f = NamedTempFile::new().unwrap();
        std::fs::write(f.path(), "").unwrap();
        f
    }

    #[test]
    fn add_user_creates_entry() {
        let f = empty_file();
        add_user(f.path(), "alice", &["alice_tg".to_string()], true).unwrap();

        let config = UsersConfig::load(f.path()).unwrap();
        assert_eq!(config.users.len(), 1);
        assert_eq!(config.users[0].system_user, "alice");
        assert_eq!(config.users[0].allowed_usernames, vec!["alice_tg"]);
        assert!(config.users[0].promote_on_first_auth);
        assert!(config.users[0].enabled);
        assert!(config.users[0].telegram_user_id.is_none());
    }

    #[test]
    fn add_duplicate_user_fails() {
        let f = empty_file();
        add_user(f.path(), "bob", &[], false).unwrap();
        let err = add_user(f.path(), "bob", &[], false).unwrap_err();
        assert!(err.to_string().contains("already exists"));
    }

    #[test]
    fn remove_user_deletes_entry() {
        let f = empty_file();
        add_user(f.path(), "carol", &[], false).unwrap();
        add_user(f.path(), "dave", &[], false).unwrap();

        remove_user(f.path(), "carol").unwrap();

        let config = UsersConfig::load(f.path()).unwrap();
        assert_eq!(config.users.len(), 1);
        assert_eq!(config.users[0].system_user, "dave");
    }

    #[test]
    fn remove_nonexistent_user_fails() {
        let f = empty_file();
        let err = remove_user(f.path(), "ghost").unwrap_err();
        assert!(err.to_string().contains("not found"));
    }

    #[test]
    fn list_users_empty() {
        let f = empty_file();
        list_users(f.path()).unwrap();
    }

    #[test]
    fn list_users_with_entries() {
        let f = empty_file();
        add_user(f.path(), "eve", &["eve_tg".to_string()], false).unwrap();
        add_user(f.path(), "frank", &[], true).unwrap();
        list_users(f.path()).unwrap();
    }
}
