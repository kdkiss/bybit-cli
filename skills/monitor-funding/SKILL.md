---
name: monitor-funding
version: 1.0.0
description: "Check current and historical funding rates for perpetual contracts."
metadata:
  openclaw:
    category: "finance"
  requires:
    bins: ["bybit"]
---

# Skill: Monitor Funding Rate

Check current and historical funding rates for perpetual contracts.

## Current funding rate (in ticker)

```bash
# Funding rate is in the ticker response
bybit market tickers --category linear --symbol BTCUSDT -o json \
  | jq '{fundingRate: .list[0].fundingRate, nextFundingTime: .list[0].nextFundingTime}'
```

## Funding rate history

```bash
# Last 10 funding periods
bybit market funding-rate --category linear --symbol BTCUSDT --limit 10

# Date range (Unix ms)
bybit market funding-rate --category linear --symbol BTCUSDT \
  --start 1700000000000 --end 1700100000000

# JSON output
bybit market funding-rate --category linear --symbol BTCUSDT --limit 20 -o json
```

## Real-time funding via WebSocket

```bash
# Stream ticker (includes live funding rate updates)
bybit ws ticker --category linear --symbol BTCUSDT
```

## Funding rate interpretation

| Rate | Meaning |
|------|---------|
| Positive (e.g., 0.01%) | Longs pay shorts. Market is bullish/leveraged long. |
| Negative (e.g., -0.01%) | Shorts pay longs. Market is bearish/leveraged short. |
| Near zero | Balanced market. |

Funding is paid every 8 hours on Bybit linear perpetuals.

## Check funding for multiple symbols

```bash
# Get all linear tickers and sort by funding rate
bybit market tickers --category linear -o json \
  | jq '[.list[] | {symbol, fundingRate: .fundingRate}] | sort_by(.fundingRate) | reverse'
```
