---
name: stream-orderbook
version: 1.0.0
description: "Stream real-time order book updates via WebSocket."
metadata:
  openclaw:
    category: "finance"
  requires:
    bins: ["bybit"]
---

# Skill: Stream Order Book

Stream real-time order book updates via WebSocket.

## Commands

```bash
# Stream order book (default depth 50)
bybit ws orderbook --category linear --symbol BTCUSDT

# Shallower book (faster, less data)
bybit ws orderbook --category linear --symbol BTCUSDT --depth 25

# Spot order book
bybit ws orderbook --category spot --symbol BTCUSDT

# Options order book
bybit ws orderbook --category option --symbol BTC-29NOV24-50000-C
```

## What you see

Each update is printed as JSON:
```json
{
  "topic": "orderbook.50.BTCUSDT",
  "type": "snapshot",
  "data": {
    "s": "BTCUSDT",
    "b": [["50000.00", "1.234"], ...],
    "a": [["50001.00", "0.567"], ...]
  }
}
```

- `b`: bids `[price, qty]` sorted descending
- `a`: asks `[price, qty]` sorted ascending
- `type`: `snapshot` (full book) then `delta` (incremental updates)

## Other streams

```bash
# Real-time ticker
bybit ws ticker --category linear --symbol BTCUSDT

# Public trade feed
bybit ws trades --category linear --symbol BTCUSDT

# Kline/candlestick stream
bybit ws kline --category linear --symbol BTCUSDT --interval 1

# Liquidation feed
bybit ws liquidation --category linear --symbol BTCUSDT
```

## Reconnection

The CLI automatically reconnects on disconnection with exponential backoff (up to 12 attempts). Press `Ctrl+C` to stop.
