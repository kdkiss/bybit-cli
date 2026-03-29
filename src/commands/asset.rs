use clap::Subcommand;
use serde_json::{json, Value};

use crate::client::BybitClient;
use crate::commands::helpers::confirm;
use crate::errors::BybitResult;
use crate::output::{print_output, OutputFormat};

#[derive(Debug, clap::Args)]
pub struct AssetArgs {
    #[command(subcommand)]
    pub command: AssetCommand,
}

#[derive(Debug, Subcommand)]
pub enum AssetCommand {
    /// Query coin info (networks, min/max withdraw, etc.)
    CoinInfo {
        #[arg(long)]
        coin: Option<String>,
    },
    /// Alias for CoinInfo — shows available networks and fees
    WithdrawalMethods {
        #[arg(long)]
        coin: Option<String>,
    },
    /// Query asset balance by account type
    Balance {
        #[arg(long, default_value = "SPOT")]
        account_type: String,
        #[arg(long)]
        coin: Option<String>,
    },
    /// Query all coins balance across account types
    AllBalance {
        #[arg(long, default_value = "UNIFIED")]
        account_type: String,
        #[arg(long)]
        coin: Option<String>,
        #[arg(long)]
        member_id: Option<String>,
    },
    /// Get list of transferable coins between two account types
    Transferable {
        #[arg(long)]
        from_account_type: String,
        #[arg(long)]
        to_account_type: String,
    },
    /// Transfer assets between account types (same UID)
    Transfer {
        #[arg(long)]
        coin: String,
        #[arg(long)]
        amount: String,
        #[arg(long)]
        from_account_type: String,
        #[arg(long)]
        to_account_type: String,
    },
    /// Get internal transfer history
    TransferHistory {
        #[arg(long)]
        coin: Option<String>,
        #[arg(long)]
        status: Option<String>,
        #[arg(long)]
        start: Option<u64>,
        #[arg(long)]
        end: Option<u64>,
        #[arg(long)]
        limit: Option<u32>,
        #[arg(long)]
        cursor: Option<String>,
    },
    /// Universal transfer (cross-UID)
    SubTransfer {
        #[arg(long)]
        coin: String,
        #[arg(long)]
        amount: String,
        #[arg(long)]
        from_member_id: String,
        #[arg(long)]
        to_member_id: String,
        #[arg(long)]
        from_account_type: String,
        #[arg(long)]
        to_account_type: String,
    },
    /// Get deposit address for a coin
    DepositAddress {
        #[arg(long)]
        coin: String,
        #[arg(long)]
        chain_type: Option<String>,
    },
    /// Get deposit history
    DepositHistory {
        #[arg(long)]
        coin: Option<String>,
        #[arg(long)]
        start: Option<u64>,
        #[arg(long)]
        end: Option<u64>,
        #[arg(long)]
        limit: Option<u32>,
        #[arg(long)]
        cursor: Option<String>,
    },
    /// Withdraw coin to external address (dangerous)
    Withdraw {
        #[arg(long)]
        coin: String,
        #[arg(long)]
        chain: String,
        #[arg(long)]
        address: String,
        #[arg(long)]
        amount: String,
        #[arg(long)]
        tag: Option<String>,
        #[arg(long, default_value = "UNIFIED")]
        account_type: String,
    },
    /// Get withdrawal history
    WithdrawHistory {
        #[arg(long)]
        coin: Option<String>,
        #[arg(long)]
        withdraw_id: Option<String>,
        #[arg(long)]
        withdraw_type: Option<u8>,
        #[arg(long)]
        start: Option<u64>,
        #[arg(long)]
        end: Option<u64>,
        #[arg(long)]
        limit: Option<u32>,
        #[arg(long)]
        cursor: Option<String>,
    },
    /// Cancel a pending withdrawal
    CancelWithdraw {
        #[arg(long)]
        id: String,
    },
    /// Query a single coin balance for any account type
    AccountBalance {
        #[arg(long, default_value = "UNIFIED")]
        account_type: String,
        #[arg(long)]
        member_id: Option<String>,
        #[arg(long)]
        coin: String,
        #[arg(long, default_value = "false")]
        with_bonus: bool,
    },
    /// Universal transfer history (cross-UID)
    SubTransferHistory {
        #[arg(long)]
        coin: Option<String>,
        #[arg(long)]
        status: Option<String>,
        #[arg(long)]
        start: Option<u64>,
        #[arg(long)]
        end: Option<u64>,
        #[arg(long)]
        limit: Option<u32>,
        #[arg(long)]
        cursor: Option<String>,
    },
}

pub async fn run(
    args: AssetArgs,
    client: &BybitClient,
    format: OutputFormat,
    force: bool,
) -> BybitResult<()> {
    let value: Value = match args.command {
        AssetCommand::CoinInfo { coin } => {
            let mut params: Vec<(&str, &str)> = vec![];
            if let Some(ref c) = coin {
                params.push(("coin", c));
            }
            client
                .private_get("/v5/asset/coin/query-info", &params)
                .await?
        }

        AssetCommand::WithdrawalMethods { coin } => {
            let mut params: Vec<(&str, &str)> = vec![];
            if let Some(ref c) = coin {
                params.push(("coin", c));
            }
            client
                .private_get("/v5/asset/coin/query-info", &params)
                .await?
        }

        AssetCommand::Balance { account_type, coin } => {
            let mut params = vec![("accountType", account_type.as_str())];
            if let Some(ref c) = coin {
                params.push(("coin", c));
            }
            client
                .private_get("/v5/asset/transfer/query-asset-info", &params)
                .await?
        }

        AssetCommand::AllBalance {
            account_type,
            coin,
            member_id,
        } => {
            let mut params = vec![("accountType", account_type.as_str())];
            if let Some(ref c) = coin {
                params.push(("coin", c));
            }
            if let Some(ref m) = member_id {
                params.push(("memberId", m));
            }
            client
                .private_get("/v5/asset/transfer/query-account-coins-balance", &params)
                .await?
        }

        AssetCommand::Transferable {
            from_account_type,
            to_account_type,
        } => {
            let params = vec![
                ("fromAccountType", from_account_type.as_str()),
                ("toAccountType", to_account_type.as_str()),
            ];
            client
                .private_get("/v5/asset/transfer/query-transfer-coin-list", &params)
                .await?
        }

        AssetCommand::Transfer {
            coin,
            amount,
            from_account_type,
            to_account_type,
        } => {
            confirm(
                &format!("Transfer {amount} {coin} from {from_account_type} to {to_account_type}?"),
                force,
            )?;
            let body = json!({
                "transferId": uuid::Uuid::new_v4().to_string(),
                "coin": coin,
                "amount": amount,
                "fromAccountType": from_account_type,
                "toAccountType": to_account_type,
            });
            client
                .private_post("/v5/asset/transfer/inter-transfer", &body)
                .await?
        }

        AssetCommand::TransferHistory {
            coin,
            status,
            start,
            end,
            limit,
            cursor,
        } => {
            let start_str = start.map(|s| s.to_string());
            let end_str = end.map(|e| e.to_string());
            let limit_str = limit.map(|l| l.to_string());
            let mut params: Vec<(&str, &str)> = vec![];
            if let Some(ref c) = coin {
                params.push(("coin", c));
            }
            if let Some(ref s) = status {
                params.push(("status", s));
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
                .private_get("/v5/asset/transfer/query-inter-transfer-list", &params)
                .await?
        }

        AssetCommand::SubTransfer {
            coin,
            amount,
            from_member_id,
            to_member_id,
            from_account_type,
            to_account_type,
        } => {
            confirm(
                &format!("Universal transfer {amount} {coin} from UID {from_member_id} to UID {to_member_id}?"),
                force,
            )?;
            let body = json!({
                "transferId": uuid::Uuid::new_v4().to_string(),
                "coin": coin,
                "amount": amount,
                "fromMemberId": from_member_id,
                "toMemberId": to_member_id,
                "fromAccountType": from_account_type,
                "toAccountType": to_account_type,
            });
            client
                .private_post("/v5/asset/transfer/universal-transfer", &body)
                .await?
        }

        AssetCommand::DepositAddress { coin, chain_type } => {
            let mut params = vec![("coin", coin.as_str())];
            if let Some(ref c) = chain_type {
                params.push(("chainType", c));
            }
            client
                .private_get("/v5/asset/deposit/query-address", &params)
                .await?
        }

        AssetCommand::DepositHistory {
            coin,
            start,
            end,
            limit,
            cursor,
        } => {
            let start_str = start.map(|s| s.to_string());
            let end_str = end.map(|e| e.to_string());
            let limit_str = limit.map(|l| l.to_string());
            let mut params: Vec<(&str, &str)> = vec![];
            if let Some(ref c) = coin {
                params.push(("coin", c));
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
                .private_get("/v5/asset/deposit/query-record", &params)
                .await?
        }

        AssetCommand::Withdraw {
            coin,
            chain,
            address,
            amount,
            tag,
            account_type,
        } => {
            confirm(
                &format!("WITHDRAW {amount} {coin} ({chain}) to {address}?"),
                force,
            )?;
            let mut body = json!({
                "coin": coin,
                "chain": chain,
                "address": address,
                "amount": amount,
                "timestamp": crate::auth::timestamp_ms(),
                "accountType": account_type,
            });
            if let Some(t) = tag {
                body["tag"] = json!(t);
            }
            client
                .private_post("/v5/asset/withdraw/create", &body)
                .await?
        }

        AssetCommand::WithdrawHistory {
            coin,
            withdraw_id,
            withdraw_type,
            start,
            end,
            limit,
            cursor,
        } => {
            let start_str = start.map(|s| s.to_string());
            let end_str = end.map(|e| e.to_string());
            let limit_str = limit.map(|l| l.to_string());
            let wtype_str = withdraw_type.map(|w| w.to_string());
            let mut params: Vec<(&str, &str)> = vec![];
            if let Some(ref c) = coin {
                params.push(("coin", c));
            }
            if let Some(ref s) = withdraw_id {
                params.push(("withdrawID", s));
            }
            if let Some(ref s) = wtype_str {
                params.push(("withdrawType", s));
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
                .private_get("/v5/asset/withdraw/query-record", &params)
                .await?
        }

        AssetCommand::CancelWithdraw { id } => {
            confirm(&format!("Cancel withdrawal {id}?"), force)?;
            let body = json!({ "id": id });
            client
                .private_post("/v5/asset/withdraw/cancel", &body)
                .await?
        }

        AssetCommand::AccountBalance {
            account_type,
            member_id,
            coin,
            with_bonus,
        } => {
            let with_bonus_str = (with_bonus as u8).to_string();
            let mut params = vec![
                ("accountType", account_type.as_str()),
                ("coin", coin.as_str()),
                ("withBonus", with_bonus_str.as_str()),
            ];
            if let Some(ref m) = member_id {
                params.push(("memberId", m));
            }
            client
                .private_get("/v5/asset/transfer/query-account-coin-balance", &params)
                .await?
        }

        AssetCommand::SubTransferHistory {
            coin,
            status,
            start,
            end,
            limit,
            cursor,
        } => {
            let start_str = start.map(|s| s.to_string());
            let end_str = end.map(|e| e.to_string());
            let limit_str = limit.map(|l| l.to_string());
            let mut params: Vec<(&str, &str)> = vec![];
            if let Some(ref c) = coin {
                params.push(("coin", c));
            }
            if let Some(ref s) = status {
                params.push(("status", s));
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
                .private_get("/v5/asset/transfer/query-universal-transfer-list", &params)
                .await?
        }
    };

    print_output(&value, format);
    Ok(())
}
