use clap::Subcommand;
use serde_json::{json, Value};

use crate::client::BybitClient;
use crate::commands::helpers::confirm;
use crate::errors::BybitResult;
use crate::output::{print_output, OutputFormat};

#[derive(Debug, clap::Args)]
pub struct EarnArgs {
    #[command(subcommand)]
    pub command: EarnCommand,
}

#[derive(Debug, Subcommand)]
pub enum EarnCommand {
    /// List available earn products
    #[command(visible_aliases = ["product", "strategies"])]
    Products {
        /// Product category: FlexibleSaving or OnChain
        #[arg(long, default_value = "FlexibleSaving")]
        category: String,
        #[arg(long)]
        coin: Option<String>,
    },
    /// Get current staked positions
    #[command(visible_alias = "allocations")]
    Positions {
        /// Product category: FlexibleSaving or OnChain
        #[arg(long, default_value = "FlexibleSaving")]
        category: String,
        #[arg(long)]
        product_id: Option<String>,
        #[arg(long)]
        coin: Option<String>,
    },
    /// Stake into an earn product (dangerous)
    #[command(visible_alias = "allocate")]
    Stake {
        /// Product category: FlexibleSaving or OnChain
        #[arg(long, default_value = "FlexibleSaving")]
        category: String,
        /// FUND or UNIFIED. OnChain only supports FUND.
        #[arg(long, default_value = "FUND")]
        account_type: String,
        #[arg(long)]
        product_id: String,
        #[arg(long)]
        coin: String,
        #[arg(long)]
        amount: String,
        /// Custom client order ID. Auto-generated if omitted.
        #[arg(long)]
        order_link_id: Option<String>,
        /// Optional destination account type for supported OnChain redeems.
        #[arg(long)]
        to_account_type: Option<String>,
    },
    /// Redeem an earn position (dangerous)
    #[command(visible_aliases = ["unstake", "deallocate"])]
    Redeem {
        /// Product category: FlexibleSaving or OnChain
        #[arg(long, default_value = "FlexibleSaving")]
        category: String,
        /// FUND or UNIFIED. OnChain only supports FUND.
        #[arg(long, default_value = "FUND")]
        account_type: String,
        #[arg(long)]
        product_id: String,
        #[arg(long)]
        coin: String,
        #[arg(long)]
        amount: String,
        /// Custom client order ID. Auto-generated if omitted.
        #[arg(long)]
        order_link_id: Option<String>,
        /// Position ID required for OnChain non-LST redeem flows.
        #[arg(long)]
        redeem_position_id: Option<String>,
        /// Optional destination account type for supported OnChain LST redeems.
        #[arg(long)]
        to_account_type: Option<String>,
    },
    /// Get stake/redeem order history or status
    #[command(visible_aliases = ["order-history", "status", "allocate-status", "deallocate-status"])]
    History {
        /// Product category: FlexibleSaving or OnChain
        #[arg(long, default_value = "FlexibleSaving")]
        category: String,
        #[arg(long)]
        order_id: Option<String>,
        #[arg(long)]
        order_link_id: Option<String>,
        #[arg(long)]
        product_id: Option<String>,
        #[arg(long)]
        start: Option<u64>,
        #[arg(long)]
        end: Option<u64>,
        #[arg(long)]
        limit: Option<u32>,
        #[arg(long)]
        cursor: Option<String>,
    },
    /// Get distributed yield history
    Yield {
        /// Product category: FlexibleSaving or OnChain
        #[arg(long, default_value = "FlexibleSaving")]
        category: String,
        #[arg(long)]
        product_id: Option<String>,
        #[arg(long)]
        start: Option<u64>,
        #[arg(long)]
        end: Option<u64>,
        #[arg(long)]
        limit: Option<u32>,
        #[arg(long)]
        cursor: Option<String>,
    },
    /// Get hourly yield history
    HourlyYield {
        /// Product category. Bybit currently documents FlexibleSaving.
        #[arg(long, default_value = "FlexibleSaving")]
        category: String,
        #[arg(long)]
        product_id: Option<String>,
        #[arg(long)]
        start: Option<u64>,
        #[arg(long)]
        end: Option<u64>,
        #[arg(long)]
        limit: Option<u32>,
        #[arg(long)]
        cursor: Option<String>,
    },
}

pub async fn run(
    args: EarnArgs,
    client: &BybitClient,
    format: OutputFormat,
    force: bool,
) -> BybitResult<()> {
    let value: Value = match args.command {
        EarnCommand::Products { category, coin } => {
            let mut params = vec![("category", category.as_str())];
            if let Some(ref coin) = coin {
                params.push(("coin", coin));
            }
            client.public_get("/v5/earn/product", &params).await?
        }
        EarnCommand::Positions {
            category,
            product_id,
            coin,
        } => {
            let mut params = vec![("category", category.as_str())];
            if let Some(ref product_id) = product_id {
                params.push(("productId", product_id));
            }
            if let Some(ref coin) = coin {
                params.push(("coin", coin));
            }
            client.private_get("/v5/earn/position", &params).await?
        }
        EarnCommand::Stake {
            category,
            account_type,
            product_id,
            coin,
            amount,
            order_link_id,
            to_account_type,
        } => {
            confirm(
                &format!("Stake {amount} {coin} into earn product {product_id} ({category})?"),
                force,
            )?;
            let mut body = json!({
                "category": category,
                "orderType": "Stake",
                "accountType": account_type,
                "amount": amount,
                "coin": coin,
                "productId": product_id,
                "orderLinkId": order_link_id.unwrap_or_else(|| uuid::Uuid::new_v4().to_string()),
            });
            if let Some(to_account_type) = to_account_type {
                body["toAccountType"] = json!(to_account_type);
            }
            client.private_post("/v5/earn/place-order", &body).await?
        }
        EarnCommand::Redeem {
            category,
            account_type,
            product_id,
            coin,
            amount,
            order_link_id,
            redeem_position_id,
            to_account_type,
        } => {
            confirm(
                &format!("Redeem {amount} {coin} from earn product {product_id} ({category})?"),
                force,
            )?;
            let mut body = json!({
                "category": category,
                "orderType": "Redeem",
                "accountType": account_type,
                "amount": amount,
                "coin": coin,
                "productId": product_id,
                "orderLinkId": order_link_id.unwrap_or_else(|| uuid::Uuid::new_v4().to_string()),
            });
            if let Some(redeem_position_id) = redeem_position_id {
                body["redeemPositionId"] = json!(redeem_position_id);
            }
            if let Some(to_account_type) = to_account_type {
                body["toAccountType"] = json!(to_account_type);
            }
            client.private_post("/v5/earn/place-order", &body).await?
        }
        EarnCommand::History {
            category,
            order_id,
            order_link_id,
            product_id,
            start,
            end,
            limit,
            cursor,
        } => {
            let start_str = start.map(|value| value.to_string());
            let end_str = end.map(|value| value.to_string());
            let limit_str = limit.map(|value| value.to_string());
            let mut params = vec![("category", category.as_str())];
            if let Some(ref order_id) = order_id {
                params.push(("orderId", order_id));
            }
            if let Some(ref order_link_id) = order_link_id {
                params.push(("orderLinkId", order_link_id));
            }
            if let Some(ref product_id) = product_id {
                params.push(("productId", product_id));
            }
            if let Some(ref start) = start_str {
                params.push(("startTime", start));
            }
            if let Some(ref end) = end_str {
                params.push(("endTime", end));
            }
            if let Some(ref limit) = limit_str {
                params.push(("limit", limit));
            }
            if let Some(ref cursor) = cursor {
                params.push(("cursor", cursor));
            }
            client.private_get("/v5/earn/order", &params).await?
        }
        EarnCommand::Yield {
            category,
            product_id,
            start,
            end,
            limit,
            cursor,
        } => {
            let start_str = start.map(|value| value.to_string());
            let end_str = end.map(|value| value.to_string());
            let limit_str = limit.map(|value| value.to_string());
            let mut params = vec![("category", category.as_str())];
            if let Some(ref product_id) = product_id {
                params.push(("productId", product_id));
            }
            if let Some(ref start) = start_str {
                params.push(("startTime", start));
            }
            if let Some(ref end) = end_str {
                params.push(("endTime", end));
            }
            if let Some(ref limit) = limit_str {
                params.push(("limit", limit));
            }
            if let Some(ref cursor) = cursor {
                params.push(("cursor", cursor));
            }
            client.private_get("/v5/earn/yield", &params).await?
        }
        EarnCommand::HourlyYield {
            category,
            product_id,
            start,
            end,
            limit,
            cursor,
        } => {
            let start_str = start.map(|value| value.to_string());
            let end_str = end.map(|value| value.to_string());
            let limit_str = limit.map(|value| value.to_string());
            let mut params = vec![("category", category.as_str())];
            if let Some(ref product_id) = product_id {
                params.push(("productId", product_id));
            }
            if let Some(ref start) = start_str {
                params.push(("startTime", start));
            }
            if let Some(ref end) = end_str {
                params.push(("endTime", end));
            }
            if let Some(ref limit) = limit_str {
                params.push(("limit", limit));
            }
            if let Some(ref cursor) = cursor {
                params.push(("cursor", cursor));
            }
            client.private_get("/v5/earn/hourly-yield", &params).await?
        }
    };

    print_output(&value, format);
    Ok(())
}
