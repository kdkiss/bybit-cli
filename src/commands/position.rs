use clap::Subcommand;
use serde_json::{json, Value};

use crate::client::BybitClient;
use crate::commands::helpers::{confirm, should_default_linear_settle_coin};
use crate::errors::BybitResult;
use crate::output::{print_output, OutputFormat};

#[derive(Debug, clap::Args)]
pub struct PositionArgs {
    #[command(subcommand)]
    pub command: PositionCommand,
}

#[derive(Debug, Subcommand)]
pub enum PositionCommand {
    /// List open positions
    List {
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
    /// Set leverage for a symbol
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
    /// Switch position mode (one-way / hedge)
    SwitchMode {
        #[arg(long, default_value = "linear")]
        category: String,
        #[arg(long)]
        symbol: Option<String>,
        #[arg(long)]
        coin: Option<String>,
        /// 0=MergedSingle (one-way), 3=BothSides (hedge)
        #[arg(long)]
        mode: u8,
    },
    /// Set take-profit / stop-loss / trailing stop
    SetTpsl {
        #[arg(long, default_value = "linear")]
        category: String,
        #[arg(long)]
        symbol: String,
        #[arg(long)]
        take_profit: Option<String>,
        #[arg(long)]
        stop_loss: Option<String>,
        #[arg(long)]
        trailing_stop: Option<String>,
        #[arg(long)]
        tp_trigger_by: Option<String>,
        #[arg(long)]
        sl_trigger_by: Option<String>,
        #[arg(long)]
        tp_size: Option<String>,
        #[arg(long)]
        sl_size: Option<String>,
        #[arg(long)]
        tp_limit_price: Option<String>,
        #[arg(long)]
        sl_limit_price: Option<String>,
        /// 0=one-way, 1=buy-side hedge, 2=sell-side hedge
        #[arg(long, default_value = "0")]
        position_idx: u8,
    },
    /// Set trailing stop for a position
    TrailingStop {
        #[arg(long, default_value = "linear")]
        category: String,
        #[arg(long)]
        symbol: String,
        /// Retracement distance
        #[arg(long)]
        trailing_stop: String,
        /// Activation price (optional)
        #[arg(long)]
        active_price: Option<String>,
        /// 0=one-way, 1=buy-side hedge, 2=sell-side hedge
        #[arg(long, default_value = "0")]
        position_idx: u8,
    },
    /// Set risk limit
    SetRiskLimit {
        #[arg(long, default_value = "linear")]
        category: String,
        #[arg(long)]
        symbol: String,
        #[arg(long)]
        risk_id: u32,
        #[arg(long, default_value = "0")]
        position_idx: u8,
    },
    /// Add or reduce margin
    AddMargin {
        #[arg(long, default_value = "linear")]
        category: String,
        #[arg(long)]
        symbol: String,
        /// Positive to add, negative to reduce
        #[arg(long)]
        margin: String,
        #[arg(long, default_value = "0")]
        position_idx: u8,
    },
    /// Get closed PnL history
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
    /// Move positions between UIDs (institutional)
    Move {
        #[arg(long, default_value = "linear")]
        category: String,
        #[arg(long)]
        from_uid: String,
        #[arg(long)]
        to_uid: String,
        /// JSON array of position objects [{symbol, side, qty}]
        #[arg(long)]
        positions: String,
    },
    /// Get move-position history
    MoveHistory {
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
    /// EMERGENCY: Cancel all orders and close all positions in a category
    Flatten {
        #[arg(long, default_value = "linear")]
        category: String,
        #[arg(long)]
        symbol: Option<String>,
    },
}

pub async fn run(
    args: PositionArgs,
    client: &BybitClient,
    format: OutputFormat,
    force: bool,
) -> BybitResult<()> {
    let value: Value = match args.command {
        PositionCommand::List {
            category,
            symbol,
            base_coin,
            settle_coin,
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
            if let Some(ref s) = limit_str {
                params.push(("limit", s));
            }
            if let Some(ref s) = cursor {
                params.push(("cursor", s));
            }
            client.private_get("/v5/position/list", &params).await?
        }

        PositionCommand::SetLeverage {
            category,
            symbol,
            buy_leverage,
            sell_leverage,
        } => {
            confirm(
                &format!("Set leverage on {symbol} to {buy_leverage}x/{sell_leverage}x?"),
                force,
            )?;
            let body = json!({
                "category": category,
                "symbol": symbol,
                "buyLeverage": buy_leverage,
                "sellLeverage": sell_leverage,
            });
            client
                .private_post("/v5/position/set-leverage", &body)
                .await?
        }

        PositionCommand::SwitchMode {
            category,
            symbol,
            coin,
            mode,
        } => {
            let mode_name = if mode == 0 { "one-way" } else { "hedge" };
            confirm(&format!("Switch to {mode_name} mode?"), force)?;
            let mut body = json!({ "category": category, "mode": mode });
            if let Some(s) = symbol {
                body["symbol"] = json!(s);
            }
            if let Some(c) = coin {
                body["coin"] = json!(c);
            }
            client
                .private_post("/v5/position/switch-mode", &body)
                .await?
        }

        PositionCommand::SetTpsl {
            category,
            symbol,
            take_profit,
            stop_loss,
            trailing_stop,
            tp_trigger_by,
            sl_trigger_by,
            tp_size,
            sl_size,
            tp_limit_price,
            sl_limit_price,
            position_idx,
        } => {
            confirm(&format!("Set TP/SL on {symbol}?"), force)?;
            let mut body = json!({
                "category": category,
                "symbol": symbol,
                "positionIdx": position_idx,
            });
            if let Some(v) = take_profit {
                body["takeProfit"] = json!(v);
            }
            if let Some(v) = stop_loss {
                body["stopLoss"] = json!(v);
            }
            if let Some(v) = trailing_stop {
                body["trailingStop"] = json!(v);
            }
            if let Some(v) = tp_trigger_by {
                body["tpTriggerBy"] = json!(v);
            }
            if let Some(v) = sl_trigger_by {
                body["slTriggerBy"] = json!(v);
            }
            if let Some(v) = tp_size {
                body["tpSize"] = json!(v);
            }
            if let Some(v) = sl_size {
                body["slSize"] = json!(v);
            }
            if let Some(v) = tp_limit_price {
                body["tpLimitPrice"] = json!(v);
            }
            if let Some(v) = sl_limit_price {
                body["slLimitPrice"] = json!(v);
            }
            client
                .private_post("/v5/position/trading-stop", &body)
                .await?
        }

        PositionCommand::TrailingStop {
            category,
            symbol,
            trailing_stop,
            active_price,
            position_idx,
        } => {
            confirm(
                &format!("Set trailing stop of {trailing_stop} on {symbol}?"),
                force,
            )?;
            let mut body = json!({
                "category": category,
                "symbol": symbol,
                "trailingStop": trailing_stop,
                "positionIdx": position_idx,
            });
            if let Some(v) = active_price {
                body["activePrice"] = json!(v);
            }
            client
                .private_post("/v5/position/trading-stop", &body)
                .await?
        }

        PositionCommand::SetRiskLimit {
            category,
            symbol,
            risk_id,
            position_idx,
        } => {
            confirm(&format!("Set risk limit {risk_id} on {symbol}?"), force)?;
            let body = json!({
                "category": category,
                "symbol": symbol,
                "riskId": risk_id,
                "positionIdx": position_idx,
            });
            client
                .private_post("/v5/position/set-risk-limit", &body)
                .await?
        }

        PositionCommand::AddMargin {
            category,
            symbol,
            margin,
            position_idx,
        } => {
            confirm(&format!("Add/reduce margin {margin} on {symbol}?"), force)?;
            let body = json!({
                "category": category,
                "symbol": symbol,
                "margin": margin,
                "positionIdx": position_idx,
            });
            client
                .private_post("/v5/position/add-margin", &body)
                .await?
        }

        PositionCommand::ClosedPnl {
            category,
            symbol,
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
                .private_get("/v5/position/closed-pnl", &params)
                .await?
        }

        PositionCommand::Move {
            category,
            from_uid,
            to_uid,
            positions,
        } => {
            confirm(
                &format!("Move positions from UID {from_uid} to UID {to_uid}?"),
                force,
            )?;
            let parsed: Value = serde_json::from_str(&positions).map_err(|e| {
                crate::errors::BybitError::Parse(format!("invalid positions JSON: {e}"))
            })?;
            let body = json!({
                "category": category,
                "fromUid": from_uid,
                "toUid": to_uid,
                "list": parsed,
            });
            client
                .private_post("/v5/position/move-positions", &body)
                .await?
        }

        PositionCommand::MoveHistory {
            category,
            symbol,
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
                .private_get("/v5/position/move-history", &params)
                .await?
        }

        PositionCommand::Flatten { category, symbol } => {
            let target = symbol.as_deref().unwrap_or("ALL symbols");
            confirm(
                &format!("EMERGENCY: Cancel all orders and CLOSE all positions on {target} in {category}?"),
                force,
            )?;

            // 1. Cancel all orders
            eprintln!("Step 1: Cancelling all open orders...");
            let mut cancel_body = json!({ "category": category });
            if let Some(ref s) = symbol {
                cancel_body["symbol"] = json!(s);
            }
            let cancel_res = client
                .private_post("/v5/order/cancel-all", &cancel_body)
                .await?;
            eprintln!("Cancel result: {}", cancel_res);

            // 2. Get open positions
            eprintln!("Step 2: Fetching open positions...");
            let mut list_params = vec![("category", category.as_str())];
            if let Some(ref s) = symbol {
                list_params.push(("symbol", s));
            }
            let pos_res = client
                .private_get("/v5/position/list", &list_params)
                .await?;

            let positions = pos_res["list"].as_array().ok_or_else(|| {
                crate::errors::BybitError::Parse("Invalid position list response".to_string())
            })?;

            if positions.is_empty() {
                eprintln!("No open positions found.");
                return Ok(());
            }

            // 3. Close each position with a market order
            eprintln!("Step 3: Closing {} positions...", positions.len());
            for pos in positions {
                let s = pos["symbol"].as_str().unwrap_or_default();
                let side = pos["side"].as_str().unwrap_or_default();
                let size = pos["size"].as_str().unwrap_or_default();
                let pos_idx = pos["positionIdx"].as_u64().unwrap_or(0);

                if size == "0" {
                    continue;
                }

                // Determine closing side
                let close_side = if side == "Buy" { "Sell" } else { "Buy" };

                eprintln!("Closing {size} {s} ({side}) with {close_side} market order...");

                let order_body = json!({
                    "category": category,
                    "symbol": s,
                    "side": close_side,
                    "orderType": "Market",
                    "qty": size,
                    "positionIdx": pos_idx,
                    "reduceOnly": true
                });

                match client.private_post("/v5/order/create", &order_body).await {
                    Ok(res) => eprintln!("Close result for {s}: {}", res),
                    Err(e) => eprintln!("Failed to close {s}: {}", e),
                }
            }

            json!({ "status": "flatten_complete", "category": category, "symbol": symbol })
        }
    };

    print_output(&value, format);
    Ok(())
}
