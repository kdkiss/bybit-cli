use clap::Subcommand;
use serde_json::{json, Value};

use crate::client::BybitClient;
use crate::commands::helpers::confirm;
use crate::errors::BybitResult;
use crate::output::{print_output, OutputFormat};

#[derive(Debug, clap::Args)]
pub struct ConvertArgs {
    #[command(subcommand)]
    pub command: ConvertCommand,
}

#[derive(Debug, Subcommand)]
pub enum ConvertCommand {
    /// List coins available for conversion
    Coins {
        /// Account type: UNIFIED, SPOT, CONTRACT, FUND
        #[arg(long, default_value = "UNIFIED")]
        account_type: String,
        /// Filter by coin
        #[arg(long)]
        coin: Option<String>,
        /// Side filter: 0 = all, 1 = from-coin list, 2 = to-coin list
        #[arg(long)]
        side: Option<u8>,
    },
    /// Request a conversion quote
    Quote {
        /// Account type performing the conversion
        #[arg(long, default_value = "UNIFIED")]
        account_type: String,
        /// Coin to convert from
        #[arg(long)]
        from_coin: String,
        /// Coin to convert to
        #[arg(long)]
        to_coin: String,
        /// Amount of from-coin to convert (mutually exclusive with to_amount)
        #[arg(
            long,
            conflicts_with = "to_amount",
            required_unless_present = "to_amount"
        )]
        from_amount: Option<String>,
        /// Amount of to-coin desired (mutually exclusive with from_amount)
        #[arg(
            long,
            conflicts_with = "from_amount",
            required_unless_present = "from_amount"
        )]
        to_amount: Option<String>,
        /// Request a quote without executing
        #[arg(long)]
        dry_run: bool,
    },
    /// Execute a previously obtained conversion quote
    Execute {
        /// Quote transaction ID returned by `convert quote`
        #[arg(long)]
        quote_tx_id: String,
    },
    /// Check the status of a conversion
    Status {
        /// Quote transaction ID
        #[arg(long)]
        quote_tx_id: String,
        /// Account type
        #[arg(long, default_value = "UNIFIED")]
        account_type: String,
    },
    /// Get conversion history
    History {
        /// Account type
        #[arg(long, default_value = "UNIFIED")]
        account_type: String,
        /// Filter by coin
        #[arg(long)]
        coin: Option<String>,
        /// Start time in milliseconds
        #[arg(long)]
        start: Option<u64>,
        /// End time in milliseconds
        #[arg(long)]
        end: Option<u64>,
        /// Page index (0-based, default 0)
        #[arg(long)]
        index: Option<u32>,
        /// Page size (default 20, max 100)
        #[arg(long)]
        limit: Option<u32>,
    },
}

pub async fn run(
    args: ConvertArgs,
    client: &BybitClient,
    format: OutputFormat,
    force: bool,
) -> BybitResult<()> {
    let value: Value = match args.command {
        ConvertCommand::Coins {
            account_type,
            coin,
            side,
        } => {
            let side_str = side.map(|s| s.to_string());
            let mut params = vec![("accountType", account_type.as_str())];
            if let Some(ref s) = side_str {
                params.push(("side", s));
            }
            if let Some(ref c) = coin {
                params.push(("coin", c));
            }
            client
                .private_get("/v5/asset/exchange/query-coin-list", &params)
                .await?
        }

        ConvertCommand::Quote {
            account_type,
            from_coin,
            to_coin,
            from_amount,
            to_amount,
            dry_run,
        } => {
            if dry_run {
                print_output(
                    &json!({
                        "dry_run": true,
                        "accountType": account_type,
                        "fromCoin": from_coin,
                        "toCoin": to_coin,
                        "fromAmount": from_amount,
                        "toAmount": to_amount,
                    }),
                    format,
                );
                return Ok(());
            }
            confirm(
                &format!("Request conversion quote: {from_coin} → {to_coin}?"),
                force,
            )?;
            let mut body = json!({
                "accountType": account_type,
                "fromCoin": from_coin,
                "toCoin": to_coin,
            });
            if let Some(ref amt) = from_amount {
                body["fromAmount"] = json!(amt);
            }
            if let Some(ref amt) = to_amount {
                body["toAmount"] = json!(amt);
            }
            client
                .private_post("/v5/asset/exchange/quote-apply", &body)
                .await?
        }

        ConvertCommand::Execute { quote_tx_id } => {
            confirm(
                &format!("Execute conversion quote {quote_tx_id}? This will convert your coins."),
                force,
            )?;
            let body = json!({ "quoteTxId": quote_tx_id });
            client
                .private_post("/v5/asset/exchange/convert-execute", &body)
                .await?
        }

        ConvertCommand::Status {
            quote_tx_id,
            account_type,
        } => {
            let params = vec![
                ("accountType", account_type.as_str()),
                ("quoteTxId", quote_tx_id.as_str()),
            ];
            client
                .private_get("/v5/asset/exchange/convert-result-query", &params)
                .await?
        }

        ConvertCommand::History {
            account_type,
            coin,
            start,
            end,
            index,
            limit,
        } => {
            let start_str = start.map(|s| s.to_string());
            let end_str = end.map(|e| e.to_string());
            let index_str = index.map(|i| i.to_string());
            let limit_str = limit.map(|l| l.to_string());
            let mut params = vec![("accountType", account_type.as_str())];
            if let Some(ref c) = coin {
                params.push(("coin", c));
            }
            if let Some(ref s) = start_str {
                params.push(("startTime", s));
            }
            if let Some(ref s) = end_str {
                params.push(("endTime", s));
            }
            if let Some(ref s) = index_str {
                params.push(("index", s));
            }
            if let Some(ref s) = limit_str {
                params.push(("limit", s));
            }
            client
                .private_get("/v5/asset/exchange/query-convert-history", &params)
                .await?
        }
    };

    print_output(&value, format);
    Ok(())
}
