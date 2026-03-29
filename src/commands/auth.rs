use clap::Subcommand;
use dialoguer::Password;
use serde_json::Value;

use crate::auth::{sign, timestamp_ms};
use crate::client::BybitClient;
use crate::config::{load_config, resolve_credentials, save_config, SecretValue};
use crate::errors::{BybitError, BybitResult};
use crate::output::print_output;
use crate::AppContext;

#[derive(Debug, clap::Args)]
pub struct AuthArgs {
    #[command(subcommand)]
    pub command: AuthCommand,
}

#[derive(Debug, Subcommand)]
pub enum AuthCommand {
    /// Save API credentials to the config file
    Set {
        #[arg(long)]
        api_key: String,
        #[arg(long)]
        api_secret: Option<String>,
    },
    /// Sign a test payload and print the HMAC-SHA256 signature
    Sign {
        #[arg(long)]
        payload: Option<String>,
    },
    /// Test credentials against the API (calls /v5/account/info)
    #[command(alias = "verify")]
    Test,
    /// Show current credential source and (masked) API key
    Show,
    /// Show active permissions and info for the current API key
    Permissions,
    /// Remove API credentials from the config file
    Reset,
}

pub async fn run(args: AuthArgs, ctx: &AppContext, client: &BybitClient) -> BybitResult<()> {
    match args.command {
        AuthCommand::Set {
            api_key,
            api_secret,
        } => {
            let secret = resolve_secret_for_set(ctx, api_secret)?;

            let mut config = load_config()?;
            config.auth.api_key = Some(api_key.clone());
            config.auth.api_secret = Some(SecretValue::new(secret));
            save_config(&config)?;

            let result = serde_json::json!({
                "status": "credentials saved",
                "api_key": mask_key(&api_key),
                "source": "config",
            });
            print_output(&result, ctx.format);
        }

        AuthCommand::Sign { payload } => {
            let key = ctx
                .api_key
                .as_deref()
                .ok_or_else(|| BybitError::Auth("--api-key is required for sign".to_string()))?;
            let secret = ctx
                .api_secret
                .as_deref()
                .ok_or_else(|| BybitError::Auth("--api-secret is required for sign".to_string()))?;
            let payload = payload.unwrap_or_default();
            let ts = timestamp_ms();
            let recv_window = ctx.recv_window.unwrap_or(5000);
            let sig = sign(secret, ts, key, recv_window, &payload);
            let result = serde_json::json!({
                "api_key": key,
                "timestamp": ts,
                "recv_window": recv_window,
                "payload": payload,
                "signature": sig,
            });
            print_output(&result, ctx.format);
        }

        AuthCommand::Test => {
            let result = client.private_get("/v5/account/info", &[]).await?;
            let output = serde_json::json!({
                "status": "success",
                "account": result,
            });
            print_output(&output, ctx.format);
        }

        AuthCommand::Show => {
            let creds = resolve_credentials(None, None)?;
            let result = match creds {
                Some(c) => serde_json::json!({
                    "source": format!("{:?}", c.source).to_lowercase(),
                    "api_key": mask_key(&c.api_key),
                    "api_secret": "[REDACTED]",
                    "secret_set": true,
                }),
                None => serde_json::json!({
                    "source": "none",
                    "api_key": null,
                    "api_secret": null,
                    "secret_set": false,
                    "hint": "Run `bybit auth set`, `bybit setup`, or set BYBIT_API_KEY / BYBIT_API_SECRET",
                }),
            };
            print_output(&result, ctx.format);
        }

        AuthCommand::Permissions => {
            let mut result = client.private_get("/v5/user/query-api", &[]).await?;
            redact_api_key_fields(&mut result);
            print_output(&result, ctx.format);
        }

        AuthCommand::Reset => {
            let mut config = load_config()?;
            let had_creds = config.auth.api_key.is_some() || config.auth.api_secret.is_some();
            config.auth.api_key = None;
            config.auth.api_secret = None;
            save_config(&config)?;
            let result = serde_json::json!({
                "status": if had_creds { "credentials removed from config file" } else { "no credentials were stored" },
                "hint": "Set BYBIT_API_KEY / BYBIT_API_SECRET, run `bybit auth set`, or use `bybit setup` to reconfigure",
            });
            print_output(&result, ctx.format);
        }
    }
    Ok(())
}

fn resolve_secret_for_set(
    ctx: &AppContext,
    local_api_secret: Option<String>,
) -> BybitResult<String> {
    let secret = if ctx.api_secret_from_input {
        ctx.api_secret.clone().unwrap_or_default()
    } else if let Some(secret) = local_api_secret {
        eprintln!(
            "Warning: passing --api-secret on the command line exposes it in process listings. Prefer --api-secret-file, --api-secret-stdin, or `bybit setup`."
        );
        secret
    } else if let Ok(secret) = std::env::var("BYBIT_API_SECRET") {
        secret
    } else if ctx.mcp_mode {
        return Err(BybitError::Validation(
            "API secret is required for `bybit auth set` in non-interactive mode. Provide --api-secret, --api-secret-stdin, or --api-secret-file.".to_string(),
        ));
    } else {
        Password::new()
            .with_prompt("API Secret")
            .allow_empty_password(false)
            .interact()
            .map_err(|e| BybitError::Io(std::io::Error::other(e.to_string())))?
    };

    if secret.trim().is_empty() {
        return Err(BybitError::Auth("Cannot save an empty API secret.".into()));
    }

    Ok(secret)
}

fn mask_key(key: &str) -> String {
    if key.len() > 8 {
        format!("{}****{}", &key[..4], &key[key.len() - 4..])
    } else if !key.is_empty() {
        format!("{}****", &key[..key.len().min(4)])
    } else {
        "****".to_string()
    }
}

fn redact_api_key_fields(value: &mut Value) {
    match value {
        Value::Object(map) => {
            for (key, nested) in map {
                match key.as_str() {
                    "apiKey" | "api_key" => {
                        *nested = match nested.as_str() {
                            Some(key) => Value::String(mask_key(key)),
                            None => Value::String("[REDACTED]".to_string()),
                        };
                    }
                    "secret" | "apiSecret" | "api_secret" => {
                        *nested = Value::String("[REDACTED]".to_string());
                    }
                    _ => redact_api_key_fields(nested),
                }
            }
        }
        Value::Array(values) => {
            for nested in values {
                redact_api_key_fields(nested);
            }
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::redact_api_key_fields;

    #[test]
    fn redact_api_key_fields_masks_auth_permission_payload() {
        let mut value = json!({
            "apiKey": "abcd1234wxyz",
            "secret": "",
            "nested": {
                "api_key": "short",
                "apiSecret": "top-secret"
            }
        });

        redact_api_key_fields(&mut value);

        assert_eq!(value["apiKey"], "abcd****wxyz");
        assert_eq!(value["secret"], "[REDACTED]");
        assert_eq!(value["nested"]["api_key"], "shor****");
        assert_eq!(value["nested"]["apiSecret"], "[REDACTED]");
    }
}
