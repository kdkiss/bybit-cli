use colored::Colorize;
use dialoguer::{Confirm, Input, Password, Select};

use crate::client::BybitClient;
use crate::config::{load_config, save_config, AuthConfig, Config, SecretValue, SettingsConfig};
use crate::errors::{BybitError, BybitResult};

const CATEGORY_OPTIONS: &[&str] = &["linear", "spot", "inverse", "option"];
const OUTPUT_OPTIONS: &[&str] = &["table", "json"];

// ---------------------------------------------------------------------------
// bybit setup
// ---------------------------------------------------------------------------

pub async fn run_setup() -> BybitResult<()> {
    println!("{}", "Bybit CLI — First-time setup".bold());
    let config_path_display = crate::config::config_path()
        .map(|p| p.display().to_string())
        .unwrap_or_else(|_| "your platform config directory".to_string());
    println!("Credentials are saved to {config_path_display}\n");

    // Load existing config so we can show current values as defaults
    let existing = load_config().unwrap_or_default();

    // API key
    let default_key = existing.auth.api_key.clone().unwrap_or_default();
    let api_key: String = Input::new()
        .with_prompt("API Key")
        .default(default_key)
        .interact_text()
        .map_err(|e| BybitError::Io(std::io::Error::other(e.to_string())))?;

    if api_key.is_empty() {
        return Err(BybitError::Validation("API key cannot be empty.".into()));
    }

    // API secret (masked input)
    let api_secret: String = Password::new()
        .with_prompt("API Secret")
        .allow_empty_password(false)
        .interact()
        .map_err(|e| BybitError::Io(std::io::Error::other(e.to_string())))?;

    // Default category
    let current_category = existing.settings.default_category.as_str();
    let default_idx = CATEGORY_OPTIONS
        .iter()
        .position(|&c| c == current_category)
        .unwrap_or(0);

    let category_idx = Select::new()
        .with_prompt("Default category")
        .items(CATEGORY_OPTIONS)
        .default(default_idx)
        .interact()
        .map_err(|e| BybitError::Io(std::io::Error::other(e.to_string())))?;

    // Output format
    let current_fmt = existing.settings.output.as_str();
    let fmt_idx = OUTPUT_OPTIONS
        .iter()
        .position(|&f| f == current_fmt)
        .unwrap_or(0);

    let fmt_idx = Select::new()
        .with_prompt("Default output format")
        .items(OUTPUT_OPTIONS)
        .default(fmt_idx)
        .interact()
        .map_err(|e| BybitError::Io(std::io::Error::other(e.to_string())))?;

    // Testnet?
    let testnet = Confirm::new()
        .with_prompt("Use testnet by default?")
        .default(existing.settings.testnet)
        .interact()
        .map_err(|e| BybitError::Io(std::io::Error::other(e.to_string())))?;

    // Verify credentials against the API before saving
    println!("\nVerifying credentials…");
    let client = BybitClient::new(
        testnet,
        None,
        Some(api_key.clone()),
        Some(api_secret.clone()),
        None,
    )?;

    match client.private_get("/v5/account/info", &[]).await {
        Ok(_) => println!("{}", "✓ Credentials verified.".green()),
        Err(e) => {
            let skip = Confirm::new()
                .with_prompt(format!(
                    "{} Save anyway?",
                    format!("Warning: could not verify credentials ({e}).").yellow()
                ))
                .default(false)
                .interact()
                .map_err(|e| BybitError::Io(std::io::Error::other(e.to_string())))?;
            if !skip {
                return Err(BybitError::Auth(
                    "Setup cancelled — credentials not saved.".into(),
                ));
            }
        }
    }

    let config = build_setup_config(
        &existing,
        api_key,
        api_secret,
        category_idx,
        fmt_idx,
        testnet,
    )?;

    save_config(&config)?;

    println!(
        "\n{} Config saved to {}",
        "✓".green(),
        crate::config::config_path()
            .map(|p| p.display().to_string())
            .unwrap_or_default()
            .dimmed()
    );

    if testnet {
        println!(
            "{} Testnet mode is on. Set BYBIT_TESTNET=0 or re-run setup to switch to mainnet.",
            "Note:".yellow()
        );
    }

    Ok(())
}

fn build_setup_config(
    existing: &Config,
    api_key: String,
    api_secret: String,
    category_idx: usize,
    fmt_idx: usize,
    testnet: bool,
) -> BybitResult<Config> {
    if api_key.trim().is_empty() {
        return Err(BybitError::Validation("API key cannot be empty.".into()));
    }

    let default_category = CATEGORY_OPTIONS
        .get(category_idx)
        .ok_or_else(|| BybitError::Validation("Invalid default category selection.".into()))?;
    let output = OUTPUT_OPTIONS
        .get(fmt_idx)
        .ok_or_else(|| BybitError::Validation("Invalid output format selection.".into()))?;

    Ok(Config {
        auth: AuthConfig {
            api_key: Some(api_key),
            api_secret: Some(SecretValue::new(api_secret)),
        },
        settings: SettingsConfig {
            default_category: (*default_category).to_string(),
            output: (*output).to_string(),
            recv_window: existing.settings.recv_window,
            testnet,
        },
    })
}

#[cfg(test)]
mod tests {
    use super::build_setup_config;
    use crate::config::{config_path, Config};

    #[test]
    fn build_setup_config_rejects_empty_api_key() {
        let existing = Config::default();
        let err = build_setup_config(&existing, "".into(), "secret".into(), 0, 0, false)
            .unwrap_err()
            .to_string();
        assert!(err.contains("API key cannot be empty"));
    }

    #[test]
    fn build_setup_config_preserves_recv_window_and_testnet() {
        let mut existing = Config::default();
        existing.settings.recv_window = 9000;

        let config = build_setup_config(&existing, "key".into(), "secret".into(), 2, 1, true)
            .expect("config should build");

        assert_eq!(config.auth.api_key.as_deref(), Some("key"));
        assert_eq!(
            config.auth.api_secret.as_ref().map(|v| v.expose()),
            Some("secret")
        );
        assert_eq!(config.settings.default_category, "inverse");
        assert_eq!(config.settings.output, "json");
        assert_eq!(config.settings.recv_window, 9000);
        assert!(config.settings.testnet);
    }

    #[test]
    fn config_path_ends_with_bybit_config_toml() {
        let path = config_path().expect("config path should resolve");
        let rendered = path.display().to_string();
        assert!(rendered.contains("bybit"));
        assert!(rendered.ends_with("config.toml"));
    }
}
