use clap::Subcommand;

use crate::client::BybitClient;
use crate::commands::{
    account::{run as run_account, AccountArgs, AccountCommand},
    market::{run as run_market, MarketArgs, MarketCommand},
    position::{run as run_position, PositionArgs, PositionCommand},
    trade::{run as run_trade, OrderArgs, TradeArgs, TradeCommand},
    websocket::{run as run_ws, WsArgs, WsCommand},
};
use crate::errors::BybitResult;
use crate::output::OutputFormat;

#[derive(Debug, clap::Args)]
pub struct FuturesArgs {
    #[command(subcommand)]
    pub command: FuturesCommand,
}

#[derive(Debug, Subcommand)]
pub enum FuturesCommand {
    /// Get tradeable futures instruments
    Instruments {
        #[arg(long, default_value = "linear")]
        category: String,
        #[arg(long)]
        symbol: Option<String>,
        #[arg(long)]
        status: Option<String>,
        #[arg(long)]
        base_coin: Option<String>,
        #[arg(long)]
        limit: Option<u32>,
        #[arg(long)]
        cursor: Option<String>,
    },
    /// Get futures tickers / 24h stats
    Tickers {
        #[arg(long, default_value = "linear")]
        category: String,
        #[arg(long)]
        symbol: Option<String>,
        #[arg(long)]
        base_coin: Option<String>,
    },
    /// Get futures order book depth
    Orderbook {
        #[arg(long, default_value = "linear")]
        category: String,
        #[arg(long)]
        symbol: String,
        #[arg(long, default_value = "50")]
        limit: u32,
    },
    /// Get futures funding rate history
    FundingRate {
        #[arg(long, default_value = "linear")]
        category: String,
        #[arg(long)]
        symbol: String,
        #[arg(long)]
        start: Option<u64>,
        #[arg(long)]
        end: Option<u64>,
        #[arg(long)]
        limit: Option<u32>,
    },
    /// Get ADL risk level for a symbol
    AdlAlert {
        #[arg(long, default_value = "linear")]
        category: String,
        #[arg(long)]
        symbol: Option<String>,
    },
    /// Get risk limit info for a symbol
    RiskLimit {
        #[arg(long, default_value = "linear")]
        category: String,
        #[arg(long)]
        symbol: Option<String>,
    },
    /// Get futures open interest
    OpenInterest {
        #[arg(long, default_value = "linear")]
        category: String,
        #[arg(long)]
        symbol: String,
        #[arg(long)]
        interval_time: String,
        #[arg(long)]
        start: Option<u64>,
        #[arg(long)]
        end: Option<u64>,
        #[arg(long)]
        limit: Option<u32>,
        #[arg(long)]
        cursor: Option<String>,
    },
    /// List open futures positions
    Positions {
        #[arg(long, default_value = "linear")]
        category: String,
        #[arg(long)]
        symbol: Option<String>,
        #[arg(long)]
        base_coin: Option<String>,
        #[arg(long)]
        settle_coin: Option<String>,
        #[arg(long)]
        limit: Option<u32>,
        #[arg(long)]
        cursor: Option<String>,
    },
    /// List open futures orders
    OpenOrders {
        #[arg(long, default_value = "linear")]
        category: String,
        #[arg(long)]
        symbol: Option<String>,
        #[arg(long)]
        base_coin: Option<String>,
        #[arg(long)]
        settle_coin: Option<String>,
        #[arg(long)]
        order_id: Option<String>,
        #[arg(long)]
        order_link_id: Option<String>,
        #[arg(long)]
        limit: Option<u32>,
        #[arg(long)]
        cursor: Option<String>,
    },
    /// Get futures order history
    History {
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
    /// Get futures fill history
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
    /// Place a futures buy order
    Buy(OrderArgs),
    /// Place a futures sell order
    Sell(OrderArgs),
    /// Cancel a futures order
    Cancel {
        #[arg(long, default_value = "linear")]
        category: String,
        #[arg(long)]
        symbol: String,
        #[arg(long)]
        order_id: Option<String>,
        #[arg(long)]
        order_link_id: Option<String>,
    },
    /// Cancel all futures orders
    CancelAll {
        #[arg(long, default_value = "linear")]
        category: String,
        #[arg(long)]
        symbol: Option<String>,
        #[arg(long)]
        base_coin: Option<String>,
        #[arg(long)]
        settle_coin: Option<String>,
    },
    /// Set futures leverage
    SetLeverage {
        #[arg(long, default_value = "linear")]
        category: String,
        #[arg(long)]
        symbol: String,
        #[arg(long)]
        buy_leverage: String,
        #[arg(long)]
        sell_leverage: String,
    },
    /// Futures websocket namespace
    Ws(FuturesWsArgs),
}

#[derive(Debug, clap::Args)]
pub struct FuturesWsArgs {
    #[command(subcommand)]
    pub command: FuturesWsCommand,
}

#[derive(Debug, Subcommand)]
pub enum FuturesWsCommand {
    Orderbook {
        #[arg(long, default_value = "linear")]
        category: String,
        #[arg(long)]
        symbol: String,
        #[arg(long, default_value = "50")]
        depth: u32,
    },
    Ticker {
        #[arg(long, default_value = "linear")]
        category: String,
        #[arg(long)]
        symbol: String,
    },
    Trades {
        #[arg(long, default_value = "linear")]
        category: String,
        #[arg(long)]
        symbol: String,
    },
    Kline {
        #[arg(long, default_value = "linear")]
        category: String,
        #[arg(long)]
        symbol: String,
        #[arg(long, default_value = "1")]
        interval: String,
    },
    Liquidation {
        #[arg(long, default_value = "linear")]
        category: String,
        #[arg(long)]
        symbol: String,
    },
    Orders,
    Positions,
    Executions,
    Wallet,
}

pub async fn run(
    args: FuturesArgs,
    client: &BybitClient,
    format: OutputFormat,
    force: bool,
    api_key: Option<&str>,
    api_secret: Option<&str>,
    testnet: bool,
) -> BybitResult<()> {
    match args.command {
        FuturesCommand::Instruments {
            category,
            symbol,
            status,
            base_coin,
            limit,
            cursor,
        } => {
            run_market(
                MarketArgs {
                    command: MarketCommand::Instruments {
                        category,
                        symbol,
                        status,
                        base_coin,
                        limit,
                        cursor,
                    },
                },
                client,
                format,
            )
            .await
        }
        FuturesCommand::Tickers {
            category,
            symbol,
            base_coin,
        } => {
            run_market(
                MarketArgs {
                    command: MarketCommand::Tickers {
                        category,
                        symbol,
                        base_coin,
                        exp_date: None,
                    },
                },
                client,
                format,
            )
            .await
        }
        FuturesCommand::Orderbook {
            category,
            symbol,
            limit,
        } => {
            run_market(
                MarketArgs {
                    command: MarketCommand::Orderbook {
                        category,
                        symbol,
                        limit,
                    },
                },
                client,
                format,
            )
            .await
        }
        FuturesCommand::FundingRate {
            category,
            symbol,
            start,
            end,
            limit,
        } => {
            run_market(
                MarketArgs {
                    command: MarketCommand::FundingRate {
                        category,
                        symbol,
                        start,
                        end,
                        limit,
                    },
                },
                client,
                format,
            )
            .await
        }
        FuturesCommand::AdlAlert { category, symbol } => {
            run_account(
                AccountArgs {
                    command: AccountCommand::AdlAlert { category, symbol },
                },
                client,
                format,
                false,
            )
            .await
        }
        FuturesCommand::RiskLimit { category, symbol } => {
            run_market(
                MarketArgs {
                    command: MarketCommand::RiskLimit {
                        category,
                        symbol,
                        cursor: None,
                    },
                },
                client,
                format,
            )
            .await
        }
        FuturesCommand::OpenInterest {
            category,
            symbol,
            interval_time,
            start,
            end,
            limit,
            cursor,
        } => {
            run_market(
                MarketArgs {
                    command: MarketCommand::OpenInterest {
                        category,
                        symbol,
                        interval_time,
                        start,
                        end,
                        limit,
                        cursor,
                    },
                },
                client,
                format,
            )
            .await
        }
        FuturesCommand::Positions {
            category,
            symbol,
            base_coin,
            settle_coin,
            limit,
            cursor,
        } => {
            run_position(
                PositionArgs {
                    command: PositionCommand::List {
                        category,
                        symbol,
                        base_coin,
                        settle_coin,
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
        FuturesCommand::OpenOrders {
            category,
            symbol,
            base_coin,
            settle_coin,
            order_id,
            order_link_id,
            limit,
            cursor,
        } => {
            run_trade(
                TradeArgs {
                    command: TradeCommand::OpenOrders {
                        category,
                        symbol,
                        base_coin,
                        settle_coin,
                        order_id,
                        order_link_id,
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
        FuturesCommand::History {
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
        FuturesCommand::Fills {
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
        FuturesCommand::Buy(order) => {
            run_trade(
                TradeArgs {
                    command: TradeCommand::Buy(order),
                },
                client,
                format,
                force,
            )
            .await
        }
        FuturesCommand::Sell(order) => {
            run_trade(
                TradeArgs {
                    command: TradeCommand::Sell(order),
                },
                client,
                format,
                force,
            )
            .await
        }
        FuturesCommand::Cancel {
            category,
            symbol,
            order_id,
            order_link_id,
        } => {
            run_trade(
                TradeArgs {
                    command: TradeCommand::Cancel {
                        category,
                        symbol,
                        order_id,
                        order_link_id,
                    },
                },
                client,
                format,
                force,
            )
            .await
        }
        FuturesCommand::CancelAll {
            category,
            symbol,
            base_coin,
            settle_coin,
        } => {
            run_trade(
                TradeArgs {
                    command: TradeCommand::CancelAll {
                        category,
                        symbol,
                        base_coin,
                        settle_coin,
                    },
                },
                client,
                format,
                force,
            )
            .await
        }
        FuturesCommand::SetLeverage {
            category,
            symbol,
            buy_leverage,
            sell_leverage,
        } => {
            run_position(
                PositionArgs {
                    command: PositionCommand::SetLeverage {
                        category,
                        symbol,
                        buy_leverage,
                        sell_leverage,
                    },
                },
                client,
                format,
                force,
            )
            .await
        }
        FuturesCommand::Ws(args) => {
            let ws_command = match args.command {
                FuturesWsCommand::Orderbook {
                    category,
                    symbol,
                    depth,
                } => WsCommand::Orderbook {
                    category,
                    symbol,
                    depth,
                },
                FuturesWsCommand::Ticker { category, symbol } => {
                    WsCommand::Ticker { category, symbol }
                }
                FuturesWsCommand::Trades { category, symbol } => {
                    WsCommand::Trades { category, symbol }
                }
                FuturesWsCommand::Kline {
                    category,
                    symbol,
                    interval,
                } => WsCommand::Kline {
                    category,
                    symbol,
                    interval,
                },
                FuturesWsCommand::Liquidation { category, symbol } => {
                    WsCommand::Liquidation { category, symbol }
                }
                FuturesWsCommand::Orders => WsCommand::Orders,
                FuturesWsCommand::Positions => WsCommand::Positions,
                FuturesWsCommand::Executions => WsCommand::Executions,
                FuturesWsCommand::Wallet => WsCommand::Wallet,
            };

            run_ws(
                WsArgs {
                    command: ws_command,
                },
                api_key,
                api_secret,
                testnet,
            )
            .await
        }
    }
}
