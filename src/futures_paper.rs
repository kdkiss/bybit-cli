// Futures paper trading state machine — simulates perpetual futures on top of
// Bybit public market data without touching real funds or credentials.

use std::collections::HashMap;
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::time::Duration;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use uuid::Uuid;

use crate::client::BybitClient;
use crate::config;
use crate::errors::{BybitError, BybitResult};

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

pub const DEFAULT_FUTURES_TAKER_FEE_RATE: f64 = 0.00055; // 0.055% Bybit taker
pub const DEFAULT_MAINTENANCE_MARGIN_RATE: f64 = 0.005; // 0.5% standard linear
pub const MAX_LEVERAGE: f64 = 100.0;
const DEFAULT_LEVERAGE: f64 = 10.0;
const DEFAULT_CATEGORY: &str = "linear";
const FUNDING_INTERVAL_HOURS: i64 = 8;
const LOCK_TIMEOUT: Duration = Duration::from_secs(5);
const LOCK_POLL_INTERVAL: Duration = Duration::from_millis(50);

// ---------------------------------------------------------------------------
// Core enums
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Side {
    Long,
    Short,
}

impl Side {
    pub fn opposite(self) -> Self {
        match self {
            Side::Long => Side::Short,
            Side::Short => Side::Long,
        }
    }

    pub fn from_buy_sell(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "buy" | "long" => Some(Side::Long),
            "sell" | "short" => Some(Side::Short),
            _ => None,
        }
    }
}

impl std::fmt::Display for Side {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Side::Long => write!(f, "long"),
            Side::Short => write!(f, "short"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FuturesOrderType {
    Market,
    Limit,
    Post,
    Stop,
    TakeProfit,
    Ioc,
    TrailingStop,
    Fok,
}

impl std::str::FromStr for FuturesOrderType {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().replace('-', "_").as_str() {
            "market" => Ok(Self::Market),
            "limit" => Ok(Self::Limit),
            "post" => Ok(Self::Post),
            "stop" => Ok(Self::Stop),
            "take_profit" | "take-profit" => Ok(Self::TakeProfit),
            "ioc" => Ok(Self::Ioc),
            "trailing_stop" | "trailing-stop" => Ok(Self::TrailingStop),
            "fok" => Ok(Self::Fok),
            other => Err(format!("unknown order type: {other}")),
        }
    }
}

impl std::fmt::Display for FuturesOrderType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::Market => "market",
            Self::Limit => "limit",
            Self::Post => "post",
            Self::Stop => "stop",
            Self::TakeProfit => "take-profit",
            Self::Ioc => "ioc",
            Self::TrailingStop => "trailing-stop",
            Self::Fok => "fok",
        };
        write!(f, "{s}")
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum OrderStatus {
    Open,
    Triggered,
    Filled,
    Cancelled,
    Rejected,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TriggerSignal {
    Mark,
    Index,
    Last,
}

impl std::str::FromStr for TriggerSignal {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "mark" => Ok(Self::Mark),
            "index" => Ok(Self::Index),
            "last" => Ok(Self::Last),
            other => Err(format!("unknown trigger signal: {other}")),
        }
    }
}

// ---------------------------------------------------------------------------
// Primary data structures
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FuturesPaperState {
    pub collateral: f64,
    pub currency: String,
    pub category: String,
    pub fee_rate: f64,
    pub starting_collateral: f64,
    pub open_orders: Vec<FuturesPaperOrder>,
    pub positions: Vec<FuturesPaperPosition>,
    pub fills: Vec<FuturesPaperFill>,
    pub history: Vec<FuturesPaperHistoryEvent>,
    pub leverage_preferences: HashMap<String, f64>,
    pub maintenance_margin_fallback_used: bool,
    next_id: u64,
    pub created_at: String,
    pub updated_at: String,
    pub last_reconciled_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FuturesPaperOrder {
    pub id: String,
    pub symbol: String,
    pub side: Side,
    pub size: f64,
    pub filled_size: f64,
    pub order_type: FuturesOrderType,
    pub price: Option<f64>,
    pub stop_price: Option<f64>,
    pub trigger_signal: Option<TriggerSignal>,
    pub client_order_id: Option<String>,
    pub reduce_only: bool,
    pub leverage: f64,
    pub reserved_margin: f64,
    pub status: OrderStatus,
    pub trailing_stop_max_deviation: Option<f64>,
    pub trailing_stop_deviation_unit: Option<String>,
    pub trailing_anchor: Option<f64>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FuturesPaperPosition {
    pub symbol: String,
    pub side: Side,
    pub size: f64,
    pub entry_price: f64,
    pub leverage: f64,
    pub unrealized_funding: f64,
    pub last_funding_time: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FuturesPaperFill {
    pub id: String,
    pub order_id: String,
    pub symbol: String,
    pub side: Side,
    pub size: f64,
    pub price: f64,
    pub fee: f64,
    pub realized_pnl: Option<f64>,
    pub fill_type: String,
    pub client_order_id: Option<String>,
    pub filled_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FuturesPaperHistoryEvent {
    pub id: String,
    pub event_type: String,
    pub symbol: Option<String>,
    pub amount: f64,
    pub details: String,
    pub timestamp: String,
}

// ---------------------------------------------------------------------------
// Supporting types
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct OrderParams {
    pub symbol: String,
    pub side: Side,
    pub size: f64,
    pub order_type: FuturesOrderType,
    pub price: Option<f64>,
    pub stop_price: Option<f64>,
    pub trigger_signal: Option<TriggerSignal>,
    pub client_order_id: Option<String>,
    pub reduce_only: bool,
    pub leverage: Option<f64>,
    pub trailing_stop_max_deviation: Option<f64>,
    pub trailing_stop_deviation_unit: Option<String>,
}

#[derive(Debug, Clone)]
pub struct MarketSnapshot {
    pub bid: f64,
    pub ask: f64,
    pub last: f64,
    pub mark: f64,
    pub index: f64,
    pub ask_levels: Vec<(f64, f64)>,
    pub bid_levels: Vec<(f64, f64)>,
}

#[derive(Debug)]
pub struct OrderPlacementResult {
    pub order_id: String,
    pub status: OrderStatus,
    pub fills: Vec<FuturesPaperFill>,
    pub message: Option<String>,
}

#[derive(Debug)]
pub struct BatchOrderResult {
    pub symbol: String,
    pub success: bool,
    pub order_id: Option<String>,
    pub error: Option<String>,
}

#[derive(Debug, Default)]
pub struct ReconcileResult {
    pub fills: Vec<FuturesPaperFill>,
    pub liquidations: Vec<FuturesPaperFill>,
    pub funding_events: Vec<FuturesPaperHistoryEvent>,
}

// ---------------------------------------------------------------------------
// File locking
// ---------------------------------------------------------------------------

pub struct StateLock {
    path: PathBuf,
    token: String,
}

impl StateLock {
    pub fn acquire() -> BybitResult<Self> {
        let path = state_lock_path()?;
        let token = Uuid::new_v4().to_string();
        let deadline = std::time::Instant::now() + LOCK_TIMEOUT;

        loop {
            // Try to write our token
            if let Ok(mut f) = fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&path)
            {
                let _ = f.write_all(token.as_bytes());
                let _ = f.sync_all();

                // Read back and verify we own it
                if let Ok(contents) = fs::read_to_string(&path) {
                    if contents.trim() == token {
                        return Ok(Self { path, token });
                    }
                }
                // Token mismatch — someone else snuck in; delete and retry
                let _ = fs::remove_file(&path);
            }

            if std::time::Instant::now() >= deadline {
                return Err(BybitError::Paper(
                    "Could not acquire futures paper state lock — another process may be running."
                        .to_string(),
                ));
            }

            std::thread::sleep(LOCK_POLL_INTERVAL);
        }
    }
}

impl Drop for StateLock {
    fn drop(&mut self) {
        if let Ok(contents) = fs::read_to_string(&self.path) {
            if contents.trim() == self.token {
                let _ = fs::remove_file(&self.path);
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Persistence helpers
// ---------------------------------------------------------------------------

fn state_path() -> BybitResult<PathBuf> {
    Ok(config::config_dir()?.join("futures-paper-state.json"))
}

pub fn futures_paper_state_path() -> BybitResult<PathBuf> {
    state_path()
}

fn state_lock_path() -> BybitResult<PathBuf> {
    Ok(config::config_dir()?.join("futures-paper-state.lock"))
}

pub fn load_state() -> BybitResult<FuturesPaperState> {
    let path = state_path()?;
    if !path.exists() {
        return Err(BybitError::Paper(
            "Futures paper account not initialized. Run `bybit futures paper init` first."
                .to_string(),
        ));
    }
    let contents = fs::read_to_string(&path)?;
    let state: FuturesPaperState = serde_json::from_str(&contents).map_err(|e| {
        BybitError::Parse(format!("Failed to parse futures paper state: {e}"))
    })?;
    Ok(state)
}

pub fn save_state(state: &FuturesPaperState) -> BybitResult<()> {
    let path = state_path()?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let data = serde_json::to_string_pretty(state)
        .map_err(|e| BybitError::Parse(format!("Failed to serialize state: {e}")))?;
    let tmp = path.with_extension("json.tmp");
    {
        let mut file = fs::File::create(&tmp)?;
        file.write_all(data.as_bytes())?;
        file.sync_all()?;
    }
    fs::rename(tmp, path)?;
    Ok(())
}

// ---------------------------------------------------------------------------
// State impl
// ---------------------------------------------------------------------------

impl FuturesPaperState {
    pub fn new(collateral: f64, currency: &str, fee_rate: f64) -> Self {
        let now = now_rfc3339();
        Self {
            collateral,
            currency: currency.to_uppercase(),
            category: DEFAULT_CATEGORY.to_string(),
            fee_rate,
            starting_collateral: collateral,
            open_orders: Vec::new(),
            positions: Vec::new(),
            fills: Vec::new(),
            history: Vec::new(),
            leverage_preferences: HashMap::new(),
            maintenance_margin_fallback_used: false,
            next_id: 1,
            created_at: now.clone(),
            updated_at: now,
            last_reconciled_at: None,
        }
    }

    pub fn reset(
        &mut self,
        collateral: Option<f64>,
        currency: Option<&str>,
        fee_rate: Option<f64>,
    ) {
        let collateral = collateral.unwrap_or(self.starting_collateral);
        let currency = currency
            .map(|c| c.to_uppercase())
            .unwrap_or_else(|| self.currency.clone());
        let fee_rate = fee_rate.unwrap_or(self.fee_rate);
        *self = Self::new(collateral, &currency, fee_rate);
    }

    fn next_id(&mut self) -> String {
        let id = self.next_id;
        self.next_id += 1;
        format!("fp-{id:06}")
    }

    // -----------------------------------------------------------------------
    // Margin accounting
    // -----------------------------------------------------------------------

    pub fn position_margin(&self) -> f64 {
        self.positions
            .iter()
            .map(|p| p.size * p.entry_price / p.leverage)
            .sum()
    }

    pub fn reserved_order_margin(&self) -> f64 {
        self.open_orders
            .iter()
            .filter(|o| o.status == OrderStatus::Open || o.status == OrderStatus::Triggered)
            .map(|o| o.reserved_margin)
            .sum()
    }

    pub fn unrealized_pnl(&self, mark_prices: &HashMap<String, f64>) -> f64 {
        self.positions
            .iter()
            .map(|p| {
                let mark = mark_prices.get(&p.symbol).copied().unwrap_or(p.entry_price);
                compute_unrealized_pnl(p, mark)
            })
            .sum()
    }

    pub fn used_margin(&self) -> f64 {
        self.position_margin() + self.reserved_order_margin()
    }

    pub fn available_margin(&self, mark_prices: &HashMap<String, f64>) -> f64 {
        (self.collateral + self.unrealized_pnl(mark_prices) - self.used_margin()).max(0.0)
    }

    pub fn resolve_leverage(
        &self,
        order_leverage: Option<f64>,
        symbol: &str,
    ) -> BybitResult<f64> {
        let lev = order_leverage
            .or_else(|| self.leverage_preferences.get(symbol).copied())
            .unwrap_or(DEFAULT_LEVERAGE);
        if lev <= 0.0 || lev > MAX_LEVERAGE {
            return Err(BybitError::Paper(format!(
                "Leverage must be between 0 and {MAX_LEVERAGE}."
            )));
        }
        Ok(lev)
    }

    // -----------------------------------------------------------------------
    // Order placement
    // -----------------------------------------------------------------------

    pub fn place_order(
        &mut self,
        params: OrderParams,
        market: &MarketSnapshot,
    ) -> BybitResult<OrderPlacementResult> {
        validate_finite_positive(params.size, "size")?;
        let symbol = params.symbol.to_uppercase();
        let leverage = self.resolve_leverage(params.leverage, &symbol)?;

        let mark_prices: HashMap<String, f64> =
            [(symbol.clone(), market.mark)].into_iter().collect();

        match params.order_type {
            FuturesOrderType::Market => self.place_market_order(&params, leverage, market),
            FuturesOrderType::Limit => {
                self.place_limit_order(&params, leverage, market, &mark_prices)
            }
            FuturesOrderType::Post => self.place_post_order(&params, leverage, market),
            FuturesOrderType::Ioc => self.place_ioc_order(&params, leverage, market),
            FuturesOrderType::Fok => self.place_fok_order(&params, leverage, market),
            FuturesOrderType::Stop
            | FuturesOrderType::TakeProfit
            | FuturesOrderType::TrailingStop => {
                self.place_triggered_order(&params, leverage, &mark_prices)
            }
        }
    }

    fn place_market_order(
        &mut self,
        params: &OrderParams,
        leverage: f64,
        market: &MarketSnapshot,
    ) -> BybitResult<OrderPlacementResult> {
        let fill_price = match params.side {
            Side::Long => market.ask,
            Side::Short => market.bid,
        };
        validate_finite_positive(fill_price, "market price")?;

        // Check reduce-only
        if params.reduce_only {
            self.validate_reduce_only(&params.symbol, params.side, params.size)?;
        }

        let available = self.available_margin(&HashMap::new());
        let required_margin = params.size * fill_price / leverage;
        if !params.reduce_only && available < required_margin {
            return Err(BybitError::Paper(format!(
                "Insufficient margin: need {required_margin:.4}, available {available:.4}"
            )));
        }

        let order_id = self.next_id();
        let fee = params.size * fill_price * self.fee_rate;
        self.collateral -= fee;

        let fill = self.apply_fill(
            &format!("fill-{order_id}"),
            &order_id,
            &params.symbol.to_uppercase(),
            params.side,
            params.size,
            fill_price,
            fee,
            params.client_order_id.clone(),
            "market",
            leverage,
        );

        self.updated_at = now_rfc3339();
        Ok(OrderPlacementResult {
            order_id,
            status: OrderStatus::Filled,
            fills: vec![fill],
            message: None,
        })
    }

    fn place_limit_order(
        &mut self,
        params: &OrderParams,
        leverage: f64,
        market: &MarketSnapshot,
        mark_prices: &HashMap<String, f64>,
    ) -> BybitResult<OrderPlacementResult> {
        let price = params.price.ok_or_else(|| {
            BybitError::Paper("Limit orders require --price".to_string())
        })?;
        validate_finite_positive(price, "price")?;

        // Check if immediately fillable
        let immediately_fillable = match params.side {
            Side::Long => price >= market.ask,
            Side::Short => price <= market.bid,
        };

        if immediately_fillable {
            let fill_price = match params.side {
                Side::Long => market.ask.min(price),
                Side::Short => market.bid.max(price),
            };

            if params.reduce_only {
                self.validate_reduce_only(&params.symbol, params.side, params.size)?;
            }

            let available = self.available_margin(mark_prices);
            let required_margin = params.size * fill_price / leverage;
            if !params.reduce_only && available < required_margin {
                return Err(BybitError::Paper(format!(
                    "Insufficient margin: need {required_margin:.4}, available {available:.4}"
                )));
            }

            let order_id = self.next_id();
            let fee = params.size * fill_price * self.fee_rate;
            self.collateral -= fee;

            let fill = self.apply_fill(
                &format!("fill-{order_id}"),
                &order_id,
                &params.symbol.to_uppercase(),
                params.side,
                params.size,
                fill_price,
                fee,
                params.client_order_id.clone(),
                "limit",
                leverage,
            );

            self.updated_at = now_rfc3339();
            return Ok(OrderPlacementResult {
                order_id,
                status: OrderStatus::Filled,
                fills: vec![fill],
                message: None,
            });
        }

        // Resting limit order
        let reserved_margin = params.size * price / leverage;
        let available = self.available_margin(mark_prices);
        if !params.reduce_only && available < reserved_margin {
            return Err(BybitError::Paper(format!(
                "Insufficient margin to reserve: need {reserved_margin:.4}, available {available:.4}"
            )));
        }

        let order_id = self.next_id();
        let order = FuturesPaperOrder {
            id: order_id.clone(),
            symbol: params.symbol.to_uppercase(),
            side: params.side,
            size: params.size,
            filled_size: 0.0,
            order_type: FuturesOrderType::Limit,
            price: Some(price),
            stop_price: None,
            trigger_signal: None,
            client_order_id: params.client_order_id.clone(),
            reduce_only: params.reduce_only,
            leverage,
            reserved_margin,
            status: OrderStatus::Open,
            trailing_stop_max_deviation: None,
            trailing_stop_deviation_unit: None,
            trailing_anchor: None,
            created_at: now_rfc3339(),
            updated_at: now_rfc3339(),
        };

        if !params.reduce_only {
            self.collateral -= reserved_margin * leverage / leverage; // noop, margin tracked via reserved_order_margin
        }
        self.open_orders.push(order);
        self.updated_at = now_rfc3339();

        Ok(OrderPlacementResult {
            order_id,
            status: OrderStatus::Open,
            fills: vec![],
            message: None,
        })
    }

    fn place_post_order(
        &mut self,
        params: &OrderParams,
        leverage: f64,
        market: &MarketSnapshot,
    ) -> BybitResult<OrderPlacementResult> {
        let price = params.price.ok_or_else(|| {
            BybitError::Paper("Post orders require --price".to_string())
        })?;
        validate_finite_positive(price, "price")?;

        // Post-only: cancel if would immediately cross
        let would_cross = match params.side {
            Side::Long => price >= market.ask,
            Side::Short => price <= market.bid,
        };

        if would_cross {
            let order_id = self.next_id();
            return Ok(OrderPlacementResult {
                order_id,
                status: OrderStatus::Cancelled,
                fills: vec![],
                message: Some("Post-only order would cross the spread and was cancelled.".to_string()),
            });
        }

        let reserved_margin = params.size * price / leverage;
        let mark_prices: HashMap<String, f64> =
            [(params.symbol.to_uppercase(), market.mark)].into_iter().collect();
        let available = self.available_margin(&mark_prices);
        if !params.reduce_only && available < reserved_margin {
            return Err(BybitError::Paper(format!(
                "Insufficient margin: need {reserved_margin:.4}, available {available:.4}"
            )));
        }

        let order_id = self.next_id();
        let order = FuturesPaperOrder {
            id: order_id.clone(),
            symbol: params.symbol.to_uppercase(),
            side: params.side,
            size: params.size,
            filled_size: 0.0,
            order_type: FuturesOrderType::Post,
            price: Some(price),
            stop_price: None,
            trigger_signal: None,
            client_order_id: params.client_order_id.clone(),
            reduce_only: params.reduce_only,
            leverage,
            reserved_margin,
            status: OrderStatus::Open,
            trailing_stop_max_deviation: None,
            trailing_stop_deviation_unit: None,
            trailing_anchor: None,
            created_at: now_rfc3339(),
            updated_at: now_rfc3339(),
        };

        self.open_orders.push(order);
        self.updated_at = now_rfc3339();

        Ok(OrderPlacementResult {
            order_id,
            status: OrderStatus::Open,
            fills: vec![],
            message: None,
        })
    }

    fn place_ioc_order(
        &mut self,
        params: &OrderParams,
        leverage: f64,
        market: &MarketSnapshot,
    ) -> BybitResult<OrderPlacementResult> {
        let price = params.price.ok_or_else(|| {
            BybitError::Paper("IOC orders require --price".to_string())
        })?;
        validate_finite_positive(price, "price")?;

        let fillable = match params.side {
            Side::Long => price >= market.ask,
            Side::Short => price <= market.bid,
        };

        if !fillable {
            let order_id = self.next_id();
            return Ok(OrderPlacementResult {
                order_id,
                status: OrderStatus::Cancelled,
                fills: vec![],
                message: Some("IOC order could not fill immediately and was cancelled.".to_string()),
            });
        }

        let fill_price = match params.side {
            Side::Long => market.ask.min(price),
            Side::Short => market.bid.max(price),
        };

        let mark_prices: HashMap<String, f64> =
            [(params.symbol.to_uppercase(), market.mark)].into_iter().collect();
        let available = self.available_margin(&mark_prices);
        let required_margin = params.size * fill_price / leverage;
        if !params.reduce_only && available < required_margin {
            return Err(BybitError::Paper(format!(
                "Insufficient margin: need {required_margin:.4}, available {available:.4}"
            )));
        }

        let order_id = self.next_id();
        let fee = params.size * fill_price * self.fee_rate;
        self.collateral -= fee;

        let fill = self.apply_fill(
            &format!("fill-{order_id}"),
            &order_id,
            &params.symbol.to_uppercase(),
            params.side,
            params.size,
            fill_price,
            fee,
            params.client_order_id.clone(),
            "ioc",
            leverage,
        );

        self.updated_at = now_rfc3339();
        Ok(OrderPlacementResult {
            order_id,
            status: OrderStatus::Filled,
            fills: vec![fill],
            message: None,
        })
    }

    fn place_fok_order(
        &mut self,
        params: &OrderParams,
        leverage: f64,
        market: &MarketSnapshot,
    ) -> BybitResult<OrderPlacementResult> {
        let price = params.price.ok_or_else(|| {
            BybitError::Paper("FOK orders require --price".to_string())
        })?;
        validate_finite_positive(price, "price")?;

        let fillable = match params.side {
            Side::Long => price >= market.ask,
            Side::Short => price <= market.bid,
        };

        if !fillable {
            let order_id = self.next_id();
            return Ok(OrderPlacementResult {
                order_id,
                status: OrderStatus::Cancelled,
                fills: vec![],
                message: Some("FOK order could not fill in full immediately and was cancelled.".to_string()),
            });
        }

        let executable = compute_executable_depth(
            if params.side == Side::Long {
                &market.ask_levels
            } else {
                &market.bid_levels
            },
            params.side,
            price,
            params.size,
        );

        if executable < params.size - 1e-9 {
            let order_id = self.next_id();
            return Ok(OrderPlacementResult {
                order_id,
                status: OrderStatus::Cancelled,
                fills: vec![],
                message: Some(format!(
                    "FOK order: insufficient depth ({executable:.4} of {:.4} fillable).",
                    params.size
                )),
            });
        }

        let fill_price = match params.side {
            Side::Long => market.ask.min(price),
            Side::Short => market.bid.max(price),
        };

        let mark_prices: HashMap<String, f64> =
            [(params.symbol.to_uppercase(), market.mark)].into_iter().collect();
        let available = self.available_margin(&mark_prices);
        let required_margin = params.size * fill_price / leverage;
        if !params.reduce_only && available < required_margin {
            return Err(BybitError::Paper(format!(
                "Insufficient margin: need {required_margin:.4}, available {available:.4}"
            )));
        }

        let order_id = self.next_id();
        let fee = params.size * fill_price * self.fee_rate;
        self.collateral -= fee;

        let fill = self.apply_fill(
            &format!("fill-{order_id}"),
            &order_id,
            &params.symbol.to_uppercase(),
            params.side,
            params.size,
            fill_price,
            fee,
            params.client_order_id.clone(),
            "fok",
            leverage,
        );

        self.updated_at = now_rfc3339();
        Ok(OrderPlacementResult {
            order_id,
            status: OrderStatus::Filled,
            fills: vec![fill],
            message: None,
        })
    }

    fn place_triggered_order(
        &mut self,
        params: &OrderParams,
        leverage: f64,
        mark_prices: &HashMap<String, f64>,
    ) -> BybitResult<OrderPlacementResult> {
        let stop_price = params.stop_price.ok_or_else(|| {
            BybitError::Paper(format!(
                "{} orders require --stop-price",
                params.order_type
            ))
        })?;
        validate_finite_positive(stop_price, "stop_price")?;

        if params.order_type == FuturesOrderType::TrailingStop {
            let dev = params.trailing_stop_max_deviation.ok_or_else(|| {
                BybitError::Paper(
                    "trailing-stop orders require --trailing-stop-max-deviation".to_string(),
                )
            })?;
            validate_finite_positive(dev, "trailing_stop_max_deviation")?;

            let unit = params
                .trailing_stop_deviation_unit
                .as_deref()
                .unwrap_or("percent");
            if unit != "percent" && unit != "quote_currency" {
                return Err(BybitError::Paper(
                    "trailing_stop_deviation_unit must be 'percent' or 'quote_currency'".to_string(),
                ));
            }
        }

        // Validate stop direction relative to mark
        if let Some(mark) = mark_prices.get(&params.symbol.to_uppercase()) {
            match (params.side, params.order_type) {
                (Side::Long, FuturesOrderType::Stop) => {
                    if stop_price >= *mark {
                        return Err(BybitError::Paper(format!(
                            "Long stop price ({stop_price}) must be below mark ({mark})"
                        )));
                    }
                }
                (Side::Short, FuturesOrderType::Stop) => {
                    if stop_price <= *mark {
                        return Err(BybitError::Paper(format!(
                            "Short stop price ({stop_price}) must be above mark ({mark})"
                        )));
                    }
                }
                (Side::Long, FuturesOrderType::TakeProfit) => {
                    if stop_price <= *mark {
                        return Err(BybitError::Paper(format!(
                            "Long take-profit price ({stop_price}) must be above mark ({mark})"
                        )));
                    }
                }
                (Side::Short, FuturesOrderType::TakeProfit) => {
                    if stop_price >= *mark {
                        return Err(BybitError::Paper(format!(
                            "Short take-profit price ({stop_price}) must be below mark ({mark})"
                        )));
                    }
                }
                _ => {}
            }
        }

        let reserved_margin = if params.reduce_only {
            0.0
        } else {
            params.size * stop_price / leverage
        };

        let available = self.available_margin(mark_prices);
        if !params.reduce_only && available < reserved_margin {
            return Err(BybitError::Paper(format!(
                "Insufficient margin: need {reserved_margin:.4}, available {available:.4}"
            )));
        }

        let mark_anchor = mark_prices
            .get(&params.symbol.to_uppercase())
            .copied()
            .or(Some(stop_price));

        let order_id = self.next_id();
        let order = FuturesPaperOrder {
            id: order_id.clone(),
            symbol: params.symbol.to_uppercase(),
            side: params.side,
            size: params.size,
            filled_size: 0.0,
            order_type: params.order_type,
            price: params.price,
            stop_price: Some(stop_price),
            trigger_signal: params.trigger_signal,
            client_order_id: params.client_order_id.clone(),
            reduce_only: params.reduce_only,
            leverage,
            reserved_margin,
            status: OrderStatus::Open,
            trailing_stop_max_deviation: params.trailing_stop_max_deviation,
            trailing_stop_deviation_unit: params.trailing_stop_deviation_unit.clone(),
            trailing_anchor: mark_anchor,
            created_at: now_rfc3339(),
            updated_at: now_rfc3339(),
        };

        self.open_orders.push(order);
        self.updated_at = now_rfc3339();

        Ok(OrderPlacementResult {
            order_id,
            status: OrderStatus::Open,
            fills: vec![],
            message: None,
        })
    }

    // -----------------------------------------------------------------------
    // Order management
    // -----------------------------------------------------------------------

    pub fn edit_order(
        &mut self,
        order_id: &str,
        new_size: Option<f64>,
        new_price: Option<f64>,
        new_stop_price: Option<f64>,
    ) -> BybitResult<()> {
        let idx = self
            .open_orders
            .iter()
            .position(|o| o.id == order_id)
            .ok_or_else(|| BybitError::Paper(format!("Order {order_id} not found.")))?;

        let order = &mut self.open_orders[idx];
        if order.status != OrderStatus::Open {
            return Err(BybitError::Paper(format!(
                "Order {order_id} is not open (status: {:?}).",
                order.status
            )));
        }

        if let Some(size) = new_size {
            validate_finite_positive(size, "size")?;
            order.size = size;
        }
        if let Some(price) = new_price {
            validate_finite_positive(price, "price")?;
            order.price = Some(price);
            // Recompute reserved margin
            order.reserved_margin = order.size * price / order.leverage;
        }
        if let Some(stop_price) = new_stop_price {
            validate_finite_positive(stop_price, "stop_price")?;
            order.stop_price = Some(stop_price);
        }

        order.updated_at = now_rfc3339();
        self.updated_at = now_rfc3339();
        Ok(())
    }

    pub fn cancel_order(
        &mut self,
        order_id: Option<&str>,
        client_order_id: Option<&str>,
    ) -> BybitResult<FuturesPaperOrder> {
        let idx = self
            .open_orders
            .iter()
            .position(|o| {
                order_id.is_some_and(|id| o.id == id)
                    || client_order_id
                        .is_some_and(|cid| o.client_order_id.as_deref() == Some(cid))
            })
            .ok_or_else(|| {
                BybitError::Paper(
                    order_id
                        .map(|id| format!("Order {id} not found."))
                        .unwrap_or_else(|| "Order not found.".to_string()),
                )
            })?;

        let mut order = self.open_orders.remove(idx);
        order.status = OrderStatus::Cancelled;
        order.updated_at = now_rfc3339();
        self.updated_at = now_rfc3339();
        Ok(order)
    }

    pub fn cancel_all(&mut self, symbol_filter: Option<&str>) -> Vec<FuturesPaperOrder> {
        let mut cancelled = Vec::new();
        let mut remaining = Vec::new();

        for mut order in self.open_orders.drain(..) {
            let matches = symbol_filter
                .map(|s| order.symbol == s.to_uppercase())
                .unwrap_or(true);
            if matches {
                order.status = OrderStatus::Cancelled;
                order.updated_at = now_rfc3339();
                cancelled.push(order);
            } else {
                remaining.push(order);
            }
        }

        self.open_orders = remaining;
        self.updated_at = now_rfc3339();
        cancelled
    }

    pub fn batch_orders(
        &mut self,
        batch: Vec<OrderParams>,
        market_snapshots: &HashMap<String, MarketSnapshot>,
    ) -> Vec<BatchOrderResult> {
        let mut results = Vec::new();

        for params in batch {
            let symbol = params.symbol.to_uppercase();
            let snapshot = match market_snapshots.get(&symbol) {
                Some(s) => s.clone(),
                None => {
                    results.push(BatchOrderResult {
                        symbol,
                        success: false,
                        order_id: None,
                        error: Some("No market snapshot available for symbol.".to_string()),
                    });
                    continue;
                }
            };

            match self.place_order(params, &snapshot) {
                Ok(result) => results.push(BatchOrderResult {
                    symbol,
                    success: true,
                    order_id: Some(result.order_id),
                    error: None,
                }),
                Err(e) => results.push(BatchOrderResult {
                    symbol,
                    success: false,
                    order_id: None,
                    error: Some(e.to_string()),
                }),
            }
        }

        results
    }

    // -----------------------------------------------------------------------
    // Fill application and position netting
    // -----------------------------------------------------------------------

    fn apply_fill(
        &mut self,
        fill_id: &str,
        order_id: &str,
        symbol: &str,
        side: Side,
        size: f64,
        price: f64,
        fee: f64,
        client_order_id: Option<String>,
        fill_type: &str,
        leverage: f64,
    ) -> FuturesPaperFill {
        let realized_pnl = self.net_position(symbol, side, size, price, leverage);

        let fill = FuturesPaperFill {
            id: fill_id.to_string(),
            order_id: order_id.to_string(),
            symbol: symbol.to_string(),
            side,
            size,
            price,
            fee,
            realized_pnl,
            fill_type: fill_type.to_string(),
            client_order_id,
            filled_at: now_rfc3339(),
        };

        self.fills.push(fill.clone());

        if let Some(pnl) = realized_pnl {
            self.collateral += pnl;
            let event = FuturesPaperHistoryEvent {
                id: format!("evt-{}", self.next_id()),
                event_type: "realized_pnl".to_string(),
                symbol: Some(symbol.to_string()),
                amount: pnl,
                details: format!(
                    "Closed {side} position: {size} @ {price:.4}, PnL = {pnl:.4}"
                ),
                timestamp: now_rfc3339(),
            };
            self.history.push(event);
        }

        fill
    }

    fn net_position(
        &mut self,
        symbol: &str,
        fill_side: Side,
        fill_size: f64,
        fill_price: f64,
        leverage: f64,
    ) -> Option<f64> {
        let pos_idx = self.positions.iter().position(|p| p.symbol == symbol);

        match pos_idx {
            None => {
                // Open new position
                self.open_new_position(symbol, fill_side, fill_size, fill_price, leverage);
                None
            }
            Some(idx) => {
                let existing_side = self.positions[idx].side;
                if existing_side == fill_side {
                    // Add to position
                    let total_size = self.positions[idx].size + fill_size;
                    let new_entry = (self.positions[idx].entry_price * self.positions[idx].size
                        + fill_price * fill_size)
                        / total_size;
                    self.positions[idx].size = total_size;
                    self.positions[idx].entry_price = new_entry;
                    self.positions[idx].leverage = leverage;
                    self.positions[idx].updated_at = now_rfc3339();
                    None
                } else {
                    // Opposite side — reduce or flip
                    let entry = self.positions[idx].entry_price;
                    let existing_size = self.positions[idx].size;

                    if fill_size < existing_size {
                        // Partial close
                        let realized =
                            compute_realized_pnl(existing_side, entry, fill_price, fill_size);
                        self.positions[idx].size -= fill_size;
                        self.positions[idx].updated_at = now_rfc3339();
                        Some(realized)
                    } else if fill_size == existing_size {
                        // Full close
                        let realized =
                            compute_realized_pnl(existing_side, entry, fill_price, fill_size);
                        self.positions.remove(idx);
                        Some(realized)
                    } else {
                        // Flip — close existing and open new
                        let close_size = existing_size;
                        let open_size = fill_size - existing_size;
                        let realized =
                            compute_realized_pnl(existing_side, entry, fill_price, close_size);
                        self.positions.remove(idx);
                        self.open_new_position(symbol, fill_side, open_size, fill_price, leverage);
                        Some(realized)
                    }
                }
            }
        }
    }

    fn open_new_position(
        &mut self,
        symbol: &str,
        side: Side,
        size: f64,
        price: f64,
        leverage: f64,
    ) {
        let margin = size * price / leverage;
        self.collateral -= margin;
        self.positions.push(FuturesPaperPosition {
            symbol: symbol.to_string(),
            side,
            size,
            entry_price: price,
            leverage,
            unrealized_funding: 0.0,
            last_funding_time: None,
            created_at: now_rfc3339(),
            updated_at: now_rfc3339(),
        });
    }

    fn validate_reduce_only(
        &self,
        symbol: &str,
        side: Side,
        size: f64,
    ) -> BybitResult<()> {
        let pos = self
            .positions
            .iter()
            .find(|p| p.symbol == symbol.to_uppercase());
        match pos {
            None => Err(BybitError::Paper(
                "reduce_only: no open position to reduce.".to_string(),
            )),
            Some(p) if p.side == side => Err(BybitError::Paper(
                "reduce_only: order side matches position side.".to_string(),
            )),
            Some(p) if size > p.size => Err(BybitError::Paper(format!(
                "reduce_only: order size ({size}) exceeds position size ({}).",
                p.size
            ))),
            _ => Ok(()),
        }
    }

    // -----------------------------------------------------------------------
    // Reconciliation
    // -----------------------------------------------------------------------

    pub fn reconcile(
        &mut self,
        mark_prices: &HashMap<String, f64>,
        last_prices: &HashMap<String, f64>,
        index_prices: &HashMap<String, f64>,
        funding_rates: &HashMap<String, f64>,
        maintenance_rates: &HashMap<String, f64>,
    ) -> ReconcileResult {
        let mut result = ReconcileResult::default();

        self.reconcile_limit_orders(last_prices, mark_prices, &mut result.fills);
        self.reconcile_triggered_orders(
            mark_prices,
            last_prices,
            index_prices,
            &mut result.fills,
        );
        self.reconcile_liquidations(
            mark_prices,
            maintenance_rates,
            &mut result.liquidations,
        );
        self.reconcile_funding(funding_rates, mark_prices, &mut result.funding_events);

        self.last_reconciled_at = Some(now_rfc3339());
        self.updated_at = now_rfc3339();

        result
    }

    fn reconcile_limit_orders(
        &mut self,
        last_prices: &HashMap<String, f64>,
        mark_prices: &HashMap<String, f64>,
        fills: &mut Vec<FuturesPaperFill>,
    ) {
        let mut filled_ids: Vec<String> = Vec::new();

        for order in &self.open_orders {
            if order.status != OrderStatus::Open {
                continue;
            }
            if order.order_type != FuturesOrderType::Limit
                && order.order_type != FuturesOrderType::Post
            {
                continue;
            }
            let last = match last_prices.get(&order.symbol) {
                Some(p) => *p,
                None => continue,
            };
            if !last.is_finite() || last <= 0.0 {
                continue;
            }
            let price = match order.price {
                Some(p) => p,
                None => continue,
            };
            let triggered = match order.side {
                Side::Long => last <= price,
                Side::Short => last >= price,
            };
            if triggered {
                filled_ids.push(order.id.clone());
            }
        }

        for order_id in filled_ids {
            if let Some(idx) = self.open_orders.iter().position(|o| o.id == order_id) {
                let order = self.open_orders.remove(idx);
                let price = order.price.unwrap_or(0.0);
                if !price.is_finite() || price <= 0.0 {
                    continue;
                }
                let fee = order.size * price * self.fee_rate;
                self.collateral += order.reserved_margin; // release reserved
                self.collateral -= fee;

                let fill = self.apply_fill(
                    &format!("fill-{}", order.id),
                    &order.id,
                    &order.symbol,
                    order.side,
                    order.size,
                    price,
                    fee,
                    order.client_order_id.clone(),
                    "limit",
                    order.leverage,
                );
                fills.push(fill);
            }
        }

        // Update trailing anchors
        for order in &mut self.open_orders {
            if order.order_type != FuturesOrderType::TrailingStop {
                continue;
            }
            let mark = match mark_prices.get(&order.symbol) {
                Some(p) => *p,
                None => continue,
            };
            if !mark.is_finite() || mark <= 0.0 {
                continue;
            }
            match order.side {
                Side::Long => {
                    if order.trailing_anchor.map_or(true, |a| mark < a) {
                        order.trailing_anchor = Some(mark);
                    }
                }
                Side::Short => {
                    if order.trailing_anchor.map_or(true, |a| mark > a) {
                        order.trailing_anchor = Some(mark);
                    }
                }
            }
        }
    }

    fn reconcile_triggered_orders(
        &mut self,
        mark_prices: &HashMap<String, f64>,
        last_prices: &HashMap<String, f64>,
        index_prices: &HashMap<String, f64>,
        fills: &mut Vec<FuturesPaperFill>,
    ) {
        let mut triggered_ids: Vec<String> = Vec::new();

        for order in &self.open_orders {
            if order.status != OrderStatus::Open {
                continue;
            }
            match order.order_type {
                FuturesOrderType::Stop
                | FuturesOrderType::TakeProfit
                | FuturesOrderType::TrailingStop => {}
                _ => continue,
            }

            if order.order_type == FuturesOrderType::TrailingStop {
                let mark = match mark_prices.get(&order.symbol) {
                    Some(p) => *p,
                    None => continue,
                };
                if check_trailing_stop_trigger(order, mark) {
                    triggered_ids.push(order.id.clone());
                }
                continue;
            }

            let stop_price = match order.stop_price {
                Some(p) => p,
                None => continue,
            };

            let trigger_price = resolve_trigger_price(
                order.trigger_signal.unwrap_or(TriggerSignal::Mark),
                &order.symbol,
                mark_prices,
                last_prices,
                index_prices,
            );

            let trigger_price = match trigger_price {
                Some(p) if p.is_finite() && p > 0.0 => p,
                _ => continue,
            };

            let triggered = match order.order_type {
                FuturesOrderType::Stop => match order.side {
                    Side::Long => trigger_price <= stop_price,
                    Side::Short => trigger_price >= stop_price,
                },
                FuturesOrderType::TakeProfit => match order.side {
                    Side::Long => trigger_price >= stop_price,
                    Side::Short => trigger_price <= stop_price,
                },
                _ => false,
            };

            if triggered {
                triggered_ids.push(order.id.clone());
            }
        }

        for order_id in triggered_ids {
            if let Some(idx) = self.open_orders.iter().position(|o| o.id == order_id) {
                let order = self.open_orders.remove(idx);
                // Trigger fill at mark price or limit price if set
                let fill_price = if let Some(lp) = order.price {
                    lp
                } else {
                    order.stop_price.unwrap_or_else(|| {
                        mark_prices
                            .get(&order.symbol)
                            .copied()
                            .unwrap_or(order.stop_price.unwrap_or(0.0))
                    })
                };
                if !fill_price.is_finite() || fill_price <= 0.0 {
                    continue;
                }
                let fee = order.size * fill_price * self.fee_rate;
                self.collateral += order.reserved_margin;
                self.collateral -= fee;

                let fill = self.apply_fill(
                    &format!("fill-{}", order.id),
                    &order.id,
                    &order.symbol,
                    order.side,
                    order.size,
                    fill_price,
                    fee,
                    order.client_order_id.clone(),
                    &format!("{}", order.order_type),
                    order.leverage,
                );
                fills.push(fill);
            }
        }
    }

    fn reconcile_liquidations(
        &mut self,
        mark_prices: &HashMap<String, f64>,
        maintenance_rates: &HashMap<String, f64>,
        liquidations: &mut Vec<FuturesPaperFill>,
    ) {
        let mut liquidated: Vec<String> = Vec::new();

        for pos in &self.positions {
            let mark = match mark_prices.get(&pos.symbol) {
                Some(p) if p.is_finite() && *p > 0.0 => *p,
                _ => continue,
            };
            let maint_rate = maintenance_rates
                .get(&pos.symbol)
                .copied()
                .unwrap_or(DEFAULT_MAINTENANCE_MARGIN_RATE);
            let liq_price = compute_liquidation_price(pos, maint_rate);
            if !liq_price.is_finite() || liq_price <= 0.0 {
                continue;
            }
            let liquidated_now = match pos.side {
                Side::Long => mark <= liq_price,
                Side::Short => mark >= liq_price,
            };
            if liquidated_now {
                liquidated.push(pos.symbol.clone());
            }
        }

        for symbol in liquidated {
            if let Some(idx) = self.positions.iter().position(|p| p.symbol == symbol) {
                let pos = self.positions.remove(idx);
                let mark = mark_prices.get(&pos.symbol).copied().unwrap_or(0.0);
                let margin_returned = pos.size * pos.entry_price / pos.leverage;
                let pnl = compute_realized_pnl(pos.side, pos.entry_price, mark, pos.size);
                self.collateral += (margin_returned + pnl).max(0.0);

                let fill = FuturesPaperFill {
                    id: format!("liq-{}", self.next_id()),
                    order_id: "liquidation".to_string(),
                    symbol: pos.symbol.clone(),
                    side: pos.side.opposite(),
                    size: pos.size,
                    price: mark,
                    fee: 0.0,
                    realized_pnl: Some(pnl),
                    fill_type: "liquidation".to_string(),
                    client_order_id: None,
                    filled_at: now_rfc3339(),
                };

                self.fills.push(fill.clone());
                liquidations.push(fill);

                let event = FuturesPaperHistoryEvent {
                    id: format!("evt-{}", self.next_id()),
                    event_type: "liquidation".to_string(),
                    symbol: Some(pos.symbol.clone()),
                    amount: pnl,
                    details: format!(
                        "Liquidated {} {}: {} @ {mark:.4}, PnL = {pnl:.4}",
                        pos.side, pos.symbol, pos.size
                    ),
                    timestamp: now_rfc3339(),
                };
                self.history.push(event);
            }
        }
    }

    fn reconcile_funding(
        &mut self,
        funding_rates: &HashMap<String, f64>,
        mark_prices: &HashMap<String, f64>,
        events: &mut Vec<FuturesPaperHistoryEvent>,
    ) {
        let now = Utc::now();

        // Collect funding charges per position to avoid borrow conflicts
        struct FundingEntry {
            symbol: String,
            side: Side,
            charge: f64,
        }
        let mut charges: Vec<FundingEntry> = Vec::new();

        for pos in &mut self.positions {
            let rate = match funding_rates.get(&pos.symbol) {
                Some(r) => *r,
                None => continue,
            };
            let mark = match mark_prices.get(&pos.symbol) {
                Some(p) if p.is_finite() && *p > 0.0 => *p,
                _ => continue,
            };

            let should_accrue = match &pos.last_funding_time {
                None => true,
                Some(ts) => {
                    if let Ok(last) = ts.parse::<DateTime<Utc>>() {
                        (now - last).num_hours() >= FUNDING_INTERVAL_HOURS
                    } else {
                        true
                    }
                }
            };

            if !should_accrue {
                continue;
            }

            let funding_payment = pos.size * mark * rate;
            let funding_charge = match pos.side {
                Side::Long => -funding_payment,
                Side::Short => funding_payment,
            };

            pos.unrealized_funding += funding_charge;
            pos.last_funding_time = Some(now.to_rfc3339());

            charges.push(FundingEntry {
                symbol: pos.symbol.clone(),
                side: pos.side,
                charge: funding_charge,
            });
        }

        // Now apply to collateral and emit events (no positions borrow active)
        for entry in charges {
            self.collateral += entry.charge;
            let id = self.next_id();
            let details = format!(
                "Funding {} {}: payment={:.4}",
                entry.side, entry.symbol, entry.charge
            );
            let event = FuturesPaperHistoryEvent {
                id: format!("evt-{id}"),
                event_type: "funding".to_string(),
                symbol: Some(entry.symbol),
                amount: entry.charge,
                details,
                timestamp: now.to_rfc3339(),
            };
            events.push(event.clone());
            self.history.push(event);
        }
    }
}

// ---------------------------------------------------------------------------
// Pure computation functions
// ---------------------------------------------------------------------------

pub fn compute_unrealized_pnl(pos: &FuturesPaperPosition, mark_price: f64) -> f64 {
    compute_realized_pnl(pos.side, pos.entry_price, mark_price, pos.size)
}

pub fn compute_liquidation_price(
    pos: &FuturesPaperPosition,
    maintenance_margin_rate: f64,
) -> f64 {
    if pos.entry_price <= 0.0 || pos.leverage <= 0.0 {
        return f64::INFINITY;
    }
    // Liq price: entry ± entry * (1/leverage - maint_rate) depending on side
    match pos.side {
        Side::Long => pos.entry_price * (1.0 - 1.0 / pos.leverage + maintenance_margin_rate),
        Side::Short => pos.entry_price * (1.0 + 1.0 / pos.leverage - maintenance_margin_rate),
    }
}

fn compute_realized_pnl(side: Side, entry: f64, exit: f64, size: f64) -> f64 {
    match side {
        Side::Long => (exit - entry) * size,
        Side::Short => (entry - exit) * size,
    }
}

fn compute_executable_depth(
    levels: &[(f64, f64)],
    side: Side,
    limit_price: f64,
    max_size: f64,
) -> f64 {
    let mut remaining = max_size;
    let mut executed = 0.0;

    for (price, qty) in levels {
        let price_ok = match side {
            Side::Long => *price <= limit_price,
            Side::Short => *price >= limit_price,
        };
        if !price_ok {
            break;
        }
        let fill = remaining.min(*qty);
        executed += fill;
        remaining -= fill;
        if remaining <= 1e-12 {
            break;
        }
    }

    executed
}

fn resolve_trigger_price(
    signal: TriggerSignal,
    symbol: &str,
    mark_prices: &HashMap<String, f64>,
    last_prices: &HashMap<String, f64>,
    index_prices: &HashMap<String, f64>,
) -> Option<f64> {
    match signal {
        TriggerSignal::Mark => mark_prices.get(symbol).copied(),
        TriggerSignal::Last => last_prices.get(symbol).copied(),
        TriggerSignal::Index => index_prices.get(symbol).copied(),
    }
}

fn check_trailing_stop_trigger(order: &FuturesPaperOrder, mark: f64) -> bool {
    let anchor = match order.trailing_anchor {
        Some(a) if a.is_finite() && a > 0.0 => a,
        _ => return false,
    };
    let max_dev = match order.trailing_stop_max_deviation {
        Some(d) if d.is_finite() && d > 0.0 => d,
        _ => return false,
    };
    let unit = order
        .trailing_stop_deviation_unit
        .as_deref()
        .unwrap_or("percent");

    let threshold = match unit {
        "quote_currency" => max_dev,
        _ => anchor * max_dev / 100.0,
    };

    match order.side {
        Side::Long => mark <= anchor - threshold,
        Side::Short => mark >= anchor + threshold,
    }
}

fn validate_finite_positive(val: f64, name: &str) -> BybitResult<()> {
    if !val.is_finite() || val <= 0.0 {
        return Err(BybitError::Paper(format!(
            "{name} must be a finite positive number."
        )));
    }
    Ok(())
}

fn now_rfc3339() -> String {
    Utc::now().to_rfc3339()
}

// ---------------------------------------------------------------------------
// Bybit market data fetching
// ---------------------------------------------------------------------------

pub async fn fetch_market_snapshot(
    client: &BybitClient,
    category: &str,
    symbol: &str,
) -> BybitResult<MarketSnapshot> {
    let symbol_upper = symbol.to_uppercase();

    // Fetch ticker
    let ticker_resp = client
        .public_get(
            "/v5/market/tickers",
            &[("category", category), ("symbol", symbol_upper.as_str())],
        )
        .await?;

    let ticker_list = ticker_resp["list"]
        .as_array()
        .and_then(|a| a.first())
        .cloned()
        .ok_or_else(|| {
            BybitError::Paper(format!("No ticker data for symbol {symbol_upper}"))
        })?;

    let parse_f = |v: &Value| -> f64 {
        v.as_str()
            .and_then(|s| s.parse::<f64>().ok())
            .unwrap_or(0.0)
    };

    let bid = parse_f(&ticker_list["bid1Price"]);
    let ask = parse_f(&ticker_list["ask1Price"]);
    let last = parse_f(&ticker_list["lastPrice"]);
    let mark = parse_f(&ticker_list["markPrice"]);
    let index = parse_f(&ticker_list["indexPrice"]);

    // Use last as fallback for missing prices
    let bid = if bid > 0.0 { bid } else { last * 0.9995 };
    let ask = if ask > 0.0 { ask } else { last * 1.0005 };
    let mark = if mark > 0.0 { mark } else { last };
    let index = if index > 0.0 { index } else { last };

    // Fetch orderbook
    let ob_resp = client
        .public_get(
            "/v5/market/orderbook",
            &[
                ("category", category),
                ("symbol", symbol_upper.as_str()),
                ("limit", "25"),
            ],
        )
        .await?;

    let parse_levels = |arr: &Value| -> Vec<(f64, f64)> {
        arr.as_array()
            .map(|levels| {
                levels
                    .iter()
                    .filter_map(|entry| {
                        let e = entry.as_array()?;
                        let p: f64 = e.first()?.as_str()?.parse().ok()?;
                        let q: f64 = e.get(1)?.as_str()?.parse().ok()?;
                        Some((p, q))
                    })
                    .collect()
            })
            .unwrap_or_default()
    };

    let mut bid_levels = parse_levels(&ob_resp["b"]);
    let mut ask_levels = parse_levels(&ob_resp["a"]);

    // Sort: bids descending, asks ascending
    bid_levels.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
    ask_levels.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));

    Ok(MarketSnapshot {
        bid,
        ask,
        last,
        mark,
        index,
        bid_levels,
        ask_levels,
    })
}

pub async fn fetch_funding_rate(
    client: &BybitClient,
    category: &str,
    symbol: &str,
) -> BybitResult<f64> {
    let symbol_upper = symbol.to_uppercase();
    let resp = client
        .public_get(
            "/v5/market/tickers",
            &[("category", category), ("symbol", symbol_upper.as_str())],
        )
        .await?;

    let rate = resp["list"]
        .as_array()
        .and_then(|a| a.first())
        .and_then(|t| t["fundingRate"].as_str())
        .and_then(|s| s.parse::<f64>().ok())
        .unwrap_or(0.0);

    Ok(rate)
}

pub async fn fetch_all_market_data(
    client: &BybitClient,
    category: &str,
    symbols: &[String],
) -> BybitResult<(
    HashMap<String, f64>,
    HashMap<String, f64>,
    HashMap<String, f64>,
    HashMap<String, f64>,
)> {
    let mut mark_prices = HashMap::new();
    let mut last_prices = HashMap::new();
    let mut index_prices = HashMap::new();
    let mut funding_rates = HashMap::new();

    for symbol in symbols {
        let resp = client
            .public_get(
                "/v5/market/tickers",
                &[("category", category), ("symbol", symbol.as_str())],
            )
            .await?;

        if let Some(ticker) = resp["list"].as_array().and_then(|a| a.first()) {
            let parse = |v: &Value| -> f64 {
                v.as_str()
                    .and_then(|s| s.parse::<f64>().ok())
                    .unwrap_or(0.0)
            };
            mark_prices.insert(symbol.clone(), parse(&ticker["markPrice"]));
            last_prices.insert(symbol.clone(), parse(&ticker["lastPrice"]));
            index_prices.insert(symbol.clone(), parse(&ticker["indexPrice"]));
            funding_rates.insert(symbol.clone(), parse(&ticker["fundingRate"]));
        }
    }

    Ok((mark_prices, last_prices, index_prices, funding_rates))
}

// ---------------------------------------------------------------------------
// JSON serialization helpers (for command output)
// ---------------------------------------------------------------------------

pub fn order_to_json(order: &FuturesPaperOrder) -> Value {
    json!({
        "order_id": order.id,
        "symbol": order.symbol,
        "side": order.side,
        "size": order.size,
        "filled_size": order.filled_size,
        "order_type": order.order_type.to_string(),
        "price": order.price,
        "stop_price": order.stop_price,
        "trigger_signal": order.trigger_signal,
        "client_order_id": order.client_order_id,
        "reduce_only": order.reduce_only,
        "leverage": order.leverage,
        "reserved_margin": order.reserved_margin,
        "status": format!("{:?}", order.status).to_lowercase(),
        "created_at": order.created_at,
        "updated_at": order.updated_at,
    })
}

pub fn position_to_json(pos: &FuturesPaperPosition, mark_price: Option<f64>) -> Value {
    let mark = mark_price.unwrap_or(pos.entry_price);
    let upnl = compute_unrealized_pnl(pos, mark);
    let liq_price = compute_liquidation_price(pos, DEFAULT_MAINTENANCE_MARGIN_RATE);
    let margin = pos.size * pos.entry_price / pos.leverage;
    json!({
        "symbol": pos.symbol,
        "side": pos.side,
        "size": pos.size,
        "entry_price": pos.entry_price,
        "mark_price": mark,
        "leverage": pos.leverage,
        "unrealized_pnl": upnl,
        "unrealized_funding": pos.unrealized_funding,
        "liquidation_price": if liq_price.is_finite() { json!(liq_price) } else { json!(null) },
        "margin": margin,
        "created_at": pos.created_at,
        "updated_at": pos.updated_at,
    })
}

pub fn fill_to_json(fill: &FuturesPaperFill) -> Value {
    json!({
        "id": fill.id,
        "order_id": fill.order_id,
        "symbol": fill.symbol,
        "side": fill.side,
        "size": fill.size,
        "price": fill.price,
        "fee": fill.fee,
        "realized_pnl": fill.realized_pnl,
        "fill_type": fill.fill_type,
        "client_order_id": fill.client_order_id,
        "filled_at": fill.filled_at,
    })
}
