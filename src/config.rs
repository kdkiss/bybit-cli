use std::fmt;
use std::fs;
use std::io::{self, Read};
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::errors::{BybitError, BybitResult};

// ---------------------------------------------------------------------------
// Secret wrapper — redacts in all debug/display/serialization contexts
// ---------------------------------------------------------------------------

#[derive(Clone, Deserialize)]
#[serde(transparent)]
pub struct SecretValue(String);

impl SecretValue {
    pub fn new(s: impl Into<String>) -> Self {
        Self(s.into())
    }

    pub fn expose(&self) -> &str {
        &self.0
    }
}

impl fmt::Debug for SecretValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[REDACTED]")
    }
}

impl fmt::Display for SecretValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[REDACTED]")
    }
}

impl Serialize for SecretValue {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str("[REDACTED]")
    }
}

// ---------------------------------------------------------------------------
// Credential source tracking
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CredentialSource {
    Flag,
    Env,
    Config,
}

#[derive(Debug, Clone)]
pub struct Credentials {
    pub api_key: String,
    pub api_secret: SecretValue,
    pub source: CredentialSource,
}

// ---------------------------------------------------------------------------
// Config file structure
// ---------------------------------------------------------------------------

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct AuthConfig {
    pub api_key: Option<String>,
    pub api_secret: Option<SecretValue>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct SettingsConfig {
    #[serde(default = "default_category")]
    pub default_category: String,
    #[serde(default = "default_output")]
    pub output: String,
    #[serde(default = "default_recv_window")]
    pub recv_window: u64,
    #[serde(default)]
    pub testnet: bool,
}

fn default_category() -> String {
    "linear".to_string()
}
fn default_output() -> String {
    "table".to_string()
}
fn default_recv_window() -> u64 {
    5000
}

impl Default for SettingsConfig {
    fn default() -> Self {
        Self {
            default_category: default_category(),
            output: default_output(),
            recv_window: default_recv_window(),
            testnet: false,
        }
    }
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct Config {
    #[serde(default)]
    pub auth: AuthConfig,
    #[serde(default)]
    pub settings: SettingsConfig,
}

// ---------------------------------------------------------------------------
// Config file path
// ---------------------------------------------------------------------------

pub fn config_dir() -> BybitResult<PathBuf> {
    if let Ok(override_dir) = std::env::var("BYBIT_CONFIG_DIR") {
        return Ok(PathBuf::from(override_dir));
    }

    let base = dirs::config_dir()
        .ok_or_else(|| BybitError::Config("Cannot determine config directory".to_string()))?;
    Ok(base.join("bybit"))
}

pub fn config_path() -> BybitResult<PathBuf> {
    Ok(config_dir()?.join("config.toml"))
}

pub fn history_path() -> BybitResult<PathBuf> {
    Ok(config_dir()?.join("history"))
}

pub fn paper_journal_path() -> BybitResult<PathBuf> {
    Ok(config_dir()?.join("paper-journal.json"))
}

pub fn instance_id_path() -> BybitResult<PathBuf> {
    Ok(config_dir()?.join("instance-id"))
}

pub fn read_secret_from_file(path: &Path) -> BybitResult<SecretValue> {
    let secret = fs::read_to_string(path)
        .map_err(|e| BybitError::Config(format!("Failed to read secret file: {e}")))?;
    let trimmed = secret.trim().to_string();
    if trimmed.is_empty() {
        return Err(BybitError::Auth("API secret file is empty.".to_string()));
    }
    Ok(SecretValue::new(trimmed))
}

pub fn read_secret_from_stdin() -> BybitResult<SecretValue> {
    let mut secret = String::new();
    io::stdin()
        .read_to_string(&mut secret)
        .map_err(BybitError::Io)?;
    let trimmed = secret.trim().to_string();
    if trimmed.is_empty() {
        return Err(BybitError::Auth(
            "API secret from stdin is empty.".to_string(),
        ));
    }
    Ok(SecretValue::new(trimmed))
}

// ---------------------------------------------------------------------------
// Load / save
// ---------------------------------------------------------------------------

pub fn load_config() -> BybitResult<Config> {
    let path = config_path()?;
    if !path.exists() {
        return Ok(Config::default());
    }
    let contents = fs::read_to_string(&path)
        .map_err(|e| BybitError::Config(format!("Failed to read config: {e}")))?;
    toml::from_str(&contents)
        .map_err(|e| BybitError::Config(format!("Failed to parse config: {e}")))
}

pub fn save_config(config: &Config) -> BybitResult<()> {
    #[derive(Serialize)]
    struct PersistedAuthConfig<'a> {
        api_key: &'a Option<String>,
        api_secret: Option<&'a str>,
    }

    #[derive(Serialize)]
    struct PersistedSettingsConfig<'a> {
        default_category: &'a str,
        output: &'a str,
        recv_window: u64,
        testnet: bool,
    }

    #[derive(Serialize)]
    struct PersistedConfig<'a> {
        auth: PersistedAuthConfig<'a>,
        settings: PersistedSettingsConfig<'a>,
    }

    let dir = config_dir()?;
    fs::create_dir_all(&dir)
        .map_err(|e| BybitError::Config(format!("Failed to create config dir: {e}")))?;

    let path = config_path()?;
    let persisted = PersistedConfig {
        auth: PersistedAuthConfig {
            api_key: &config.auth.api_key,
            api_secret: config.auth.api_secret.as_ref().map(SecretValue::expose),
        },
        settings: PersistedSettingsConfig {
            default_category: &config.settings.default_category,
            output: &config.settings.output,
            recv_window: config.settings.recv_window,
            testnet: config.settings.testnet,
        },
    };
    let contents = toml::to_string_pretty(&persisted)
        .map_err(|e| BybitError::Config(format!("Failed to serialize config: {e}")))?;

    // Write with restricted permissions on Unix
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        let mut opts = fs::OpenOptions::new();
        opts.write(true).create(true).truncate(true).mode(0o600);
        let mut file = opts
            .open(&path)
            .map_err(|e| BybitError::Config(format!("Failed to open config for write: {e}")))?;
        use std::io::Write;
        file.write_all(contents.as_bytes())
            .map_err(|e| BybitError::Config(format!("Failed to write config: {e}")))?;
    }

    #[cfg(not(unix))]
    fs::write(&path, contents)
        .map_err(|e| BybitError::Config(format!("Failed to write config: {e}")))?;

    Ok(())
}

// ---------------------------------------------------------------------------
// Credential resolution: Flag > Env > Config
// ---------------------------------------------------------------------------

pub fn resolve_credentials(
    flag_key: Option<&str>,
    flag_secret: Option<&str>,
) -> BybitResult<Option<Credentials>> {
    // 1. CLI flags
    if let (Some(key), Some(secret)) = (flag_key, flag_secret) {
        return Ok(Some(Credentials {
            api_key: key.to_string(),
            api_secret: SecretValue::new(secret),
            source: CredentialSource::Flag,
        }));
    }

    // 2. Environment variables
    let env_key = std::env::var("BYBIT_API_KEY").ok();
    let env_secret = std::env::var("BYBIT_API_SECRET").ok();
    if let (Some(key), Some(secret)) = (env_key, env_secret) {
        return Ok(Some(Credentials {
            api_key: key,
            api_secret: SecretValue::new(secret),
            source: CredentialSource::Env,
        }));
    }

    // 3. Config file
    let config = load_config()?;
    if let (Some(key), Some(secret)) = (config.auth.api_key, config.auth.api_secret) {
        return Ok(Some(Credentials {
            api_key: key,
            api_secret: secret,
            source: CredentialSource::Config,
        }));
    }

    Ok(None)
}
