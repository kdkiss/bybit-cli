use std::collections::HashMap;
use std::str::FromStr;

use clap::Subcommand;
use serde_json::{json, Value};

use crate::client::BybitClient;
use crate::errors::{BybitError, BybitResult};
use crate::futures_paper::{
    self, fill_to_json, order_to_json, position_to_json, FuturesOrderType, FuturesPaperState,
    MarketSnapshot, OrderParams, Side, TriggerSignal, DEFAULT_FUTURES_TAKER_FEE_RATE, MAX_LEVERAGE,
};
use crate::output::{print_output, OutputFormat};

// ---------------------------------------------------------------------------
// Subcommand definitions
// ---------------------------------------------------------------------------

#[derive(Debug, clap::Args)]
pub struct FuturesPaperArgs {
    #[command(subcommand)]
    pub command: FuturesPaperCommand,
}

#[derive(Debug, Subcommand)]
pub enum FuturesPaperCommand {
    /// Initialize futures paper trading account
    Init {
        /// Starting collateral balance (default: 10000)
        #[arg(long, default_value = "10000")]
        balance: f64,
        /// Collateral currency (default: USDT)
        #[arg(long, default_value = "USDT")]
        currency: String,
        /// Taker fee rate as a decimal (default: 0.00055 = 0.055%)
        #[arg(long)]
        fee_rate: Option<f64>,
        /// Overwrite an existing account without error
        #[arg(long)]
        force: bool,
    },

    /// Reset futures paper account to initial state
    Reset {
        /// New starting balance (default: keep current)
        #[arg(long)]
        balance: Option<f64>,
        /// New collateral currency (default: keep current)
        #[arg(long)]
        currency: Option<String>,
        /// New taker fee rate (default: keep current)
        #[arg(long)]
        fee_rate: Option<f64>,
    },

    /// Show collateral balance and margin summary
    Balance,

    /// Show full futures paper account summary
    Status,

    /// Place a futures paper long (buy) order
    Buy {
        /// Futures symbol (e.g. BTCUSDT)
        symbol: String,
        /// Order size in base asset
        size: String,
        /// Order type
        #[arg(long, default_value = "limit", value_parser = ["limit", "market", "post", "stop", "take-profit", "ioc", "trailing-stop", "fok"])]
        r#type: String,
        /// Limit price (required for limit/post/ioc/fok orders)
        #[arg(long)]
        price: Option<String>,
        /// Stop/trigger price (required for stop/take-profit/trailing-stop orders)
        #[arg(long)]
        stop_price: Option<String>,
        /// Trigger signal: mark, index, or last
        #[arg(long, value_parser = ["mark", "index", "last"])]
        trigger_signal: Option<String>,
        /// Client order ID
        #[arg(long)]
        client_order_id: Option<String>,
        /// Reduce-only: can only reduce an existing position
        #[arg(long)]
        reduce_only: bool,
        /// Leverage override for this order (1–100)
        #[arg(long)]
        leverage: Option<String>,
        /// Trailing stop max deviation
        #[arg(long)]
        trailing_stop_max_deviation: Option<String>,
        /// Trailing stop deviation unit (percent or quote_currency)
        #[arg(long)]
        trailing_stop_deviation_unit: Option<String>,
        /// Asset category (default: linear)
        #[arg(long, default_value = "linear")]
        category: String,
    },

    /// Place a futures paper short (sell) order
    Sell {
        /// Futures symbol (e.g. BTCUSDT)
        symbol: String,
        /// Order size in base asset
        size: String,
        /// Order type
        #[arg(long, default_value = "limit", value_parser = ["limit", "market", "post", "stop", "take-profit", "ioc", "trailing-stop", "fok"])]
        r#type: String,
        #[arg(long)]
        price: Option<String>,
        #[arg(long)]
        stop_price: Option<String>,
        #[arg(long, value_parser = ["mark", "index", "last"])]
        trigger_signal: Option<String>,
        #[arg(long)]
        client_order_id: Option<String>,
        #[arg(long)]
        reduce_only: bool,
        #[arg(long)]
        leverage: Option<String>,
        #[arg(long)]
        trailing_stop_max_deviation: Option<String>,
        #[arg(long)]
        trailing_stop_deviation_unit: Option<String>,
        /// Asset category (default: linear)
        #[arg(long, default_value = "linear")]
        category: String,
    },

    /// Show open futures paper orders (reconciles against current market first)
    Orders {
        /// Asset category (default: linear)
        #[arg(long, default_value = "linear")]
        category: String,
    },

    /// Get status of a specific futures paper order
    OrderStatus {
        /// Order ID to query
        order_id: String,
    },

    /// Edit a resting futures paper order
    EditOrder {
        /// Order ID to edit
        #[arg(long)]
        order_id: String,
        /// New order size
        #[arg(long)]
        size: Option<String>,
        /// New limit price
        #[arg(long)]
        price: Option<String>,
        /// New stop price
        #[arg(long)]
        stop_price: Option<String>,
    },

    /// Cancel a specific futures paper order
    Cancel {
        /// Exchange order ID
        #[arg(long, required_unless_present = "cli_ord_id")]
        order_id: Option<String>,
        /// Client order ID
        #[arg(long, required_unless_present = "order_id")]
        cli_ord_id: Option<String>,
    },

    /// Cancel all open futures paper orders
    CancelAll {
        /// Filter by symbol
        #[arg(long)]
        symbol: Option<String>,
    },

    /// Place a batch of futures paper orders from JSON
    BatchOrder {
        /// Orders as a JSON array string, or path to JSON file (prefix with @)
        orders_json: String,
    },

    /// Show open futures paper positions (with current PnL)
    Positions {
        /// Asset category (default: linear)
        #[arg(long, default_value = "linear")]
        category: String,
    },

    /// Show futures paper fill history
    Fills,

    /// Show futures paper account history (PnL events, funding, liquidations)
    History,

    /// Get leverage preferences
    Leverage {
        /// Filter by symbol
        #[arg(long)]
        symbol: Option<String>,
    },

    /// Set leverage preference for a symbol
    SetLeverage {
        /// Futures symbol (e.g. BTCUSDT)
        symbol: String,
        /// Max leverage (1–100)
        leverage: String,
    },
}

// ---------------------------------------------------------------------------
// Entry point
// ---------------------------------------------------------------------------

pub async fn run(
    args: FuturesPaperArgs,
    client: &BybitClient,
    format: OutputFormat,
) -> BybitResult<()> {
    let value = match args.command {
        FuturesPaperCommand::Init {
            balance,
            currency,
            fee_rate,
            force,
        } => cmd_init(balance, &currency, fee_rate, force)?,

        FuturesPaperCommand::Reset {
            balance,
            currency,
            fee_rate,
        } => cmd_reset(balance, currency.as_deref(), fee_rate)?,

        FuturesPaperCommand::Balance => cmd_balance()?,

        FuturesPaperCommand::Status => cmd_status(client).await?,

        FuturesPaperCommand::Buy {
            symbol,
            size,
            r#type,
            price,
            stop_price,
            trigger_signal,
            client_order_id,
            reduce_only,
            leverage,
            trailing_stop_max_deviation,
            trailing_stop_deviation_unit,
            category,
        } => {
            cmd_place_order(
                client,
                &symbol,
                &size,
                Side::Long,
                &r#type,
                price.as_deref(),
                stop_price.as_deref(),
                trigger_signal.as_deref(),
                client_order_id,
                reduce_only,
                leverage.as_deref(),
                trailing_stop_max_deviation.as_deref(),
                trailing_stop_deviation_unit.as_deref(),
                &category,
            )
            .await?
        }

        FuturesPaperCommand::Sell {
            symbol,
            size,
            r#type,
            price,
            stop_price,
            trigger_signal,
            client_order_id,
            reduce_only,
            leverage,
            trailing_stop_max_deviation,
            trailing_stop_deviation_unit,
            category,
        } => {
            cmd_place_order(
                client,
                &symbol,
                &size,
                Side::Short,
                &r#type,
                price.as_deref(),
                stop_price.as_deref(),
                trigger_signal.as_deref(),
                client_order_id,
                reduce_only,
                leverage.as_deref(),
                trailing_stop_max_deviation.as_deref(),
                trailing_stop_deviation_unit.as_deref(),
                &category,
            )
            .await?
        }

        FuturesPaperCommand::Orders { category } => cmd_orders(client, &category).await?,

        FuturesPaperCommand::OrderStatus { order_id } => cmd_order_status(&order_id)?,

        FuturesPaperCommand::EditOrder {
            order_id,
            size,
            price,
            stop_price,
        } => cmd_edit_order(&order_id, size.as_deref(), price.as_deref(), stop_price.as_deref())?,

        FuturesPaperCommand::Cancel {
            order_id,
            cli_ord_id,
        } => cmd_cancel(order_id.as_deref(), cli_ord_id.as_deref())?,

        FuturesPaperCommand::CancelAll { symbol } => cmd_cancel_all(symbol.as_deref())?,

        FuturesPaperCommand::BatchOrder { orders_json } => {
            cmd_batch_order(client, &orders_json).await?
        }

        FuturesPaperCommand::Positions { category } => cmd_positions(client, &category).await?,

        FuturesPaperCommand::Fills => cmd_fills()?,

        FuturesPaperCommand::History => cmd_history()?,

        FuturesPaperCommand::Leverage { symbol } => cmd_leverage(symbol.as_deref())?,

        FuturesPaperCommand::SetLeverage { symbol, leverage } => {
            cmd_set_leverage(&symbol, &leverage)?
        }
    };

    print_output(&value, format);
    Ok(())
}

// ---------------------------------------------------------------------------
// Command implementations
// ---------------------------------------------------------------------------

fn cmd_init(
    balance: f64,
    currency: &str,
    fee_rate: Option<f64>,
    force: bool,
) -> BybitResult<Value> {
    let path = futures_paper::futures_paper_state_path()?;

    if path.exists() && !force {
        return Err(BybitError::Paper(
            "Futures paper account already initialized. Use --force to overwrite, or `bybit futures paper reset`.".to_string(),
        ));
    }

    if !balance.is_finite() || balance <= 0.0 {
        return Err(BybitError::Paper(
            "Starting balance must be a positive number.".to_string(),
        ));
    }

    let fee_rate = fee_rate.unwrap_or(DEFAULT_FUTURES_TAKER_FEE_RATE);
    if !fee_rate.is_finite() || fee_rate < 0.0 || fee_rate > 0.1 {
        return Err(BybitError::Paper(
            "fee_rate must be between 0 and 0.1 (0% – 10%).".to_string(),
        ));
    }

    let state = FuturesPaperState::new(balance, currency, fee_rate);
    futures_paper::save_state(&state)?;

    Ok(json!({
        "mode": "futures_paper",
        "status": "initialized",
        "currency": state.currency,
        "collateral": state.collateral,
        "fee_rate": state.fee_rate,
        "max_leverage": MAX_LEVERAGE,
        "created_at": state.created_at,
    }))
}

fn cmd_reset(
    balance: Option<f64>,
    currency: Option<&str>,
    fee_rate: Option<f64>,
) -> BybitResult<Value> {
    let _lock = futures_paper::StateLock::acquire()?;
    let mut state = futures_paper::load_state()?;
    state.reset(balance, currency, fee_rate);
    futures_paper::save_state(&state)?;

    Ok(json!({
        "mode": "futures_paper",
        "status": "reset",
        "currency": state.currency,
        "starting_collateral": state.starting_collateral,
        "fee_rate": state.fee_rate,
    }))
}

fn cmd_balance() -> BybitResult<Value> {
    let state = futures_paper::load_state()?;
    let position_margin = state.position_margin();
    let order_margin = state.reserved_order_margin();
    Ok(json!({
        "mode": "futures_paper",
        "currency": state.currency,
        "collateral": state.collateral,
        "starting_collateral": state.starting_collateral,
        "position_margin": position_margin,
        "reserved_order_margin": order_margin,
        "used_margin": position_margin + order_margin,
        "open_positions": state.positions.len(),
        "open_orders": state.open_orders.len(),
    }))
}

async fn cmd_status(client: &BybitClient) -> BybitResult<Value> {
    let state = futures_paper::load_state()?;
    let category = state.category.clone();
    let symbols: Vec<String> = state.positions.iter().map(|p| p.symbol.clone()).collect();

    let mark_prices = if symbols.is_empty() {
        HashMap::new()
    } else {
        let (marks, _, _, _) =
            futures_paper::fetch_all_market_data(client, &category, &symbols).await?;
        marks
    };

    let upnl = state.unrealized_pnl(&mark_prices);
    let position_margin = state.position_margin();
    let order_margin = state.reserved_order_margin();
    let equity = state.collateral + upnl;
    let total_fees: f64 = state.fills.iter().map(|f| f.fee).sum();

    let positions: Vec<Value> = state
        .positions
        .iter()
        .map(|p| position_to_json(p, mark_prices.get(&p.symbol).copied()))
        .collect();

    Ok(json!({
        "mode": "futures_paper",
        "currency": state.currency,
        "collateral": state.collateral,
        "equity": equity,
        "unrealized_pnl": upnl,
        "position_margin": position_margin,
        "reserved_order_margin": order_margin,
        "used_margin": position_margin + order_margin,
        "total_fees_paid": total_fees,
        "starting_collateral": state.starting_collateral,
        "open_positions": positions,
        "open_orders_count": state.open_orders.len(),
        "fills_count": state.fills.len(),
        "created_at": state.created_at,
        "last_reconciled_at": state.last_reconciled_at,
    }))
}

#[allow(clippy::too_many_arguments)]
async fn cmd_place_order(
    client: &BybitClient,
    symbol: &str,
    size_str: &str,
    side: Side,
    order_type_str: &str,
    price_str: Option<&str>,
    stop_price_str: Option<&str>,
    trigger_signal_str: Option<&str>,
    client_order_id: Option<String>,
    reduce_only: bool,
    leverage_str: Option<&str>,
    trailing_max_dev_str: Option<&str>,
    trailing_dev_unit: Option<&str>,
    category: &str,
) -> BybitResult<Value> {
    let size: f64 = size_str
        .parse()
        .map_err(|_| BybitError::Paper(format!("Invalid size: {size_str}")))?;

    let order_type = FuturesOrderType::from_str(order_type_str)
        .map_err(|e| BybitError::Paper(e))?;

    let price = price_str
        .map(|s| s.parse::<f64>())
        .transpose()
        .map_err(|_| BybitError::Paper(format!("Invalid price: {}", price_str.unwrap_or(""))))?;

    let stop_price = stop_price_str
        .map(|s| s.parse::<f64>())
        .transpose()
        .map_err(|_| BybitError::Paper(format!("Invalid stop_price: {}", stop_price_str.unwrap_or(""))))?;

    let trigger_signal = trigger_signal_str
        .map(|s| TriggerSignal::from_str(s))
        .transpose()
        .map_err(|e| BybitError::Paper(e))?;

    let leverage = leverage_str
        .map(|s| s.parse::<f64>())
        .transpose()
        .map_err(|_| BybitError::Paper(format!("Invalid leverage: {}", leverage_str.unwrap_or(""))))?;

    let trailing_max_dev = trailing_max_dev_str
        .map(|s| s.parse::<f64>())
        .transpose()
        .map_err(|_| BybitError::Paper(format!("Invalid trailing_stop_max_deviation")))?;

    let market = futures_paper::fetch_market_snapshot(client, category, symbol).await?;

    let params = OrderParams {
        symbol: symbol.to_uppercase(),
        side,
        size,
        order_type,
        price,
        stop_price,
        trigger_signal,
        client_order_id,
        reduce_only,
        leverage,
        trailing_stop_max_deviation: trailing_max_dev,
        trailing_stop_deviation_unit: trailing_dev_unit.map(str::to_string),
    };

    let _lock = futures_paper::StateLock::acquire()?;
    let mut state = futures_paper::load_state()?;
    let result = state.place_order(params, &market)?;
    futures_paper::save_state(&state)?;

    let fills: Vec<Value> = result.fills.iter().map(fill_to_json).collect();

    Ok(json!({
        "mode": "futures_paper",
        "order_id": result.order_id,
        "status": format!("{:?}", result.status).to_lowercase(),
        "symbol": symbol.to_uppercase(),
        "side": side,
        "size": size,
        "order_type": order_type_str,
        "fills": fills,
        "message": result.message,
    }))
}

async fn cmd_orders(client: &BybitClient, category: &str) -> BybitResult<Value> {
    // Reconcile first
    {
        let _lock = futures_paper::StateLock::acquire()?;
        let mut state = futures_paper::load_state()?;
        let symbols: Vec<String> = state
            .open_orders
            .iter()
            .map(|o| o.symbol.clone())
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();

        if !symbols.is_empty() {
            if let Ok((marks, lasts, indexes, fundings)) =
                futures_paper::fetch_all_market_data(client, category, &symbols).await
            {
                state.reconcile(&marks, &lasts, &indexes, &fundings, &HashMap::new());
                let _ = futures_paper::save_state(&state);
            }
        }
    }

    let state = futures_paper::load_state()?;
    let orders: Vec<Value> = state.open_orders.iter().map(order_to_json).collect();
    Ok(json!({
        "mode": "futures_paper",
        "orders": orders,
        "count": orders.len(),
    }))
}

fn cmd_order_status(order_id: &str) -> BybitResult<Value> {
    let state = futures_paper::load_state()?;
    let order = state
        .open_orders
        .iter()
        .find(|o| o.id == order_id)
        .ok_or_else(|| BybitError::Paper(format!("Order {order_id} not found in open orders.")))?;

    Ok(order_to_json(order))
}

fn cmd_edit_order(
    order_id: &str,
    size_str: Option<&str>,
    price_str: Option<&str>,
    stop_price_str: Option<&str>,
) -> BybitResult<Value> {
    let size = size_str
        .map(|s| s.parse::<f64>())
        .transpose()
        .map_err(|_| BybitError::Paper(format!("Invalid size: {}", size_str.unwrap_or(""))))?;

    let price = price_str
        .map(|s| s.parse::<f64>())
        .transpose()
        .map_err(|_| BybitError::Paper(format!("Invalid price: {}", price_str.unwrap_or(""))))?;

    let stop_price = stop_price_str
        .map(|s| s.parse::<f64>())
        .transpose()
        .map_err(|_| BybitError::Paper(format!("Invalid stop_price: {}", stop_price_str.unwrap_or(""))))?;

    let _lock = futures_paper::StateLock::acquire()?;
    let mut state = futures_paper::load_state()?;
    state.edit_order(order_id, size, price, stop_price)?;
    futures_paper::save_state(&state)?;

    let order = state
        .open_orders
        .iter()
        .find(|o| o.id == order_id)
        .map(order_to_json)
        .unwrap_or(json!({"order_id": order_id, "status": "edited"}));

    Ok(json!({
        "mode": "futures_paper",
        "status": "edited",
        "order": order,
    }))
}

fn cmd_cancel(order_id: Option<&str>, cli_ord_id: Option<&str>) -> BybitResult<Value> {
    let _lock = futures_paper::StateLock::acquire()?;
    let mut state = futures_paper::load_state()?;
    let order = state.cancel_order(order_id, cli_ord_id)?;
    futures_paper::save_state(&state)?;

    Ok(json!({
        "mode": "futures_paper",
        "status": "cancelled",
        "order": order_to_json(&order),
    }))
}

fn cmd_cancel_all(symbol: Option<&str>) -> BybitResult<Value> {
    let _lock = futures_paper::StateLock::acquire()?;
    let mut state = futures_paper::load_state()?;
    let cancelled = state.cancel_all(symbol);
    futures_paper::save_state(&state)?;

    let orders: Vec<Value> = cancelled.iter().map(order_to_json).collect();
    Ok(json!({
        "mode": "futures_paper",
        "status": "cancelled",
        "cancelled_count": orders.len(),
        "orders": orders,
    }))
}

async fn cmd_batch_order(client: &BybitClient, orders_json: &str) -> BybitResult<Value> {
    // Support @file.json syntax
    let json_str = if let Some(path) = orders_json.strip_prefix('@') {
        std::fs::read_to_string(path)
            .map_err(|e| BybitError::Paper(format!("Could not read orders file: {e}")))?
    } else {
        orders_json.to_string()
    };

    let raw: Vec<serde_json::Value> = serde_json::from_str(&json_str)
        .map_err(|e| BybitError::Paper(format!("Invalid JSON for batch orders: {e}")))?;

    let mut params_list: Vec<OrderParams> = Vec::new();
    for (i, item) in raw.iter().enumerate() {
        let symbol = item["symbol"]
            .as_str()
            .ok_or_else(|| BybitError::Paper(format!("Order {i}: missing symbol")))?
            .to_uppercase();
        let side_str = item["side"]
            .as_str()
            .ok_or_else(|| BybitError::Paper(format!("Order {i}: missing side")))?;
        let side = Side::from_buy_sell(side_str)
            .ok_or_else(|| BybitError::Paper(format!("Order {i}: invalid side: {side_str}")))?;
        let size: f64 = item["size"]
            .as_str()
            .and_then(|s| s.parse().ok())
            .or_else(|| item["size"].as_f64())
            .ok_or_else(|| BybitError::Paper(format!("Order {i}: missing/invalid size")))?;
        let order_type_str = item["type"].as_str().unwrap_or("limit");
        let order_type = FuturesOrderType::from_str(order_type_str)
            .map_err(|e| BybitError::Paper(format!("Order {i}: {e}")))?;
        let price = item["price"]
            .as_str()
            .and_then(|s| s.parse().ok())
            .or_else(|| item["price"].as_f64());
        let stop_price = item["stop_price"]
            .as_str()
            .and_then(|s| s.parse().ok())
            .or_else(|| item["stop_price"].as_f64());
        let leverage = item["leverage"]
            .as_str()
            .and_then(|s| s.parse().ok())
            .or_else(|| item["leverage"].as_f64());

        params_list.push(OrderParams {
            symbol,
            side,
            size,
            order_type,
            price,
            stop_price,
            trigger_signal: None,
            client_order_id: item["client_order_id"].as_str().map(str::to_string),
            reduce_only: item["reduce_only"].as_bool().unwrap_or(false),
            leverage,
            trailing_stop_max_deviation: None,
            trailing_stop_deviation_unit: None,
        });
    }

    // Collect unique symbols and fetch market snapshots
    let symbols: Vec<String> = params_list
        .iter()
        .map(|p| p.symbol.clone())
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .collect();

    let state_category = futures_paper::load_state()?.category.clone();
    let mut snapshots: HashMap<String, MarketSnapshot> = HashMap::new();
    for sym in &symbols {
        if let Ok(snap) =
            futures_paper::fetch_market_snapshot(client, &state_category, sym).await
        {
            snapshots.insert(sym.clone(), snap);
        }
    }

    let _lock = futures_paper::StateLock::acquire()?;
    let mut state = futures_paper::load_state()?;
    let results = state.batch_orders(params_list, &snapshots);
    futures_paper::save_state(&state)?;

    let results_json: Vec<Value> = results
        .iter()
        .map(|r| {
            json!({
                "symbol": r.symbol,
                "success": r.success,
                "order_id": r.order_id,
                "error": r.error,
            })
        })
        .collect();

    Ok(json!({
        "mode": "futures_paper",
        "results": results_json,
        "total": results_json.len(),
        "succeeded": results.iter().filter(|r| r.success).count(),
        "failed": results.iter().filter(|r| !r.success).count(),
    }))
}

async fn cmd_positions(client: &BybitClient, category: &str) -> BybitResult<Value> {
    let state = futures_paper::load_state()?;
    let symbols: Vec<String> = state.positions.iter().map(|p| p.symbol.clone()).collect();

    let mark_prices = if symbols.is_empty() {
        HashMap::new()
    } else {
        let (marks, _, _, _) =
            futures_paper::fetch_all_market_data(client, category, &symbols).await?;
        marks
    };

    let positions: Vec<Value> = state
        .positions
        .iter()
        .map(|p| position_to_json(p, mark_prices.get(&p.symbol).copied()))
        .collect();

    Ok(json!({
        "mode": "futures_paper",
        "positions": positions,
        "count": positions.len(),
    }))
}

fn cmd_fills() -> BybitResult<Value> {
    let state = futures_paper::load_state()?;
    let fills: Vec<Value> = state.fills.iter().map(fill_to_json).collect();
    Ok(json!({
        "mode": "futures_paper",
        "fills": fills,
        "count": fills.len(),
    }))
}

fn cmd_history() -> BybitResult<Value> {
    let state = futures_paper::load_state()?;
    let events: Vec<Value> = state
        .history
        .iter()
        .map(|e| {
            json!({
                "id": e.id,
                "event_type": e.event_type,
                "symbol": e.symbol,
                "amount": e.amount,
                "details": e.details,
                "timestamp": e.timestamp,
            })
        })
        .collect();
    Ok(json!({
        "mode": "futures_paper",
        "history": events,
        "count": events.len(),
    }))
}

fn cmd_leverage(symbol: Option<&str>) -> BybitResult<Value> {
    let state = futures_paper::load_state()?;
    let prefs: Vec<Value> = if let Some(sym) = symbol {
        let sym_upper = sym.to_uppercase();
        let lev = state
            .leverage_preferences
            .get(&sym_upper)
            .copied()
            .unwrap_or(crate::futures_paper::MAX_LEVERAGE / 10.0);
        vec![json!({"symbol": sym_upper, "leverage": lev})]
    } else {
        state
            .leverage_preferences
            .iter()
            .map(|(k, v)| json!({"symbol": k, "leverage": v}))
            .collect()
    };

    Ok(json!({
        "mode": "futures_paper",
        "leverage_preferences": prefs,
        "default_leverage": 10.0,
        "max_leverage": MAX_LEVERAGE,
    }))
}

fn cmd_set_leverage(symbol: &str, leverage_str: &str) -> BybitResult<Value> {
    let leverage: f64 = leverage_str
        .parse()
        .map_err(|_| BybitError::Paper(format!("Invalid leverage: {leverage_str}")))?;

    if leverage <= 0.0 || leverage > MAX_LEVERAGE {
        return Err(BybitError::Paper(format!(
            "Leverage must be between 1 and {MAX_LEVERAGE}."
        )));
    }

    let _lock = futures_paper::StateLock::acquire()?;
    let mut state = futures_paper::load_state()?;
    let sym_upper = symbol.to_uppercase();
    state
        .leverage_preferences
        .insert(sym_upper.clone(), leverage);
    futures_paper::save_state(&state)?;

    Ok(json!({
        "mode": "futures_paper",
        "symbol": sym_upper,
        "leverage": leverage,
        "status": "set",
    }))
}

// ---------------------------------------------------------------------------
// Path helper (re-exported for use by other modules)
// ---------------------------------------------------------------------------

pub fn futures_paper_state_path() -> BybitResult<std::path::PathBuf> {
    futures_paper::futures_paper_state_path()
}
