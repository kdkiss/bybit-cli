// MCP tool registry — defines all tools exposed by the MCP server.

use serde_json::json;
use serde_json::Value;

use super::{DEFAULT_SERVICES, VALID_SERVICES};
use super::schema::{bool_prop, enum_prop, int_prop, num_prop, object_schema, str_prop};

// ---------------------------------------------------------------------------
// Tool descriptor
// ---------------------------------------------------------------------------

#[derive(Clone)]
pub struct McpTool {
    pub name: &'static str,
    pub description: &'static str,
    pub input_schema: Value,
    /// Which service group this tool belongs to (market / account / trade / position / asset / funding / reports / subaccount / futures / paper / auth)
    pub service: &'static str,
    /// In guarded mode, dangerous tools stay visible but require
    /// `acknowledged=true`. `--allow-dangerous` removes the per-call gate.
    pub dangerous: bool,
}

// ---------------------------------------------------------------------------
// Category enum helper
// ---------------------------------------------------------------------------

fn category_prop() -> Value {
    enum_prop("Asset category", &["linear", "spot", "inverse", "option"])
}

fn category_required_schema(extra: Vec<(&str, Value)>, extra_required: &[&str]) -> Value {
    let mut props = vec![("category", category_prop())];
    props.extend(extra);
    let mut required = vec!["category"];
    required.extend_from_slice(extra_required);
    object_schema(props, &required)
}

// ---------------------------------------------------------------------------
// Market tools
// ---------------------------------------------------------------------------

fn market_tools() -> Vec<McpTool> {
    vec![
        McpTool {
            name: "market_server_time",
            description: "Get the Bybit server time",
            input_schema: object_schema(vec![], &[]),
            service: "market",
            dangerous: false,
        },
        McpTool {
            name: "market_tickers",
            description: "Get ticker data (price, 24h stats, funding rate) for one or all symbols",
            input_schema: object_schema(
                vec![
                    ("category", category_prop()),
                    ("symbol", str_prop("Trading pair symbol, e.g. BTCUSDT")),
                    ("base_coin", str_prop("Base coin filter, e.g. BTC")),
                    (
                        "exp_date",
                        str_prop("Expiry date for options, e.g. 25DEC23"),
                    ),
                ],
                &["category"],
            ),
            service: "market",
            dangerous: false,
        },
        McpTool {
            name: "market_orderbook",
            description: "Get the order book for a symbol",
            input_schema: object_schema(
                vec![
                    ("category", category_prop()),
                    ("symbol", str_prop("Trading pair symbol, e.g. BTCUSDT")),
                    ("limit", int_prop("Depth limit (1–500, default 25)")),
                ],
                &["category", "symbol"],
            ),
            service: "market",
            dangerous: false,
        },
        McpTool {
            name: "market_kline",
            description: "Get OHLCV candlestick data",
            input_schema: object_schema(
                vec![
                    ("category", category_prop()),
                    ("symbol", str_prop("Trading pair symbol")),
                    (
                        "interval",
                        enum_prop(
                            "Candle interval",
                            &[
                                "1", "3", "5", "15", "30", "60", "120", "240", "360", "720", "D",
                                "W", "M",
                            ],
                        ),
                    ),
                    ("start", int_prop("Start time in milliseconds")),
                    ("end", int_prop("End time in milliseconds")),
                    (
                        "limit",
                        int_prop("Number of candles (default 200, max 1000)"),
                    ),
                ],
                &["category", "symbol", "interval"],
            ),
            service: "market",
            dangerous: false,
        },
        McpTool {
            name: "market_funding_rate",
            description: "Get historical funding rate data for a perpetual symbol",
            input_schema: object_schema(
                vec![
                    ("category", category_prop()),
                    ("symbol", str_prop("Perpetual symbol, e.g. BTCUSDT")),
                    ("start", int_prop("Start time in milliseconds")),
                    ("end", int_prop("End time in milliseconds")),
                    (
                        "limit",
                        int_prop("Number of records (default 200, max 200)"),
                    ),
                ],
                &["category", "symbol"],
            ),
            service: "market",
            dangerous: false,
        },
        McpTool {
            name: "market_trades",
            description: "Get recent public trade history for a symbol",
            input_schema: object_schema(
                vec![
                    ("category", category_prop()),
                    ("symbol", str_prop("Trading pair symbol")),
                    ("limit", int_prop("Number of trades (default 60, max 1000)")),
                    ("base_coin", str_prop("Base coin for options")),
                    (
                        "option_type",
                        enum_prop("Option type filter", &["Call", "Put"]),
                    ),
                ],
                &["category", "symbol"],
            ),
            service: "market",
            dangerous: false,
        },
        McpTool {
            name: "market_instruments",
            description: "Get instrument (trading pair) specifications",
            input_schema: object_schema(
                vec![
                    ("category", category_prop()),
                    ("symbol", str_prop("Specific symbol to query")),
                    (
                        "status",
                        enum_prop("Instrument status", &["Trading", "PreLaunch"]),
                    ),
                    ("base_coin", str_prop("Base coin filter, e.g. BTC")),
                    ("limit", int_prop("Page size (default 500)")),
                    ("cursor", str_prop("Pagination cursor")),
                ],
                &["category"],
            ),
            service: "market",
            dangerous: false,
        },
        McpTool {
            name: "market_open_interest",
            description: "Get open interest data for a symbol",
            input_schema: object_schema(
                vec![
                    ("category", category_prop()),
                    ("symbol", str_prop("Symbol, e.g. BTCUSDT")),
                    (
                        "interval_time",
                        enum_prop("Interval", &["5min", "15min", "30min", "1h", "4h", "1d"]),
                    ),
                    ("start", int_prop("Start time in milliseconds")),
                    ("end", int_prop("End time in milliseconds")),
                    ("limit", int_prop("Number of records (default 50, max 200)")),
                ],
                &["category", "symbol", "interval_time"],
            ),
            service: "market",
            dangerous: false,
        },
        McpTool {
            name: "market_risk_limit",
            description: "Get risk limit tiers for a symbol",
            input_schema: category_required_schema(
                vec![("symbol", str_prop("Symbol, e.g. BTCUSDT"))],
                &[],
            ),
            service: "market",
            dangerous: false,
        },
        McpTool {
            name: "market_insurance",
            description: "Get insurance fund data",
            input_schema: object_schema(
                vec![("coin", str_prop("Coin, e.g. BTC (default: all)"))],
                &[],
            ),
            service: "market",
            dangerous: false,
        },
        McpTool {
            name: "market_delivery_price",
            description: "Get delivery price for futures/options",
            input_schema: category_required_schema(
                vec![
                    ("symbol", str_prop("Symbol")),
                    ("base_coin", str_prop("Base coin")),
                    ("limit", int_prop("Page size")),
                    ("cursor", str_prop("Pagination cursor")),
                ],
                &[],
            ),
            service: "market",
            dangerous: false,
        },
        McpTool {
            name: "market_ls_ratio",
            description: "Get long/short ratio for a symbol",
            input_schema: object_schema(
                vec![
                    ("category", category_prop()),
                    ("symbol", str_prop("Symbol, e.g. BTCUSDT")),
                    (
                        "period",
                        enum_prop("Period", &["5min", "15min", "30min", "1h", "4h", "1d"]),
                    ),
                    ("limit", int_prop("Number of records (max 500)")),
                ],
                &["category", "symbol", "period"],
            ),
            service: "market",
            dangerous: false,
        },
        McpTool {
            name: "market_spread",
            description: "Get current bid-ask spread for a symbol",
            input_schema: object_schema(
                vec![
                    ("category", category_prop()),
                    ("symbol", str_prop("Symbol, e.g. BTCUSDT")),
                ],
                &["category", "symbol"],
            ),
            service: "market",
            dangerous: false,
        },
    ]
}

// ---------------------------------------------------------------------------
// Account tools
// ---------------------------------------------------------------------------

fn account_tools() -> Vec<McpTool> {
    vec![
        McpTool {
            name: "account_balance",
            description: "Get unified trading account balance",
            input_schema: object_schema(
                vec![
                    (
                        "account_type",
                        enum_prop("Account type", &["UNIFIED", "CONTRACT"]),
                    ),
                    ("coin", str_prop("Filter by coin, e.g. USDT")),
                ],
                &[],
            ),
            service: "account",
            dangerous: false,
        },
        McpTool {
            name: "account_info",
            description: "Get account margin mode and status",
            input_schema: object_schema(vec![], &[]),
            service: "account",
            dangerous: false,
        },
        McpTool {
            name: "account_fee_rate",
            description: "Get maker/taker fee rates",
            input_schema: object_schema(
                vec![
                    ("category", category_prop()),
                    ("symbol", str_prop("Symbol to get fee rate for")),
                    ("base_coin", str_prop("Base coin (options only)")),
                ],
                &["category"],
            ),
            service: "account",
            dangerous: false,
        },
        McpTool {
            name: "account_transaction_log",
            description: "Get account transaction / P&L history",
            input_schema: object_schema(
                vec![
                    ("account_type", str_prop("Account type, e.g. UNIFIED")),
                    ("category", category_prop()),
                    ("currency", str_prop("Settlement currency, e.g. USDT")),
                    ("start", int_prop("Start time in milliseconds")),
                    ("end", int_prop("End time in milliseconds")),
                    ("limit", int_prop("Page size (default 20, max 50)")),
                    ("cursor", str_prop("Pagination cursor")),
                ],
                &[],
            ),
            service: "account",
            dangerous: false,
        },
        McpTool {
            name: "account_borrow_history",
            description: "Get borrow history for the account",
            input_schema: object_schema(
                vec![
                    ("currency", str_prop("Coin, e.g. USDT")),
                    ("start", int_prop("Start time in milliseconds")),
                    ("end", int_prop("End time in milliseconds")),
                    ("limit", int_prop("Page size (default 20, max 50)")),
                    ("cursor", str_prop("Pagination cursor")),
                ],
                &[],
            ),
            service: "account",
            dangerous: false,
        },
        McpTool {
            name: "account_collateral_info",
            description: "Get collateral information for margin trading",
            input_schema: object_schema(vec![("currency", str_prop("Coin, e.g. BTC"))], &[]),
            service: "account",
            dangerous: false,
        },
        McpTool {
            name: "account_volume",
            description: "Get trading volume for a period (e.g. last 30 days)",
            input_schema: object_schema(
                vec![
                    ("category", category_prop()),
                    ("symbol", str_prop("Symbol filter")),
                    ("base_coin", str_prop("Base coin filter")),
                    ("days", int_prop("Lookback days (default 30)")),
                ],
                &["category"],
            ),
            service: "account",
            dangerous: false,
        },
        McpTool {
            name: "account_set_usdc_settlement",
            description: "Set USDC settlement coin (USDC or USDT) for UTA",
            input_schema: object_schema(
                vec![("coin", enum_prop("Settlement coin", &["USDC", "USDT"]))],
                &["coin"],
            ),
            service: "account",
            dangerous: true,
        },
        McpTool {
            name: "account_adl_alert",
            description: "Get ADL risk level for a symbol (Private)",
            input_schema: category_required_schema(
                vec![("symbol", str_prop("Specific symbol (optional)"))],
                &[],
            ),
            service: "account",
            dangerous: false,
        },
        McpTool {
            name: "account_borrow",
            description: "Manually borrow funds (UTA Pro / Portfolio Margin accounts only)",
            input_schema: object_schema(
                vec![
                    ("coin", str_prop("Coin to borrow, e.g. USDT")),
                    ("amount", str_prop("Amount to borrow")),
                ],
                &["coin", "amount"],
            ),
            service: "account",
            dangerous: true,
        },
        McpTool {
            name: "account_repay",
            description: "Manually repay borrowed funds",
            input_schema: object_schema(
                vec![
                    ("coin", str_prop("Coin to repay, e.g. USDT")),
                    ("amount", str_prop("Amount to repay")),
                ],
                &["coin", "amount"],
            ),
            service: "account",
            dangerous: true,
        },
        McpTool {
            name: "account_quick_repay",
            description: "Quick-repay liability (auto-select repayment amount)",
            input_schema: object_schema(
                vec![("coin", str_prop("Coin to repay; omit to auto-select"))],
                &[],
            ),
            service: "account",
            dangerous: true,
        },
    ]
}

// ---------------------------------------------------------------------------
// Earn tools
// ---------------------------------------------------------------------------

fn earn_tools() -> Vec<McpTool> {
    vec![
        McpTool {
            name: "earn_product",
            description: "List available savings/earn products",
            input_schema: object_schema(
                vec![
                    (
                        "product_type",
                        str_prop("Product type: FlexibleSavings, FixedSavings"),
                    ),
                    ("coin", str_prop("Coin filter, e.g. USDT")),
                ],
                &[],
            ),
            service: "earn",
            dangerous: false,
        },
        McpTool {
            name: "earn_positions",
            description: "Get current staking/earn positions",
            input_schema: object_schema(
                vec![
                    ("coin", str_prop("Coin filter")),
                    ("order_id", str_prop("Specific order ID")),
                ],
                &[],
            ),
            service: "earn",
            dangerous: false,
        },
        McpTool {
            name: "earn_stake",
            description: "Stake funds into an earn product",
            input_schema: object_schema(
                vec![
                    ("product_id", str_prop("Product ID")),
                    ("coin", str_prop("Coin symbol")),
                    ("amount", str_prop("Amount to stake")),
                ],
                &["product_id", "coin", "amount"],
            ),
            service: "earn",
            dangerous: true,
        },
        McpTool {
            name: "earn_unstake",
            description: "Unstake funds from an earn product",
            input_schema: object_schema(
                vec![
                    ("order_id", str_prop("Order ID to unstake from")),
                    ("coin", str_prop("Coin symbol")),
                    ("amount", str_prop("Amount to unstake")),
                ],
                &["order_id", "coin", "amount"],
            ),
            service: "earn",
            dangerous: true,
        },
        McpTool {
            name: "earn_history",
            description: "Get staking/earn history",
            input_schema: object_schema(
                vec![
                    ("coin", str_prop("Coin filter")),
                    ("start", int_prop("Start time in milliseconds")),
                    ("end", int_prop("End time in milliseconds")),
                    ("limit", int_prop("Page size")),
                    ("cursor", str_prop("Pagination cursor")),
                ],
                &[],
            ),
            service: "earn",
            dangerous: false,
        },
    ]
}

// ---------------------------------------------------------------------------
// Trade tools (dangerous)
// ---------------------------------------------------------------------------

fn trade_tools() -> Vec<McpTool> {
    let order_schema = |_side: &str| {
        object_schema(
            vec![
                ("category", category_prop()),
                ("symbol", str_prop("Trading pair, e.g. BTCUSDT")),
                ("qty", str_prop("Order quantity")),
                ("price", str_prop("Limit price (omit for market order)")),
                ("order_type", enum_prop("Order type", &["Limit", "Market"])),
                (
                    "time_in_force",
                    enum_prop("Time in force", &["GTC", "IOC", "FOK", "PostOnly"]),
                ),
                ("take_profit", str_prop("Take-profit price")),
                ("stop_loss", str_prop("Stop-loss price")),
                ("order_link_id", str_prop("Client order ID")),
                (
                    "position_idx",
                    int_prop("Position index: 0=one-way, 1=buy-side hedge, 2=sell-side hedge"),
                ),
                ("reduce_only", bool_prop("Reduce-only order")),
                (
                    "post_only",
                    bool_prop("Convenience flag for PostOnly time-in-force"),
                ),
                (
                    "display_qty",
                    str_prop("Visible quantity for Iceberg orders"),
                ),
                (
                    "trigger_price",
                    str_prop("Trigger price for conditional orders"),
                ),
                ("tp_limit_price", str_prop("Limit price for take profit")),
                ("sl_limit_price", str_prop("Limit price for stop loss")),
                (
                    "tp_trigger_by",
                    str_prop("TP trigger source: LastPrice, IndexPrice, MarkPrice"),
                ),
                (
                    "sl_trigger_by",
                    str_prop("SL trigger source: LastPrice, IndexPrice, MarkPrice"),
                ),
            ],
            &["category", "symbol", "qty"],
        )
    };

    vec![
        McpTool {
            name: "trade_buy",
            description: "Place a buy (long) order. Always --validate first to dry-run.",
            input_schema: order_schema("buy"),
            service: "trade",
            dangerous: true,
        },
        McpTool {
            name: "trade_sell",
            description: "Place a sell (short) order. Always --validate first to dry-run.",
            input_schema: order_schema("sell"),
            service: "trade",
            dangerous: true,
        },
        McpTool {
            name: "trade_validate_buy",
            description: "Dry-run a buy order (validate only, does not submit)",
            input_schema: order_schema("buy"),
            service: "trade",
            dangerous: false,
        },
        McpTool {
            name: "trade_validate_sell",
            description: "Dry-run a sell order (validate only, does not submit)",
            input_schema: order_schema("sell"),
            service: "trade",
            dangerous: false,
        },
        McpTool {
            name: "trade_amend",
            description: "Amend an existing open order",
            input_schema: object_schema(
                vec![
                    ("category", category_prop()),
                    ("symbol", str_prop("Symbol")),
                    ("order_id", str_prop("Order ID to amend")),
                    ("order_link_id", str_prop("Client order ID")),
                    ("qty", str_prop("New quantity")),
                    ("price", str_prop("New price")),
                    ("take_profit", str_prop("New take-profit price")),
                    ("stop_loss", str_prop("New stop-loss price")),
                    ("trigger_price", str_prop("New trigger price")),
                ],
                &["category", "symbol"],
            ),
            service: "trade",
            dangerous: true,
        },
        McpTool {
            name: "trade_cancel",
            description: "Cancel an open order by order_id or order_link_id",
            input_schema: object_schema(
                vec![
                    ("category", category_prop()),
                    ("symbol", str_prop("Symbol")),
                    ("order_id", str_prop("Order ID to cancel")),
                    ("order_link_id", str_prop("Client order ID to cancel")),
                ],
                &["category", "symbol"],
            ),
            service: "trade",
            dangerous: true,
        },
        McpTool {
            name: "trade_cancel_all",
            description: "Cancel all open orders for a symbol or category",
            input_schema: object_schema(
                vec![
                    ("category", category_prop()),
                    (
                        "symbol",
                        str_prop("Symbol (omit to cancel all in category)"),
                    ),
                    ("base_coin", str_prop("Base coin filter")),
                    ("settle_coin", str_prop("Settle coin filter")),
                ],
                &["category"],
            ),
            service: "trade",
            dangerous: true,
        },
        McpTool {
            name: "trade_open_orders",
            description: "List open orders",
            input_schema: object_schema(
                vec![
                    ("category", category_prop()),
                    ("symbol", str_prop("Symbol filter")),
                    ("base_coin", str_prop("Base coin filter")),
                    ("order_id", str_prop("Specific order ID")),
                    ("order_link_id", str_prop("Specific client order ID")),
                    ("limit", int_prop("Page size (default 20, max 50)")),
                    ("cursor", str_prop("Pagination cursor")),
                ],
                &["category"],
            ),
            service: "trade",
            dangerous: false,
        },
        McpTool {
            name: "trade_history",
            description: "Get order history",
            input_schema: object_schema(
                vec![
                    ("category", category_prop()),
                    ("symbol", str_prop("Symbol filter")),
                    ("order_id", str_prop("Specific order ID")),
                    ("order_status", str_prop("Status filter, e.g. Filled")),
                    ("start", int_prop("Start time in milliseconds")),
                    ("end", int_prop("End time in milliseconds")),
                    ("limit", int_prop("Page size (default 20, max 50)")),
                    ("cursor", str_prop("Pagination cursor")),
                ],
                &["category"],
            ),
            service: "trade",
            dangerous: false,
        },
        McpTool {
            name: "trade_fills",
            description: "Get execution (fill) history",
            input_schema: object_schema(
                vec![
                    ("category", category_prop()),
                    ("symbol", str_prop("Symbol filter")),
                    ("order_id", str_prop("Order ID filter")),
                    ("start", int_prop("Start time in milliseconds")),
                    ("end", int_prop("End time in milliseconds")),
                    ("exec_type", str_prop("Execution type filter")),
                    ("limit", int_prop("Page size (default 20, max 100)")),
                    ("cursor", str_prop("Pagination cursor")),
                ],
                &["category"],
            ),
            service: "trade",
            dangerous: false,
        },
        McpTool {
            name: "trade_cancel_after",
            description: "Dead man's switch — cancel all open orders after N seconds (0 = disable)",
            input_schema: object_schema(
                vec![(
                    "seconds",
                    int_prop("Seconds until all orders are cancelled. 0 to disable."),
                )],
                &["seconds"],
            ),
            service: "trade",
            dangerous: true,
        },
        McpTool {
            name: "trade_dcp_info",
            description: "Get current DCP (Disconnect Cancel All) window configuration",
            input_schema: object_schema(vec![], &[]),
            service: "trade",
            dangerous: false,
        },
        McpTool {
            name: "trade_batch_place",
            description: "Place up to 20 orders in a single request",
            input_schema: object_schema(
                vec![
                    ("category", category_prop()),
                    ("orders", str_prop("JSON array of order objects")),
                ],
                &["category", "orders"],
            ),
            service: "trade",
            dangerous: true,
        },
        McpTool {
            name: "trade_batch_amend",
            description: "Amend up to 20 orders in a single request",
            input_schema: object_schema(
                vec![
                    ("category", category_prop()),
                    ("orders", str_prop("JSON array of amend objects")),
                ],
                &["category", "orders"],
            ),
            service: "trade",
            dangerous: true,
        },
        McpTool {
            name: "trade_batch_cancel",
            description: "Cancel up to 20 orders in a single request",
            input_schema: object_schema(
                vec![
                    ("category", category_prop()),
                    (
                        "orders",
                        str_prop(
                            "JSON array of cancel objects (each needs orderId or orderLinkId)",
                        ),
                    ),
                ],
                &["category", "orders"],
            ),
            service: "trade",
            dangerous: true,
        },
    ]
}

// ---------------------------------------------------------------------------
// Position tools
// ---------------------------------------------------------------------------

fn position_tools() -> Vec<McpTool> {
    vec![
        McpTool {
            name: "position_list",
            description: "List open positions",
            input_schema: object_schema(
                vec![
                    ("category", category_prop()),
                    ("symbol", str_prop("Symbol filter")),
                    ("base_coin", str_prop("Base coin filter")),
                    ("settle_coin", str_prop("Settle coin filter")),
                    ("limit", int_prop("Page size (default 20, max 200)")),
                    ("cursor", str_prop("Pagination cursor")),
                ],
                &["category"],
            ),
            service: "position",
            dangerous: false,
        },
        McpTool {
            name: "position_set_leverage",
            description: "Set leverage for a position",
            input_schema: object_schema(
                vec![
                    ("category", category_prop()),
                    ("symbol", str_prop("Symbol")),
                    ("buy_leverage", num_prop("Buy/long leverage multiplier")),
                    ("sell_leverage", num_prop("Sell/short leverage multiplier")),
                ],
                &["category", "symbol", "buy_leverage", "sell_leverage"],
            ),
            service: "position",
            dangerous: true,
        },
        McpTool {
            name: "position_set_tpsl",
            description: "Set take-profit and/or stop-loss on a position",
            input_schema: object_schema(
                vec![
                    ("category", category_prop()),
                    ("symbol", str_prop("Symbol")),
                    ("take_profit", str_prop("Take-profit price")),
                    ("stop_loss", str_prop("Stop-loss price")),
                    ("trailing_stop", str_prop("Trailing stop distance")),
                    (
                        "position_idx",
                        int_prop("0=one-way, 1=buy-side hedge, 2=sell-side hedge"),
                    ),
                ],
                &["category", "symbol"],
            ),
            service: "position",
            dangerous: true,
        },
        McpTool {
            name: "position_closed_pnl",
            description: "Get closed P&L history",
            input_schema: object_schema(
                vec![
                    ("category", category_prop()),
                    ("symbol", str_prop("Symbol filter")),
                    ("start", int_prop("Start time in milliseconds")),
                    ("end", int_prop("End time in milliseconds")),
                    ("limit", int_prop("Page size (default 50, max 100)")),
                    ("cursor", str_prop("Pagination cursor")),
                ],
                &["category"],
            ),
            service: "position",
            dangerous: false,
        },
        McpTool {
            name: "position_switch_mode",
            description: "Switch position mode (one-way vs hedge mode)",
            input_schema: object_schema(
                vec![
                    ("category", category_prop()),
                    ("symbol", str_prop("Symbol")),
                    ("coin", str_prop("Coin (for no-symbol mode switch)")),
                    ("mode", int_prop("0 = one-way mode, 3 = hedge mode")),
                ],
                &["category", "mode"],
            ),
            service: "position",
            dangerous: true,
        },
        McpTool {
            name: "position_add_margin",
            description: "Manually add margin to a position",
            input_schema: object_schema(
                vec![
                    ("category", category_prop()),
                    ("symbol", str_prop("Symbol")),
                    ("margin", str_prop("Margin amount to add")),
                    (
                        "position_idx",
                        int_prop("0=one-way, 1=buy-hedge, 2=sell-hedge"),
                    ),
                ],
                &["category", "symbol", "margin"],
            ),
            service: "position",
            dangerous: true,
        },
        McpTool {
            name: "position_flatten",
            description: "EMERGENCY: Cancel all orders and close all positions in a category",
            input_schema: object_schema(
                vec![
                    ("category", category_prop()),
                    ("symbol", str_prop("Specific symbol to flatten (optional)")),
                ],
                &["category"],
            ),
            service: "position",
            dangerous: true,
        },
    ]
}

// ---------------------------------------------------------------------------
// Asset tools
// ---------------------------------------------------------------------------

fn asset_tools() -> Vec<McpTool> {
    vec![
        McpTool {
            name: "asset_coin_info",
            description: "Get coin information (deposit/withdraw status, chain info)",
            input_schema: object_schema(vec![("coin", str_prop("Coin name, e.g. BTC"))], &[]),
            service: "asset",
            dangerous: false,
        },
        McpTool {
            name: "asset_balance",
            description: "Get asset balance for an account type",
            input_schema: object_schema(
                vec![
                    (
                        "account_type",
                        enum_prop(
                            "Account type",
                            &["SPOT", "UNIFIED", "CONTRACT", "FUND", "OPTION"],
                        ),
                    ),
                    ("coin", str_prop("Coin filter, e.g. USDT")),
                ],
                &["account_type"],
            ),
            service: "asset",
            dangerous: false,
        },
        McpTool {
            name: "asset_all_balance",
            description: "Get all asset balances across all account types",
            input_schema: object_schema(
                vec![
                    ("member_id", str_prop("Sub-account UID")),
                    (
                        "account_type",
                        enum_prop("Account type", &["SPOT", "UNIFIED", "CONTRACT", "FUND"]),
                    ),
                    ("coin", str_prop("Coin filter")),
                    ("with_bonus", bool_prop("Include bonus assets")),
                ],
                &[],
            ),
            service: "asset",
            dangerous: false,
        },
        McpTool {
            name: "asset_transferable",
            description: "Get transferable amount between account types",
            input_schema: object_schema(
                vec![
                    ("from_account_type", str_prop("Source account type")),
                    ("to_account_type", str_prop("Destination account type")),
                    ("coin", str_prop("Coin to check")),
                ],
                &["from_account_type", "to_account_type", "coin"],
            ),
            service: "asset",
            dangerous: false,
        },
        McpTool {
            name: "asset_deposit_address",
            description: "Get deposit address for a coin/chain",
            input_schema: object_schema(
                vec![
                    ("coin", str_prop("Coin, e.g. USDT")),
                    ("chain_type", str_prop("Chain, e.g. ETH")),
                ],
                &["coin"],
            ),
            service: "asset",
            dangerous: false,
        },
        McpTool {
            name: "asset_deposit_history",
            description: "Get deposit history",
            input_schema: object_schema(
                vec![
                    ("coin", str_prop("Coin filter")),
                    ("start", int_prop("Start time in milliseconds")),
                    ("end", int_prop("End time in milliseconds")),
                    ("limit", int_prop("Page size (default 50, max 50)")),
                    ("cursor", str_prop("Pagination cursor")),
                ],
                &[],
            ),
            service: "asset",
            dangerous: false,
        },
        McpTool {
            name: "asset_withdraw_history",
            description: "Get withdrawal history",
            input_schema: object_schema(
                vec![
                    ("coin", str_prop("Coin filter")),
                    ("withdraw_id", str_prop("Specific withdrawal ID")),
                    ("start", int_prop("Start time in milliseconds")),
                    ("end", int_prop("End time in milliseconds")),
                    ("limit", int_prop("Page size (default 50, max 50)")),
                    ("cursor", str_prop("Pagination cursor")),
                ],
                &[],
            ),
            service: "asset",
            dangerous: false,
        },
        McpTool {
            name: "asset_transfer",
            description: "Transfer assets between account types",
            input_schema: object_schema(
                vec![
                    ("coin", str_prop("Coin to transfer, e.g. USDT")),
                    ("amount", str_prop("Amount to transfer")),
                    (
                        "from_account_type",
                        enum_prop("Source account", &["SPOT", "UNIFIED", "CONTRACT", "FUND"]),
                    ),
                    (
                        "to_account_type",
                        enum_prop(
                            "Destination account",
                            &["SPOT", "UNIFIED", "CONTRACT", "FUND"],
                        ),
                    ),
                    (
                        "transfer_id",
                        str_prop("Client transfer ID (optional UUID)"),
                    ),
                ],
                &["coin", "amount", "from_account_type", "to_account_type"],
            ),
            service: "asset",
            dangerous: true,
        },
        McpTool {
            name: "asset_withdraw",
            description: "Submit a withdrawal request",
            input_schema: object_schema(
                vec![
                    ("coin", str_prop("Coin to withdraw, e.g. USDT")),
                    ("chain", str_prop("Blockchain network, e.g. ETH")),
                    ("address", str_prop("Destination wallet address")),
                    ("tag", str_prop("Memo/tag (required for some chains)")),
                    ("amount", str_prop("Amount to withdraw")),
                    ("timestamp", int_prop("Request timestamp in milliseconds")),
                    (
                        "account_type",
                        enum_prop("Source account", &["SPOT", "UNIFIED", "FUND"]),
                    ),
                ],
                &["coin", "chain", "address", "amount", "timestamp"],
            ),
            service: "asset",
            dangerous: true,
        },
        McpTool {
            name: "asset_transfer_history",
            description: "Get internal transfer history",
            input_schema: object_schema(
                vec![
                    ("coin", str_prop("Coin filter")),
                    ("transfer_id", str_prop("Specific transfer ID")),
                    ("start", int_prop("Start time in milliseconds")),
                    ("end", int_prop("End time in milliseconds")),
                    ("limit", int_prop("Page size (default 20, max 50)")),
                    ("cursor", str_prop("Pagination cursor")),
                ],
                &[],
            ),
            service: "asset",
            dangerous: false,
        },
        McpTool {
            name: "asset_withdrawal_methods",
            description: "Get available withdrawal networks and fees for a coin",
            input_schema: object_schema(vec![("coin", str_prop("Coin filter, e.g. USDT"))], &[]),
            service: "asset",
            dangerous: false,
        },
    ]
}

// ---------------------------------------------------------------------------
// Funding tools
// ---------------------------------------------------------------------------

fn funding_tools() -> Vec<McpTool> {
    vec![
        McpTool {
            name: "funding_coin_info",
            description: "Get coin and chain metadata for funding operations",
            input_schema: object_schema(vec![("coin", str_prop("Coin filter, e.g. USDT"))], &[]),
            service: "funding",
            dangerous: false,
        },
        McpTool {
            name: "funding_balance",
            description: "Get funding balance by account type",
            input_schema: object_schema(
                vec![
                    (
                        "account_type",
                        enum_prop(
                            "Account type",
                            &["SPOT", "UNIFIED", "CONTRACT", "FUND", "OPTION"],
                        ),
                    ),
                    ("coin", str_prop("Coin filter, e.g. USDT")),
                ],
                &[],
            ),
            service: "funding",
            dangerous: false,
        },
        McpTool {
            name: "funding_all_balance",
            description: "Get balances across account types",
            input_schema: object_schema(
                vec![
                    ("account_type", str_prop("Account type, e.g. UNIFIED")),
                    ("coin", str_prop("Coin filter, e.g. USDT")),
                    ("member_id", str_prop("Optional member/subaccount ID")),
                ],
                &[],
            ),
            service: "funding",
            dangerous: false,
        },
        McpTool {
            name: "funding_account_balance",
            description: "Get a single funding coin balance",
            input_schema: object_schema(
                vec![
                    ("account_type", str_prop("Account type, e.g. UNIFIED")),
                    ("member_id", str_prop("Optional member/subaccount ID")),
                    ("coin", str_prop("Coin symbol, e.g. USDT")),
                    ("with_bonus", bool_prop("Include bonus balances")),
                ],
                &["coin"],
            ),
            service: "funding",
            dangerous: false,
        },
        McpTool {
            name: "funding_transferable",
            description: "List coins transferable between wallet types",
            input_schema: object_schema(
                vec![
                    ("from_account_type", str_prop("Source account type")),
                    ("to_account_type", str_prop("Destination account type")),
                ],
                &["from_account_type", "to_account_type"],
            ),
            service: "funding",
            dangerous: false,
        },
        McpTool {
            name: "funding_transfer",
            description: "Transfer funds between wallet types on the same UID",
            input_schema: object_schema(
                vec![
                    ("coin", str_prop("Coin to transfer, e.g. USDT")),
                    ("amount", str_prop("Amount to transfer")),
                    (
                        "from_account_type",
                        enum_prop("Source account", &["SPOT", "UNIFIED", "CONTRACT", "FUND"]),
                    ),
                    (
                        "to_account_type",
                        enum_prop(
                            "Destination account",
                            &["SPOT", "UNIFIED", "CONTRACT", "FUND"],
                        ),
                    ),
                ],
                &["coin", "amount", "from_account_type", "to_account_type"],
            ),
            service: "funding",
            dangerous: true,
        },
        McpTool {
            name: "funding_transfer_history",
            description: "Get internal transfer history",
            input_schema: object_schema(
                vec![
                    ("coin", str_prop("Coin filter")),
                    ("status", str_prop("Transfer status filter")),
                    ("start", int_prop("Start time in milliseconds")),
                    ("end", int_prop("End time in milliseconds")),
                    ("limit", int_prop("Page size")),
                    ("cursor", str_prop("Pagination cursor")),
                ],
                &[],
            ),
            service: "funding",
            dangerous: false,
        },
        McpTool {
            name: "funding_sub_transfer",
            description: "Transfer funds across UIDs using universal transfer",
            input_schema: object_schema(
                vec![
                    ("coin", str_prop("Coin to transfer, e.g. USDT")),
                    ("amount", str_prop("Amount to transfer")),
                    ("from_member_id", str_prop("Source member ID")),
                    ("to_member_id", str_prop("Destination member ID")),
                    ("from_account_type", str_prop("Source account type")),
                    ("to_account_type", str_prop("Destination account type")),
                ],
                &[
                    "coin",
                    "amount",
                    "from_member_id",
                    "to_member_id",
                    "from_account_type",
                    "to_account_type",
                ],
            ),
            service: "funding",
            dangerous: true,
        },
        McpTool {
            name: "funding_sub_transfer_history",
            description: "Get universal transfer history",
            input_schema: object_schema(
                vec![
                    ("coin", str_prop("Coin filter")),
                    ("status", str_prop("Transfer status filter")),
                    ("start", int_prop("Start time in milliseconds")),
                    ("end", int_prop("End time in milliseconds")),
                    ("limit", int_prop("Page size")),
                    ("cursor", str_prop("Pagination cursor")),
                ],
                &[],
            ),
            service: "funding",
            dangerous: false,
        },
        McpTool {
            name: "funding_deposit_address",
            description: "Get deposit address for a coin and optional chain",
            input_schema: object_schema(
                vec![
                    ("coin", str_prop("Coin, e.g. BTC")),
                    ("chain_type", str_prop("Chain/network, e.g. BTC")),
                ],
                &["coin"],
            ),
            service: "funding",
            dangerous: false,
        },
        McpTool {
            name: "funding_deposit_history",
            description: "Get deposit history",
            input_schema: object_schema(
                vec![
                    ("coin", str_prop("Coin filter")),
                    ("start", int_prop("Start time in milliseconds")),
                    ("end", int_prop("End time in milliseconds")),
                    ("limit", int_prop("Page size")),
                    ("cursor", str_prop("Pagination cursor")),
                ],
                &[],
            ),
            service: "funding",
            dangerous: false,
        },
        McpTool {
            name: "funding_withdraw",
            description: "Withdraw funds to an external address",
            input_schema: object_schema(
                vec![
                    ("coin", str_prop("Coin, e.g. USDT")),
                    ("chain", str_prop("Chain/network, e.g. TRX")),
                    ("address", str_prop("Destination address")),
                    ("amount", str_prop("Amount to withdraw")),
                    ("tag", str_prop("Memo/tag when required by the chain")),
                    (
                        "account_type",
                        str_prop("Source account type, e.g. UNIFIED"),
                    ),
                ],
                &["coin", "chain", "address", "amount"],
            ),
            service: "funding",
            dangerous: true,
        },
        McpTool {
            name: "funding_withdraw_history",
            description: "Get withdrawal history",
            input_schema: object_schema(
                vec![
                    ("coin", str_prop("Coin filter")),
                    ("withdraw_id", str_prop("Specific withdrawal ID")),
                    ("withdraw_type", int_prop("Withdrawal type filter")),
                    ("start", int_prop("Start time in milliseconds")),
                    ("end", int_prop("End time in milliseconds")),
                    ("limit", int_prop("Page size")),
                    ("cursor", str_prop("Pagination cursor")),
                ],
                &[],
            ),
            service: "funding",
            dangerous: false,
        },
        McpTool {
            name: "funding_cancel_withdraw",
            description: "Cancel a pending withdrawal request",
            input_schema: object_schema(vec![("id", str_prop("Withdrawal request ID"))], &["id"]),
            service: "funding",
            dangerous: true,
        },
    ]
}

// ---------------------------------------------------------------------------
// Reports tools
// ---------------------------------------------------------------------------

fn reports_tools() -> Vec<McpTool> {
    vec![
        McpTool {
            name: "reports_transactions",
            description: "Get account transaction log history",
            input_schema: object_schema(
                vec![
                    ("account_type", str_prop("Account type, e.g. UNIFIED")),
                    ("category", category_prop()),
                    ("currency", str_prop("Settlement currency, e.g. USDT")),
                    ("base_coin", str_prop("Base coin filter")),
                    ("tx_type", str_prop("Transaction type filter")),
                    ("start", int_prop("Start time in milliseconds")),
                    ("end", int_prop("End time in milliseconds")),
                    ("limit", int_prop("Page size")),
                    ("cursor", str_prop("Pagination cursor")),
                ],
                &[],
            ),
            service: "reports",
            dangerous: false,
        },
        McpTool {
            name: "reports_borrow_history",
            description: "Get borrow history",
            input_schema: object_schema(
                vec![
                    ("currency", str_prop("Coin filter")),
                    ("start", int_prop("Start time in milliseconds")),
                    ("end", int_prop("End time in milliseconds")),
                    ("limit", int_prop("Page size")),
                    ("cursor", str_prop("Pagination cursor")),
                ],
                &[],
            ),
            service: "reports",
            dangerous: false,
        },
        McpTool {
            name: "reports_orders",
            description: "Get order history",
            input_schema: object_schema(
                vec![
                    ("category", category_prop()),
                    ("symbol", str_prop("Symbol filter")),
                    ("order_id", str_prop("Specific order ID")),
                    ("order_status", str_prop("Order status filter")),
                    ("start", int_prop("Start time in milliseconds")),
                    ("end", int_prop("End time in milliseconds")),
                    ("limit", int_prop("Page size")),
                    ("cursor", str_prop("Pagination cursor")),
                ],
                &["category"],
            ),
            service: "reports",
            dangerous: false,
        },
        McpTool {
            name: "reports_fills",
            description: "Get execution/fill history",
            input_schema: object_schema(
                vec![
                    ("category", category_prop()),
                    ("symbol", str_prop("Symbol filter")),
                    ("order_id", str_prop("Order ID filter")),
                    ("exec_type", str_prop("Execution type filter")),
                    ("start", int_prop("Start time in milliseconds")),
                    ("end", int_prop("End time in milliseconds")),
                    ("limit", int_prop("Page size")),
                    ("cursor", str_prop("Pagination cursor")),
                ],
                &["category"],
            ),
            service: "reports",
            dangerous: false,
        },
        McpTool {
            name: "reports_closed_pnl",
            description: "Get closed P&L history",
            input_schema: object_schema(
                vec![
                    ("category", category_prop()),
                    ("symbol", str_prop("Symbol filter")),
                    ("start", int_prop("Start time in milliseconds")),
                    ("end", int_prop("End time in milliseconds")),
                    ("limit", int_prop("Page size")),
                    ("cursor", str_prop("Pagination cursor")),
                ],
                &["category"],
            ),
            service: "reports",
            dangerous: false,
        },
        McpTool {
            name: "reports_moves",
            description: "Get position move history",
            input_schema: object_schema(
                vec![
                    ("category", category_prop()),
                    ("symbol", str_prop("Symbol filter")),
                    ("start", int_prop("Start time in milliseconds")),
                    ("end", int_prop("End time in milliseconds")),
                    ("limit", int_prop("Page size")),
                    ("cursor", str_prop("Pagination cursor")),
                ],
                &["category"],
            ),
            service: "reports",
            dangerous: false,
        },
        McpTool {
            name: "reports_deposits",
            description: "Get deposit history",
            input_schema: object_schema(
                vec![
                    ("coin", str_prop("Coin filter")),
                    ("start", int_prop("Start time in milliseconds")),
                    ("end", int_prop("End time in milliseconds")),
                    ("limit", int_prop("Page size")),
                    ("cursor", str_prop("Pagination cursor")),
                ],
                &[],
            ),
            service: "reports",
            dangerous: false,
        },
        McpTool {
            name: "reports_withdrawals",
            description: "Get withdrawal history",
            input_schema: object_schema(
                vec![
                    ("coin", str_prop("Coin filter")),
                    ("withdraw_id", str_prop("Specific withdrawal ID")),
                    ("withdraw_type", int_prop("Withdrawal type filter")),
                    ("start", int_prop("Start time in milliseconds")),
                    ("end", int_prop("End time in milliseconds")),
                    ("limit", int_prop("Page size")),
                    ("cursor", str_prop("Pagination cursor")),
                ],
                &[],
            ),
            service: "reports",
            dangerous: false,
        },
        McpTool {
            name: "reports_transfers",
            description: "Get internal transfer history",
            input_schema: object_schema(
                vec![
                    ("coin", str_prop("Coin filter")),
                    ("status", str_prop("Transfer status filter")),
                    ("start", int_prop("Start time in milliseconds")),
                    ("end", int_prop("End time in milliseconds")),
                    ("limit", int_prop("Page size")),
                    ("cursor", str_prop("Pagination cursor")),
                ],
                &[],
            ),
            service: "reports",
            dangerous: false,
        },
        McpTool {
            name: "reports_sub_transfers",
            description: "Get universal transfer history",
            input_schema: object_schema(
                vec![
                    ("coin", str_prop("Coin filter")),
                    ("status", str_prop("Transfer status filter")),
                    ("start", int_prop("Start time in milliseconds")),
                    ("end", int_prop("End time in milliseconds")),
                    ("limit", int_prop("Page size")),
                    ("cursor", str_prop("Pagination cursor")),
                ],
                &[],
            ),
            service: "reports",
            dangerous: false,
        },
    ]
}

// ---------------------------------------------------------------------------
// Subaccount tools
// ---------------------------------------------------------------------------

fn subaccount_tools() -> Vec<McpTool> {
    vec![
        McpTool {
            name: "subaccount_list",
            description: "List subaccounts for the master account",
            input_schema: object_schema(vec![], &[]),
            service: "subaccount",
            dangerous: false,
        },
        McpTool {
            name: "subaccount_list_all",
            description: "Paginated list of all subaccounts",
            input_schema: object_schema(
                vec![
                    ("page_size", int_prop("Page size")),
                    ("next_cursor", str_prop("Pagination cursor")),
                ],
                &[],
            ),
            service: "subaccount",
            dangerous: false,
        },
        McpTool {
            name: "subaccount_api_keys",
            description: "List API keys for a subaccount",
            input_schema: object_schema(
                vec![
                    ("sub_member_id", str_prop("Subaccount member ID")),
                    ("limit", int_prop("Page size")),
                    ("cursor", str_prop("Pagination cursor")),
                ],
                &["sub_member_id"],
            ),
            service: "subaccount",
            dangerous: false,
        },
        McpTool {
            name: "subaccount_wallet_types",
            description: "List wallet types for the master account or selected subaccounts",
            input_schema: object_schema(
                vec![(
                    "member_ids",
                    str_prop("Comma-separated subaccount member IDs"),
                )],
                &[],
            ),
            service: "subaccount",
            dangerous: false,
        },
        McpTool {
            name: "subaccount_create",
            description: "Create a new subaccount",
            input_schema: object_schema(
                vec![
                    ("username", str_prop("Subaccount username")),
                    ("password", str_prop("Optional password")),
                    (
                        "member_type",
                        int_prop("Member type: 1 normal, 6 custodial"),
                    ),
                    ("quick_login", bool_prop("Enable quick login")),
                ],
                &["username"],
            ),
            service: "subaccount",
            dangerous: true,
        },
        McpTool {
            name: "subaccount_delete",
            description: "Delete a subaccount",
            input_schema: object_schema(
                vec![("sub_member_id", str_prop("Subaccount member ID"))],
                &["sub_member_id"],
            ),
            service: "subaccount",
            dangerous: true,
        },
        McpTool {
            name: "subaccount_freeze",
            description: "Freeze a subaccount (Master account only)",
            input_schema: object_schema(
                vec![("sub_member_id", str_prop("Subaccount member ID"))],
                &["sub_member_id"],
            ),
            service: "subaccount",
            dangerous: true,
        },
        McpTool {
            name: "subaccount_unfreeze",
            description: "Unfreeze a subaccount (Master account only)",
            input_schema: object_schema(
                vec![("sub_member_id", str_prop("Subaccount member ID"))],
                &["sub_member_id"],
            ),
            service: "subaccount",
            dangerous: true,
        },
    ]
}

// ---------------------------------------------------------------------------
// Futures tools
// ---------------------------------------------------------------------------

fn futures_tools() -> Vec<McpTool> {
    let order_schema = || {
        object_schema(
            vec![
                (
                    "category",
                    enum_prop("Futures category", &["linear", "inverse"]),
                ),
                ("symbol", str_prop("Trading pair, e.g. BTCUSDT")),
                ("qty", str_prop("Order quantity")),
                ("price", str_prop("Limit price (omit for market order)")),
                ("order_type", enum_prop("Order type", &["Limit", "Market"])),
                (
                    "time_in_force",
                    enum_prop("Time in force", &["GTC", "IOC", "FOK", "PostOnly"]),
                ),
                ("take_profit", str_prop("Take-profit price")),
                ("stop_loss", str_prop("Stop-loss price")),
                ("order_link_id", str_prop("Client order ID")),
                ("position_idx", int_prop("Position index")),
                ("reduce_only", bool_prop("Reduce-only order")),
            ],
            &["symbol", "qty"],
        )
    };

    vec![
        McpTool {
            name: "futures_instruments",
            description: "Get futures instrument metadata",
            input_schema: object_schema(
                vec![
                    (
                        "category",
                        enum_prop("Futures category", &["linear", "inverse"]),
                    ),
                    ("symbol", str_prop("Specific symbol")),
                    ("status", str_prop("Status filter")),
                    ("base_coin", str_prop("Base coin filter")),
                    ("limit", int_prop("Page size")),
                    ("cursor", str_prop("Pagination cursor")),
                ],
                &[],
            ),
            service: "futures",
            dangerous: false,
        },
        McpTool {
            name: "futures_tickers",
            description: "Get futures tickers / 24h stats",
            input_schema: object_schema(
                vec![
                    (
                        "category",
                        enum_prop("Futures category", &["linear", "inverse"]),
                    ),
                    ("symbol", str_prop("Specific symbol")),
                    ("base_coin", str_prop("Base coin filter")),
                ],
                &[],
            ),
            service: "futures",
            dangerous: false,
        },
        McpTool {
            name: "futures_orderbook",
            description: "Get futures order book depth",
            input_schema: object_schema(
                vec![
                    (
                        "category",
                        enum_prop("Futures category", &["linear", "inverse"]),
                    ),
                    ("symbol", str_prop("Symbol, e.g. BTCUSDT")),
                    ("limit", int_prop("Depth limit")),
                ],
                &["symbol"],
            ),
            service: "futures",
            dangerous: false,
        },
        McpTool {
            name: "futures_funding_rate",
            description: "Get futures funding rate history",
            input_schema: object_schema(
                vec![
                    (
                        "category",
                        enum_prop("Futures category", &["linear", "inverse"]),
                    ),
                    ("symbol", str_prop("Symbol, e.g. BTCUSDT")),
                    ("start", int_prop("Start time in milliseconds")),
                    ("end", int_prop("End time in milliseconds")),
                    ("limit", int_prop("Page size")),
                ],
                &["symbol"],
            ),
            service: "futures",
            dangerous: false,
        },
        McpTool {
            name: "futures_adl_alert",
            description: "Get ADL risk level for a symbol",
            input_schema: object_schema(
                vec![
                    (
                        "category",
                        enum_prop("Futures category", &["linear", "inverse"]),
                    ),
                    ("symbol", str_prop("Specific symbol (optional)")),
                ],
                &[],
            ),
            service: "futures",
            dangerous: false,
        },
        McpTool {
            name: "futures_risk_limit",
            description: "Get risk limit info for a symbol",
            input_schema: object_schema(
                vec![
                    (
                        "category",
                        enum_prop("Futures category", &["linear", "inverse"]),
                    ),
                    ("symbol", str_prop("Specific symbol (optional)")),
                ],
                &[],
            ),
            service: "futures",
            dangerous: false,
        },
        McpTool {
            name: "futures_open_interest",
            description: "Get futures open interest history",
            input_schema: object_schema(
                vec![
                    (
                        "category",
                        enum_prop("Futures category", &["linear", "inverse"]),
                    ),
                    ("symbol", str_prop("Symbol, e.g. BTCUSDT")),
                    (
                        "interval_time",
                        enum_prop("Interval", &["5min", "15min", "30min", "1h", "4h", "1d"]),
                    ),
                    ("start", int_prop("Start time in milliseconds")),
                    ("end", int_prop("End time in milliseconds")),
                    ("limit", int_prop("Page size")),
                    ("cursor", str_prop("Pagination cursor")),
                ],
                &["symbol", "interval_time"],
            ),
            service: "futures",
            dangerous: false,
        },
        McpTool {
            name: "futures_positions",
            description: "List open futures positions",
            input_schema: object_schema(
                vec![
                    (
                        "category",
                        enum_prop("Futures category", &["linear", "inverse"]),
                    ),
                    ("symbol", str_prop("Symbol filter")),
                    ("base_coin", str_prop("Base coin filter")),
                    ("settle_coin", str_prop("Settle coin filter")),
                    ("limit", int_prop("Page size")),
                    ("cursor", str_prop("Pagination cursor")),
                ],
                &[],
            ),
            service: "futures",
            dangerous: false,
        },
        McpTool {
            name: "futures_open_orders",
            description: "List open futures orders",
            input_schema: object_schema(
                vec![
                    (
                        "category",
                        enum_prop("Futures category", &["linear", "inverse"]),
                    ),
                    ("symbol", str_prop("Symbol filter")),
                    ("base_coin", str_prop("Base coin filter")),
                    ("settle_coin", str_prop("Settle coin filter")),
                    ("order_id", str_prop("Specific order ID")),
                    ("order_link_id", str_prop("Specific client order ID")),
                    ("limit", int_prop("Page size")),
                    ("cursor", str_prop("Pagination cursor")),
                ],
                &[],
            ),
            service: "futures",
            dangerous: false,
        },
        McpTool {
            name: "futures_history",
            description: "Get futures order history",
            input_schema: object_schema(
                vec![
                    (
                        "category",
                        enum_prop("Futures category", &["linear", "inverse"]),
                    ),
                    ("symbol", str_prop("Symbol filter")),
                    ("order_id", str_prop("Specific order ID")),
                    ("order_status", str_prop("Order status filter")),
                    ("start", int_prop("Start time in milliseconds")),
                    ("end", int_prop("End time in milliseconds")),
                    ("limit", int_prop("Page size")),
                    ("cursor", str_prop("Pagination cursor")),
                ],
                &[],
            ),
            service: "futures",
            dangerous: false,
        },
        McpTool {
            name: "futures_fills",
            description: "Get futures execution/fill history",
            input_schema: object_schema(
                vec![
                    (
                        "category",
                        enum_prop("Futures category", &["linear", "inverse"]),
                    ),
                    ("symbol", str_prop("Symbol filter")),
                    ("order_id", str_prop("Order ID filter")),
                    ("exec_type", str_prop("Execution type filter")),
                    ("start", int_prop("Start time in milliseconds")),
                    ("end", int_prop("End time in milliseconds")),
                    ("limit", int_prop("Page size")),
                    ("cursor", str_prop("Pagination cursor")),
                ],
                &[],
            ),
            service: "futures",
            dangerous: false,
        },
        McpTool {
            name: "futures_buy",
            description: "Place a futures buy order",
            input_schema: order_schema(),
            service: "futures",
            dangerous: true,
        },
        McpTool {
            name: "futures_sell",
            description: "Place a futures sell order",
            input_schema: order_schema(),
            service: "futures",
            dangerous: true,
        },
        McpTool {
            name: "futures_cancel",
            description: "Cancel a futures order",
            input_schema: object_schema(
                vec![
                    (
                        "category",
                        enum_prop("Futures category", &["linear", "inverse"]),
                    ),
                    ("symbol", str_prop("Symbol")),
                    ("order_id", str_prop("Specific order ID")),
                    ("order_link_id", str_prop("Specific client order ID")),
                ],
                &["symbol"],
            ),
            service: "futures",
            dangerous: true,
        },
        McpTool {
            name: "futures_cancel_all",
            description: "Cancel all open futures orders for a symbol or category",
            input_schema: object_schema(
                vec![
                    (
                        "category",
                        enum_prop("Futures category", &["linear", "inverse"]),
                    ),
                    ("symbol", str_prop("Symbol filter")),
                    ("base_coin", str_prop("Base coin filter")),
                    ("settle_coin", str_prop("Settle coin filter")),
                ],
                &[],
            ),
            service: "futures",
            dangerous: true,
        },
        McpTool {
            name: "futures_set_leverage",
            description: "Set futures leverage",
            input_schema: object_schema(
                vec![
                    (
                        "category",
                        enum_prop("Futures category", &["linear", "inverse"]),
                    ),
                    ("symbol", str_prop("Symbol")),
                    ("buy_leverage", num_prop("Buy/long leverage multiplier")),
                    ("sell_leverage", num_prop("Sell/short leverage multiplier")),
                ],
                &["symbol", "buy_leverage", "sell_leverage"],
            ),
            service: "futures",
            dangerous: true,
        },
    ]
}

// ---------------------------------------------------------------------------
// Paper trading tools (always safe)
// ---------------------------------------------------------------------------

fn paper_tools() -> Vec<McpTool> {
    vec![
        McpTool {
            name: "paper_init",
            description: "Initialise a paper trading account with a configurable fee/slippage model. Use --force to overwrite an existing journal.",
            input_schema: object_schema(
                vec![
                    ("usdt", num_prop("Starting balance in settle-coin (default 10000)")),
                    ("settle_coin", str_prop("Settlement currency (default: USDT)")),
                    ("taker_fee_bps", int_prop("Taker fee basis points for market fills (default: 6 = 0.06%)")),
                    ("maker_fee_bps", int_prop("Maker fee basis points for limit fills (default: 1 = 0.01%)")),
                    ("slippage_bps", int_prop("One-way market order slippage in bps (default: 5 = 0.05%)")),
                    ("force", bool_prop("Overwrite existing journal without error")),
                ],
                &[],
            ),
            service: "paper",
            dangerous: false,
        },
        McpTool {
            name: "paper_buy",
            description: "Place a paper buy order at market or limit price",
            input_schema: object_schema(
                vec![
                    ("category", category_prop()),
                    ("symbol", str_prop("Symbol, e.g. BTCUSDT")),
                    ("qty", num_prop("Quantity to buy")),
                    ("price", num_prop("Limit price (omit for market fill)")),
                ],
                &["symbol", "qty"],
            ),
            service: "paper",
            dangerous: false,
        },
        McpTool {
            name: "paper_sell",
            description: "Place a paper sell order at market or limit price",
            input_schema: object_schema(
                vec![
                    ("category", category_prop()),
                    ("symbol", str_prop("Symbol, e.g. BTCUSDT")),
                    ("qty", num_prop("Quantity to sell")),
                    ("price", num_prop("Limit price (omit for market fill)")),
                ],
                &["symbol", "qty"],
            ),
            service: "paper",
            dangerous: false,
        },
        McpTool {
            name: "paper_status",
            description: "Show paper trading account summary: balance, positions, P&L",
            input_schema: object_schema(vec![], &[]),
            service: "paper",
            dangerous: false,
        },
        McpTool {
            name: "paper_balance",
            description: "Show paper trading USDT balance",
            input_schema: object_schema(vec![], &[]),
            service: "paper",
            dangerous: false,
        },
        McpTool {
            name: "paper_positions",
            description: "List open paper trading positions",
            input_schema: object_schema(vec![], &[]),
            service: "paper",
            dangerous: false,
        },
        McpTool {
            name: "paper_orders",
            description: "List pending paper limit orders (checks for fills first)",
            input_schema: object_schema(vec![], &[]),
            service: "paper",
            dangerous: false,
        },
        McpTool {
            name: "paper_cancel",
            description: "Cancel a specific pending paper limit order",
            input_schema: object_schema(
                vec![("order_id", int_prop("Order ID to cancel"))],
                &["order_id"],
            ),
            service: "paper",
            dangerous: false,
        },
        McpTool {
            name: "paper_cancel_all",
            description: "Cancel all pending paper limit orders",
            input_schema: object_schema(vec![], &[]),
            service: "paper",
            dangerous: false,
        },
        McpTool {
            name: "paper_history",
            description: "Show paper trading filled trade history (includes fee per trade)",
            input_schema: object_schema(vec![], &[]),
            service: "paper",
            dangerous: false,
        },
        McpTool {
            name: "paper_cancelled",
            description: "Show cancelled paper limit order history",
            input_schema: object_schema(vec![], &[]),
            service: "paper",
            dangerous: false,
        },
        McpTool {
            name: "paper_reset",
            description: "Reset paper trading account (wipe all positions and history)",
            input_schema: object_schema(vec![], &[]),
            service: "paper",
            dangerous: false,
        },
    ]
}

// ---------------------------------------------------------------------------
// Convert tools
// ---------------------------------------------------------------------------

fn convert_tools() -> Vec<McpTool> {
    vec![
        McpTool {
            name: "convert_coins",
            description: "List coins available for conversion and their supported pairs",
            input_schema: object_schema(
                vec![
                    (
                        "account_type",
                        enum_prop("Account type", &["UNIFIED", "SPOT", "CONTRACT", "FUND"]),
                    ),
                    ("coin", str_prop("Filter by coin name, e.g. BTC")),
                    (
                        "side",
                        int_prop("0 = all, 1 = from-coin list, 2 = to-coin list"),
                    ),
                ],
                &[],
            ),
            service: "convert",
            dangerous: false,
        },
        McpTool {
            name: "convert_quote",
            description: "Request a conversion quote. Returns a quoteTxId needed to execute.",
            input_schema: object_schema(
                vec![
                    (
                        "account_type",
                        enum_prop(
                            "Account type performing the conversion",
                            &["UNIFIED", "SPOT", "CONTRACT", "FUND"],
                        ),
                    ),
                    ("from_coin", str_prop("Coin to convert from, e.g. BTC")),
                    ("to_coin", str_prop("Coin to convert to, e.g. USDT")),
                    (
                        "from_amount",
                        str_prop(
                            "Amount of from-coin to convert (mutually exclusive with to_amount)",
                        ),
                    ),
                    (
                        "to_amount",
                        str_prop("Desired amount of to-coin (mutually exclusive with from_amount)"),
                    ),
                    (
                        "dry_run",
                        bool_prop("Preview the request without calling the API"),
                    ),
                ],
                &["from_coin", "to_coin"],
            ),
            service: "convert",
            dangerous: true,
        },
        McpTool {
            name: "convert_execute",
            description: "Execute a previously obtained conversion quote. Irreversible.",
            input_schema: object_schema(
                vec![(
                    "quote_tx_id",
                    str_prop("Quote transaction ID from convert_quote"),
                )],
                &["quote_tx_id"],
            ),
            service: "convert",
            dangerous: true,
        },
        McpTool {
            name: "convert_status",
            description: "Check the status of a conversion by quoteTxId",
            input_schema: object_schema(
                vec![
                    ("quote_tx_id", str_prop("Quote transaction ID to check")),
                    (
                        "account_type",
                        enum_prop("Account type", &["UNIFIED", "SPOT", "CONTRACT", "FUND"]),
                    ),
                ],
                &["quote_tx_id"],
            ),
            service: "convert",
            dangerous: false,
        },
        McpTool {
            name: "convert_history",
            description: "Get coin conversion history",
            input_schema: object_schema(
                vec![
                    (
                        "account_type",
                        enum_prop("Account type", &["UNIFIED", "SPOT", "CONTRACT", "FUND"]),
                    ),
                    ("coin", str_prop("Filter by coin, e.g. USDT")),
                    ("start", int_prop("Start time in milliseconds")),
                    ("end", int_prop("End time in milliseconds")),
                    ("index", int_prop("Page index (0-based)")),
                    ("limit", int_prop("Page size (default 20, max 100)")),
                ],
                &[],
            ),
            service: "convert",
            dangerous: false,
        },
    ]
}

// ---------------------------------------------------------------------------
// Margin tools
// ---------------------------------------------------------------------------

fn margin_tools() -> Vec<McpTool> {
    vec![
        McpTool {
            name: "margin_vip_data",
            description: "Get spot margin VIP borrow and leverage data",
            input_schema: object_schema(
                vec![
                    ("vip_level", str_prop("VIP level label, e.g. No VIP")),
                    ("currency", str_prop("Coin filter, e.g. BTC")),
                ],
                &[],
            ),
            service: "margin",
            dangerous: false,
        },
        McpTool {
            name: "margin_status",
            description: "Get current unified account spot margin state and leverage",
            input_schema: object_schema(vec![], &[]),
            service: "margin",
            dangerous: false,
        },
        McpTool {
            name: "margin_toggle",
            description: "Enable or disable unified account spot margin trading",
            input_schema: object_schema(
                vec![("mode", enum_prop("Desired mode", &["on", "off"]))],
                &["mode"],
            ),
            service: "margin",
            dangerous: true,
        },
        McpTool {
            name: "margin_set_leverage",
            description: "Set unified account spot margin leverage",
            input_schema: object_schema(
                vec![
                    ("leverage", str_prop("Leverage, typically 2-10")),
                    (
                        "currency",
                        str_prop("Optional coin-specific leverage setting"),
                    ),
                ],
                &["leverage"],
            ),
            service: "margin",
            dangerous: true,
        },
    ]
}

// ---------------------------------------------------------------------------
// Auth tools
// ---------------------------------------------------------------------------

fn auth_tools() -> Vec<McpTool> {
    vec![
        McpTool {
            name: "auth_test",
            description: "Test that the configured API credentials are valid",
            input_schema: object_schema(vec![], &[]),
            service: "auth",
            dangerous: false,
        },
        McpTool {
            name: "auth_show",
            description: "Show the current credential source and masked API key",
            input_schema: object_schema(vec![], &[]),
            service: "auth",
            dangerous: false,
        },
        McpTool {
            name: "auth_permissions",
            description: "Show active permissions and info for the current API key",
            input_schema: object_schema(vec![], &[]),
            service: "auth",
            dangerous: false,
        },
    ]
}

// ---------------------------------------------------------------------------
// WebSocket tools
// ---------------------------------------------------------------------------

fn ws_tools() -> Vec<McpTool> {
    vec![McpTool {
        name: "ws_notifications",
        description: "Stream all private notifications (orders, positions, executions, wallet)",
        input_schema: object_schema(vec![], &[]),
        service: "ws",
        dangerous: false,
    }]
}

// ---------------------------------------------------------------------------
// ---------------------------------------------------------------------------
// Futures paper tools
// ---------------------------------------------------------------------------

fn futures_paper_tools() -> Vec<McpTool> {
    vec![
        McpTool {
            name: "futures_paper_init",
            description: "Initialize a futures paper trading account with starting collateral. No auth required. Use --force to overwrite.",
            input_schema: object_schema(
                vec![
                    ("balance", num_prop("Starting collateral (default: 10000)")),
                    ("currency", str_prop("Collateral currency (default: USDT)")),
                    ("fee_rate", num_prop("Taker fee rate as decimal (default: 0.00055)")),
                    ("force", bool_prop("Overwrite existing account")),
                ],
                &[],
            ),
            service: "futures-paper",
            dangerous: false,
        },
        McpTool {
            name: "futures_paper_reset",
            description: "Reset futures paper account to initial state. Wipes all positions, orders, and fill history.",
            input_schema: object_schema(
                vec![
                    ("balance", num_prop("New starting collateral (default: keep current)")),
                    ("currency", str_prop("New collateral currency (default: keep current)")),
                    ("fee_rate", num_prop("New fee rate (default: keep current)")),
                ],
                &[],
            ),
            service: "futures-paper",
            dangerous: false,
        },
        McpTool {
            name: "futures_paper_balance",
            description: "Show futures paper collateral balance and margin summary.",
            input_schema: object_schema(vec![], &[]),
            service: "futures-paper",
            dangerous: false,
        },
        McpTool {
            name: "futures_paper_status",
            description: "Show full futures paper account summary: equity, PnL, positions, margin usage.",
            input_schema: object_schema(vec![], &[]),
            service: "futures-paper",
            dangerous: false,
        },
        McpTool {
            name: "futures_paper_buy",
            description: "Place a futures paper long (buy) order. Supports market, limit, post, stop, take-profit, ioc, trailing-stop, fok order types.",
            input_schema: object_schema(
                vec![
                    ("symbol", str_prop("Futures symbol, e.g. BTCUSDT")),
                    ("size", str_prop("Order size in base asset")),
                    ("type", str_prop("Order type: limit, market, post, stop, take-profit, ioc, trailing-stop, fok (default: limit)")),
                    ("price", str_prop("Limit price (required for limit/post/ioc/fok)")),
                    ("stop_price", str_prop("Stop/trigger price (required for stop/take-profit/trailing-stop)")),
                    ("trigger_signal", str_prop("Trigger signal: mark, index, or last")),
                    ("client_order_id", str_prop("Client order ID")),
                    ("reduce_only", bool_prop("Reduce-only flag")),
                    ("leverage", str_prop("Leverage override (1–100)")),
                    ("trailing_stop_max_deviation", str_prop("Trailing stop max deviation")),
                    ("trailing_stop_deviation_unit", str_prop("Trailing stop unit: percent or quote_currency")),
                    ("category", str_prop("Asset category (default: linear)")),
                ],
                &["symbol", "size"],
            ),
            service: "futures-paper",
            dangerous: false,
        },
        McpTool {
            name: "futures_paper_sell",
            description: "Place a futures paper short (sell) order. Supports all 8 order types.",
            input_schema: object_schema(
                vec![
                    ("symbol", str_prop("Futures symbol, e.g. BTCUSDT")),
                    ("size", str_prop("Order size in base asset")),
                    ("type", str_prop("Order type: limit, market, post, stop, take-profit, ioc, trailing-stop, fok (default: limit)")),
                    ("price", str_prop("Limit price")),
                    ("stop_price", str_prop("Stop/trigger price")),
                    ("trigger_signal", str_prop("Trigger signal: mark, index, or last")),
                    ("client_order_id", str_prop("Client order ID")),
                    ("reduce_only", bool_prop("Reduce-only flag")),
                    ("leverage", str_prop("Leverage override (1–100)")),
                    ("trailing_stop_max_deviation", str_prop("Trailing stop max deviation")),
                    ("trailing_stop_deviation_unit", str_prop("Trailing stop unit: percent or quote_currency")),
                    ("category", str_prop("Asset category (default: linear)")),
                ],
                &["symbol", "size"],
            ),
            service: "futures-paper",
            dangerous: false,
        },
        McpTool {
            name: "futures_paper_orders",
            description: "Show open futures paper orders. Reconciles against current market prices first.",
            input_schema: object_schema(
                vec![("category", str_prop("Asset category (default: linear)"))],
                &[],
            ),
            service: "futures-paper",
            dangerous: false,
        },
        McpTool {
            name: "futures_paper_order_status",
            description: "Get the status of a specific open futures paper order by ID.",
            input_schema: object_schema(
                vec![("order_id", str_prop("Order ID to query"))],
                &["order_id"],
            ),
            service: "futures-paper",
            dangerous: false,
        },
        McpTool {
            name: "futures_paper_edit_order",
            description: "Edit a resting futures paper order (size, price, or stop_price).",
            input_schema: object_schema(
                vec![
                    ("order_id", str_prop("Order ID to edit")),
                    ("size", str_prop("New size")),
                    ("price", str_prop("New limit price")),
                    ("stop_price", str_prop("New stop price")),
                ],
                &["order_id"],
            ),
            service: "futures-paper",
            dangerous: false,
        },
        McpTool {
            name: "futures_paper_cancel",
            description: "Cancel a specific open futures paper order.",
            input_schema: object_schema(
                vec![
                    ("order_id", str_prop("Exchange order ID")),
                    ("cli_ord_id", str_prop("Client order ID")),
                ],
                &[],
            ),
            service: "futures-paper",
            dangerous: false,
        },
        McpTool {
            name: "futures_paper_cancel_all",
            description: "Cancel all open futures paper orders, optionally filtered by symbol.",
            input_schema: object_schema(
                vec![("symbol", str_prop("Filter by symbol (optional)"))],
                &[],
            ),
            service: "futures-paper",
            dangerous: false,
        },
        McpTool {
            name: "futures_paper_batch_order",
            description: "Place a batch of futures paper orders from a JSON array string.",
            input_schema: object_schema(
                vec![("orders_json", str_prop("JSON array of order objects, or @path to JSON file"))],
                &["orders_json"],
            ),
            service: "futures-paper",
            dangerous: false,
        },
        McpTool {
            name: "futures_paper_positions",
            description: "Show open futures paper positions with unrealized PnL and liquidation prices.",
            input_schema: object_schema(
                vec![("category", str_prop("Asset category (default: linear)"))],
                &[],
            ),
            service: "futures-paper",
            dangerous: false,
        },
        McpTool {
            name: "futures_paper_fills",
            description: "Show futures paper fill history.",
            input_schema: object_schema(vec![], &[]),
            service: "futures-paper",
            dangerous: false,
        },
        McpTool {
            name: "futures_paper_history",
            description: "Show futures paper account history (realized PnL events, funding payments, liquidations).",
            input_schema: object_schema(vec![], &[]),
            service: "futures-paper",
            dangerous: false,
        },
        McpTool {
            name: "futures_paper_leverage",
            description: "Get current leverage preferences for futures paper trading.",
            input_schema: object_schema(
                vec![("symbol", str_prop("Filter by symbol (optional)"))],
                &[],
            ),
            service: "futures-paper",
            dangerous: false,
        },
        McpTool {
            name: "futures_paper_set_leverage",
            description: "Set leverage preference for a symbol in futures paper trading.",
            input_schema: object_schema(
                vec![
                    ("symbol", str_prop("Futures symbol, e.g. BTCUSDT")),
                    ("leverage", str_prop("Leverage value (1–100)")),
                ],
                &["symbol", "leverage"],
            ),
            service: "futures-paper",
            dangerous: false,
        },
    ]
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

pub fn all_tools() -> Vec<McpTool> {
    let mut tools = Vec::new();
    tools.extend(market_tools());
    tools.extend(account_tools());
    tools.extend(earn_tools());
    tools.extend(trade_tools());
    tools.extend(position_tools());
    tools.extend(asset_tools());
    tools.extend(funding_tools());
    tools.extend(convert_tools());
    tools.extend(margin_tools());
    tools.extend(reports_tools());
    tools.extend(subaccount_tools());
    tools.extend(futures_tools());
    tools.extend(paper_tools());
    tools.extend(futures_paper_tools());
    tools.extend(auth_tools());
    tools.extend(ws_tools());
    tools
}

pub fn runtime_tool_catalog() -> Value {
    let tools: Vec<Value> = all_tools()
        .into_iter()
        .map(|tool| {
            json!({
                "name": tool.name,
                "description": tool.description,
                "service": tool.service,
                "dangerous": tool.dangerous,
                "input_schema": tool.input_schema,
            })
        })
        .collect();

    json!({
        "schema_version": "1.0",
        "transport": "stdio",
        "default_services": DEFAULT_SERVICES.split(',').collect::<Vec<_>>(),
        "valid_services": VALID_SERVICES,
        "tools": tools,
    })
}

// ---------------------------------------------------------------------------
// Tool name → CLI args converter
// ---------------------------------------------------------------------------

/// Convert an MCP tool call (name + JSON params) to bybit CLI arguments.
/// Returns `None` for unknown tool names.
pub fn tool_to_args(name: &str, p: &Value) -> Option<Vec<String>> {
    let gs = |key: &str| p.get(key).and_then(Value::as_str).map(String::from);
    let gn = |key: &str| p.get(key).and_then(Value::as_f64).map(|v| v.to_string());
    let gi = |key: &str| {
        p.get(key)
            .and_then(|v| v.as_i64().or_else(|| v.as_f64().map(|f| f as i64)))
            .map(|v| v.to_string())
    };
    let gb = |key: &str| p.get(key).and_then(Value::as_bool).unwrap_or(false);

    // Helper: push --flag value if value is Some
    macro_rules! opt {
        ($args:expr, $flag:literal, $val:expr) => {
            if let Some(v) = $val {
                $args.push($flag.to_string());
                $args.push(v);
            }
        };
    }

    let mut args: Vec<String> = Vec::new();

    match name {
        // ----------------------------------------------------------------
        // market
        // ----------------------------------------------------------------
        "market_server_time" => {
            args.extend(["market", "server-time"].map(String::from));
        }
        "market_tickers" => {
            args.extend(["market", "tickers"].map(String::from));
            opt!(args, "--category", gs("category"));
            opt!(args, "--symbol", gs("symbol"));
            opt!(args, "--base-coin", gs("base_coin"));
            opt!(args, "--exp-date", gs("exp_date"));
        }
        "market_orderbook" => {
            args.extend(["market", "orderbook"].map(String::from));
            opt!(args, "--category", gs("category"));
            opt!(args, "--symbol", gs("symbol"));
            opt!(args, "--limit", gi("limit"));
        }
        "market_kline" => {
            args.extend(["market", "kline"].map(String::from));
            opt!(args, "--category", gs("category"));
            opt!(args, "--symbol", gs("symbol"));
            opt!(args, "--interval", gs("interval"));
            opt!(args, "--start", gi("start"));
            opt!(args, "--end", gi("end"));
            opt!(args, "--limit", gi("limit"));
        }
        "market_funding_rate" => {
            args.extend(["market", "funding-rate"].map(String::from));
            opt!(args, "--category", gs("category"));
            opt!(args, "--symbol", gs("symbol"));
            opt!(args, "--start", gi("start"));
            opt!(args, "--end", gi("end"));
            opt!(args, "--limit", gi("limit"));
        }
        "market_trades" => {
            args.extend(["market", "trades"].map(String::from));
            opt!(args, "--category", gs("category"));
            opt!(args, "--symbol", gs("symbol"));
            opt!(args, "--limit", gi("limit"));
            opt!(args, "--base-coin", gs("base_coin"));
            opt!(args, "--option-type", gs("option_type"));
        }
        "market_instruments" => {
            args.extend(["market", "instruments"].map(String::from));
            opt!(args, "--category", gs("category"));
            opt!(args, "--symbol", gs("symbol"));
            opt!(args, "--status", gs("status"));
            opt!(args, "--base-coin", gs("base_coin"));
            opt!(args, "--limit", gi("limit"));
            opt!(args, "--cursor", gs("cursor"));
        }
        "market_open_interest" => {
            args.extend(["market", "open-interest"].map(String::from));
            opt!(args, "--category", gs("category"));
            opt!(args, "--symbol", gs("symbol"));
            opt!(args, "--interval-time", gs("interval_time"));
            opt!(args, "--start", gi("start"));
            opt!(args, "--end", gi("end"));
            opt!(args, "--limit", gi("limit"));
        }
        "market_risk_limit" => {
            args.extend(["market", "risk-limit"].map(String::from));
            opt!(args, "--category", gs("category"));
            opt!(args, "--symbol", gs("symbol"));
        }
        "market_insurance" => {
            args.extend(["market", "insurance"].map(String::from));
            opt!(args, "--coin", gs("coin"));
        }
        "market_delivery_price" => {
            args.extend(["market", "delivery-price"].map(String::from));
            opt!(args, "--category", gs("category"));
            opt!(args, "--symbol", gs("symbol"));
            opt!(args, "--base-coin", gs("base_coin"));
            opt!(args, "--limit", gi("limit"));
            opt!(args, "--cursor", gs("cursor"));
        }
        "market_ls_ratio" => {
            args.extend(["market", "ls-ratio"].map(String::from));
            opt!(args, "--category", gs("category"));
            opt!(args, "--symbol", gs("symbol"));
            opt!(args, "--period", gs("period"));
            opt!(args, "--limit", gi("limit"));
        }
        "market_spread" => {
            args.extend(["market", "spread"].map(String::from));
            opt!(args, "--category", gs("category"));
            opt!(args, "--symbol", gs("symbol"));
        }

        // ----------------------------------------------------------------
        // account
        // ----------------------------------------------------------------
        "account_balance" => {
            args.extend(["account", "balance"].map(String::from));
            opt!(args, "--account-type", gs("account_type"));
            opt!(args, "--coin", gs("coin"));
        }
        "account_info" => {
            args.extend(["account", "info"].map(String::from));
        }
        "account_fee_rate" => {
            args.extend(["account", "fee-rate"].map(String::from));
            opt!(args, "--category", gs("category"));
            opt!(args, "--symbol", gs("symbol"));
            opt!(args, "--base-coin", gs("base_coin"));
        }
        "account_transaction_log" => {
            args.extend(["account", "transaction-log"].map(String::from));
            opt!(args, "--account-type", gs("account_type"));
            opt!(args, "--category", gs("category"));
            opt!(args, "--currency", gs("currency"));
            opt!(args, "--start", gi("start"));
            opt!(args, "--end", gi("end"));
            opt!(args, "--limit", gi("limit"));
            opt!(args, "--cursor", gs("cursor"));
        }
        "account_borrow_history" => {
            args.extend(["account", "borrow-history"].map(String::from));
            opt!(args, "--currency", gs("currency"));
            opt!(args, "--start", gi("start"));
            opt!(args, "--end", gi("end"));
            opt!(args, "--limit", gi("limit"));
            opt!(args, "--cursor", gs("cursor"));
        }
        "account_collateral_info" => {
            args.extend(["account", "collateral-info"].map(String::from));
            opt!(args, "--currency", gs("currency"));
        }
        "account_volume" => {
            args.extend(["account", "volume"].map(String::from));
            opt!(args, "--category", gs("category"));
            opt!(args, "--symbol", gs("symbol"));
            opt!(args, "--base-coin", gs("base_coin"));
            opt!(args, "--days", gi("days"));
        }
        "account_set_usdc_settlement" => {
            args.extend(["account", "set-usdc-settlement"].map(String::from));
            opt!(args, "--coin", gs("coin"));
        }
        "account_adl_alert" => {
            args.extend(["account", "adl-alert"].map(String::from));
            opt!(args, "--category", gs("category"));
            opt!(args, "--symbol", gs("symbol"));
        }
        "account_borrow" => {
            args.extend(["account", "borrow"].map(String::from));
            opt!(args, "--coin", gs("coin"));
            opt!(args, "--amount", gs("amount"));
        }
        "account_repay" => {
            args.extend(["account", "repay"].map(String::from));
            opt!(args, "--coin", gs("coin"));
            opt!(args, "--amount", gs("amount"));
        }
        "account_quick_repay" => {
            args.extend(["account", "quick-repay"].map(String::from));
            opt!(args, "--coin", gs("coin"));
        }

        // ----------------------------------------------------------------
        // earn
        // ----------------------------------------------------------------
        "earn_product" => {
            args.extend(["earn", "product"].map(String::from));
            opt!(args, "--product-type", gs("product_type"));
            opt!(args, "--coin", gs("coin"));
        }
        "earn_positions" => {
            args.extend(["earn", "positions"].map(String::from));
            opt!(args, "--coin", gs("coin"));
            opt!(args, "--order-id", gs("order_id"));
        }
        "earn_stake" => {
            args.extend(["earn", "stake"].map(String::from));
            opt!(args, "--product-id", gs("product_id"));
            opt!(args, "--coin", gs("coin"));
            opt!(args, "--amount", gs("amount"));
        }
        "earn_unstake" => {
            args.extend(["earn", "unstake"].map(String::from));
            opt!(args, "--order-id", gs("order_id"));
            opt!(args, "--coin", gs("coin"));
            opt!(args, "--amount", gs("amount"));
        }
        "earn_history" => {
            args.extend(["earn", "history"].map(String::from));
            opt!(args, "--coin", gs("coin"));
            opt!(args, "--start", gi("start"));
            opt!(args, "--end", gi("end"));
            opt!(args, "--limit", gi("limit"));
            opt!(args, "--cursor", gs("cursor"));
        }

        // ----------------------------------------------------------------
        // trade
        // ----------------------------------------------------------------
        "trade_buy" | "trade_validate_buy" => {
            args.extend(["trade", "buy"].map(String::from));
            opt!(args, "--category", gs("category"));
            opt!(args, "--symbol", gs("symbol"));
            opt!(args, "--qty", gs("qty"));
            opt!(args, "--price", gs("price"));
            opt!(args, "--order-type", gs("order_type"));
            opt!(args, "--time-in-force", gs("time_in_force"));
            opt!(args, "--take-profit", gs("take_profit"));
            opt!(args, "--stop-loss", gs("stop_loss"));
            opt!(args, "--tp-limit-price", gs("tp_limit_price"));
            opt!(args, "--sl-limit-price", gs("sl_limit_price"));
            opt!(args, "--tp-trigger-by", gs("tp_trigger_by"));
            opt!(args, "--sl-trigger-by", gs("sl_trigger_by"));
            opt!(args, "--display-qty", gs("display_qty"));
            opt!(args, "--trigger-price", gs("trigger_price"));
            opt!(args, "--order-link-id", gs("order_link_id"));
            opt!(args, "--position-idx", gi("position_idx"));
            if gb("reduce_only") {
                args.push("--reduce-only".to_string());
            }
            if gb("post_only") {
                args.push("--post-only".to_string());
            }
            if name == "trade_validate_buy" {
                args.push("--validate".to_string());
            }
        }
        "trade_sell" | "trade_validate_sell" => {
            args.extend(["trade", "sell"].map(String::from));
            opt!(args, "--category", gs("category"));
            opt!(args, "--symbol", gs("symbol"));
            opt!(args, "--qty", gs("qty"));
            opt!(args, "--price", gs("price"));
            opt!(args, "--order-type", gs("order_type"));
            opt!(args, "--time-in-force", gs("time_in_force"));
            opt!(args, "--take-profit", gs("take_profit"));
            opt!(args, "--stop-loss", gs("stop_loss"));
            opt!(args, "--tp-limit-price", gs("tp_limit_price"));
            opt!(args, "--sl-limit-price", gs("sl_limit_price"));
            opt!(args, "--tp-trigger-by", gs("tp_trigger_by"));
            opt!(args, "--sl-trigger-by", gs("sl_trigger_by"));
            opt!(args, "--display-qty", gs("display_qty"));
            opt!(args, "--trigger-price", gs("trigger_price"));
            opt!(args, "--order-link-id", gs("order_link_id"));
            opt!(args, "--position-idx", gi("position_idx"));
            if gb("reduce_only") {
                args.push("--reduce-only".to_string());
            }
            if gb("post_only") {
                args.push("--post-only".to_string());
            }
            if name == "trade_validate_sell" {
                args.push("--validate".to_string());
            }
        }
        "trade_amend" => {
            args.extend(["trade", "amend"].map(String::from));
            opt!(args, "--category", gs("category"));
            opt!(args, "--symbol", gs("symbol"));
            opt!(args, "--order-id", gs("order_id"));
            opt!(args, "--order-link-id", gs("order_link_id"));
            opt!(args, "--qty", gs("qty"));
            opt!(args, "--price", gs("price"));
            opt!(args, "--take-profit", gs("take_profit"));
            opt!(args, "--stop-loss", gs("stop_loss"));
            opt!(args, "--trigger-price", gs("trigger_price"));
        }
        "trade_cancel" => {
            args.extend(["trade", "cancel"].map(String::from));
            opt!(args, "--category", gs("category"));
            opt!(args, "--symbol", gs("symbol"));
            opt!(args, "--order-id", gs("order_id"));
            opt!(args, "--order-link-id", gs("order_link_id"));
        }
        "trade_cancel_all" => {
            args.extend(["trade", "cancel-all"].map(String::from));
            opt!(args, "--category", gs("category"));
            opt!(args, "--symbol", gs("symbol"));
            opt!(args, "--base-coin", gs("base_coin"));
            opt!(args, "--settle-coin", gs("settle_coin"));
        }
        "trade_open_orders" => {
            args.extend(["trade", "open-orders"].map(String::from));
            opt!(args, "--category", gs("category"));
            opt!(args, "--symbol", gs("symbol"));
            opt!(args, "--base-coin", gs("base_coin"));
            opt!(args, "--order-id", gs("order_id"));
            opt!(args, "--order-link-id", gs("order_link_id"));
            opt!(args, "--limit", gi("limit"));
            opt!(args, "--cursor", gs("cursor"));
        }
        "trade_history" => {
            args.extend(["trade", "history"].map(String::from));
            opt!(args, "--category", gs("category"));
            opt!(args, "--symbol", gs("symbol"));
            opt!(args, "--order-id", gs("order_id"));
            opt!(args, "--order-status", gs("order_status"));
            opt!(args, "--start", gi("start"));
            opt!(args, "--end", gi("end"));
            opt!(args, "--limit", gi("limit"));
            opt!(args, "--cursor", gs("cursor"));
        }
        "trade_fills" => {
            args.extend(["trade", "fills"].map(String::from));
            opt!(args, "--category", gs("category"));
            opt!(args, "--symbol", gs("symbol"));
            opt!(args, "--order-id", gs("order_id"));
            opt!(args, "--start", gi("start"));
            opt!(args, "--end", gi("end"));
            opt!(args, "--exec-type", gs("exec_type"));
            opt!(args, "--limit", gi("limit"));
            opt!(args, "--cursor", gs("cursor"));
        }
        "trade_cancel_after" => {
            args.extend(["trade", "cancel-after"].map(String::from));
            if let Some(s) = gi("seconds") {
                args.push(s);
            } else {
                args.push("0".to_string());
            }
        }
        "trade_dcp_info" => {
            args.extend(["trade", "dcp-info"].map(String::from));
        }
        "trade_batch_place" => {
            args.extend(["trade", "batch-place"].map(String::from));
            opt!(args, "--category", gs("category"));
            opt!(args, "--orders", gs("orders"));
        }
        "trade_batch_amend" => {
            args.extend(["trade", "batch-amend"].map(String::from));
            opt!(args, "--category", gs("category"));
            opt!(args, "--orders", gs("orders"));
        }
        "trade_batch_cancel" => {
            args.extend(["trade", "batch-cancel"].map(String::from));
            opt!(args, "--category", gs("category"));
            opt!(args, "--orders", gs("orders"));
        }

        // ----------------------------------------------------------------
        // position
        // ----------------------------------------------------------------
        "position_list" => {
            args.extend(["position", "list"].map(String::from));
            opt!(args, "--category", gs("category"));
            opt!(args, "--symbol", gs("symbol"));
            opt!(args, "--base-coin", gs("base_coin"));
            opt!(args, "--settle-coin", gs("settle_coin"));
            opt!(args, "--limit", gi("limit"));
            opt!(args, "--cursor", gs("cursor"));
        }
        "position_set_leverage" => {
            args.extend(["position", "set-leverage"].map(String::from));
            opt!(args, "--category", gs("category"));
            opt!(args, "--symbol", gs("symbol"));
            opt!(args, "--buy-leverage", gn("buy_leverage"));
            opt!(args, "--sell-leverage", gn("sell_leverage"));
        }
        "position_set_tpsl" => {
            args.extend(["position", "set-tpsl"].map(String::from));
            opt!(args, "--category", gs("category"));
            opt!(args, "--symbol", gs("symbol"));
            opt!(args, "--take-profit", gs("take_profit"));
            opt!(args, "--stop-loss", gs("stop_loss"));
            opt!(args, "--trailing-stop", gs("trailing_stop"));
            opt!(args, "--position-idx", gi("position_idx"));
        }
        "position_closed_pnl" => {
            args.extend(["position", "closed-pnl"].map(String::from));
            opt!(args, "--category", gs("category"));
            opt!(args, "--symbol", gs("symbol"));
            opt!(args, "--start", gi("start"));
            opt!(args, "--end", gi("end"));
            opt!(args, "--limit", gi("limit"));
            opt!(args, "--cursor", gs("cursor"));
        }
        "position_switch_mode" => {
            args.extend(["position", "switch-mode"].map(String::from));
            opt!(args, "--category", gs("category"));
            opt!(args, "--symbol", gs("symbol"));
            opt!(args, "--coin", gs("coin"));
            opt!(args, "--mode", gi("mode"));
        }
        "position_add_margin" => {
            args.extend(["position", "add-margin"].map(String::from));
            opt!(args, "--category", gs("category"));
            opt!(args, "--symbol", gs("symbol"));
            opt!(args, "--margin", gs("margin"));
            opt!(args, "--position-idx", gi("position_idx"));
        }
        "position_flatten" => {
            args.extend(["position", "flatten"].map(String::from));
            opt!(args, "--category", gs("category"));
            opt!(args, "--symbol", gs("symbol"));
        }

        // ----------------------------------------------------------------
        // asset
        // ----------------------------------------------------------------
        "asset_coin_info" => {
            args.extend(["asset", "coin-info"].map(String::from));
            opt!(args, "--coin", gs("coin"));
        }
        "asset_balance" => {
            args.extend(["asset", "balance"].map(String::from));
            opt!(args, "--account-type", gs("account_type"));
            opt!(args, "--coin", gs("coin"));
        }
        "asset_all_balance" => {
            args.extend(["asset", "all-balance"].map(String::from));
            opt!(args, "--member-id", gs("member_id"));
            opt!(args, "--account-type", gs("account_type"));
            opt!(args, "--coin", gs("coin"));
            if gb("with_bonus") {
                args.push("--with-bonus".to_string());
            }
        }
        "asset_transferable" => {
            args.extend(["asset", "transferable"].map(String::from));
            opt!(args, "--from-account-type", gs("from_account_type"));
            opt!(args, "--to-account-type", gs("to_account_type"));
            opt!(args, "--coin", gs("coin"));
        }
        "asset_deposit_address" => {
            args.extend(["asset", "deposit-address"].map(String::from));
            opt!(args, "--coin", gs("coin"));
            opt!(args, "--chain-type", gs("chain_type"));
        }
        "asset_withdrawal_methods" => {
            args.extend(["asset", "withdrawal-methods"].map(String::from));
            opt!(args, "--coin", gs("coin"));
        }
        "asset_deposit_history" => {
            args.extend(["asset", "deposit-history"].map(String::from));
            opt!(args, "--coin", gs("coin"));
            opt!(args, "--start", gi("start"));
            opt!(args, "--end", gi("end"));
            opt!(args, "--limit", gi("limit"));
            opt!(args, "--cursor", gs("cursor"));
        }
        "asset_withdraw_history" => {
            args.extend(["asset", "withdraw-history"].map(String::from));
            opt!(args, "--coin", gs("coin"));
            opt!(args, "--withdraw-id", gs("withdraw_id"));
            opt!(args, "--start", gi("start"));
            opt!(args, "--end", gi("end"));
            opt!(args, "--limit", gi("limit"));
            opt!(args, "--cursor", gs("cursor"));
        }
        "asset_transfer" => {
            args.extend(["asset", "transfer"].map(String::from));
            opt!(args, "--coin", gs("coin"));
            opt!(args, "--amount", gs("amount"));
            opt!(args, "--from-account-type", gs("from_account_type"));
            opt!(args, "--to-account-type", gs("to_account_type"));
            opt!(args, "--transfer-id", gs("transfer_id"));
        }
        "asset_withdraw" => {
            args.extend(["asset", "withdraw"].map(String::from));
            opt!(args, "--coin", gs("coin"));
            opt!(args, "--chain", gs("chain"));
            opt!(args, "--address", gs("address"));
            opt!(args, "--tag", gs("tag"));
            opt!(args, "--amount", gs("amount"));
            opt!(args, "--timestamp", gi("timestamp"));
            opt!(args, "--account-type", gs("account_type"));
        }
        "asset_transfer_history" => {
            args.extend(["asset", "transfer-history"].map(String::from));
            opt!(args, "--coin", gs("coin"));
            opt!(args, "--transfer-id", gs("transfer_id"));
            opt!(args, "--start", gi("start"));
            opt!(args, "--end", gi("end"));
            opt!(args, "--limit", gi("limit"));
            opt!(args, "--cursor", gs("cursor"));
        }

        // ----------------------------------------------------------------
        // funding
        // ----------------------------------------------------------------
        "funding_coin_info" => {
            args.extend(["funding", "coin-info"].map(String::from));
            opt!(args, "--coin", gs("coin"));
        }
        "funding_balance" => {
            args.extend(["funding", "balance"].map(String::from));
            opt!(args, "--account-type", gs("account_type"));
            opt!(args, "--coin", gs("coin"));
        }
        "funding_all_balance" => {
            args.extend(["funding", "all-balance"].map(String::from));
            opt!(args, "--account-type", gs("account_type"));
            opt!(args, "--coin", gs("coin"));
            opt!(args, "--member-id", gs("member_id"));
        }
        "funding_account_balance" => {
            args.extend(["funding", "account-balance"].map(String::from));
            opt!(args, "--account-type", gs("account_type"));
            opt!(args, "--member-id", gs("member_id"));
            opt!(args, "--coin", gs("coin"));
            if gb("with_bonus") {
                args.push("--with-bonus".to_string());
            }
        }
        "funding_transferable" => {
            args.extend(["funding", "transferable"].map(String::from));
            opt!(args, "--from-account-type", gs("from_account_type"));
            opt!(args, "--to-account-type", gs("to_account_type"));
        }
        "funding_transfer" => {
            args.extend(["funding", "transfer"].map(String::from));
            opt!(args, "--coin", gs("coin"));
            opt!(args, "--amount", gs("amount"));
            opt!(args, "--from-account-type", gs("from_account_type"));
            opt!(args, "--to-account-type", gs("to_account_type"));
        }
        "funding_transfer_history" => {
            args.extend(["funding", "transfer-history"].map(String::from));
            opt!(args, "--coin", gs("coin"));
            opt!(args, "--status", gs("status"));
            opt!(args, "--start", gi("start"));
            opt!(args, "--end", gi("end"));
            opt!(args, "--limit", gi("limit"));
            opt!(args, "--cursor", gs("cursor"));
        }
        "funding_sub_transfer" => {
            args.extend(["funding", "sub-transfer"].map(String::from));
            opt!(args, "--coin", gs("coin"));
            opt!(args, "--amount", gs("amount"));
            opt!(args, "--from-member-id", gs("from_member_id"));
            opt!(args, "--to-member-id", gs("to_member_id"));
            opt!(args, "--from-account-type", gs("from_account_type"));
            opt!(args, "--to-account-type", gs("to_account_type"));
        }
        "funding_sub_transfer_history" => {
            args.extend(["funding", "sub-transfer-history"].map(String::from));
            opt!(args, "--coin", gs("coin"));
            opt!(args, "--status", gs("status"));
            opt!(args, "--start", gi("start"));
            opt!(args, "--end", gi("end"));
            opt!(args, "--limit", gi("limit"));
            opt!(args, "--cursor", gs("cursor"));
        }
        "funding_deposit_address" => {
            args.extend(["funding", "deposit-address"].map(String::from));
            opt!(args, "--coin", gs("coin"));
            opt!(args, "--chain-type", gs("chain_type"));
        }
        "funding_deposit_history" => {
            args.extend(["funding", "deposit-history"].map(String::from));
            opt!(args, "--coin", gs("coin"));
            opt!(args, "--start", gi("start"));
            opt!(args, "--end", gi("end"));
            opt!(args, "--limit", gi("limit"));
            opt!(args, "--cursor", gs("cursor"));
        }
        "funding_withdraw" => {
            args.extend(["funding", "withdraw"].map(String::from));
            opt!(args, "--coin", gs("coin"));
            opt!(args, "--chain", gs("chain"));
            opt!(args, "--address", gs("address"));
            opt!(args, "--amount", gs("amount"));
            opt!(args, "--tag", gs("tag"));
            opt!(args, "--account-type", gs("account_type"));
        }
        "funding_withdraw_history" => {
            args.extend(["funding", "withdraw-history"].map(String::from));
            opt!(args, "--coin", gs("coin"));
            opt!(args, "--withdraw-id", gs("withdraw_id"));
            opt!(args, "--withdraw-type", gi("withdraw_type"));
            opt!(args, "--start", gi("start"));
            opt!(args, "--end", gi("end"));
            opt!(args, "--limit", gi("limit"));
            opt!(args, "--cursor", gs("cursor"));
        }
        "funding_cancel_withdraw" => {
            args.extend(["funding", "cancel-withdraw"].map(String::from));
            opt!(args, "--id", gs("id"));
        }

        // ----------------------------------------------------------------
        // convert
        // ----------------------------------------------------------------
        "convert_coins" => {
            args.extend(["convert", "coins"].map(String::from));
            opt!(args, "--account-type", gs("account_type"));
            opt!(args, "--coin", gs("coin"));
            opt!(args, "--side", gi("side"));
        }
        "convert_quote" => {
            args.extend(["convert", "quote"].map(String::from));
            opt!(args, "--account-type", gs("account_type"));
            opt!(args, "--from-coin", gs("from_coin"));
            opt!(args, "--to-coin", gs("to_coin"));
            opt!(args, "--from-amount", gs("from_amount"));
            opt!(args, "--to-amount", gs("to_amount"));
            if gb("dry_run") {
                args.push("--dry-run".to_string());
            }
        }
        "convert_execute" => {
            args.extend(["convert", "execute"].map(String::from));
            opt!(args, "--quote-tx-id", gs("quote_tx_id"));
        }
        "convert_status" => {
            args.extend(["convert", "status"].map(String::from));
            opt!(args, "--quote-tx-id", gs("quote_tx_id"));
            opt!(args, "--account-type", gs("account_type"));
        }
        "convert_history" => {
            args.extend(["convert", "history"].map(String::from));
            opt!(args, "--account-type", gs("account_type"));
            opt!(args, "--coin", gs("coin"));
            opt!(args, "--start", gi("start"));
            opt!(args, "--end", gi("end"));
            opt!(args, "--index", gi("index"));
            opt!(args, "--limit", gi("limit"));
        }

        // ----------------------------------------------------------------
        // margin
        // ----------------------------------------------------------------
        "margin_vip_data" => {
            args.extend(["margin", "vip-data"].map(String::from));
            opt!(args, "--vip-level", gs("vip_level"));
            opt!(args, "--currency", gs("currency"));
        }
        "margin_status" => {
            args.extend(["margin", "status"].map(String::from));
        }
        "margin_toggle" => {
            args.extend(["margin", "toggle"].map(String::from));
            opt!(args, "--mode", gs("mode"));
        }
        "margin_set_leverage" => {
            args.extend(["margin", "set-leverage"].map(String::from));
            opt!(args, "--leverage", gs("leverage"));
            opt!(args, "--currency", gs("currency"));
        }

        // ----------------------------------------------------------------
        // reports
        // ----------------------------------------------------------------
        "reports_transactions" => {
            args.extend(["reports", "transactions"].map(String::from));
            opt!(args, "--account-type", gs("account_type"));
            opt!(args, "--category", gs("category"));
            opt!(args, "--currency", gs("currency"));
            opt!(args, "--base-coin", gs("base_coin"));
            opt!(args, "--tx-type", gs("tx_type"));
            opt!(args, "--start", gi("start"));
            opt!(args, "--end", gi("end"));
            opt!(args, "--limit", gi("limit"));
            opt!(args, "--cursor", gs("cursor"));
        }
        "reports_borrow_history" => {
            args.extend(["reports", "borrow-history"].map(String::from));
            opt!(args, "--currency", gs("currency"));
            opt!(args, "--start", gi("start"));
            opt!(args, "--end", gi("end"));
            opt!(args, "--limit", gi("limit"));
            opt!(args, "--cursor", gs("cursor"));
        }
        "reports_orders" => {
            args.extend(["reports", "orders"].map(String::from));
            opt!(args, "--category", gs("category"));
            opt!(args, "--symbol", gs("symbol"));
            opt!(args, "--order-id", gs("order_id"));
            opt!(args, "--order-status", gs("order_status"));
            opt!(args, "--start", gi("start"));
            opt!(args, "--end", gi("end"));
            opt!(args, "--limit", gi("limit"));
            opt!(args, "--cursor", gs("cursor"));
        }
        "reports_fills" => {
            args.extend(["reports", "fills"].map(String::from));
            opt!(args, "--category", gs("category"));
            opt!(args, "--symbol", gs("symbol"));
            opt!(args, "--order-id", gs("order_id"));
            opt!(args, "--exec-type", gs("exec_type"));
            opt!(args, "--start", gi("start"));
            opt!(args, "--end", gi("end"));
            opt!(args, "--limit", gi("limit"));
            opt!(args, "--cursor", gs("cursor"));
        }
        "reports_closed_pnl" => {
            args.extend(["reports", "closed-pnl"].map(String::from));
            opt!(args, "--category", gs("category"));
            opt!(args, "--symbol", gs("symbol"));
            opt!(args, "--start", gi("start"));
            opt!(args, "--end", gi("end"));
            opt!(args, "--limit", gi("limit"));
            opt!(args, "--cursor", gs("cursor"));
        }
        "reports_moves" => {
            args.extend(["reports", "moves"].map(String::from));
            opt!(args, "--category", gs("category"));
            opt!(args, "--symbol", gs("symbol"));
            opt!(args, "--start", gi("start"));
            opt!(args, "--end", gi("end"));
            opt!(args, "--limit", gi("limit"));
            opt!(args, "--cursor", gs("cursor"));
        }
        "reports_deposits" => {
            args.extend(["reports", "deposits"].map(String::from));
            opt!(args, "--coin", gs("coin"));
            opt!(args, "--start", gi("start"));
            opt!(args, "--end", gi("end"));
            opt!(args, "--limit", gi("limit"));
            opt!(args, "--cursor", gs("cursor"));
        }
        "reports_withdrawals" => {
            args.extend(["reports", "withdrawals"].map(String::from));
            opt!(args, "--coin", gs("coin"));
            opt!(args, "--withdraw-id", gs("withdraw_id"));
            opt!(args, "--withdraw-type", gi("withdraw_type"));
            opt!(args, "--start", gi("start"));
            opt!(args, "--end", gi("end"));
            opt!(args, "--limit", gi("limit"));
            opt!(args, "--cursor", gs("cursor"));
        }
        "reports_transfers" => {
            args.extend(["reports", "transfers"].map(String::from));
            opt!(args, "--coin", gs("coin"));
            opt!(args, "--status", gs("status"));
            opt!(args, "--start", gi("start"));
            opt!(args, "--end", gi("end"));
            opt!(args, "--limit", gi("limit"));
            opt!(args, "--cursor", gs("cursor"));
        }
        "reports_sub_transfers" => {
            args.extend(["reports", "sub-transfers"].map(String::from));
            opt!(args, "--coin", gs("coin"));
            opt!(args, "--status", gs("status"));
            opt!(args, "--start", gi("start"));
            opt!(args, "--end", gi("end"));
            opt!(args, "--limit", gi("limit"));
            opt!(args, "--cursor", gs("cursor"));
        }

        // ----------------------------------------------------------------
        // subaccount
        // ----------------------------------------------------------------
        "subaccount_list" => {
            args.extend(["subaccount", "list"].map(String::from));
        }
        "subaccount_list_all" => {
            args.extend(["subaccount", "list-all"].map(String::from));
            opt!(args, "--page-size", gi("page_size"));
            opt!(args, "--next-cursor", gs("next_cursor"));
        }
        "subaccount_api_keys" => {
            args.extend(["subaccount", "api-keys"].map(String::from));
            opt!(args, "--sub-member-id", gs("sub_member_id"));
            opt!(args, "--limit", gi("limit"));
            opt!(args, "--cursor", gs("cursor"));
        }
        "subaccount_wallet_types" => {
            args.extend(["subaccount", "wallet-types"].map(String::from));
            opt!(args, "--member-ids", gs("member_ids"));
        }
        "subaccount_create" => {
            args.extend(["subaccount", "create"].map(String::from));
            opt!(args, "--username", gs("username"));
            opt!(args, "--password", gs("password"));
            opt!(args, "--member-type", gi("member_type"));
            if gb("quick_login") {
                args.push("--quick-login".to_string());
            }
        }
        "subaccount_delete" => {
            args.extend(["subaccount", "delete"].map(String::from));
            opt!(args, "--sub-member-id", gs("sub_member_id"));
        }
        "subaccount_freeze" => {
            args.extend(["subaccount", "freeze"].map(String::from));
            opt!(args, "--sub-member-id", gs("sub_member_id"));
        }
        "subaccount_unfreeze" => {
            args.extend(["subaccount", "unfreeze"].map(String::from));
            opt!(args, "--sub-member-id", gs("sub_member_id"));
        }

        // ----------------------------------------------------------------
        // futures
        // ----------------------------------------------------------------
        "futures_instruments" => {
            args.extend(["futures", "instruments"].map(String::from));
            opt!(args, "--category", gs("category"));
            opt!(args, "--symbol", gs("symbol"));
            opt!(args, "--status", gs("status"));
            opt!(args, "--base-coin", gs("base_coin"));
            opt!(args, "--limit", gi("limit"));
            opt!(args, "--cursor", gs("cursor"));
        }
        "futures_tickers" => {
            args.extend(["futures", "tickers"].map(String::from));
            opt!(args, "--category", gs("category"));
            opt!(args, "--symbol", gs("symbol"));
            opt!(args, "--base-coin", gs("base_coin"));
        }
        "futures_orderbook" => {
            args.extend(["futures", "orderbook"].map(String::from));
            opt!(args, "--category", gs("category"));
            opt!(args, "--symbol", gs("symbol"));
            opt!(args, "--limit", gi("limit"));
        }
        "futures_funding_rate" => {
            args.extend(["futures", "funding-rate"].map(String::from));
            opt!(args, "--category", gs("category"));
            opt!(args, "--symbol", gs("symbol"));
            opt!(args, "--start", gi("start"));
            opt!(args, "--end", gi("end"));
            opt!(args, "--limit", gi("limit"));
        }
        "futures_adl_alert" => {
            args.extend(["futures", "adl-alert"].map(String::from));
            opt!(args, "--category", gs("category"));
            opt!(args, "--symbol", gs("symbol"));
        }
        "futures_risk_limit" => {
            args.extend(["futures", "risk-limit"].map(String::from));
            opt!(args, "--category", gs("category"));
            opt!(args, "--symbol", gs("symbol"));
        }
        "futures_open_interest" => {
            args.extend(["futures", "open-interest"].map(String::from));
            opt!(args, "--category", gs("category"));
            opt!(args, "--symbol", gs("symbol"));
            opt!(args, "--interval-time", gs("interval_time"));
            opt!(args, "--start", gi("start"));
            opt!(args, "--end", gi("end"));
            opt!(args, "--limit", gi("limit"));
            opt!(args, "--cursor", gs("cursor"));
        }
        "futures_positions" => {
            args.extend(["futures", "positions"].map(String::from));
            opt!(args, "--category", gs("category"));
            opt!(args, "--symbol", gs("symbol"));
            opt!(args, "--base-coin", gs("base_coin"));
            opt!(args, "--settle-coin", gs("settle_coin"));
            opt!(args, "--limit", gi("limit"));
            opt!(args, "--cursor", gs("cursor"));
        }
        "futures_open_orders" => {
            args.extend(["futures", "open-orders"].map(String::from));
            opt!(args, "--category", gs("category"));
            opt!(args, "--symbol", gs("symbol"));
            opt!(args, "--base-coin", gs("base_coin"));
            opt!(args, "--settle-coin", gs("settle_coin"));
            opt!(args, "--order-id", gs("order_id"));
            opt!(args, "--order-link-id", gs("order_link_id"));
            opt!(args, "--limit", gi("limit"));
            opt!(args, "--cursor", gs("cursor"));
        }
        "futures_history" => {
            args.extend(["futures", "history"].map(String::from));
            opt!(args, "--category", gs("category"));
            opt!(args, "--symbol", gs("symbol"));
            opt!(args, "--order-id", gs("order_id"));
            opt!(args, "--order-status", gs("order_status"));
            opt!(args, "--start", gi("start"));
            opt!(args, "--end", gi("end"));
            opt!(args, "--limit", gi("limit"));
            opt!(args, "--cursor", gs("cursor"));
        }
        "futures_fills" => {
            args.extend(["futures", "fills"].map(String::from));
            opt!(args, "--category", gs("category"));
            opt!(args, "--symbol", gs("symbol"));
            opt!(args, "--order-id", gs("order_id"));
            opt!(args, "--exec-type", gs("exec_type"));
            opt!(args, "--start", gi("start"));
            opt!(args, "--end", gi("end"));
            opt!(args, "--limit", gi("limit"));
            opt!(args, "--cursor", gs("cursor"));
        }
        "futures_buy" => {
            args.extend(["futures", "buy"].map(String::from));
            opt!(args, "--category", gs("category"));
            opt!(args, "--symbol", gs("symbol"));
            opt!(args, "--qty", gs("qty"));
            opt!(args, "--price", gs("price"));
            opt!(args, "--order-type", gs("order_type"));
            opt!(args, "--time-in-force", gs("time_in_force"));
            opt!(args, "--take-profit", gs("take_profit"));
            opt!(args, "--stop-loss", gs("stop_loss"));
            opt!(args, "--order-link-id", gs("order_link_id"));
            opt!(args, "--position-idx", gi("position_idx"));
            if gb("reduce_only") {
                args.push("--reduce-only".to_string());
            }
        }
        "futures_sell" => {
            args.extend(["futures", "sell"].map(String::from));
            opt!(args, "--category", gs("category"));
            opt!(args, "--symbol", gs("symbol"));
            opt!(args, "--qty", gs("qty"));
            opt!(args, "--price", gs("price"));
            opt!(args, "--order-type", gs("order_type"));
            opt!(args, "--time-in-force", gs("time_in_force"));
            opt!(args, "--take-profit", gs("take_profit"));
            opt!(args, "--stop-loss", gs("stop_loss"));
            opt!(args, "--order-link-id", gs("order_link_id"));
            opt!(args, "--position-idx", gi("position_idx"));
            if gb("reduce_only") {
                args.push("--reduce-only".to_string());
            }
        }
        "futures_cancel" => {
            args.extend(["futures", "cancel"].map(String::from));
            opt!(args, "--category", gs("category"));
            opt!(args, "--symbol", gs("symbol"));
            opt!(args, "--order-id", gs("order_id"));
            opt!(args, "--order-link-id", gs("order_link_id"));
        }
        "futures_cancel_all" => {
            args.extend(["futures", "cancel-all"].map(String::from));
            opt!(args, "--category", gs("category"));
            opt!(args, "--symbol", gs("symbol"));
            opt!(args, "--base-coin", gs("base_coin"));
            opt!(args, "--settle-coin", gs("settle_coin"));
        }
        "futures_set_leverage" => {
            args.extend(["futures", "set-leverage"].map(String::from));
            opt!(args, "--category", gs("category"));
            opt!(args, "--symbol", gs("symbol"));
            opt!(args, "--buy-leverage", gn("buy_leverage"));
            opt!(args, "--sell-leverage", gn("sell_leverage"));
        }

        // ----------------------------------------------------------------
        // paper
        // ----------------------------------------------------------------
        "paper_init" => {
            args.extend(["paper", "init"].map(String::from));
            opt!(args, "--usdt", gn("usdt"));
            opt!(args, "--settle-coin", gs("settle_coin"));
            opt!(args, "--taker-fee-bps", gi("taker_fee_bps"));
            opt!(args, "--maker-fee-bps", gi("maker_fee_bps"));
            opt!(args, "--slippage-bps", gi("slippage_bps"));
            if gb("force") {
                args.push("--force".to_string());
            }
        }
        "paper_buy" => {
            args.extend(["paper", "buy"].map(String::from));
            opt!(args, "--category", gs("category"));
            opt!(args, "--symbol", gs("symbol"));
            opt!(args, "--qty", gn("qty"));
            opt!(args, "--price", gn("price"));
        }
        "paper_sell" => {
            args.extend(["paper", "sell"].map(String::from));
            opt!(args, "--category", gs("category"));
            opt!(args, "--symbol", gs("symbol"));
            opt!(args, "--qty", gn("qty"));
            opt!(args, "--price", gn("price"));
        }
        "paper_status" => {
            args.extend(["paper", "status"].map(String::from));
        }
        "paper_balance" => {
            args.extend(["paper", "balance"].map(String::from));
        }
        "paper_positions" => {
            args.extend(["paper", "positions"].map(String::from));
        }
        "paper_orders" => {
            args.extend(["paper", "orders"].map(String::from));
        }
        "paper_cancel" => {
            args.extend(["paper", "cancel"].map(String::from));
            // order_id is a positional argument, not a flag
            if let Some(id) = gi("order_id") {
                args.push(id);
            }
        }
        "paper_cancel_all" => {
            args.extend(["paper", "cancel-all"].map(String::from));
        }
        "paper_history" => {
            args.extend(["paper", "history"].map(String::from));
        }
        "paper_cancelled" => {
            args.extend(["paper", "cancelled"].map(String::from));
        }
        "paper_reset" => {
            args.extend(["paper", "reset"].map(String::from));
        }

        // ----------------------------------------------------------------
        // futures-paper
        // ----------------------------------------------------------------
        "futures_paper_init" => {
            args.extend(["futures", "paper", "init"].map(String::from));
            opt!(args, "--balance", gn("balance"));
            opt!(args, "--currency", gs("currency"));
            opt!(args, "--fee-rate", gn("fee_rate"));
            if gb("force") {
                args.push("--force".to_string());
            }
        }
        "futures_paper_reset" => {
            args.extend(["futures", "paper", "reset"].map(String::from));
            opt!(args, "--balance", gn("balance"));
            opt!(args, "--currency", gs("currency"));
            opt!(args, "--fee-rate", gn("fee_rate"));
        }
        "futures_paper_balance" => {
            args.extend(["futures", "paper", "balance"].map(String::from));
        }
        "futures_paper_status" => {
            args.extend(["futures", "paper", "status"].map(String::from));
        }
        "futures_paper_buy" => {
            args.extend(["futures", "paper", "buy"].map(String::from));
            if let Some(sym) = gs("symbol") {
                args.push(sym);
            }
            if let Some(sz) = gs("size") {
                args.push(sz);
            }
            opt!(args, "--type", gs("type"));
            opt!(args, "--price", gs("price"));
            opt!(args, "--stop-price", gs("stop_price"));
            opt!(args, "--trigger-signal", gs("trigger_signal"));
            opt!(args, "--client-order-id", gs("client_order_id"));
            opt!(args, "--leverage", gs("leverage"));
            opt!(
                args,
                "--trailing-stop-max-deviation",
                gs("trailing_stop_max_deviation")
            );
            opt!(
                args,
                "--trailing-stop-deviation-unit",
                gs("trailing_stop_deviation_unit")
            );
            opt!(args, "--category", gs("category"));
            if gb("reduce_only") {
                args.push("--reduce-only".to_string());
            }
        }
        "futures_paper_sell" => {
            args.extend(["futures", "paper", "sell"].map(String::from));
            if let Some(sym) = gs("symbol") {
                args.push(sym);
            }
            if let Some(sz) = gs("size") {
                args.push(sz);
            }
            opt!(args, "--type", gs("type"));
            opt!(args, "--price", gs("price"));
            opt!(args, "--stop-price", gs("stop_price"));
            opt!(args, "--trigger-signal", gs("trigger_signal"));
            opt!(args, "--client-order-id", gs("client_order_id"));
            opt!(args, "--leverage", gs("leverage"));
            opt!(
                args,
                "--trailing-stop-max-deviation",
                gs("trailing_stop_max_deviation")
            );
            opt!(
                args,
                "--trailing-stop-deviation-unit",
                gs("trailing_stop_deviation_unit")
            );
            opt!(args, "--category", gs("category"));
            if gb("reduce_only") {
                args.push("--reduce-only".to_string());
            }
        }
        "futures_paper_orders" => {
            args.extend(["futures", "paper", "orders"].map(String::from));
            opt!(args, "--category", gs("category"));
        }
        "futures_paper_order_status" => {
            args.extend(["futures", "paper", "order-status"].map(String::from));
            if let Some(id) = gs("order_id") {
                args.push(id);
            }
        }
        "futures_paper_edit_order" => {
            args.extend(["futures", "paper", "edit-order"].map(String::from));
            opt!(args, "--order-id", gs("order_id"));
            opt!(args, "--size", gs("size"));
            opt!(args, "--price", gs("price"));
            opt!(args, "--stop-price", gs("stop_price"));
        }
        "futures_paper_cancel" => {
            args.extend(["futures", "paper", "cancel"].map(String::from));
            opt!(args, "--order-id", gs("order_id"));
            opt!(args, "--cli-ord-id", gs("cli_ord_id"));
        }
        "futures_paper_cancel_all" => {
            args.extend(["futures", "paper", "cancel-all"].map(String::from));
            opt!(args, "--symbol", gs("symbol"));
        }
        "futures_paper_batch_order" => {
            args.extend(["futures", "paper", "batch-order"].map(String::from));
            if let Some(json) = gs("orders_json") {
                args.push(json);
            }
        }
        "futures_paper_positions" => {
            args.extend(["futures", "paper", "positions"].map(String::from));
            opt!(args, "--category", gs("category"));
        }
        "futures_paper_fills" => {
            args.extend(["futures", "paper", "fills"].map(String::from));
        }
        "futures_paper_history" => {
            args.extend(["futures", "paper", "history"].map(String::from));
        }
        "futures_paper_leverage" => {
            args.extend(["futures", "paper", "leverage"].map(String::from));
            opt!(args, "--symbol", gs("symbol"));
        }
        "futures_paper_set_leverage" => {
            args.extend(["futures", "paper", "set-leverage"].map(String::from));
            if let Some(sym) = gs("symbol") {
                args.push(sym);
            }
            if let Some(lev) = gs("leverage") {
                args.push(lev);
            }
        }

        // ----------------------------------------------------------------
        // auth
        // ----------------------------------------------------------------
        "auth_test" => {
            args.extend(["auth", "test"].map(String::from));
        }
        "auth_verify" => {
            args.extend(["auth", "test"].map(String::from));
        }
        "auth_show" => {
            args.extend(["auth", "show"].map(String::from));
        }
        "auth_permissions" => {
            args.extend(["auth", "permissions"].map(String::from));
        }
        "ws_notifications" => {
            args.extend(["ws", "notifications"].map(String::from));
        }

        _ => return None,
    }

    Some(args)
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::{all_tools, tool_to_args};

    #[test]
    fn every_registered_tool_has_an_argv_mapping() {
        for tool in all_tools() {
            assert!(
                tool_to_args(tool.name, &json!({})).is_some(),
                "missing argv mapping for {}",
                tool.name
            );
        }
    }
}
