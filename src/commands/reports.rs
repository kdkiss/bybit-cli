use std::path::{Component, Path, PathBuf};
use std::time::Duration;

use clap::Subcommand;
use reqwest::ClientBuilder;
use serde::Deserialize;
use serde_json::{json, Value};

use crate::client::BybitClient;
use crate::commands::{
    account::{run as run_account, AccountArgs, AccountCommand},
    asset::{run as run_asset, AssetArgs, AssetCommand},
    position::{run as run_position, PositionArgs, PositionCommand},
    trade::{run as run_trade, TradeArgs, TradeCommand},
};
use crate::errors::{BybitError, BybitResult};
use crate::output::{print_output, OutputFormat};

const MAX_EXPORT_RANGE_SECONDS: u64 = 60 * 24 * 60 * 60;

#[derive(Debug, clap::Args)]
pub struct ReportsArgs {
    #[command(subcommand)]
    pub command: ReportsCommand,
}

#[derive(Debug, Subcommand)]
pub enum ReportsCommand {
    /// Account transaction log / ledger-style history
    Transactions {
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
    /// Borrow history
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
    /// Order history
    Orders {
        #[arg(long, default_value = "linear")]
        category: String,
        #[arg(long)]
        symbol: Option<String>,
        #[arg(long)]
        order_id: Option<String>,
        #[arg(long)]
        order_status: Option<String>,
        #[arg(long)]
        start: Option<u64>,
        #[arg(long)]
        end: Option<u64>,
        #[arg(long)]
        limit: Option<u32>,
        #[arg(long)]
        cursor: Option<String>,
    },
    /// Execution / fill history
    Fills {
        #[arg(long, default_value = "linear")]
        category: String,
        #[arg(long)]
        symbol: Option<String>,
        #[arg(long)]
        order_id: Option<String>,
        #[arg(long)]
        start: Option<u64>,
        #[arg(long)]
        end: Option<u64>,
        #[arg(long)]
        exec_type: Option<String>,
        #[arg(long)]
        limit: Option<u32>,
        #[arg(long)]
        cursor: Option<String>,
    },
    /// Closed PnL history
    ClosedPnl {
        #[arg(long, default_value = "linear")]
        category: String,
        #[arg(long)]
        symbol: Option<String>,
        #[arg(long)]
        start: Option<u64>,
        #[arg(long)]
        end: Option<u64>,
        #[arg(long)]
        limit: Option<u32>,
        #[arg(long)]
        cursor: Option<String>,
    },
    /// Position move history
    Moves {
        #[arg(long, default_value = "linear")]
        category: String,
        #[arg(long)]
        symbol: Option<String>,
        #[arg(long)]
        start: Option<u64>,
        #[arg(long)]
        end: Option<u64>,
        #[arg(long)]
        limit: Option<u32>,
        #[arg(long)]
        cursor: Option<String>,
    },
    /// Deposit history
    Deposits {
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
    /// Withdrawal history
    Withdrawals {
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
    /// Internal transfer history
    Transfers {
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
    /// Universal transfer history
    SubTransfers {
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
    /// Get the Bybit tax API register time for the current account
    RegisterTime,
    /// Request a Bybit tax export report
    ExportRequest {
        /// TRADE, P&L, EARN, DEPOSIT&WITHDRAWAL, BONUS, AIRDROP
        #[arg(long = "report-type")]
        report_type: String,
        /// Report number for the chosen type, per Bybit's tax enums
        #[arg(long = "report-number")]
        report_number: String,
        /// Start time in UNIX seconds
        #[arg(long)]
        start: u64,
        /// End time in UNIX seconds
        #[arg(long)]
        end: u64,
    },
    /// Check a Bybit tax export report status
    ExportStatus {
        #[arg(long)]
        query_id: String,
    },
    /// Retrieve a Bybit tax export manifest, and optionally download the files
    ExportRetrieve {
        #[arg(long)]
        query_id: String,
        /// Directory to download the export files into
        #[arg(long)]
        download_dir: Option<PathBuf>,
    },
}

#[derive(Debug, Deserialize)]
struct ExportLocator {
    #[serde(rename = "Basepath")]
    basepath: String,
    #[serde(rename = "Files")]
    files: Vec<String>,
}

pub async fn run(
    args: ReportsArgs,
    client: &BybitClient,
    format: OutputFormat,
    force: bool,
) -> BybitResult<()> {
    match args.command {
        ReportsCommand::Transactions {
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
            run_account(
                AccountArgs {
                    command: AccountCommand::TransactionLog {
                        account_type,
                        category,
                        currency,
                        base_coin,
                        tx_type,
                        start,
                        end,
                        limit,
                        cursor,
                    },
                },
                client,
                format,
                force,
            )
            .await
        }
        ReportsCommand::BorrowHistory {
            currency,
            start,
            end,
            limit,
            cursor,
        } => {
            run_account(
                AccountArgs {
                    command: AccountCommand::BorrowHistory {
                        currency,
                        start,
                        end,
                        limit,
                        cursor,
                    },
                },
                client,
                format,
                force,
            )
            .await
        }
        ReportsCommand::Orders {
            category,
            symbol,
            order_id,
            order_status,
            start,
            end,
            limit,
            cursor,
        } => {
            run_trade(
                TradeArgs {
                    command: TradeCommand::History {
                        category,
                        symbol,
                        order_id,
                        order_status,
                        start,
                        end,
                        limit,
                        cursor,
                    },
                },
                client,
                format,
                force,
            )
            .await
        }
        ReportsCommand::Fills {
            category,
            symbol,
            order_id,
            start,
            end,
            exec_type,
            limit,
            cursor,
        } => {
            run_trade(
                TradeArgs {
                    command: TradeCommand::Fills {
                        category,
                        symbol,
                        order_id,
                        start,
                        end,
                        exec_type,
                        limit,
                        cursor,
                    },
                },
                client,
                format,
                force,
            )
            .await
        }
        ReportsCommand::ClosedPnl {
            category,
            symbol,
            start,
            end,
            limit,
            cursor,
        } => {
            run_position(
                PositionArgs {
                    command: PositionCommand::ClosedPnl {
                        category,
                        symbol,
                        start,
                        end,
                        limit,
                        cursor,
                    },
                },
                client,
                format,
                force,
            )
            .await
        }
        ReportsCommand::Moves {
            category,
            symbol,
            start,
            end,
            limit,
            cursor,
        } => {
            run_position(
                PositionArgs {
                    command: PositionCommand::MoveHistory {
                        category,
                        symbol,
                        start,
                        end,
                        limit,
                        cursor,
                    },
                },
                client,
                format,
                force,
            )
            .await
        }
        ReportsCommand::Deposits {
            coin,
            start,
            end,
            limit,
            cursor,
        } => {
            run_asset(
                AssetArgs {
                    command: AssetCommand::DepositHistory {
                        coin,
                        start,
                        end,
                        limit,
                        cursor,
                    },
                },
                client,
                format,
                force,
            )
            .await
        }
        ReportsCommand::Withdrawals {
            coin,
            withdraw_id,
            withdraw_type,
            start,
            end,
            limit,
            cursor,
        } => {
            run_asset(
                AssetArgs {
                    command: AssetCommand::WithdrawHistory {
                        coin,
                        withdraw_id,
                        withdraw_type,
                        start,
                        end,
                        limit,
                        cursor,
                    },
                },
                client,
                format,
                force,
            )
            .await
        }
        ReportsCommand::Transfers {
            coin,
            status,
            start,
            end,
            limit,
            cursor,
        } => {
            run_asset(
                AssetArgs {
                    command: AssetCommand::TransferHistory {
                        coin,
                        status,
                        start,
                        end,
                        limit,
                        cursor,
                    },
                },
                client,
                format,
                force,
            )
            .await
        }
        ReportsCommand::SubTransfers {
            coin,
            status,
            start,
            end,
            limit,
            cursor,
        } => {
            run_asset(
                AssetArgs {
                    command: AssetCommand::SubTransferHistory {
                        coin,
                        status,
                        start,
                        end,
                        limit,
                        cursor,
                    },
                },
                client,
                format,
                force,
            )
            .await
        }
        ReportsCommand::RegisterTime => {
            let value = client
                .private_post("/fht/compliance/tax/v3/private/registertime", &json!({}))
                .await?;
            print_output(&value, format);
            Ok(())
        }
        ReportsCommand::ExportRequest {
            report_type,
            report_number,
            start,
            end,
        } => {
            validate_export_range(start, end)?;
            let value = client
                .private_post(
                    "/fht/compliance/tax/v3/private/create",
                    &json!({
                        "type": report_type,
                        "number": report_number,
                        "startTime": start.to_string(),
                        "endTime": end.to_string(),
                    }),
                )
                .await?;
            print_output(&value, format);
            Ok(())
        }
        ReportsCommand::ExportStatus { query_id } => {
            let value = client
                .private_post(
                    "/fht/compliance/tax/v3/private/status",
                    &json!({ "queryId": query_id }),
                )
                .await?;
            print_output(&value, format);
            Ok(())
        }
        ReportsCommand::ExportRetrieve {
            query_id,
            download_dir,
        } => {
            let value = retrieve_export(client, &query_id, download_dir.as_deref()).await?;
            print_output(&value, format);
            Ok(())
        }
    }
}

fn validate_export_range(start: u64, end: u64) -> BybitResult<()> {
    if end <= start {
        return Err(BybitError::Validation(
            "`--end` must be greater than `--start` for tax exports.".to_string(),
        ));
    }

    if end - start > MAX_EXPORT_RANGE_SECONDS {
        return Err(BybitError::Validation(
            "Bybit tax exports only allow up to 60 days per request.".to_string(),
        ));
    }

    Ok(())
}

async fn retrieve_export(
    client: &BybitClient,
    query_id: &str,
    download_dir: Option<&Path>,
) -> BybitResult<Value> {
    let response = client
        .private_post(
            "/fht/compliance/tax/v3/private/url",
            &json!({ "queryId": query_id }),
        )
        .await?;

    let locator_raw = response
        .get("url")
        .and_then(Value::as_str)
        .ok_or_else(|| BybitError::Parse("tax export response is missing result.url".into()))?;
    let locator: ExportLocator = serde_json::from_str(locator_raw)?;
    let urls = build_export_urls(&locator);

    let mut value = json!({
        "queryId": query_id,
        "basepath": locator.basepath,
        "files": locator.files,
        "urls": urls,
    });

    if let Some(download_dir) = download_dir {
        let downloads = download_export_files(&locator, download_dir).await?;
        value["downloadDir"] = json!(download_dir.display().to_string());
        value["downloaded"] = Value::Array(downloads);
    }

    Ok(value)
}

fn build_export_urls(locator: &ExportLocator) -> Vec<String> {
    let basepath = locator.basepath.trim_end_matches('/');
    locator
        .files
        .iter()
        .map(|path| format!("{basepath}/{}", path.trim_start_matches('/')))
        .collect()
}

async fn download_export_files(
    locator: &ExportLocator,
    download_dir: &Path,
) -> BybitResult<Vec<Value>> {
    let http = ClientBuilder::new()
        .use_rustls_tls()
        .timeout(Duration::from_secs(60))
        .build()
        .map_err(|error| {
            BybitError::Network(format!("Failed to build download client: {error}"))
        })?;

    tokio::fs::create_dir_all(download_dir).await?;

    let mut downloads = Vec::with_capacity(locator.files.len());
    for remote_path in &locator.files {
        let relative_path = sanitize_export_path(remote_path)?;
        let local_path = download_dir.join(&relative_path);
        if let Some(parent) = local_path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }

        let url = format!(
            "{}/{}",
            locator.basepath.trim_end_matches('/'),
            remote_path.trim_start_matches('/')
        );
        let response = http.get(&url).send().await.map_err(BybitError::from)?;
        let status = response.status();
        if !status.is_success() {
            return Err(BybitError::Network(format!(
                "Failed to download export file {remote_path}: HTTP {status}"
            )));
        }

        let bytes = response.bytes().await.map_err(BybitError::from)?;
        tokio::fs::write(&local_path, &bytes).await?;
        downloads.push(json!({
            "remotePath": remote_path,
            "localPath": local_path.display().to_string(),
            "sizeBytes": bytes.len(),
        }));
    }

    Ok(downloads)
}

fn sanitize_export_path(remote_path: &str) -> BybitResult<PathBuf> {
    let mut sanitized = PathBuf::new();
    for component in Path::new(remote_path).components() {
        match component {
            Component::Normal(part) => sanitized.push(part),
            Component::CurDir => {}
            Component::ParentDir | Component::RootDir | Component::Prefix(_) => {
                return Err(BybitError::Validation(format!(
                    "Refusing to write unsafe export path: {remote_path}"
                )));
            }
        }
    }

    if sanitized.as_os_str().is_empty() {
        return Err(BybitError::Validation(
            "Refusing to write an empty export path.".to_string(),
        ));
    }

    Ok(sanitized)
}
