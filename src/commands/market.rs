use clap::Subcommand;
use serde_json::{json, Value};

use crate::client::BybitClient;
use crate::errors::BybitResult;
use crate::output::{print_output, OutputFormat};

#[allow(clippy::too_many_arguments)]
async fn kline_request(
    client: &BybitClient,
    endpoint: &str,
    category: &str,
    symbol: &str,
    interval: &str,
    start: Option<u64>,
    end: Option<u64>,
    limit: Option<u32>,
) -> BybitResult<Value> {
    let start_str = start.map(|v| v.to_string());
    let end_str = end.map(|v| v.to_string());
    let limit_str = limit.map(|v| v.to_string());
    let mut params = vec![
        ("category", category),
        ("symbol", symbol),
        ("interval", interval),
    ];
    if let Some(ref s) = start_str {
        params.push(("start", s));
    }
    if let Some(ref s) = end_str {
        params.push(("end", s));
    }
    if let Some(ref s) = limit_str {
        params.push(("limit", s));
    }
    client.public_get(endpoint, &params).await
}

#[derive(Debug, clap::Args)]
pub struct MarketArgs {
    #[command(subcommand)]
    pub command: MarketCommand,
}

#[derive(Debug, Subcommand)]
pub enum MarketCommand {
    /// Get server time
    ServerTime,

    /// Get tradeable instruments
    Instruments {
        /// Asset category: spot, linear, inverse, option
        #[arg(long, default_value = "linear")]
        category: String,
        /// Filter by symbol
        #[arg(long)]
        symbol: Option<String>,
        /// Filter by status
        #[arg(long)]
        status: Option<String>,
        /// Filter by base coin
        #[arg(long)]
        base_coin: Option<String>,
        /// Max results per page
        #[arg(long)]
        limit: Option<u32>,
        /// Pagination cursor
        #[arg(long)]
        cursor: Option<String>,
    },

    /// Get order book depth
    Orderbook {
        #[arg(long, default_value = "linear")]
        category: String,
        #[arg(long)]
        symbol: String,
        /// Depth (1/50/200/500 depending on category)
        #[arg(long, default_value = "50")]
        limit: u32,
    },

    /// Get tickers / 24h stats
    Tickers {
        #[arg(long, default_value = "linear")]
        category: String,
        #[arg(long)]
        symbol: Option<String>,
        #[arg(long)]
        base_coin: Option<String>,
        #[arg(long)]
        exp_date: Option<String>,
    },

    /// Get OHLCV kline data
    Kline {
        #[arg(long, default_value = "linear")]
        category: String,
        #[arg(long)]
        symbol: String,
        /// Interval: 1 3 5 15 30 60 120 240 360 720 D W M
        #[arg(long, default_value = "60")]
        interval: String,
        #[arg(long)]
        start: Option<u64>,
        #[arg(long)]
        end: Option<u64>,
        #[arg(long)]
        limit: Option<u32>,
    },

    /// Get mark price kline
    MarkPriceKline {
        #[arg(long, default_value = "linear")]
        category: String,
        #[arg(long)]
        symbol: String,
        #[arg(long, default_value = "60")]
        interval: String,
        #[arg(long)]
        start: Option<u64>,
        #[arg(long)]
        end: Option<u64>,
        #[arg(long)]
        limit: Option<u32>,
    },

    /// Get index price kline
    IndexPriceKline {
        #[arg(long, default_value = "linear")]
        category: String,
        #[arg(long)]
        symbol: String,
        #[arg(long, default_value = "60")]
        interval: String,
        #[arg(long)]
        start: Option<u64>,
        #[arg(long)]
        end: Option<u64>,
        #[arg(long)]
        limit: Option<u32>,
    },

    /// Get premium index price kline
    PremiumIndexKline {
        #[arg(long, default_value = "linear")]
        category: String,
        #[arg(long)]
        symbol: String,
        #[arg(long, default_value = "60")]
        interval: String,
        #[arg(long)]
        start: Option<u64>,
        #[arg(long)]
        end: Option<u64>,
        #[arg(long)]
        limit: Option<u32>,
    },

    /// Get funding rate history (linear/inverse only)
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

    /// Get recent public trades
    Trades {
        #[arg(long, default_value = "linear")]
        category: String,
        #[arg(long)]
        symbol: Option<String>,
        #[arg(long)]
        base_coin: Option<String>,
        #[arg(long)]
        option_type: Option<String>,
        #[arg(long)]
        limit: Option<u32>,
    },

    /// Get open interest
    OpenInterest {
        #[arg(long, default_value = "linear")]
        category: String,
        #[arg(long)]
        symbol: String,
        /// Interval time: 5min 15min 30min 1h 4h 1d
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

    /// Get historical volatility (option only)
    Volatility {
        #[arg(long, default_value = "option")]
        category: String,
        #[arg(long)]
        base_coin: Option<String>,
        #[arg(long)]
        period: Option<u32>,
        #[arg(long)]
        start: Option<u64>,
        #[arg(long)]
        end: Option<u64>,
    },

    /// Get insurance pool data
    Insurance {
        #[arg(long)]
        coin: Option<String>,
    },

    /// Get risk limit info for a symbol
    RiskLimit {
        #[arg(long, default_value = "linear")]
        category: String,
        #[arg(long)]
        symbol: Option<String>,
        #[arg(long)]
        cursor: Option<String>,
    },

    /// Get delivery price
    DeliveryPrice {
        #[arg(long, default_value = "option")]
        category: String,
        #[arg(long)]
        symbol: Option<String>,
        #[arg(long)]
        base_coin: Option<String>,
        #[arg(long)]
        limit: Option<u32>,
        #[arg(long)]
        cursor: Option<String>,
    },

    /// Get long/short ratio
    LsRatio {
        #[arg(long, default_value = "linear")]
        category: String,
        #[arg(long)]
        symbol: String,
        /// 5min, 15min, 30min, 1h, 4h, 1d
        #[arg(long)]
        period: String,
        #[arg(long)]
        limit: Option<u32>,
    },
    /// Get current bid-ask spread for a symbol
    Spread {
        #[arg(long, default_value = "linear")]
        category: String,
        #[arg(long)]
        symbol: String,
    },
}

pub async fn run(args: MarketArgs, client: &BybitClient, format: OutputFormat) -> BybitResult<()> {
    let value: Value = match args.command {
        MarketCommand::ServerTime => client.public_get("/v5/market/time", &[]).await?,

        MarketCommand::Instruments {
            category,
            symbol,
            status,
            base_coin,
            limit,
            cursor,
        } => {
            let limit_str = limit.map(|l| l.to_string());
            let mut params = vec![("category", category.as_str())];
            if let Some(ref s) = symbol {
                params.push(("symbol", s));
            }
            if let Some(ref s) = status {
                params.push(("status", s));
            }
            if let Some(ref s) = base_coin {
                params.push(("baseCoin", s));
            }
            if let Some(ref s) = limit_str {
                params.push(("limit", s));
            }
            if let Some(ref s) = cursor {
                params.push(("cursor", s));
            }
            client
                .public_get("/v5/market/instruments-info", &params)
                .await?
        }

        MarketCommand::Orderbook {
            category,
            symbol,
            limit,
        } => {
            let limit_str = limit.to_string();
            let params = vec![
                ("category", category.as_str()),
                ("symbol", symbol.as_str()),
                ("limit", limit_str.as_str()),
            ];
            client.public_get("/v5/market/orderbook", &params).await?
        }

        MarketCommand::Tickers {
            category,
            symbol,
            base_coin,
            exp_date,
        } => {
            let mut params = vec![("category", category.as_str())];
            if let Some(ref s) = symbol {
                params.push(("symbol", s));
            }
            if let Some(ref s) = base_coin {
                params.push(("baseCoin", s));
            }
            if let Some(ref s) = exp_date {
                params.push(("expDate", s));
            }
            client.public_get("/v5/market/tickers", &params).await?
        }

        MarketCommand::Kline {
            category,
            symbol,
            interval,
            start,
            end,
            limit,
        } => {
            kline_request(
                client,
                "/v5/market/kline",
                &category,
                &symbol,
                &interval,
                start,
                end,
                limit,
            )
            .await?
        }

        MarketCommand::MarkPriceKline {
            category,
            symbol,
            interval,
            start,
            end,
            limit,
        } => {
            kline_request(
                client,
                "/v5/market/mark-price-kline",
                &category,
                &symbol,
                &interval,
                start,
                end,
                limit,
            )
            .await?
        }

        MarketCommand::IndexPriceKline {
            category,
            symbol,
            interval,
            start,
            end,
            limit,
        } => {
            kline_request(
                client,
                "/v5/market/index-price-kline",
                &category,
                &symbol,
                &interval,
                start,
                end,
                limit,
            )
            .await?
        }

        MarketCommand::PremiumIndexKline {
            category,
            symbol,
            interval,
            start,
            end,
            limit,
        } => {
            kline_request(
                client,
                "/v5/market/premium-index-price-kline",
                &category,
                &symbol,
                &interval,
                start,
                end,
                limit,
            )
            .await?
        }

        MarketCommand::FundingRate {
            category,
            symbol,
            start,
            end,
            limit,
        } => {
            let start_str = start.map(|s| s.to_string());
            let end_str = end.map(|e| e.to_string());
            let limit_str = limit.map(|l| l.to_string());
            let mut params = vec![("category", category.as_str()), ("symbol", symbol.as_str())];
            if let Some(ref s) = start_str {
                params.push(("startTime", s));
            }
            if let Some(ref s) = end_str {
                params.push(("endTime", s));
            }
            if let Some(ref s) = limit_str {
                params.push(("limit", s));
            }
            client
                .public_get("/v5/market/funding/history", &params)
                .await?
        }

        MarketCommand::Trades {
            category,
            symbol,
            base_coin,
            option_type,
            limit,
        } => {
            let limit_str = limit.map(|l| l.to_string());
            let mut params = vec![("category", category.as_str())];
            if let Some(ref s) = symbol {
                params.push(("symbol", s));
            }
            if let Some(ref s) = base_coin {
                params.push(("baseCoin", s));
            }
            if let Some(ref s) = option_type {
                params.push(("optionType", s));
            }
            if let Some(ref s) = limit_str {
                params.push(("limit", s));
            }
            client
                .public_get("/v5/market/recent-trade", &params)
                .await?
        }

        MarketCommand::OpenInterest {
            category,
            symbol,
            interval_time,
            start,
            end,
            limit,
            cursor,
        } => {
            let start_str = start.map(|s| s.to_string());
            let end_str = end.map(|e| e.to_string());
            let limit_str = limit.map(|l| l.to_string());
            let mut params = vec![
                ("category", category.as_str()),
                ("symbol", symbol.as_str()),
                ("intervalTime", interval_time.as_str()),
            ];
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
                .public_get("/v5/market/open-interest", &params)
                .await?
        }

        MarketCommand::Volatility {
            category,
            base_coin,
            period,
            start,
            end,
        } => {
            let period_str = period.map(|p| p.to_string());
            let start_str = start.map(|s| s.to_string());
            let end_str = end.map(|e| e.to_string());
            let mut params = vec![("category", category.as_str())];
            if let Some(ref s) = base_coin {
                params.push(("baseCoin", s));
            }
            if let Some(ref s) = period_str {
                params.push(("period", s));
            }
            if let Some(ref s) = start_str {
                params.push(("startTime", s));
            }
            if let Some(ref s) = end_str {
                params.push(("endTime", s));
            }
            client
                .public_get("/v5/market/historical-volatility", &params)
                .await?
        }

        MarketCommand::Insurance { coin } => {
            let mut params: Vec<(&str, &str)> = vec![];
            if let Some(ref c) = coin {
                params.push(("coin", c));
            }
            client.public_get("/v5/market/insurance", &params).await?
        }

        MarketCommand::RiskLimit {
            category,
            symbol,
            cursor,
        } => {
            let mut params = vec![("category", category.as_str())];
            if let Some(ref s) = symbol {
                params.push(("symbol", s));
            }
            if let Some(ref s) = cursor {
                params.push(("cursor", s));
            }
            client.public_get("/v5/market/risk-limit", &params).await?
        }

        MarketCommand::DeliveryPrice {
            category,
            symbol,
            base_coin,
            limit,
            cursor,
        } => {
            let limit_str = limit.map(|l| l.to_string());
            let mut params = vec![("category", category.as_str())];
            if let Some(ref s) = symbol {
                params.push(("symbol", s));
            }
            if let Some(ref s) = base_coin {
                params.push(("baseCoin", s));
            }
            if let Some(ref s) = limit_str {
                params.push(("limit", s));
            }
            if let Some(ref s) = cursor {
                params.push(("cursor", s));
            }
            client
                .public_get("/v5/market/delivery-price", &params)
                .await?
        }

        MarketCommand::LsRatio {
            category,
            symbol,
            period,
            limit,
        } => {
            let limit_str = limit.map(|l| l.to_string());
            let mut params = vec![
                ("category", category.as_str()),
                ("symbol", symbol.as_str()),
                ("period", period.as_str()),
            ];
            if let Some(ref s) = limit_str {
                params.push(("limit", s));
            }
            client
                .public_get("/v5/market/account-ratio", &params)
                .await?
        }

        MarketCommand::Spread { category, symbol } => {
            let params = [("category", category.as_str()), ("symbol", symbol.as_str())];
            let res = client.public_get("/v5/market/tickers", &params).await?;

            let ticker = res["list"]
                .as_array()
                .and_then(|l| l.first())
                .ok_or_else(|| crate::errors::BybitError::Parse("Symbol not found".to_string()))?;

            let bid: f64 = ticker["bid1Price"]
                .as_str()
                .ok_or_else(|| crate::errors::BybitError::Parse("bid1Price missing".to_string()))?
                .parse()
                .map_err(|_| {
                    crate::errors::BybitError::Parse("bid1Price is not a number".to_string())
                })?;
            let ask: f64 = ticker["ask1Price"]
                .as_str()
                .ok_or_else(|| crate::errors::BybitError::Parse("ask1Price missing".to_string()))?
                .parse()
                .map_err(|_| {
                    crate::errors::BybitError::Parse("ask1Price is not a number".to_string())
                })?;
            let spread = ask - bid;
            let mid = (ask + bid) / 2.0;
            let spread_pct = if mid > 0.0 {
                (spread / mid) * 100.0
            } else {
                0.0
            };

            json!({
                "symbol": symbol,
                "bid": bid,
                "ask": ask,
                "spread": spread,
                "spread_pct": format!("{:.4}%", spread_pct),
                "mid_price": mid
            })
        }
    };

    print_output(&value, format);
    Ok(())
}
