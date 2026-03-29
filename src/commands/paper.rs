use clap::Subcommand;

use crate::client::BybitClient;
use crate::errors::BybitResult;
use crate::output::{print_output, OutputFormat};
use crate::paper;

#[derive(Debug, clap::Args)]
pub struct PaperArgs {
    #[command(subcommand)]
    pub command: PaperCommand,
}

#[derive(Debug, Subcommand)]
pub enum PaperCommand {
    /// Initialize paper trading with a configurable starting balance and fee model
    Init {
        /// Starting balance in settle-coin
        #[arg(long, default_value = "10000")]
        usdt: f64,
        /// Settlement currency (default: USDT)
        #[arg(long, default_value = "USDT")]
        settle_coin: String,
        /// Taker fee in basis points applied to market fills (default: 6 = 0.06%)
        #[arg(long, default_value = "6")]
        taker_fee_bps: u32,
        /// Maker fee in basis points applied to limit fills (default: 1 = 0.01%)
        #[arg(long, default_value = "1")]
        maker_fee_bps: u32,
        /// One-way market order slippage in basis points (default: 5 = 0.05%)
        #[arg(long, default_value = "5")]
        slippage_bps: u32,
        /// Overwrite an existing journal without error
        #[arg(long)]
        force: bool,
    },
    /// Simulate a buy order (market fill, or limit if --price given)
    Buy {
        #[arg(long, default_value = "linear")]
        category: String,
        #[arg(long)]
        symbol: String,
        /// Quantity to buy
        #[arg(long)]
        qty: f64,
        /// Limit price — order waits until market price ≤ this value
        #[arg(long)]
        price: Option<f64>,
    },
    /// Simulate a sell order (market fill, or limit if --price given)
    Sell {
        #[arg(long, default_value = "linear")]
        category: String,
        #[arg(long)]
        symbol: String,
        #[arg(long)]
        qty: f64,
        /// Limit price — order waits until market price ≥ this value
        #[arg(long)]
        price: Option<f64>,
    },
    /// Show simulated coin balances (includes reserved and available amounts)
    Balance,
    /// Show paper trade history (filled trades with fees)
    History,
    /// Show cancelled order history
    Cancelled,
    /// Show open paper positions
    Positions,
    /// Show open limit orders (checks for fills at current prices first)
    Orders,
    /// Cancel a specific open limit order
    Cancel {
        /// Order ID to cancel
        order_id: u64,
    },
    /// Cancel all open limit orders
    CancelAll,
    /// Show full account summary: balance, positions, PnL, fees
    Status,
    /// Wipe all paper trading state and start fresh
    Reset {
        /// New starting balance in settle-coin (default: keep current)
        #[arg(long, alias = "usdt")]
        balance: Option<f64>,
        /// New settlement currency (default: keep current)
        #[arg(long, alias = "currency")]
        settle_coin: Option<String>,
        /// New taker fee in basis points (default: keep current)
        #[arg(long)]
        taker_fee_bps: Option<u32>,
        /// New maker fee in basis points (default: keep current)
        #[arg(long)]
        maker_fee_bps: Option<u32>,
        /// New one-way market slippage in basis points (default: keep current)
        #[arg(long)]
        slippage_bps: Option<u32>,
    },
}

pub async fn run(args: PaperArgs, client: &BybitClient, format: OutputFormat) -> BybitResult<()> {
    let value = match args.command {
        PaperCommand::Init {
            usdt,
            settle_coin,
            taker_fee_bps,
            maker_fee_bps,
            slippage_bps,
            force,
        } => {
            let journal = paper::init(
                usdt,
                settle_coin,
                taker_fee_bps,
                maker_fee_bps,
                slippage_bps,
                force,
            )?;
            serde_json::json!({
                "mode": "paper",
                "status": "initialized",
                "settle_coin": journal.settle_coin,
                "balance": journal.balance.coins,
                "settings": {
                    "taker_fee_bps": journal.taker_fee_bps,
                    "maker_fee_bps": journal.maker_fee_bps,
                    "slippage_bps": journal.slippage_bps,
                },
            })
        }

        PaperCommand::Buy {
            category,
            symbol,
            qty,
            price,
        } => paper::buy(client, &category, &symbol, qty, price).await?,

        PaperCommand::Sell {
            category,
            symbol,
            qty,
            price,
        } => paper::sell(client, &category, &symbol, qty, price).await?,

        PaperCommand::Balance => paper::get_balance(client).await?,

        PaperCommand::Positions => paper::get_positions(client).await?,

        PaperCommand::History => paper::get_trades(client).await?,

        PaperCommand::Cancelled => paper::get_cancelled(client).await?,

        PaperCommand::Orders => paper::get_orders(client).await?,

        PaperCommand::Cancel { order_id } => paper::cancel_order(client, order_id).await?,

        PaperCommand::CancelAll => paper::cancel_all_orders(client).await?,

        PaperCommand::Status => paper::status(client).await?,

        PaperCommand::Reset {
            balance,
            settle_coin,
            taker_fee_bps,
            maker_fee_bps,
            slippage_bps,
        } => {
            let journal = paper::reset(
                client,
                paper::ResetOptions {
                    balance,
                    settle_coin,
                    taker_fee_bps,
                    maker_fee_bps,
                    slippage_bps,
                },
            )
            .await?;
            serde_json::json!({
                "mode": "paper",
                "status": "reset",
                "settle_coin": journal.settle_coin,
                "starting_balance": journal.starting_balance,
                "settings": {
                    "taker_fee_bps": journal.taker_fee_bps,
                    "maker_fee_bps": journal.maker_fee_bps,
                    "slippage_bps": journal.slippage_bps,
                },
            })
        }
    };

    print_output(&value, format);
    Ok(())
}
