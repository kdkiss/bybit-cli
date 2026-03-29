use std::ffi::OsString;
use std::fs;
use std::path::Path;
use std::sync::{Mutex, OnceLock};

use tempfile::TempDir;

use bybit_cli::config::{
    config_path, load_config, resolve_credentials, save_config, AuthConfig, Config,
    CredentialSource, SecretValue, SettingsConfig,
};

fn env_lock() -> std::sync::MutexGuard<'static, ()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
        .lock()
        .unwrap_or_else(|err| err.into_inner())
}

struct EnvReset {
    values: Vec<(&'static str, Option<OsString>)>,
}

impl EnvReset {
    fn capture(vars: &[&'static str]) -> Self {
        Self {
            values: vars
                .iter()
                .map(|name| (*name, std::env::var_os(name)))
                .collect(),
        }
    }
}

impl Drop for EnvReset {
    fn drop(&mut self) {
        for (name, value) in self.values.drain(..) {
            if let Some(value) = value {
                std::env::set_var(name, value);
            } else {
                std::env::remove_var(name);
            }
        }
    }
}

fn set_config_home(path: &Path) {
    std::env::set_var("BYBIT_CONFIG_DIR", path.join("bybit"));
    std::env::set_var("APPDATA", path);
    std::env::set_var("XDG_CONFIG_HOME", path);
    std::env::set_var("HOME", path);
}

fn clear_credentials_env() {
    std::env::remove_var("BYBIT_API_KEY");
    std::env::remove_var("BYBIT_API_SECRET");
}

// ---------------------------------------------------------------------------
// SecretValue redaction
// ---------------------------------------------------------------------------

#[test]
fn secret_value_debug_is_redacted() {
    let s = SecretValue::new("supersecret");
    assert_eq!(format!("{s:?}"), "[REDACTED]");
}

#[test]
fn secret_value_display_is_redacted() {
    let s = SecretValue::new("supersecret");
    assert_eq!(format!("{s}"), "[REDACTED]");
}

#[test]
fn secret_value_expose_returns_inner() {
    let s = SecretValue::new("my_secret_key");
    assert_eq!(s.expose(), "my_secret_key");
}

#[test]
fn secret_value_serializes_as_redacted() {
    let s = SecretValue::new("supersecret");
    let json = serde_json::to_string(&s).unwrap();
    assert_eq!(json, "\"[REDACTED]\"");
}

// ---------------------------------------------------------------------------
// Config persistence
// ---------------------------------------------------------------------------

#[test]
fn config_save_and_load_round_trip_uses_real_secret() {
    let _guard = env_lock();
    let dir = TempDir::new().unwrap();
    let _reset = EnvReset::capture(&["BYBIT_CONFIG_DIR", "APPDATA", "XDG_CONFIG_HOME", "HOME"]);
    set_config_home(dir.path());

    let config = Config {
        auth: AuthConfig {
            api_key: Some("mykey".to_string()),
            api_secret: Some(SecretValue::new("mysecret")),
        },
        settings: SettingsConfig {
            default_category: "spot".to_string(),
            output: "json".to_string(),
            recv_window: 10000,
            testnet: true,
        },
    };

    save_config(&config).unwrap();

    let raw = fs::read_to_string(config_path().unwrap()).unwrap();
    assert!(raw.contains("api_secret = \"mysecret\""));
    assert!(!raw.contains("[REDACTED]"));

    let loaded = load_config().unwrap();
    assert_eq!(loaded.auth.api_key.as_deref(), Some("mykey"));
    assert_eq!(
        loaded.auth.api_secret.as_ref().map(SecretValue::expose),
        Some("mysecret")
    );
    assert_eq!(loaded.settings.default_category, "spot");
    assert_eq!(loaded.settings.output, "json");
    assert_eq!(loaded.settings.recv_window, 10000);
    assert!(loaded.settings.testnet);
}

// ---------------------------------------------------------------------------
// Config defaults
// ---------------------------------------------------------------------------

#[test]
fn settings_config_defaults() {
    let s = SettingsConfig::default();
    assert_eq!(s.default_category, "linear");
    assert_eq!(s.output, "table");
    assert_eq!(s.recv_window, 5000);
    assert!(!s.testnet);
}

#[test]
fn config_default_has_no_credentials() {
    let c = Config::default();
    assert!(c.auth.api_key.is_none());
    assert!(c.auth.api_secret.is_none());
}

// ---------------------------------------------------------------------------
// Config parse: missing fields use defaults
// ---------------------------------------------------------------------------

#[test]
fn config_parses_with_missing_settings_fields() {
    let toml = r#"
[auth]
api_key = "k"
api_secret = "s"
"#;
    let c: Config = toml::from_str(toml).unwrap();
    assert_eq!(c.settings.default_category, "linear");
    assert_eq!(c.settings.output, "table");
    assert_eq!(c.settings.recv_window, 5000);
    assert!(!c.settings.testnet);
}

#[test]
fn config_parses_empty_toml_as_default() {
    let c: Config = toml::from_str("").unwrap();
    assert!(c.auth.api_key.is_none());
    assert_eq!(c.settings.recv_window, 5000);
    assert!(!c.settings.testnet);
}

// ---------------------------------------------------------------------------
// Credential resolution order
// ---------------------------------------------------------------------------

#[test]
fn resolve_credentials_flags_take_priority() {
    let _guard = env_lock();
    let dir = TempDir::new().unwrap();
    let _reset = EnvReset::capture(&[
        "APPDATA",
        "XDG_CONFIG_HOME",
        "HOME",
        "BYBIT_CONFIG_DIR",
        "BYBIT_API_KEY",
        "BYBIT_API_SECRET",
    ]);
    set_config_home(dir.path());
    clear_credentials_env();
    std::env::set_var("BYBIT_API_KEY", "env-key");
    std::env::set_var("BYBIT_API_SECRET", "env-secret");

    let creds = resolve_credentials(Some("flag-key"), Some("flag-secret"))
        .unwrap()
        .unwrap();

    assert_eq!(creds.api_key, "flag-key");
    assert_eq!(creds.api_secret.expose(), "flag-secret");
    assert_eq!(creds.source, CredentialSource::Flag);
}

#[test]
fn resolve_credentials_env_beats_config() {
    let _guard = env_lock();
    let dir = TempDir::new().unwrap();
    let _reset = EnvReset::capture(&[
        "APPDATA",
        "XDG_CONFIG_HOME",
        "HOME",
        "BYBIT_CONFIG_DIR",
        "BYBIT_API_KEY",
        "BYBIT_API_SECRET",
    ]);
    set_config_home(dir.path());
    clear_credentials_env();

    save_config(&Config {
        auth: AuthConfig {
            api_key: Some("config-key".to_string()),
            api_secret: Some(SecretValue::new("config-secret")),
        },
        settings: SettingsConfig::default(),
    })
    .unwrap();

    std::env::set_var("BYBIT_API_KEY", "env-key");
    std::env::set_var("BYBIT_API_SECRET", "env-secret");

    let creds = resolve_credentials(None, None).unwrap().unwrap();
    assert_eq!(creds.api_key, "env-key");
    assert_eq!(creds.api_secret.expose(), "env-secret");
    assert_eq!(creds.source, CredentialSource::Env);
}

#[test]
fn resolve_credentials_uses_config_when_env_missing() {
    let _guard = env_lock();
    let dir = TempDir::new().unwrap();
    let _reset = EnvReset::capture(&[
        "APPDATA",
        "XDG_CONFIG_HOME",
        "HOME",
        "BYBIT_CONFIG_DIR",
        "BYBIT_API_KEY",
        "BYBIT_API_SECRET",
    ]);
    set_config_home(dir.path());
    clear_credentials_env();

    save_config(&Config {
        auth: AuthConfig {
            api_key: Some("config-key".to_string()),
            api_secret: Some(SecretValue::new("config-secret")),
        },
        settings: SettingsConfig::default(),
    })
    .unwrap();

    let creds = resolve_credentials(None, None).unwrap().unwrap();
    assert_eq!(creds.api_key, "config-key");
    assert_eq!(creds.api_secret.expose(), "config-secret");
    assert_eq!(creds.source, CredentialSource::Config);
}

#[test]
fn resolve_credentials_returns_none_when_missing() {
    let _guard = env_lock();
    let dir = TempDir::new().unwrap();
    let _reset = EnvReset::capture(&[
        "APPDATA",
        "XDG_CONFIG_HOME",
        "HOME",
        "BYBIT_CONFIG_DIR",
        "BYBIT_API_KEY",
        "BYBIT_API_SECRET",
    ]);
    set_config_home(dir.path());
    clear_credentials_env();

    let result = resolve_credentials(None, None).unwrap();
    assert!(result.is_none());
}

#[test]
fn resolve_credentials_partial_flags_fall_through() {
    let _guard = env_lock();
    let dir = TempDir::new().unwrap();
    let _reset = EnvReset::capture(&[
        "APPDATA",
        "XDG_CONFIG_HOME",
        "HOME",
        "BYBIT_CONFIG_DIR",
        "BYBIT_API_KEY",
        "BYBIT_API_SECRET",
    ]);
    set_config_home(dir.path());
    clear_credentials_env();
    std::env::set_var("BYBIT_API_KEY", "env-key");
    std::env::set_var("BYBIT_API_SECRET", "env-secret");

    let creds = resolve_credentials(Some("only-key"), None)
        .unwrap()
        .unwrap();
    assert_eq!(creds.api_key, "env-key");
    assert_eq!(creds.api_secret.expose(), "env-secret");
    assert_eq!(creds.source, CredentialSource::Env);
}
