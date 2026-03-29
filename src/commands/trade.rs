use clap::Subcommand;
use serde_json::{json, Value};

use crate::client::BybitClient;
use crate::commands::helpers::{confirm, should_default_linear_settle_coin};
use crate::errors::BybitResult;
use crate::output::{print_output, OutputFormat};

#[derive(Debug, clap::Args)]
pub struct TradeArgs {
    #[command(subcommand)]
    pub command: TradeCommand,
}

#[derive(Debug, Subcommand)]
pub enum TradeCommand {
    /// Place a buy order
    Buy(OrderArgs),
    /// Place a sell order
    Sell(OrderArgs),
    /// Amend an existing open order
    Amend {
        #[arg(long, default_value = "linear")]
        category: String,
        #[arg(long)]
        symbol: String,
        #[arg(long)]
        order_id: Option<String>,
        #[arg(long)]
        order_link_id: Option<String>,
        #[arg(long)]
        qty: Option<String>,
        #[arg(long)]
        price: Option<String>,
        #[arg(long)]
        take_profit: Option<String>,
        #[arg(long)]
        stop_loss: Option<String>,
        #[arg(long)]
        trigger_price: Option<String>,
    },
    /// Cancel an order
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
    /// Cancel all open orders
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
    /// List open orders
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
    /// Get order history
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
    /// Get execution/fill history
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
    /// Batch place up to 20 orders
    BatchPlace {
        #[arg(long, default_value = "linear")]
        category: String,
        /// JSON array of order objects
        #[arg(long)]
        orders: String,
    },
    /// Batch amend up to 20 orders
    BatchAmend {
        #[arg(long, default_value = "linear")]
        category: String,
        /// JSON array of amend objects
        #[arg(long)]
        orders: String,
    },
    /// Batch cancel up to 20 orders
    BatchCancel {
        #[arg(long, default_value = "linear")]
        category: String,
        /// JSON array of cancel objects (each needs orderId or orderLinkId)
        #[arg(long)]
        orders: String,
    },
    /// Dead man's switch — cancel all open orders after N seconds (0 = disable)
    CancelAfter {
        /// Seconds until all open orders are cancelled. Pass 0 to disable.
        seconds: u32,
    },
}

#[derive(Debug, clap::Args)]
pub struct OrderArgs {
    #[arg(long, default_value = "linear")]
    pub category: String,
    #[arg(long)]
    pub symbol: String,
    /// Order quantity
    #[arg(long)]
    pub qty: String,
    /// Limit price (omit for market orders)
    #[arg(long)]
    pub price: Option<String>,
    /// Market or Limit
    #[arg(long, default_value = "Limit")]
    pub order_type: String,
    /// GTC, IOC, FOK, PostOnly
    #[arg(long, default_value = "GTC")]
    pub time_in_force: String,
    #[arg(long)]
    pub order_link_id: Option<String>,
    #[arg(long)]
    pub take_profit: Option<String>,
    #[arg(long)]
    pub stop_loss: Option<String>,
    /// Limit price for take profit
    #[arg(long)]
    pub tp_limit_price: Option<String>,
    /// Limit price for stop loss
    #[arg(long)]
    pub sl_limit_price: Option<String>,
    /// LastPrice, IndexPrice, MarkPrice
    #[arg(long)]
    pub tp_trigger_by: Option<String>,
    /// LastPrice, IndexPrice, MarkPrice
    #[arg(long)]
    pub sl_trigger_by: Option<String>,
    /// 0=one-way, 1=buy-side hedge, 2=sell-side hedge
    #[arg(long, default_value = "0")]
    pub position_idx: u8,
    /// Reduce-only order
    #[arg(long)]
    pub reduce_only: bool,
    /// Convenience flag for PostOnly time-in-force
    #[arg(long)]
    pub post_only: bool,
    /// Visible quantity for Iceberg orders
    #[arg(long)]
    pub display_qty: Option<String>,
    /// Trigger price for conditional orders
    #[arg(long)]
    pub trigger_price: Option<String>,
    /// Dry-run: validate without submitting
    #[arg(long)]
    pub validate: bool,
}

pub async fn run(
    args: TradeArgs,
    client: &BybitClient,
    format: OutputFormat,
    force: bool,
) -> BybitResult<()> {
    let value: Value = match args.command {
        TradeCommand::Buy(order) => place_order("Buy", order, client, force).await?,
        TradeCommand::Sell(order) => place_order("Sell", order, client, force).await?,

        TradeCommand::Amend {
            category,
            symbol,
            order_id,
            order_link_id,
            qty,
            price,
            take_profit,
            stop_loss,
            trigger_price,
        } => {
            confirm(&format!("Amend order on {symbol}?"), force)?;
            let mut body = json!({ "category": category, "symbol": symbol });
            if let Some(v) = order_id {
                body["orderId"] = json!(v);
            }
            if let Some(v) = order_link_id {
                body["orderLinkId"] = json!(v);
            }
            if let Some(v) = qty {
                body["qty"] = json!(v);
            }
            if let Some(v) = price {
                body["price"] = json!(v);
            }
            if let Some(v) = take_profit {
                body["takeProfit"] = json!(v);
            }
            if let Some(v) = stop_loss {
                body["stopLoss"] = json!(v);
            }
            if let Some(v) = trigger_price {
                body["triggerPrice"] = json!(v);
            }
            client.private_post("/v5/order/amend", &body).await?
        }

        TradeCommand::Cancel {
            category,
            symbol,
            order_id,
            order_link_id,
        } => {
            confirm(&format!("Cancel order on {symbol}?"), force)?;
            let mut body = json!({ "category": category, "symbol": symbol });
            if let Some(v) = order_id {
                body["orderId"] = json!(v);
            }
            if let Some(v) = order_link_id {
                body["orderLinkId"] = json!(v);
            }
            client.private_post("/v5/order/cancel", &body).await?
        }

        TradeCommand::CancelAll {
            category,
            symbol,
            base_coin,
            settle_coin,
        } => {
            confirm("Cancel ALL open orders?", force)?;
            let mut body = json!({ "category": category });
            if let Some(v) = symbol {
                body["symbol"] = json!(v);
            }
            if let Some(v) = base_coin {
                body["baseCoin"] = json!(v);
            }
            if let Some(v) = settle_coin {
                body["settleCoin"] = json!(v);
            }
            client.private_post("/v5/order/cancel-all", &body).await?
        }

        TradeCommand::OpenOrders {
            category,
            symbol,
            base_coin,
            settle_coin,
            order_id,
            order_link_id,
            limit,
            cursor,
        } => {
            let limit_str = limit.map(|l| l.to_string());
            let default_settle_coin =
                should_default_linear_settle_coin(&category, &symbol, &base_coin, &settle_coin)
                    .then_some("USDT");
            let mut params = vec![("category", category.as_str())];
            if let Some(ref s) = symbol {
                params.push(("symbol", s));
            }
            if let Some(ref s) = base_coin {
                params.push(("baseCoin", s));
            }
            if let Some(ref s) = settle_coin {
                params.push(("settleCoin", s));
            } else if let Some(s) = default_settle_coin {
                params.push(("settleCoin", s));
            }
            if let Some(ref s) = order_id {
                params.push(("orderId", s));
            }
            if let Some(ref s) = order_link_id {
                params.push(("orderLinkId", s));
            }
            if let Some(ref s) = limit_str {
                params.push(("limit", s));
            }
            if let Some(ref s) = cursor {
                params.push(("cursor", s));
            }
            client.private_get("/v5/order/realtime", &params).await?
        }

        TradeCommand::History {
            category,
            symbol,
            order_id,
            order_status,
            start,
            end,
            limit,
            cursor,
        } => {
            let start_str = start.map(|s| s.to_string());
            let end_str = end.map(|e| e.to_string());
            let limit_str = limit.map(|l| l.to_string());
            let mut params = vec![("category", category.as_str())];
            if let Some(ref s) = symbol {
                params.push(("symbol", s));
            }
            if let Some(ref s) = order_id {
                params.push(("orderId", s));
            }
            if let Some(ref s) = order_status {
                params.push(("orderStatus", s));
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
            client.private_get("/v5/order/history", &params).await?
        }

        TradeCommand::Fills {
            category,
            symbol,
            order_id,
            start,
            end,
            exec_type,
            limit,
            cursor,
        } => {
            let start_str = start.map(|s| s.to_string());
            let end_str = end.map(|e| e.to_string());
            let limit_str = limit.map(|l| l.to_string());
            let mut params = vec![("category", category.as_str())];
            if let Some(ref s) = symbol {
                params.push(("symbol", s));
            }
            if let Some(ref s) = order_id {
                params.push(("orderId", s));
            }
            if let Some(ref s) = exec_type {
                params.push(("execType", s));
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
            client.private_get("/v5/execution/list", &params).await?
        }

        TradeCommand::BatchPlace { category, orders } => {
            confirm("Batch place orders?", force)?;
            let parsed: Value = serde_json::from_str(&orders).map_err(|e| {
                crate::errors::BybitError::Parse(format!("invalid orders JSON: {e}"))
            })?;
            let body = json!({ "category": category, "request": parsed });
            client.private_post("/v5/order/create-batch", &body).await?
        }

        TradeCommand::BatchAmend { category, orders } => {
            confirm("Batch amend orders?", force)?;
            let parsed: Value = serde_json::from_str(&orders).map_err(|e| {
                crate::errors::BybitError::Parse(format!("invalid orders JSON: {e}"))
            })?;
            let body = json!({ "category": category, "request": parsed });
            client.private_post("/v5/order/amend-batch", &body).await?
        }

        TradeCommand::BatchCancel { category, orders } => {
            confirm("Batch cancel orders?", force)?;
            let parsed: Value = serde_json::from_str(&orders).map_err(|e| {
                crate::errors::BybitError::Parse(format!("invalid orders JSON: {e}"))
            })?;
            let body = json!({ "category": category, "request": parsed });
            client.private_post("/v5/order/cancel-batch", &body).await?
        }

        TradeCommand::CancelAfter { seconds } => {
            if seconds == 0 {
                confirm(
                    "Disable the cancel-after timer (orders will NOT be auto-cancelled)?",
                    force,
                )?;
            } else {
                confirm(&format!("Set cancel-after timer to {seconds}s? All open orders will be cancelled if not refreshed."), force)?;
            }
            let body = json!({ "timeOut": seconds });
            client
                .private_post("/v5/order/cancel-all-after", &body)
                .await?
        }
    };

    print_output(&value, format);
    Ok(())
}

async fn place_order(
    side: &str,
    args: OrderArgs,
    client: &BybitClient,
    force: bool,
) -> BybitResult<Value> {
    if args.validate {
        eprintln!(
            "[validate] Would place {} {} {} @ {:?} (post_only: {}, display_qty: {:?}, trigger_price: {:?})",
            side, args.qty, args.symbol, args.price, args.post_only, args.display_qty, args.trigger_price
        );
        return Ok(json!({
            "validated": true,
            "side": side,
            "symbol": args.symbol,
            "qty": args.qty,
            "postOnly": args.post_only,
            "displayQty": args.display_qty,
            "triggerPrice": args.trigger_price,
            "tpLimitPrice": args.tp_limit_price,
            "slLimitPrice": args.sl_limit_price
        }));
    }

    confirm(
        &format!(
            "{side} {qty} {symbol} @ {price}{iceberg}{trigger}{post_only}{tpsl}?",
            qty = args.qty,
            symbol = args.symbol,
            price = args.price.as_deref().unwrap_or("MARKET"),
            iceberg = args
                .display_qty
                .as_ref()
                .map(|q| format!(" (Iceberg: {q} visible)"))
                .unwrap_or_default(),
            trigger = args
                .trigger_price
                .as_ref()
                .map(|p| format!(" (Trigger: {p})"))
                .unwrap_or_default(),
            post_only = if args.post_only { " [PostOnly]" } else { "" },
            tpsl = match (&args.take_profit, &args.stop_loss) {
                (Some(tp), Some(sl)) => format!(" (TP: {tp}, SL: {sl})"),
                (Some(tp), None) => format!(" (TP: {tp})"),
                (None, Some(sl)) => format!(" (SL: {sl})"),
                _ => "".to_string(),
            },
        ),
        force,
    )?;

    let order_type = if args.price.is_some() {
        &args.order_type
    } else {
        "Market"
    };

    let tif = if args.post_only {
        "PostOnly"
    } else {
        &args.time_in_force
    };

    let mut body = json!({
        "category": args.category,
        "symbol": args.symbol,
        "side": side,
        "orderType": order_type,
        "qty": args.qty,
        "timeInForce": tif,
        "positionIdx": args.position_idx,
    });

    if let Some(price) = &args.price {
        body["price"] = json!(price);
    }
    if let Some(tp) = &args.take_profit {
        body["takeProfit"] = json!(tp);
    }
    if let Some(sl) = &args.stop_loss {
        body["stopLoss"] = json!(sl);
    }
    if let Some(v) = &args.tp_limit_price {
        body["tpLimitPrice"] = json!(v);
    }
    if let Some(v) = &args.sl_limit_price {
        body["slLimitPrice"] = json!(v);
    }
    if let Some(v) = &args.tp_trigger_by {
        body["tpTriggerBy"] = json!(v);
    }
    if let Some(v) = &args.sl_trigger_by {
        body["slTriggerBy"] = json!(v);
    }
    if let Some(link_id) = &args.order_link_id {
        body["orderLinkId"] = json!(link_id);
    }
    if let Some(dq) = &args.display_qty {
        body["displayQty"] = json!(dq);
    }
    if let Some(tp) = &args.trigger_price {
        body["triggerPrice"] = json!(tp);
    }
    if args.reduce_only {
        body["reduceOnly"] = json!(true);
    }

    client.private_post("/v5/order/create", &body).await
}
