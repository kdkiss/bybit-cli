pub mod auth;
pub mod client;
pub mod commands;
pub mod config;
pub mod errors;
pub mod mcp;
pub mod output;
pub mod paper;
pub mod shell;
pub mod telemetry;

use clap::{Parser, Subcommand};

use commands::{
    account::{run as run_account, AccountArgs},
    asset::{run as run_asset, AssetArgs},
    auth::{run as run_auth, AuthArgs},
    convert::{run as run_convert, ConvertArgs},
    earn::{run as run_earn, EarnArgs},
    funding::{run as run_funding, FundingArgs},
    futures::{run as run_futures, FuturesArgs},
    margin::{run as run_margin, MarginArgs},
    market::{run as run_market, MarketArgs},
    paper::{run as run_paper, PaperArgs},
    position::{run as run_position, PositionArgs},
    reports::{run as run_reports, ReportsArgs},
    subaccount::{run as run_subaccount, SubaccountArgs},
    trade::{run as run_trade, TradeArgs},
    utility::run_setup,
    websocket::{run as run_ws, WsArgs},
};
use mcp::server::{
    McpTransportKind, DEFAULT_MCP_HTTP_HOST, DEFAULT_MCP_HTTP_PATH, DEFAULT_MCP_HTTP_PORT,
};
use output::OutputFormat;

// ---------------------------------------------------------------------------
// Runtime context passed to every command handler
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct AppContext {
    pub format: OutputFormat,
    pub verbose: bool,
    pub api_url: Option<String>,
    pub api_key: Option<String>,
    pub api_secret: Option<String>,
    pub api_secret_from_input: bool,
    pub default_category: String,
    pub recv_window: Option<u64>,
    pub testnet: bool,
    pub force: bool,
    pub mcp_mode: bool,
}

// ---------------------------------------------------------------------------
// Top-level CLI definition
// ---------------------------------------------------------------------------

#[derive(Debug, Parser)]
#[command(
    name = "bybit",
    about = "Bybit CLI — trade, query, and manage your Bybit account from the terminal",
    version,
    propagate_version = true
)]
pub struct Cli {
    /// Output format
    #[arg(short = 'o', long, global = true, default_value = "table")]
    pub output: OutputFormat,

    /// Verbose mode (show request/response details on stderr)
    #[arg(short = 'v', long, global = true)]
    pub verbose: bool,

    /// API key (overrides env and config)
    #[arg(long, global = true, env = "BYBIT_API_KEY")]
    pub api_key: Option<String>,

    /// API secret (overrides env and config)
    #[arg(long, global = true, env = "BYBIT_API_SECRET")]
    pub api_secret: Option<String>,

    /// Read API secret from stdin
    #[arg(long, global = true, conflicts_with_all = ["api_secret", "api_secret_file"])]
    pub api_secret_stdin: bool,

    /// Path to file containing the API secret
    #[arg(long, global = true, conflicts_with_all = ["api_secret", "api_secret_stdin"])]
    pub api_secret_file: Option<std::path::PathBuf>,

    /// Override API base URL
    #[arg(long, global = true, env = "BYBIT_API_URL")]
    pub api_url: Option<String>,

    /// Use testnet endpoints
    #[arg(long, global = true, env = "BYBIT_TESTNET")]
    pub testnet: bool,

    /// recv_window in milliseconds (default 5000)
    #[arg(long, global = true)]
    pub recv_window: Option<u64>,

    /// Skip confirmation prompts (dangerous — use with care)
    #[arg(short = 'y', long, global = true)]
    pub yes: bool,

    #[command(subcommand)]
    pub command: Command,
}

pub fn resolve_cli_api_secret(
    api_secret: Option<String>,
    api_secret_stdin: bool,
    api_secret_file: Option<&std::path::Path>,
) -> errors::BybitResult<Option<String>> {
    if let Some(path) = api_secret_file {
        return Ok(Some(
            config::read_secret_from_file(path)?.expose().to_string(),
        ));
    }

    if api_secret_stdin {
        return Ok(Some(config::read_secret_from_stdin()?.expose().to_string()));
    }

    Ok(api_secret)
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Public market data (no auth required)
    #[command(subcommand_value_name = "MARKET_CMD")]
    Market(MarketArgs),

    /// Order placement and management
    #[command(subcommand_value_name = "TRADE_CMD")]
    Trade(TradeArgs),

    /// Account and wallet information
    #[command(subcommand_value_name = "ACCOUNT_CMD")]
    Account(AccountArgs),

    /// Position management
    #[command(subcommand_value_name = "POSITION_CMD")]
    Position(PositionArgs),

    /// Asset transfers, deposits, and withdrawals
    #[command(subcommand_value_name = "ASSET_CMD")]
    Asset(AssetArgs),

    /// Coin conversion (quote, execute, status, history)
    #[command(subcommand_value_name = "CONVERT_CMD")]
    Convert(ConvertArgs),

    /// Spot margin trade workflows for UTA
    #[command(subcommand_value_name = "MARGIN_CMD")]
    Margin(MarginArgs),

    /// Funding and wallet workflows
    #[command(subcommand_value_name = "FUNDING_CMD")]
    Funding(FundingArgs),

    /// Subaccount management for master accounts
    #[command(subcommand_value_name = "SUBACCOUNT_CMD")]
    Subaccount(SubaccountArgs),

    /// Staking and savings products (Earn)
    #[command(subcommand_value_name = "EARN_CMD")]
    Earn(EarnArgs),

    /// WebSocket real-time streaming
    #[command(subcommand_value_name = "WS_CMD")]
    Ws(WsArgs),

    /// Futures-focused namespace for derivatives market data, trading, positions, and streaming
    #[command(subcommand_value_name = "FUTURES_CMD")]
    Futures(FuturesArgs),

    /// Simulated paper trading (no real money)
    #[command(subcommand_value_name = "PAPER_CMD")]
    Paper(PaperArgs),

    /// Read-only account, order, transfer, and PnL histories
    #[command(subcommand_value_name = "REPORTS_CMD")]
    Reports(ReportsArgs),

    /// API credential management
    #[command(subcommand_value_name = "AUTH_CMD")]
    Auth(AuthArgs),

    /// Interactive first-time setup
    Setup,

    /// Start interactive REPL shell
    Shell,

    /// Start MCP server for AI tool use
    Mcp {
        /// Comma-separated service groups to expose, or "all"
        #[arg(short = 's', long, default_value = "market,account,paper")]
        services: String,
        /// Skip per-call confirmation for dangerous tools
        #[arg(long)]
        allow_dangerous: bool,
        /// MCP transport to serve
        #[arg(long, value_enum, default_value_t = McpTransportKind::Stdio)]
        transport: McpTransportKind,
        /// Host to bind when using HTTP transport
        #[arg(long, default_value = DEFAULT_MCP_HTTP_HOST)]
        host: String,
        /// Port to bind when using HTTP transport
        #[arg(long, default_value_t = DEFAULT_MCP_HTTP_PORT)]
        port: u16,
        /// Request path to serve when using HTTP transport
        #[arg(long, default_value = DEFAULT_MCP_HTTP_PATH)]
        path: String,
    },
}

pub fn has_option_flag(args: &[String], short: Option<char>, long: &str) -> bool {
    let short_prefix = short.map(|value| format!("-{value}"));
    let long_with_value = format!("{long}=");

    args.iter().any(|arg| {
        arg == long
            || arg.starts_with(&long_with_value)
            || short_prefix.as_ref().is_some_and(|prefix| {
                arg == prefix || (arg.starts_with(prefix) && arg.len() > prefix.len())
            })
    })
}

pub fn has_switch_flag(args: &[String], long: &str) -> bool {
    args.iter().any(|arg| arg == long)
}

pub fn env_flag(name: &str) -> Option<bool> {
    std::env::var(name).ok().map(|value| {
        matches!(
            value.trim().to_ascii_lowercase().as_str(),
            "1" | "true" | "yes" | "on"
        )
    })
}

impl Command {
    pub fn apply_default_category(&mut self, default_category: &str) {
        use commands::{
            account::AccountCommand,
            futures::{FuturesCommand, FuturesWsCommand},
            market::MarketCommand,
            paper::PaperCommand,
            position::PositionCommand,
            reports::ReportsCommand,
            trade::TradeCommand,
            websocket::WsCommand,
        };

        match self {
            Command::Market(args) => match &mut args.command {
                MarketCommand::Instruments { category, .. }
                | MarketCommand::Orderbook { category, .. }
                | MarketCommand::Tickers { category, .. }
                | MarketCommand::Kline { category, .. }
                | MarketCommand::MarkPriceKline { category, .. }
                | MarketCommand::IndexPriceKline { category, .. }
                | MarketCommand::PremiumIndexKline { category, .. }
                | MarketCommand::FundingRate { category, .. }
                | MarketCommand::Trades { category, .. }
                | MarketCommand::OpenInterest { category, .. }
                | MarketCommand::RiskLimit { category, .. }
                | MarketCommand::LsRatio { category, .. } => {
                    *category = default_category.to_string();
                }
                _ => {}
            },
            Command::Trade(args) => match &mut args.command {
                TradeCommand::Buy(order) | TradeCommand::Sell(order) => {
                    order.category = default_category.to_string();
                }
                TradeCommand::Amend { category, .. }
                | TradeCommand::Cancel { category, .. }
                | TradeCommand::CancelAll { category, .. }
                | TradeCommand::OpenOrders { category, .. }
                | TradeCommand::History { category, .. }
                | TradeCommand::Fills { category, .. }
                | TradeCommand::BatchPlace { category, .. }
                | TradeCommand::BatchAmend { category, .. }
                | TradeCommand::BatchCancel { category, .. } => {
                    *category = default_category.to_string();
                }
                TradeCommand::CancelAfter { .. } | TradeCommand::DcpInfo => {}
            },
            Command::Account(args) => match &mut args.command {
                AccountCommand::FeeRate { category, .. }
                | AccountCommand::Volume { category, .. } => {
                    *category = default_category.to_string();
                }
                AccountCommand::Balance { .. }
                | AccountCommand::ExtendedBalance { .. }
                | AccountCommand::Info
                | AccountCommand::TransactionLog { .. }
                | AccountCommand::BorrowHistory { .. }
                | AccountCommand::CollateralInfo { .. }
                | AccountCommand::Greeks { .. }
                | AccountCommand::SetMarginMode { .. }
                | AccountCommand::SetSpotHedging { .. }
                | AccountCommand::SetUsdcSettlement { .. }
                | AccountCommand::AdlAlert { .. }
                | AccountCommand::Borrow { .. }
                | AccountCommand::Repay { .. }
                | AccountCommand::QuickRepay { .. } => {}
            },
            Command::Position(args) => match &mut args.command {
                PositionCommand::List { category, .. }
                | PositionCommand::SetLeverage { category, .. }
                | PositionCommand::SwitchMode { category, .. }
                | PositionCommand::SetTpsl { category, .. }
                | PositionCommand::TrailingStop { category, .. }
                | PositionCommand::SetRiskLimit { category, .. }
                | PositionCommand::AddMargin { category, .. }
                | PositionCommand::ClosedPnl { category, .. }
                | PositionCommand::Move { category, .. }
                | PositionCommand::MoveHistory { category, .. }
                | PositionCommand::Flatten { category, .. } => {
                    *category = default_category.to_string();
                }
            },
            Command::Ws(args) => match &mut args.command {
                WsCommand::Orderbook { category, .. }
                | WsCommand::Ticker { category, .. }
                | WsCommand::Trades { category, .. }
                | WsCommand::Kline { category, .. }
                | WsCommand::Liquidation { category, .. } => {
                    *category = default_category.to_string();
                }
                _ => {}
            },
            Command::Futures(args) => {
                if matches!(default_category, "linear" | "inverse") {
                    match &mut args.command {
                        FuturesCommand::Instruments { category, .. }
                        | FuturesCommand::Tickers { category, .. }
                        | FuturesCommand::Orderbook { category, .. }
                        | FuturesCommand::FundingRate { category, .. }
                        | FuturesCommand::OpenInterest { category, .. }
                        | FuturesCommand::Positions { category, .. }
                        | FuturesCommand::OpenOrders { category, .. }
                        | FuturesCommand::History { category, .. }
                        | FuturesCommand::Fills { category, .. }
                        | FuturesCommand::AdlAlert { category, .. }
                        | FuturesCommand::RiskLimit { category, .. }
                        | FuturesCommand::Cancel { category, .. }
                        | FuturesCommand::CancelAll { category, .. }
                        | FuturesCommand::SetLeverage { category, .. } => {
                            *category = default_category.to_string();
                        }
                        FuturesCommand::Buy(order) | FuturesCommand::Sell(order) => {
                            order.category = default_category.to_string();
                        }
                        FuturesCommand::Ws(ws_args) => match &mut ws_args.command {
                            FuturesWsCommand::Orderbook { category, .. }
                            | FuturesWsCommand::Ticker { category, .. }
                            | FuturesWsCommand::Trades { category, .. }
                            | FuturesWsCommand::Kline { category, .. }
                            | FuturesWsCommand::Liquidation { category, .. } => {
                                *category = default_category.to_string();
                            }
                            FuturesWsCommand::Orders
                            | FuturesWsCommand::Positions
                            | FuturesWsCommand::Executions
                            | FuturesWsCommand::Wallet => {}
                        },
                    }
                }
            }
            Command::Paper(args) => match &mut args.command {
                PaperCommand::Buy { category, .. } | PaperCommand::Sell { category, .. } => {
                    *category = default_category.to_string();
                }
                PaperCommand::Init { .. }
                | PaperCommand::Balance
                | PaperCommand::Positions
                | PaperCommand::History
                | PaperCommand::Cancelled
                | PaperCommand::Orders
                | PaperCommand::Cancel { .. }
                | PaperCommand::CancelAll
                | PaperCommand::Status
                | PaperCommand::Reset { .. } => {}
            },
            Command::Reports(args) => match &mut args.command {
                ReportsCommand::Orders { category, .. }
                | ReportsCommand::Fills { category, .. }
                | ReportsCommand::ClosedPnl { category, .. }
                | ReportsCommand::Moves { category, .. } => {
                    *category = default_category.to_string();
                }
                ReportsCommand::Transactions { .. }
                | ReportsCommand::BorrowHistory { .. }
                | ReportsCommand::Deposits { .. }
                | ReportsCommand::Withdrawals { .. }
                | ReportsCommand::Transfers { .. }
                | ReportsCommand::SubTransfers { .. }
                | ReportsCommand::RegisterTime
                | ReportsCommand::ExportRequest { .. }
                | ReportsCommand::ExportStatus { .. }
                | ReportsCommand::ExportRetrieve { .. } => {}
            },
            _ => {}
        }
    }
}

// ---------------------------------------------------------------------------
// Command dispatch
// ---------------------------------------------------------------------------

pub async fn dispatch(ctx: AppContext, command: Command) -> errors::BybitResult<()> {
    use client::BybitClient;

    let client = BybitClient::new(
        ctx.testnet,
        ctx.api_url.as_deref(),
        ctx.api_key.clone(),
        ctx.api_secret.clone(),
        ctx.recv_window,
    )?;

    match command {
        Command::Market(args) => run_market(args, &client, ctx.format).await,
        Command::Trade(args) => run_trade(args, &client, ctx.format, ctx.force).await,
        Command::Account(args) => run_account(args, &client, ctx.format, ctx.force).await,
        Command::Position(args) => run_position(args, &client, ctx.format, ctx.force).await,
        Command::Asset(args) => run_asset(args, &client, ctx.format, ctx.force).await,
        Command::Convert(args) => run_convert(args, &client, ctx.format, ctx.force).await,
        Command::Margin(args) => run_margin(args, &client, ctx.format, ctx.force).await,
        Command::Funding(args) => run_funding(args, &client, ctx.format, ctx.force).await,
        Command::Subaccount(args) => run_subaccount(args, &client, ctx.format, ctx.force).await,
        Command::Earn(args) => run_earn(args, &client, ctx.format, ctx.force).await,
        Command::Ws(args) => {
            run_ws(
                args,
                ctx.api_key.as_deref(),
                ctx.api_secret.as_deref(),
                ctx.testnet,
            )
            .await
        }
        Command::Futures(args) => {
            run_futures(
                args,
                &client,
                ctx.format,
                ctx.force,
                ctx.api_key.as_deref(),
                ctx.api_secret.as_deref(),
                ctx.testnet,
            )
            .await
        }
        Command::Paper(args) => run_paper(args, &client, ctx.format).await,
        Command::Reports(args) => run_reports(args, &client, ctx.format, ctx.force).await,
        Command::Auth(args) => run_auth(args, &ctx, &client).await,
        Command::Setup => run_setup().await,
        Command::Shell => shell::run_shell(ctx).await,
        Command::Mcp {
            services,
            allow_dangerous,
            transport,
            host,
            port,
            path,
        } => {
            let mut mcp_ctx = ctx;
            mcp_ctx.mcp_mode = true;
            mcp::server::run_mcp_server(
                mcp_ctx,
                &services,
                allow_dangerous,
                transport,
                &host,
                port,
                &path,
            )
            .await
        }
    }
}
