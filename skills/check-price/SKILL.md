---
name: check-price
version: 1.0.0
description: "Get the current price, 24h volume, and stats for any symbol."
metadata:
  openclaw:
    category: "finance"
  requires:
    bins: ["bybit"]
---

# Skill: Check Price

Get the current price, 24h volume, and stats for any symbol.

## Commands

```bash
# Latest ticker for a linear perpetual
bybit market tickers --category linear --symbol BTCUSDT

# Spot price
bybit market tickers --category spot --symbol BTCUSDT

# JSON for scripting
bybit market tickers --category linear --symbol BTCUSDT -o json

# Order book snapshot (top 5 levels)
bybit market orderbook --category linear --symbol BTCUSDT --limit 5

# Recent trades
bybit market trades --category linear --symbol BTCUSDT --limit 10
```

## Key Fields (JSON output)

| Field | Description |
|-------|-------------|
| `lastPrice` | Last traded price |
| `markPrice` | Mark price (used for liquidation) |
| `indexPrice` | Underlying index price |
| `bid1Price` / `ask1Price` | Best bid/ask |
| `price24hPcnt` | 24h price change % |
| `volume24h` | 24h trading volume |
| `openInterestValue` | Open interest in USD |
| `fundingRate` | Current funding rate |
| `nextFundingTime` | Next funding timestamp (ms) |

## Example: Extract last price

```bash
bybit market tickers --category linear --symbol BTCUSDT -o json \
  | jq -r '.list[0].lastPrice'
```
