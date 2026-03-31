use clap::Subcommand;
use serde_json::{json, Value};

use crate::client::BybitClient;
use crate::commands::helpers::confirm;
use crate::errors::BybitResult;
use crate::output::{print_output, OutputFormat};

#[derive(Debug, clap::Args)]
pub struct AccountArgs {
    #[command(subcommand)]
    pub command: AccountCommand,
}

#[derive(Debug, Subcommand)]
pub enum AccountCommand {
    /// Get wallet balances
    Balance {
        /// UNIFIED, CONTRACT, SPOT, OPTION, INVESTMENT, FUND, COPYTRADING
        #[arg(long, default_value = "UNIFIED")]
        account_type: String,
        #[arg(long)]
        coin: Option<String>,
        /// Show extended margin and risk fields
        #[arg(long)]
        extended: bool,
    },
    /// Get extended per-coin balances across the account
    ExtendedBalance {
        #[arg(long, default_value = "UNIFIED")]
        account_type: String,
        #[arg(long)]
        coin: Option<String>,
        #[arg(long)]
        member_id: Option<String>,
    },
    /// Get account information
    Info,
    /// Get fee rates
    FeeRate {
        #[arg(long, default_value = "linear")]
        category: String,
        #[arg(long)]
        symbol: Option<String>,
        #[arg(long)]
        base_coin: Option<String>,
    },
    /// Get transaction log (UTA)
    TransactionLog {
        #[arg(long, default_value = "UNIFIED")]
        account_type: String,
        #[arg(long)]
        category: Option<String>,
        #[arg(long)]
        currency: Option<String>,
        #[arg(long)]
        base_coin: Option<String>,
        #[arg(long, name = "type")]
        tx_type: Option<String>,
        #[arg(long)]
        start: Option<u64>,
        #[arg(long)]
        end: Option<u64>,
        #[arg(long)]
        limit: Option<u32>,
        #[arg(long)]
        cursor: Option<String>,
    },
    /// Get borrow history
    BorrowHistory {
        #[arg(long)]
        currency: Option<String>,
        #[arg(long)]
        start: Option<u64>,
        #[arg(long)]
        end: Option<u64>,
        #[arg(long)]
        limit: Option<u32>,
        #[arg(long)]
        cursor: Option<String>,
    },
    /// Get collateral info
    CollateralInfo {
        #[arg(long)]
        currency: Option<String>,
    },
    /// Get option greeks
    Greeks {
        #[arg(long)]
        base_coin: Option<String>,
    },
    /// Set margin mode (dangerous)
    SetMarginMode {
        /// ISOLATED_MARGIN, REGULAR_MARGIN, PORTFOLIO_MARGIN
        #[arg(long)]
        margin_mode: String,
    },
    /// Set spot hedging mode (dangerous)
    SetSpotHedging {
        /// ON or OFF
        #[arg(long)]
        mode: String,
    },
    /// Set USDC settlement coin (USDC or USDT) for UTA
    SetUsdcSettlement {
        /// USDC or USDT
        #[arg(long)]
        coin: String,
    },
    /// Get trading volume for a period (e.g. last 30 days)
    Volume {
        #[arg(long, default_value = "linear")]
        category: String,
        #[arg(long)]
        symbol: Option<String>,
        #[arg(long)]
        base_coin: Option<String>,
        /// Lookback days (default 30)
        #[arg(long, default_value = "30")]
        days: u32,
    },
    /// Get ADL (Auto-Deleveraging) risk alerts (Private)
    AdlAlert {
        #[arg(long, default_value = "linear")]
        category: String,
        #[arg(long)]
        symbol: Option<String>,
    },
    /// Manually borrow funds (UTA Pro / Portfolio Margin)
    Borrow {
        /// Coin to borrow, e.g. USDT
        #[arg(long)]
        coin: String,
        /// Amount to borrow
        #[arg(long)]
        amount: String,
    },
    /// Manually repay borrowed funds
    Repay {
        /// Coin to repay, e.g. USDT
        #[arg(long)]
        coin: String,
        /// Amount to repay
        #[arg(long)]
        amount: String,
    },
    /// Quick-repay liability (auto-select coin and amount)
    QuickRepay {
        /// Coin to repay; omit to auto-select
        #[arg(long)]
        coin: Option<String>,
    },
}

pub async fn run(
    args: AccountArgs,
    client: &BybitClient,
    format: OutputFormat,
    force: bool,
) -> BybitResult<()> {
    let value: Value = match args.command {
        AccountCommand::Balance {
            account_type,
            coin,
            extended: _,
        } => {
            let mut params = vec![("accountType", account_type.as_str())];
            // coin needs to live long enough
            let coin_ref = coin.clone();
            if let Some(ref c) = coin_ref {
                params.push(("coin", c));
            }
            client
                .private_get("/v5/account/wallet-balance", &params)
                .await?
        }

        AccountCommand::ExtendedBalance {
            account_type,
            coin,
            member_id,
        } => {
            let mut params = vec![("accountType", account_type.as_str())];
            if let Some(ref coin) = coin {
                params.push(("coin", coin));
            }
            if let Some(ref member_id) = member_id {
                params.push(("memberId", member_id));
            }
            client
                .private_get("/v5/asset/transfer/query-account-coins-balance", &params)
                .await?
        }

        AccountCommand::Info => client.private_get("/v5/account/info", &[]).await?,

        AccountCommand::FeeRate {
            category,
            symbol,
            base_coin,
        } => {
            let mut params = vec![("category", category.as_str())];
            if let Some(ref s) = symbol {
                params.push(("symbol", s));
            }
            if let Some(ref s) = base_coin {
                params.push(("baseCoin", s));
            }
            client.private_get("/v5/account/fee-rate", &params).await?
        }

        AccountCommand::TransactionLog {
            account_type,
            category,
            currency,
            base_coin,
            tx_type,
            start,
            end,
            limit,
            cursor,
        } => {
            let start_str = start.map(|s| s.to_string());
            let end_str = end.map(|e| e.to_string());
            let limit_str = limit.map(|l| l.to_string());
            let mut params = vec![("accountType", account_type.as_str())];
            if let Some(ref s) = category {
                params.push(("category", s));
            }
            if let Some(ref s) = currency {
                params.push(("currency", s));
            }
            if let Some(ref s) = base_coin {
                params.push(("baseCoin", s));
            }
            if let Some(ref s) = tx_type {
                params.push(("type", s));
            }
            if let Some(ref s) = start_str {
                params.push(("startTime", s));
            }
            if let Some(ref s) = end_str {
                params.push(("endTime", s));
            }
            if let Some(ref s) = limit_str {
                params.push(("limit", s));
            }
            if let Some(ref s) = cursor {
                params.push(("cursor", s));
            }
            client
                .private_get("/v5/account/transaction-log", &params)
                .await?
        }

        AccountCommand::BorrowHistory {
            currency,
            start,
            end,
            limit,
            cursor,
        } => {
            let start_str = start.map(|s| s.to_string());
            let end_str = end.map(|e| e.to_string());
            let limit_str = limit.map(|l| l.to_string());
            let mut params: Vec<(&str, &str)> = vec![];
            if let Some(ref s) = currency {
                params.push(("currency", s));
            }
            if let Some(ref s) = start_str {
                params.push(("startTime", s));
            }
            if let Some(ref s) = end_str {
                params.push(("endTime", s));
            }
            if let Some(ref s) = limit_str {
                params.push(("limit", s));
            }
            if let Some(ref s) = cursor {
                params.push(("cursor", s));
            }
            client
                .private_get("/v5/account/borrow-history", &params)
                .await?
        }

        AccountCommand::CollateralInfo { currency } => {
            let mut params: Vec<(&str, &str)> = vec![];
            if let Some(ref c) = currency {
                params.push(("currency", c));
            }
            client
                .private_get("/v5/account/collateral-info", &params)
                .await?
        }

        AccountCommand::Greeks { base_coin } => {
            let mut params: Vec<(&str, &str)> = vec![];
            if let Some(ref s) = base_coin {
                params.push(("baseCoin", s));
            }
            client.private_get("/v5/asset/coin-greeks", &params).await?
        }

        AccountCommand::SetMarginMode { margin_mode } => {
            confirm(&format!("Set margin mode to {margin_mode}?"), force)?;
            let body = json!({ "setMarginMode": margin_mode });
            client
                .private_post("/v5/account/set-margin-mode", &body)
                .await?
        }

        AccountCommand::SetSpotHedging { mode } => {
            confirm(&format!("Set spot hedging to {mode}?"), force)?;
            let body = json!({ "setHedgingMode": mode });
            client
                .private_post("/v5/account/set-hedging-mode", &body)
                .await?
        }

        AccountCommand::SetUsdcSettlement { coin } => {
            confirm(&format!("Set USDC settlement coin to {coin}?"), force)?;
            let body = json!({ "settlementCoin": coin });
            client
                .private_post("/v5/account/set-usdc-settlement-mode", &body)
                .await?
        }

        AccountCommand::Volume {
            category,
            symbol,
            base_coin,
            days,
        } => {
            let now = crate::auth::timestamp_ms();
            let start_time = now - (days as u64 * 24 * 60 * 60 * 1000);
            let start_time_str = start_time.to_string();

            let mut params = vec![
                ("category", category.as_str()),
                ("startTime", start_time_str.as_str()),
                ("limit", "100"),
            ];

            let symbol_str = symbol.clone();
            if let Some(ref s) = symbol_str {
                params.push(("symbol", s));
            }

            let base_coin_str = base_coin.clone();
            if let Some(ref c) = base_coin_str {
                params.push(("baseCoin", c));
            }

            eprintln!("Calculating volume for the last {days} days in {category}...");

            let mut total_volume = 0.0;
            let mut cursor: Option<String> = None;
            let mut page = 1;

            loop {
                let mut current_params = params.clone();
                let cursor_str;
                if let Some(ref c) = cursor {
                    cursor_str = c.clone();
                    current_params.push(("cursor", &cursor_str));
                }

                let res = client
                    .private_get("/v5/execution/list", &current_params)
                    .await?;
                let list = res["list"].as_array().ok_or_else(|| {
                    crate::errors::BybitError::Parse("Invalid execution list response".to_string())
                })?;

                if list.is_empty() {
                    break;
                }

                for exec in list {
                    let val_str = exec["execValue"].as_str().unwrap_or("0");
                    let val: f64 = val_str.parse().unwrap_or(0.0);
                    total_volume += val;
                }

                eprintln!("Processed page {page}, current total: ${:.2}", total_volume);

                let next_cursor = res["nextPageCursor"].as_str().unwrap_or("");
                if next_cursor.is_empty() {
                    break;
                }

                cursor = Some(next_cursor.to_string());
                page += 1;

                // Safety break to prevent infinite loops on massive histories
                if page > 50 {
                    eprintln!("Reached max pages (50). Result may be partial.");
                    break;
                }
            }

            json!({
                "category": category,
                "days": days,
                "totalVolume": total_volume,
                "currency": "USD"
            })
        }

        AccountCommand::AdlAlert { category, symbol } => {
            let _ = category;
            let mut params: Vec<(&str, &str)> = vec![];
            if let Some(ref s) = symbol {
                params.push(("symbol", s));
            }
            client.private_get("/v5/market/adlAlert", &params).await?
        }

        AccountCommand::Borrow { coin, amount } => {
            confirm(&format!("Borrow {amount} {coin}?"), force)?;
            let body = json!({ "coin": coin, "qty": amount });
            client
                .private_post("/v5/account/manual-borrow", &body)
                .await?
        }

        AccountCommand::Repay { coin, amount } => {
            confirm(&format!("Repay {amount} {coin}?"), force)?;
            let body = json!({ "coin": coin, "qty": amount });
            client
                .private_post("/v5/account/manual-repay", &body)
                .await?
        }

        AccountCommand::QuickRepay { coin } => {
            let msg = match &coin {
                Some(c) => format!("Quick-repay liability for {c}?"),
                None => "Quick-repay all liabilities?".to_string(),
            };
            confirm(&msg, force)?;
            let mut body = json!({});
            if let Some(c) = coin {
                body["coin"] = json!(c);
            }
            client
                .private_post("/v5/account/quick-repayment", &body)
                .await?
        }
    };

    print_output(&value, format);
    Ok(())
}
