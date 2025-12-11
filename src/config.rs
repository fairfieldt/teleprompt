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
    let home = std::env::var_os("HOME").context("HOME environment variable is not set")?;
    Ok(PathBuf::from(home).join(".teleprompt"))
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
}
