use std::collections::HashMap;
use std::fs;
use std::io::Write;
use std::path::PathBuf;

use chrono::Utc;
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::json;

use crate::client::BybitClient;
use crate::config;
use crate::errors::{BybitError, BybitResult};

const DUST_THRESHOLD: f64 = 1e-12;
const LEGACY_RESERVED_KEY: &str = "__legacy_reserved__";
const DEFAULT_TAKER_FEE_BPS: u32 = 6;
const DEFAULT_MAKER_FEE_BPS: u32 = 1;
const DEFAULT_SLIPPAGE_BPS: u32 = 5;
const KNOWN_QUOTES: &[&str] = &["USDT", "USDC", "USD", "EUR", "GBP", "BTC", "ETH"];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum OrderSide {
    Buy,
    Sell,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum OrderType {
    Market,
    Limit,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaperBalance {
    #[serde(default)]
    pub coins: HashMap<String, f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaperPosition {
    #[serde(default = "default_category")]
    pub category: String,
    pub symbol: String,
    #[serde(default)]
    pub base_asset: String,
    pub qty: f64,
    pub avg_entry_price: f64,
    #[serde(default)]
    pub mark_price: f64,
    #[serde(default)]
    pub market_value: f64,
    #[serde(default)]
    pub unrealized_pnl: f64,
    #[serde(default = "now_rfc3339")]
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaperTrade {
    pub id: u64,
    #[serde(default)]
    pub order_id: Option<u64>,
    #[serde(default = "default_category")]
    pub category: String,
    pub symbol: String,
    #[serde(default)]
    pub base_asset: String,
    #[serde(default)]
    pub settle_coin: String,
    pub side: OrderSide,
    pub qty: f64,
    pub price: f64,
    #[serde(default, alias = "fee")]
    pub fee_paid: f64,
    #[serde(default, alias = "cost")]
    pub gross_value: f64,
    #[serde(default)]
    pub net_value: f64,
    #[serde(default)]
    pub realized_pnl: f64,
    #[serde(default = "now_rfc3339")]
    pub filled_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaperOrder {
    pub id: u64,
    #[serde(default = "default_category")]
    pub category: String,
    pub symbol: String,
    #[serde(default)]
    pub base_asset: String,
    #[serde(default)]
    pub settle_coin: String,
    pub side: OrderSide,
    #[serde(default = "default_limit_order_type")]
    pub order_type: OrderType,
    pub qty: f64,
    pub price: f64,
    #[serde(default)]
    pub reserved_asset: String,
    #[serde(default)]
    pub reserved_amount: f64,
    #[serde(default = "now_rfc3339")]
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaperJournal {
    pub balance: PaperBalance,
    #[serde(default)]
    pub positions: Vec<PaperPosition>,
    #[serde(default)]
    pub trades: Vec<PaperTrade>,
    #[serde(default)]
    pub pending_orders: Vec<PaperOrder>,
    #[serde(default)]
    pub cancelled_orders: Vec<PaperOrder>,
    #[serde(default = "default_settle_coin")]
    pub settle_coin: String,
    #[serde(default = "default_taker_fee_bps")]
    pub taker_fee_bps: u32,
    #[serde(default = "default_maker_fee_bps")]
    pub maker_fee_bps: u32,
    #[serde(default = "default_slippage_bps")]
    pub slippage_bps: u32,
    #[serde(default, deserialize_with = "deserialize_reserved")]
    pub reserved: HashMap<String, f64>,
    #[serde(default)]
    pub total_fees_paid: f64,
    #[serde(default)]
    pub starting_balance: f64,
    #[serde(default = "now_rfc3339")]
    pub created_at: String,
    #[serde(default = "now_rfc3339")]
    pub updated_at: String,
    #[serde(default = "default_next_order_id")]
    pub next_order_id: u64,
}

#[derive(Debug, Clone, Default)]
pub struct ResetOptions {
    pub balance: Option<f64>,
    pub settle_coin: Option<String>,
    pub taker_fee_bps: Option<u32>,
    pub maker_fee_bps: Option<u32>,
    pub slippage_bps: Option<u32>,
}

#[derive(Debug, Clone, Copy)]
struct TickerSnapshot {
    last: f64,
    bid: f64,
    ask: f64,
}

fn default_category() -> String {
    "linear".to_string()
}

fn default_settle_coin() -> String {
    "USDT".to_string()
}

fn default_taker_fee_bps() -> u32 {
    DEFAULT_TAKER_FEE_BPS
}

fn default_maker_fee_bps() -> u32 {
    DEFAULT_MAKER_FEE_BPS
}

fn default_slippage_bps() -> u32 {
    DEFAULT_SLIPPAGE_BPS
}

fn default_next_order_id() -> u64 {
    1
}

fn default_limit_order_type() -> OrderType {
    OrderType::Limit
}

fn now_rfc3339() -> String {
    Utc::now().to_rfc3339()
}

fn deserialize_reserved<'de, D>(deserializer: D) -> Result<HashMap<String, f64>, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum ReservedValue {
        Number(f64),
        Map(HashMap<String, f64>),
    }

    match Option::<ReservedValue>::deserialize(deserializer)? {
        Some(ReservedValue::Map(map)) => Ok(map),
        Some(ReservedValue::Number(value)) => {
            let mut map = HashMap::new();
            if value.abs() > DUST_THRESHOLD {
                map.insert(LEGACY_RESERVED_KEY.to_string(), value);
            }
            Ok(map)
        }
        None => Ok(HashMap::new()),
    }
}

fn maker_fee_rate(journal: &PaperJournal) -> f64 {
    journal.maker_fee_bps as f64 / 10_000.0
}

fn taker_fee_rate(journal: &PaperJournal) -> f64 {
    journal.taker_fee_bps as f64 / 10_000.0
}

fn slippage_rate(journal: &PaperJournal) -> f64 {
    journal.slippage_bps as f64 / 10_000.0
}

fn paper_mode(data: serde_json::Value) -> serde_json::Value {
    let mut value = data;
    if let Some(obj) = value.as_object_mut() {
        obj.insert("mode".to_string(), json!("paper"));
    }
    value
}

fn normalize_symbol(symbol: &str) -> String {
    symbol.trim().to_uppercase()
}

fn infer_base_asset(symbol: &str, settle_coin: &str) -> String {
    let normalized = normalize_symbol(symbol);
    let settle = settle_coin.to_uppercase();

    if normalized.ends_with(&settle) && normalized.len() > settle.len() {
        return normalized[..normalized.len() - settle.len()].to_string();
    }

    for quote in KNOWN_QUOTES {
        if normalized.ends_with(quote) && normalized.len() > quote.len() {
            return normalized[..normalized.len() - quote.len()].to_string();
        }
    }

    normalized
}

fn parse_base_asset(symbol: &str, settle_coin: &str) -> BybitResult<String> {
    let normalized = normalize_symbol(symbol);
    let base_asset = infer_base_asset(&normalized, settle_coin);
    if base_asset == normalized {
        return Err(BybitError::Paper(format!(
            "Could not infer base asset from symbol {normalized}. Use a symbol ending in {settle_coin}."
        )));
    }
    Ok(base_asset)
}

fn journal_path() -> BybitResult<PathBuf> {
    config::paper_journal_path()
}

impl PaperJournal {
    fn new(
        starting_balance: f64,
        settle_coin: String,
        taker_fee_bps: u32,
        maker_fee_bps: u32,
        slippage_bps: u32,
    ) -> Self {
        let settle_coin = settle_coin.to_uppercase();
        let mut coins = HashMap::new();
        coins.insert(settle_coin.clone(), starting_balance);

        Self {
            balance: PaperBalance { coins },
            positions: Vec::new(),
            trades: Vec::new(),
            pending_orders: Vec::new(),
            cancelled_orders: Vec::new(),
            settle_coin,
            taker_fee_bps,
            maker_fee_bps,
            slippage_bps,
            reserved: HashMap::new(),
            total_fees_paid: 0.0,
            starting_balance,
            created_at: now_rfc3339(),
            updated_at: now_rfc3339(),
            next_order_id: 1,
        }
    }

    fn next_id(&mut self) -> u64 {
        let id = self.next_order_id.max(1);
        self.next_order_id = id + 1;
        id
    }
}

fn repair_journal(mut journal: PaperJournal) -> PaperJournal {
    journal.settle_coin = journal.settle_coin.to_uppercase();
    if journal.settle_coin.is_empty() {
        journal.settle_coin = default_settle_coin();
    }

    let mut normalized_coins = HashMap::new();
    for (asset, amount) in journal.balance.coins.drain() {
        normalized_coins.insert(asset.to_uppercase(), amount);
    }
    journal.balance.coins = normalized_coins;
    journal
        .balance
        .coins
        .entry(journal.settle_coin.clone())
        .or_insert(journal.starting_balance.max(0.0));

    if let Some(legacy_reserved) = journal.reserved.remove(LEGACY_RESERVED_KEY) {
        *journal
            .reserved
            .entry(journal.settle_coin.clone())
            .or_insert(0.0) += legacy_reserved;
    }

    if journal.starting_balance <= 0.0 {
        journal.starting_balance = journal
            .balance
            .coins
            .get(&journal.settle_coin)
            .copied()
            .unwrap_or(10_000.0);
    }

    for position in &mut journal.positions {
        position.category = position.category.to_lowercase();
        if position.category.is_empty() {
            position.category = default_category();
        }
        position.symbol = normalize_symbol(&position.symbol);
        if position.base_asset.is_empty() {
            position.base_asset = infer_base_asset(&position.symbol, &journal.settle_coin);
        }
        position.base_asset = position.base_asset.to_uppercase();
    }

    for trade in &mut journal.trades {
        trade.category = trade.category.to_lowercase();
        if trade.category.is_empty() {
            trade.category = default_category();
        }
        trade.symbol = normalize_symbol(&trade.symbol);
        if trade.base_asset.is_empty() {
            trade.base_asset = infer_base_asset(&trade.symbol, &journal.settle_coin);
        }
        trade.base_asset = trade.base_asset.to_uppercase();
        if trade.settle_coin.is_empty() {
            trade.settle_coin = journal.settle_coin.clone();
        } else {
            trade.settle_coin = trade.settle_coin.to_uppercase();
        }
    }

    let maker_fee_rate = maker_fee_rate(&journal);
    for order in journal
        .pending_orders
        .iter_mut()
        .chain(journal.cancelled_orders.iter_mut())
    {
        order.category = order.category.to_lowercase();
        if order.category.is_empty() {
            order.category = default_category();
        }
        order.symbol = normalize_symbol(&order.symbol);
        if order.base_asset.is_empty() {
            order.base_asset = infer_base_asset(&order.symbol, &journal.settle_coin);
        }
        order.base_asset = order.base_asset.to_uppercase();
        if order.settle_coin.is_empty() {
            order.settle_coin = journal.settle_coin.clone();
        } else {
            order.settle_coin = order.settle_coin.to_uppercase();
        }
        if order.reserved_asset.is_empty() {
            order.reserved_asset = match order.side {
                OrderSide::Buy => journal.settle_coin.clone(),
                OrderSide::Sell => order.base_asset.clone(),
            };
        }
        if order.reserved_amount <= 0.0 {
            order.reserved_amount = match order.side {
                OrderSide::Buy => order.qty * order.price * (1.0 + maker_fee_rate),
                OrderSide::Sell => order.qty,
            };
        }
    }

    let mut recomputed_reserved = HashMap::new();
    for order in &journal.pending_orders {
        *recomputed_reserved
            .entry(order.reserved_asset.clone())
            .or_insert(0.0) += order.reserved_amount;
    }
    journal.reserved = recomputed_reserved;

    let mut position_balances: HashMap<String, f64> = HashMap::new();
    for position in &journal.positions {
        *position_balances
            .entry(position.base_asset.clone())
            .or_insert(0.0) += position.qty.max(0.0);
    }
    let mut reserved_sell_balances: HashMap<String, f64> = HashMap::new();
    for order in &journal.pending_orders {
        if order.side == OrderSide::Sell {
            *reserved_sell_balances
                .entry(order.base_asset.clone())
                .or_insert(0.0) += order.qty.max(0.0);
        }
    }
    for asset in position_balances
        .keys()
        .chain(reserved_sell_balances.keys())
        .cloned()
        .collect::<std::collections::BTreeSet<_>>()
    {
        let required = position_balances
            .get(&asset)
            .copied()
            .unwrap_or_default()
            .max(
                reserved_sell_balances
                    .get(&asset)
                    .copied()
                    .unwrap_or_default(),
            );
        let entry = journal.balance.coins.entry(asset).or_insert(0.0);
        if *entry < required {
            *entry = required;
        }
    }

    if journal.next_order_id == 0 {
        let max_id = journal
            .pending_orders
            .iter()
            .chain(journal.cancelled_orders.iter())
            .map(|order| order.id)
            .chain(journal.trades.iter().map(|trade| trade.id))
            .max()
            .unwrap_or(0);
        journal.next_order_id = max_id + 1;
    }

    journal
}

fn load_journal() -> BybitResult<PaperJournal> {
    let path = journal_path()?;
    if !path.exists() {
        return Err(BybitError::Paper(
            "Paper account not initialized. Run `bybit paper init` first.".to_string(),
        ));
    }

    let contents = fs::read_to_string(&path)?;
    let journal: PaperJournal = serde_json::from_str(&contents)?;
    Ok(repair_journal(journal))
}

fn save_journal(journal: &PaperJournal) -> BybitResult<()> {
    let path = journal_path()?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let data = serde_json::to_string_pretty(journal)?;
    let tmp = path.with_extension("json.tmp");
    {
        let mut file = fs::File::create(&tmp)?;
        file.write_all(data.as_bytes())?;
        file.sync_all()?;
    }
    fs::rename(tmp, path)?;
    Ok(())
}

fn available_balance(journal: &PaperJournal, asset: &str) -> f64 {
    let total = journal
        .balance
        .coins
        .get(asset)
        .copied()
        .unwrap_or_default();
    let reserved = journal.reserved.get(asset).copied().unwrap_or_default();
    (total - reserved).max(0.0)
}

fn reserve_asset(journal: &mut PaperJournal, asset: &str, amount: f64) {
    *journal.reserved.entry(asset.to_string()).or_insert(0.0) += amount;
}

fn release_reservation(journal: &mut PaperJournal, asset: &str, amount: f64) {
    if let Some(entry) = journal.reserved.get_mut(asset) {
        *entry = (*entry - amount).max(0.0);
        if entry.abs() <= DUST_THRESHOLD {
            journal.reserved.remove(asset);
        }
    }
}

fn update_position_after_buy(
    journal: &mut PaperJournal,
    category: &str,
    symbol: &str,
    base_asset: &str,
    qty: f64,
    price: f64,
) {
    if let Some(position) = journal
        .positions
        .iter_mut()
        .find(|position| position.category == category && position.symbol == symbol)
    {
        let total_cost = position.avg_entry_price * position.qty + price * qty;
        position.qty += qty;
        position.avg_entry_price = if position.qty > DUST_THRESHOLD {
            total_cost / position.qty
        } else {
            price
        };
        position.updated_at = now_rfc3339();
        return;
    }

    journal.positions.push(PaperPosition {
        category: category.to_string(),
        symbol: symbol.to_string(),
        base_asset: base_asset.to_string(),
        qty,
        avg_entry_price: price,
        mark_price: price,
        market_value: qty * price,
        unrealized_pnl: 0.0,
        updated_at: now_rfc3339(),
    });
}

fn update_position_after_sell(
    journal: &mut PaperJournal,
    category: &str,
    symbol: &str,
    qty: f64,
    price: f64,
) -> BybitResult<f64> {
    if let Some(index) = journal
        .positions
        .iter()
        .position(|position| position.category == category && position.symbol == symbol)
    {
        let average_entry = journal.positions[index].avg_entry_price;
        let realized_pnl = (price - average_entry) * qty;
        journal.positions[index].qty -= qty;
        journal.positions[index].updated_at = now_rfc3339();

        if journal.positions[index].qty <= DUST_THRESHOLD {
            journal.positions.remove(index);
        }

        return Ok(realized_pnl);
    }

    Ok(0.0)
}

fn validate_finite_positive(name: &str, value: f64) -> BybitResult<()> {
    if !value.is_finite() || value <= 0.0 {
        return Err(BybitError::Paper(format!(
            "{name} must be a finite positive number."
        )));
    }
    Ok(())
}

fn validate_bps(name: &str, value: u32) -> BybitResult<()> {
    if value > 10_000 {
        return Err(BybitError::Paper(format!(
            "{name} must be between 0 and 10000 basis points."
        )));
    }
    Ok(())
}

async fn fetch_ticker_snapshot(
    client: &BybitClient,
    category: &str,
    symbol: &str,
) -> BybitResult<TickerSnapshot> {
    let result = client
        .public_get(
            "/v5/market/tickers",
            &[("category", category), ("symbol", symbol)],
        )
        .await?;

    let list = result
        .get("list")
        .and_then(|value| value.as_array())
        .ok_or_else(|| BybitError::Parse("Unexpected ticker payload format.".to_string()))?;

    let ticker = list
        .first()
        .ok_or_else(|| BybitError::Parse(format!("No ticker data returned for {symbol}.")))?;

    fn parse_price(ticker: &serde_json::Value, key: &str) -> BybitResult<f64> {
        ticker
            .get(key)
            .and_then(|value| value.as_str())
            .ok_or_else(|| BybitError::Parse(format!("Missing ticker field `{key}`.")))?
            .parse::<f64>()
            .map_err(|error| BybitError::Parse(format!("Invalid ticker field `{key}`: {error}")))
    }

    let last = parse_price(ticker, "lastPrice")?;
    let bid = parse_price(ticker, "bid1Price").unwrap_or(last);
    let ask = parse_price(ticker, "ask1Price").unwrap_or(last);

    Ok(TickerSnapshot { last, bid, ask })
}

async fn refresh_positions(client: &BybitClient, journal: &mut PaperJournal) -> bool {
    let mut valuation_complete = true;

    for position in &mut journal.positions {
        match fetch_ticker_snapshot(client, &position.category, &position.symbol).await {
            Ok(snapshot) => {
                let mark = if snapshot.bid > 0.0 {
                    snapshot.bid
                } else {
                    snapshot.last
                };
                position.mark_price = mark;
                position.market_value = mark * position.qty;
                position.unrealized_pnl = (mark - position.avg_entry_price) * position.qty;
                position.updated_at = now_rfc3339();
            }
            Err(_) => valuation_complete = false,
        }
    }

    valuation_complete
}

#[allow(clippy::too_many_arguments)]
fn apply_fill(
    journal: &mut PaperJournal,
    category: &str,
    symbol: &str,
    qty: f64,
    price: f64,
    side: OrderSide,
    fee_rate: f64,
    order_id: Option<u64>,
) -> BybitResult<PaperTrade> {
    validate_finite_positive("Quantity", qty)?;
    validate_finite_positive("Price", price)?;

    let settle_coin = journal.settle_coin.clone();
    let base_asset = parse_base_asset(symbol, &settle_coin)?;
    let gross_value = qty * price;
    let fee_paid = gross_value * fee_rate;
    let net_value;
    let realized_pnl;

    match side {
        OrderSide::Buy => {
            let total_cost = gross_value + fee_paid;
            if available_balance(journal, &settle_coin) + DUST_THRESHOLD < total_cost {
                return Err(BybitError::Paper(format!(
                    "Insufficient {settle_coin} balance. Available: {:.8}, required: {:.8}.",
                    available_balance(journal, &settle_coin),
                    total_cost
                )));
            }

            *journal
                .balance
                .coins
                .entry(settle_coin.clone())
                .or_insert(0.0) -= total_cost;
            *journal
                .balance
                .coins
                .entry(base_asset.clone())
                .or_insert(0.0) += qty;

            update_position_after_buy(journal, category, symbol, &base_asset, qty, price);
            net_value = -total_cost;
            realized_pnl = 0.0;
        }
        OrderSide::Sell => {
            if available_balance(journal, &base_asset) + DUST_THRESHOLD < qty {
                return Err(BybitError::Paper(format!(
                    "Insufficient {base_asset} balance. Available: {:.8}, required: {:.8}.",
                    available_balance(journal, &base_asset),
                    qty
                )));
            }

            *journal
                .balance
                .coins
                .entry(base_asset.clone())
                .or_insert(0.0) -= qty;
            *journal
                .balance
                .coins
                .entry(settle_coin.clone())
                .or_insert(0.0) += gross_value - fee_paid;

            realized_pnl = update_position_after_sell(journal, category, symbol, qty, price)?;
            net_value = gross_value - fee_paid;
        }
    }

    journal.total_fees_paid += fee_paid;
    journal.updated_at = now_rfc3339();

    let trade_id = journal.next_id();
    let trade = PaperTrade {
        id: trade_id,
        order_id,
        category: category.to_string(),
        symbol: symbol.to_string(),
        base_asset,
        settle_coin,
        side,
        qty,
        price,
        fee_paid,
        gross_value,
        net_value,
        realized_pnl: if side == OrderSide::Sell {
            realized_pnl - fee_paid
        } else {
            realized_pnl
        },
        filled_at: now_rfc3339(),
    };
    journal.trades.push(trade.clone());
    Ok(trade)
}

async fn reconcile_pending_orders(
    client: &BybitClient,
    journal: &mut PaperJournal,
) -> Vec<PaperTrade> {
    let mut fills = Vec::new();
    let mut index = 0;

    while index < journal.pending_orders.len() {
        let order = journal.pending_orders[index].clone();
        let snapshot = match fetch_ticker_snapshot(client, &order.category, &order.symbol).await {
            Ok(snapshot) => snapshot,
            Err(_) => {
                index += 1;
                continue;
            }
        };

        let should_fill = match order.side {
            OrderSide::Buy => snapshot.ask <= order.price,
            OrderSide::Sell => snapshot.bid >= order.price,
        };

        if should_fill {
            let order = journal.pending_orders.remove(index);
            release_reservation(journal, &order.reserved_asset, order.reserved_amount);
            let fill = apply_fill(
                journal,
                &order.category,
                &order.symbol,
                order.qty,
                order.price,
                order.side,
                maker_fee_rate(journal),
                Some(order.id),
            );
            if let Ok(trade) = fill {
                fills.push(trade);
            }
        } else {
            index += 1;
        }
    }

    fills
}

async fn reconcile_best_effort(client: &BybitClient, journal: &mut PaperJournal) {
    let fills = reconcile_pending_orders(client, journal).await;
    if !fills.is_empty() {
        let _ = save_journal(journal);
    }
}

fn balances_json(journal: &PaperJournal) -> serde_json::Value {
    let mut balances = serde_json::Map::new();
    let mut assets: Vec<_> = journal.balance.coins.keys().cloned().collect();
    assets.sort();
    assets.dedup();

    for asset in assets {
        let total = journal
            .balance
            .coins
            .get(&asset)
            .copied()
            .unwrap_or_default();
        if total.abs() <= DUST_THRESHOLD
            && available_balance(journal, &asset).abs() <= DUST_THRESHOLD
        {
            continue;
        }

        let reserved = journal.reserved.get(&asset).copied().unwrap_or_default();
        balances.insert(
            asset.clone(),
            json!({
                "total": total,
                "reserved": reserved,
                "available": available_balance(journal, &asset),
            }),
        );
    }

    serde_json::Value::Object(balances)
}

fn positions_json(positions: &[PaperPosition]) -> serde_json::Value {
    json!(positions
        .iter()
        .map(|position| json!({
            "category": position.category,
            "symbol": position.symbol,
            "base_asset": position.base_asset,
            "qty": position.qty,
            "avg_entry_price": position.avg_entry_price,
            "mark_price": position.mark_price,
            "market_value": position.market_value,
            "unrealized_pnl": position.unrealized_pnl,
            "updated_at": position.updated_at,
        }))
        .collect::<Vec<_>>())
}

fn orders_json(orders: &[PaperOrder]) -> serde_json::Value {
    json!(orders
        .iter()
        .map(|order| json!({
            "id": order.id,
            "category": order.category,
            "symbol": order.symbol,
            "base_asset": order.base_asset,
            "settle_coin": order.settle_coin,
            "side": order.side,
            "type": order.order_type,
            "qty": order.qty,
            "price": order.price,
            "reserved_asset": order.reserved_asset,
            "reserved_amount": order.reserved_amount,
            "created_at": order.created_at,
        }))
        .collect::<Vec<_>>())
}

fn trades_json(trades: &[PaperTrade]) -> serde_json::Value {
    json!(trades
        .iter()
        .map(|trade| json!({
            "id": trade.id,
            "order_id": trade.order_id,
            "category": trade.category,
            "symbol": trade.symbol,
            "base_asset": trade.base_asset,
            "settle_coin": trade.settle_coin,
            "side": trade.side,
            "qty": trade.qty,
            "price": trade.price,
            "fee_paid": trade.fee_paid,
            "gross_value": trade.gross_value,
            "net_value": trade.net_value,
            "realized_pnl": trade.realized_pnl,
            "filled_at": trade.filled_at,
        }))
        .collect::<Vec<_>>())
}

pub fn init(
    starting_balance: f64,
    settle_coin: String,
    taker_fee_bps: u32,
    maker_fee_bps: u32,
    slippage_bps: u32,
    force: bool,
) -> BybitResult<PaperJournal> {
    validate_finite_positive("Starting balance", starting_balance)?;
    validate_bps("Taker fee", taker_fee_bps)?;
    validate_bps("Maker fee", maker_fee_bps)?;
    validate_bps("Slippage", slippage_bps)?;

    let path = journal_path()?;
    if path.exists() && !force {
        return Err(BybitError::Paper(
            "Paper account already initialized. Use `bybit paper init --force` or `bybit paper reset`."
                .to_string(),
        ));
    }

    let journal = PaperJournal::new(
        starting_balance,
        settle_coin,
        taker_fee_bps,
        maker_fee_bps,
        slippage_bps,
    );
    save_journal(&journal)?;
    Ok(journal)
}

pub async fn buy(
    client: &BybitClient,
    category: &str,
    symbol: &str,
    qty: f64,
    price: Option<f64>,
) -> BybitResult<serde_json::Value> {
    validate_finite_positive("Quantity", qty)?;

    let mut journal = load_journal()?;
    reconcile_best_effort(client, &mut journal).await;

    let category = category.to_lowercase();
    let symbol = normalize_symbol(symbol);
    let base_asset = parse_base_asset(&symbol, &journal.settle_coin)?;

    let result = if let Some(limit_price) = price {
        validate_finite_positive("Price", limit_price)?;
        let reserved_amount = qty * limit_price * (1.0 + maker_fee_rate(&journal));
        let settle_coin = journal.settle_coin.clone();

        if available_balance(&journal, &settle_coin) + DUST_THRESHOLD < reserved_amount {
            return Err(BybitError::Paper(format!(
                "Insufficient {settle_coin} balance. Available: {:.8}, required: {:.8}.",
                available_balance(&journal, &settle_coin),
                reserved_amount
            )));
        }

        let order_id = journal.next_id();
        reserve_asset(&mut journal, &settle_coin, reserved_amount);
        journal.pending_orders.push(PaperOrder {
            id: order_id,
            category,
            symbol,
            base_asset,
            settle_coin,
            side: OrderSide::Buy,
            order_type: OrderType::Limit,
            qty,
            price: limit_price,
            reserved_asset: journal.settle_coin.clone(),
            reserved_amount,
            created_at: now_rfc3339(),
        });
        journal.updated_at = now_rfc3339();
        save_journal(&journal)?;

        paper_mode(json!({
            "status": "open",
            "action": "limit_order_placed",
            "order_id": order_id,
            "side": "buy",
            "category": journal.pending_orders.last().map(|order| order.category.clone()).unwrap_or_default(),
            "symbol": journal.pending_orders.last().map(|order| order.symbol.clone()).unwrap_or_default(),
            "qty": qty,
            "price": limit_price,
            "reserved_asset": journal.pending_orders.last().map(|order| order.reserved_asset.clone()).unwrap_or_default(),
            "reserved_amount": reserved_amount,
        }))
    } else {
        let snapshot = fetch_ticker_snapshot(client, &category, &symbol).await?;
        let fill_price = snapshot.ask * (1.0 + slippage_rate(&journal));
        let fee_rate = taker_fee_rate(&journal);
        let order_id = journal.next_id();
        let trade = apply_fill(
            &mut journal,
            &category,
            &symbol,
            qty,
            fill_price,
            OrderSide::Buy,
            fee_rate,
            Some(order_id),
        )?;
        save_journal(&journal)?;

        paper_mode(json!({
            "status": "filled",
            "action": "market_order_filled",
            "trade_id": trade.id,
            "order_id": trade.order_id,
            "side": "buy",
            "category": trade.category,
            "symbol": trade.symbol,
            "base_asset": trade.base_asset,
            "qty": trade.qty,
            "price": trade.price,
            "fee_paid": trade.fee_paid,
            "gross_value": trade.gross_value,
            "net_value": trade.net_value,
            "balances": balances_json(&journal),
        }))
    };

    Ok(result)
}

pub async fn sell(
    client: &BybitClient,
    category: &str,
    symbol: &str,
    qty: f64,
    price: Option<f64>,
) -> BybitResult<serde_json::Value> {
    validate_finite_positive("Quantity", qty)?;

    let mut journal = load_journal()?;
    reconcile_best_effort(client, &mut journal).await;

    let category = category.to_lowercase();
    let symbol = normalize_symbol(symbol);
    let base_asset = parse_base_asset(&symbol, &journal.settle_coin)?;

    let result = if let Some(limit_price) = price {
        validate_finite_positive("Price", limit_price)?;

        if available_balance(&journal, &base_asset) + DUST_THRESHOLD < qty {
            return Err(BybitError::Paper(format!(
                "Insufficient {base_asset} balance. Available: {:.8}, required: {:.8}.",
                available_balance(&journal, &base_asset),
                qty
            )));
        }

        let order_id = journal.next_id();
        reserve_asset(&mut journal, &base_asset, qty);
        journal.pending_orders.push(PaperOrder {
            id: order_id,
            category,
            symbol,
            base_asset: base_asset.clone(),
            settle_coin: journal.settle_coin.clone(),
            side: OrderSide::Sell,
            order_type: OrderType::Limit,
            qty,
            price: limit_price,
            reserved_asset: base_asset.clone(),
            reserved_amount: qty,
            created_at: now_rfc3339(),
        });
        journal.updated_at = now_rfc3339();
        save_journal(&journal)?;

        paper_mode(json!({
            "status": "open",
            "action": "limit_order_placed",
            "order_id": order_id,
            "side": "sell",
            "category": journal.pending_orders.last().map(|order| order.category.clone()).unwrap_or_default(),
            "symbol": journal.pending_orders.last().map(|order| order.symbol.clone()).unwrap_or_default(),
            "qty": qty,
            "price": limit_price,
            "reserved_asset": base_asset,
            "reserved_amount": qty,
        }))
    } else {
        let snapshot = fetch_ticker_snapshot(client, &category, &symbol).await?;
        let fill_price = snapshot.bid * (1.0 - slippage_rate(&journal));
        let fee_rate = taker_fee_rate(&journal);
        let order_id = journal.next_id();
        let trade = apply_fill(
            &mut journal,
            &category,
            &symbol,
            qty,
            fill_price,
            OrderSide::Sell,
            fee_rate,
            Some(order_id),
        )?;
        save_journal(&journal)?;

        paper_mode(json!({
            "status": "filled",
            "action": "market_order_filled",
            "trade_id": trade.id,
            "order_id": trade.order_id,
            "side": "sell",
            "category": trade.category,
            "symbol": trade.symbol,
            "base_asset": trade.base_asset,
            "qty": trade.qty,
            "price": trade.price,
            "fee_paid": trade.fee_paid,
            "gross_value": trade.gross_value,
            "net_value": trade.net_value,
            "realized_pnl": trade.realized_pnl,
            "balances": balances_json(&journal),
        }))
    };

    Ok(result)
}

pub async fn get_balance(client: &BybitClient) -> BybitResult<serde_json::Value> {
    let mut journal = load_journal()?;
    reconcile_best_effort(client, &mut journal).await;

    Ok(paper_mode(json!({
        "settle_coin": journal.settle_coin,
        "starting_balance": journal.starting_balance,
        "balances": balances_json(&journal),
        "total_fees_paid": journal.total_fees_paid,
    })))
}

pub async fn get_positions(client: &BybitClient) -> BybitResult<serde_json::Value> {
    let mut journal = load_journal()?;
    reconcile_best_effort(client, &mut journal).await;
    let valuation_complete = refresh_positions(client, &mut journal).await;

    Ok(paper_mode(json!({
        "positions": positions_json(&journal.positions),
        "count": journal.positions.len(),
        "valuation_complete": valuation_complete,
    })))
}

pub async fn get_trades(client: &BybitClient) -> BybitResult<serde_json::Value> {
    let mut journal = load_journal()?;
    reconcile_best_effort(client, &mut journal).await;

    Ok(paper_mode(json!({
        "trades": trades_json(&journal.trades),
        "count": journal.trades.len(),
        "total_fees_paid": journal.total_fees_paid,
    })))
}

pub async fn get_cancelled(client: &BybitClient) -> BybitResult<serde_json::Value> {
    let mut journal = load_journal()?;
    reconcile_best_effort(client, &mut journal).await;

    Ok(paper_mode(json!({
        "cancelled": orders_json(&journal.cancelled_orders),
        "count": journal.cancelled_orders.len(),
    })))
}

pub async fn get_orders(client: &BybitClient) -> BybitResult<serde_json::Value> {
    let mut journal = load_journal()?;
    reconcile_best_effort(client, &mut journal).await;

    Ok(paper_mode(json!({
        "open_orders": orders_json(&journal.pending_orders),
        "count": journal.pending_orders.len(),
        "balances": balances_json(&journal),
    })))
}

pub async fn cancel_order(client: &BybitClient, order_id: u64) -> BybitResult<serde_json::Value> {
    let mut journal = load_journal()?;
    reconcile_best_effort(client, &mut journal).await;

    let position = journal
        .pending_orders
        .iter()
        .position(|order| order.id == order_id)
        .ok_or_else(|| BybitError::Paper(format!("Paper order {order_id} not found.")))?;

    let order = journal.pending_orders.remove(position);
    release_reservation(&mut journal, &order.reserved_asset, order.reserved_amount);
    journal.cancelled_orders.push(order.clone());
    journal.updated_at = now_rfc3339();
    save_journal(&journal)?;

    Ok(paper_mode(json!({
        "status": "cancelled",
        "action": "order_cancelled",
        "order": {
            "id": order.id,
            "category": order.category,
            "symbol": order.symbol,
            "side": order.side,
            "qty": order.qty,
            "price": order.price,
            "reserved_asset": order.reserved_asset,
            "reserved_amount": order.reserved_amount,
        },
        "balances": balances_json(&journal),
    })))
}

pub async fn cancel_all_orders(client: &BybitClient) -> BybitResult<serde_json::Value> {
    let mut journal = load_journal()?;
    reconcile_best_effort(client, &mut journal).await;

    let cancelled_orders: Vec<_> = journal.pending_orders.drain(..).collect();
    for order in &cancelled_orders {
        release_reservation(&mut journal, &order.reserved_asset, order.reserved_amount);
    }
    journal.cancelled_orders.extend(cancelled_orders.clone());
    journal.updated_at = now_rfc3339();
    save_journal(&journal)?;

    Ok(paper_mode(json!({
        "status": "cancelled",
        "action": "all_orders_cancelled",
        "cancelled_count": cancelled_orders.len(),
        "cancelled_orders": orders_json(&cancelled_orders),
        "balances": balances_json(&journal),
    })))
}

pub async fn status(client: &BybitClient) -> BybitResult<serde_json::Value> {
    let mut journal = load_journal()?;
    reconcile_best_effort(client, &mut journal).await;
    let positions_complete = refresh_positions(client, &mut journal).await;

    let mut current_value = journal
        .balance
        .coins
        .get(&journal.settle_coin)
        .copied()
        .unwrap_or_default();
    let mut valuation_complete = positions_complete;

    let mut symbol_by_asset = HashMap::new();
    for position in &journal.positions {
        symbol_by_asset.insert(
            position.base_asset.clone(),
            (position.category.clone(), position.symbol.clone()),
        );
    }

    for (asset, total) in &journal.balance.coins {
        if asset == &journal.settle_coin || total.abs() <= DUST_THRESHOLD {
            continue;
        }

        let ticker = if let Some((category, symbol)) = symbol_by_asset.get(asset) {
            fetch_ticker_snapshot(client, category, symbol).await.ok()
        } else {
            let synthetic_symbol = format!("{asset}{}", journal.settle_coin);
            fetch_ticker_snapshot(client, "linear", &synthetic_symbol)
                .await
                .ok()
        };

        if let Some(snapshot) = ticker {
            let mark = if snapshot.bid > 0.0 {
                snapshot.bid
            } else {
                snapshot.last
            };
            current_value += total * mark;
        } else {
            valuation_complete = false;
        }
    }

    let realized_pnl: f64 = journal.trades.iter().map(|trade| trade.realized_pnl).sum();
    let unrealized_pnl: f64 = journal
        .positions
        .iter()
        .map(|position| position.unrealized_pnl)
        .sum();
    let total_pnl = realized_pnl + unrealized_pnl;

    Ok(paper_mode(json!({
        "settle_coin": journal.settle_coin,
        "starting_balance": journal.starting_balance,
        "current_value": current_value,
        "valuation_complete": valuation_complete,
        "realized_pnl": realized_pnl,
        "unrealized_pnl": unrealized_pnl,
        "total_pnl": total_pnl,
        "taker_fee_bps": journal.taker_fee_bps,
        "maker_fee_bps": journal.maker_fee_bps,
        "slippage_bps": journal.slippage_bps,
        "total_fees_paid": journal.total_fees_paid,
        "open_orders": journal.pending_orders.len(),
        "filled_trades": journal.trades.len(),
        "cancelled_orders": journal.cancelled_orders.len(),
        "assets_held": journal
            .balance
            .coins
            .iter()
            .filter(|(asset, amount)| *asset != &journal.settle_coin && amount.abs() > DUST_THRESHOLD)
            .count(),
        "balances": balances_json(&journal),
        "positions": positions_json(&journal.positions),
    })))
}

pub async fn reset(client: &BybitClient, options: ResetOptions) -> BybitResult<PaperJournal> {
    let mut journal = load_journal()?;
    reconcile_best_effort(client, &mut journal).await;

    let starting_balance = options.balance.unwrap_or(journal.starting_balance);
    let settle_coin = options
        .settle_coin
        .unwrap_or_else(|| journal.settle_coin.clone());
    let taker_fee_bps = options.taker_fee_bps.unwrap_or(journal.taker_fee_bps);
    let maker_fee_bps = options.maker_fee_bps.unwrap_or(journal.maker_fee_bps);
    let slippage_bps = options.slippage_bps.unwrap_or(journal.slippage_bps);

    validate_finite_positive("Starting balance", starting_balance)?;
    validate_bps("Taker fee", taker_fee_bps)?;
    validate_bps("Maker fee", maker_fee_bps)?;
    validate_bps("Slippage", slippage_bps)?;

    let reset = PaperJournal::new(
        starting_balance,
        settle_coin,
        taker_fee_bps,
        maker_fee_bps,
        slippage_bps,
    );
    save_journal(&reset)?;
    Ok(reset)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reserved_map_accepts_legacy_number() {
        let json = r#"{
            "balance": {"coins": {"USDT": 1000.0}},
            "settle_coin": "USDT",
            "reserved": 25.0
        }"#;
        let journal: PaperJournal = serde_json::from_str(json).unwrap();
        assert_eq!(
            journal.reserved.get(LEGACY_RESERVED_KEY).copied(),
            Some(25.0)
        );
    }

    #[test]
    fn repair_rebuilds_sell_reservations() {
        let journal = PaperJournal {
            balance: PaperBalance {
                coins: HashMap::from([("USDT".to_string(), 1000.0), ("BTC".to_string(), 1.0)]),
            },
            positions: vec![],
            trades: vec![],
            pending_orders: vec![PaperOrder {
                id: 1,
                category: "linear".to_string(),
                symbol: "BTCUSDT".to_string(),
                base_asset: "".to_string(),
                settle_coin: "".to_string(),
                side: OrderSide::Sell,
                order_type: OrderType::Limit,
                qty: 0.25,
                price: 60_000.0,
                reserved_asset: "".to_string(),
                reserved_amount: 0.0,
                created_at: now_rfc3339(),
            }],
            cancelled_orders: vec![],
            settle_coin: "USDT".to_string(),
            taker_fee_bps: DEFAULT_TAKER_FEE_BPS,
            maker_fee_bps: DEFAULT_MAKER_FEE_BPS,
            slippage_bps: DEFAULT_SLIPPAGE_BPS,
            reserved: HashMap::new(),
            total_fees_paid: 0.0,
            starting_balance: 1000.0,
            created_at: now_rfc3339(),
            updated_at: now_rfc3339(),
            next_order_id: 1,
        };

        let repaired = repair_journal(journal);
        assert_eq!(repaired.pending_orders[0].reserved_asset, "BTC");
        assert_eq!(repaired.reserved.get("BTC").copied(), Some(0.25));
    }

    #[test]
    fn apply_fill_rejects_reserved_sell_overcommit() {
        let mut journal = PaperJournal::new(10_000.0, "USDT".to_string(), 6, 1, 5);
        journal.balance.coins.insert("BTC".to_string(), 1.0);
        reserve_asset(&mut journal, "BTC", 0.75);
        let result = apply_fill(
            &mut journal,
            "linear",
            "BTCUSDT",
            0.5,
            60_000.0,
            OrderSide::Sell,
            0.0,
            Some(1),
        );
        assert!(result.is_err());
    }
}
