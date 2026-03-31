use clap::Subcommand;
use serde_json::{json, Value};

use crate::client::BybitClient;
use crate::commands::helpers::confirm;
use crate::errors::BybitResult;
use crate::output::{print_output, OutputFormat};

#[derive(Debug, clap::Args)]
pub struct MarginArgs {
    #[command(subcommand)]
    pub command: MarginCommand,
}

#[derive(Debug, Subcommand)]
pub enum MarginCommand {
    /// Get VIP margin data for a unified account coin
    VipData {
        /// VIP tier label, e.g. "No VIP"
        #[arg(long)]
        vip_level: Option<String>,
        /// Coin name, uppercase only
        #[arg(long)]
        currency: Option<String>,
    },
    /// Toggle spot margin on or off
    Toggle {
        /// on or off
        #[arg(long)]
        mode: String,
    },
    /// Set the maximum spot cross-margin leverage
    SetLeverage {
        /// Leverage, supports 2-10
        #[arg(long)]
        leverage: String,
        /// Optional coin-specific leverage override
        #[arg(long)]
        currency: Option<String>,
    },
    /// Get the current spot margin state and leverage
    Status,
}

pub async fn run(
    args: MarginArgs,
    client: &BybitClient,
    format: OutputFormat,
    force: bool,
) -> BybitResult<()> {
    let value: Value = match args.command {
        MarginCommand::VipData {
            vip_level,
            currency,
        } => {
            let mut params = vec![];
            if let Some(ref vip_level) = vip_level {
                params.push(("vipLevel", vip_level.as_str()));
            }
            if let Some(ref currency) = currency {
                params.push(("currency", currency.as_str()));
            }
            client
                .public_get("/v5/spot-margin-trade/data", &params)
                .await?
        }
        MarginCommand::Toggle { mode } => {
            let normalized = mode.to_ascii_lowercase();
            let spot_margin_mode = match normalized.as_str() {
                "on" => "1",
                "off" => "0",
                _ => {
                    return Err(crate::errors::BybitError::Validation(
                        "mode must be 'on' or 'off'".to_string(),
                    ));
                }
            };
            let action = if spot_margin_mode == "1" {
                "Enable"
            } else {
                "Disable"
            };
            confirm(&format!("{action} spot margin trading?"), force)?;
            let body = json!({ "spotMarginMode": spot_margin_mode });
            client
                .private_post("/v5/spot-margin-trade/switch-mode", &body)
                .await?
        }
        MarginCommand::SetLeverage { leverage, currency } => {
            confirm(&format!("Set spot margin leverage to {leverage}x?"), force)?;
            let mut body = json!({ "leverage": leverage });
            if let Some(currency) = currency {
                body["currency"] = json!(currency);
            }
            client
                .private_post("/v5/spot-margin-trade/set-leverage", &body)
                .await?
        }
        MarginCommand::Status => {
            client
                .private_get("/v5/spot-margin-trade/state", &[])
                .await?
        }
    };

    print_output(&value, format);
    Ok(())
}
