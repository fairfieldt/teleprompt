use anyhow::{Context, Result};
use serde::Deserialize;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub bot_token: String,
    pub user_id: i64,
    #[serde(default = "default_timeout_minutes")]
    pub timeout_minutes: u64,
}

fn default_timeout_minutes() -> u64 {
    60
}

pub fn default_config_path() -> Result<PathBuf> {
    default_config_path_impl()
}

#[cfg(target_os = "linux")]
fn default_config_path_impl() -> Result<PathBuf> {
    let base = match std::env::var_os("XDG_CONFIG_HOME") {
        Some(xdg) if !xdg.is_empty() => PathBuf::from(xdg),
        _ => {
            let home = std::env::var_os("HOME").context("HOME environment variable is not set")?;
            PathBuf::from(home).join(".config")
        }
    };

    Ok(base.join("teleprompt").join("config.toml"))
}

#[cfg(target_os = "macos")]
fn default_config_path_impl() -> Result<PathBuf> {
    let home = std::env::var_os("HOME").context("HOME environment variable is not set")?;
    Ok(PathBuf::from(home)
        .join("Library")
        .join("Application Support")
        .join("teleprompt")
        .join("config.toml"))
}

#[cfg(target_os = "windows")]
fn default_config_path_impl() -> Result<PathBuf> {
    let appdata = std::env::var_os("APPDATA").context("APPDATA environment variable is not set")?;
    Ok(PathBuf::from(appdata)
        .join("teleprompt")
        .join("config.toml"))
}

#[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
fn default_config_path_impl() -> Result<PathBuf> {
    anyhow::bail!("unsupported OS for default config path resolution")
}

pub fn load(path: &Path) -> Result<Config> {
    let raw = std::fs::read_to_string(path)
        .with_context(|| format!("read config file: {}", path.display()))?;
    let cfg: Config =
        toml::from_str(&raw).with_context(|| format!("parse TOML config: {}", path.display()))?;
    Ok(cfg)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::OsString;
    use std::sync::{Mutex, OnceLock};

    fn env_lock() -> std::sync::MutexGuard<'static, ()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(()))
            .lock()
            .expect("env lock")
    }

    fn set_env(key: &str, value: Option<OsString>) -> Option<OsString> {
        let old = std::env::var_os(key);

        // std::env::set_var/remove_var are unsafe in Rust 2024 due to potential UB when
        // accessed concurrently; tests guard env mutation with env_lock().
        unsafe {
            match value {
                Some(v) => std::env::set_var(key, v),
                None => std::env::remove_var(key),
            }
        }
        old
    }

    #[test]
    fn parses_minimal_config_with_default_timeout() {
        let raw = r#"
bot_token = "t"
user_id = 123
"#;
        let cfg: Config = toml::from_str(raw).unwrap();
        assert_eq!(cfg.bot_token, "t");
        assert_eq!(cfg.user_id, 123);
        assert_eq!(cfg.timeout_minutes, 60);
    }

    #[test]
    fn parses_config_with_timeout_override() {
        let raw = r#"
bot_token = "t"
user_id = 123
timeout_minutes = 5
"#;
        let cfg: Config = toml::from_str(raw).unwrap();
        assert_eq!(cfg.timeout_minutes, 5);
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn default_config_path_linux_prefers_xdg_config_home() {
        let _lock = env_lock();

        let old_home = set_env("HOME", Some(OsString::from("/home/test")));
        let old_xdg = set_env("XDG_CONFIG_HOME", Some(OsString::from("/xdg")));

        let path = default_config_path().unwrap();
        assert_eq!(path, PathBuf::from("/xdg/teleprompt/config.toml"));

        set_env("HOME", old_home);
        set_env("XDG_CONFIG_HOME", old_xdg);
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn default_config_path_linux_falls_back_to_home_dot_config() {
        let _lock = env_lock();

        let old_home = set_env("HOME", Some(OsString::from("/home/test")));
        let old_xdg = set_env("XDG_CONFIG_HOME", None);

        let path = default_config_path().unwrap();
        assert_eq!(
            path,
            PathBuf::from("/home/test/.config/teleprompt/config.toml")
        );

        set_env("HOME", old_home);
        set_env("XDG_CONFIG_HOME", old_xdg);
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn default_config_path_macos_uses_application_support() {
        let _lock = env_lock();

        let old_home = set_env("HOME", Some(OsString::from("/Users/test")));

        let path = default_config_path().unwrap();
        assert_eq!(
            path,
            PathBuf::from("/Users/test/Library/Application Support/teleprompt/config.toml")
        );

        set_env("HOME", old_home);
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn default_config_path_windows_uses_appdata() {
        let _lock = env_lock();

        let old_appdata = set_env(
            "APPDATA",
            Some(OsString::from(r"C:\\Users\\test\\AppData\\Roaming")),
        );

        let path = default_config_path().unwrap();
        assert_eq!(
            path,
            PathBuf::from(r"C:\\Users\\test\\AppData\\Roaming\\teleprompt\\config.toml")
        );

        set_env("APPDATA", old_appdata);
    }
}
