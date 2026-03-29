---
name: bybit-recipe-grid-trading
version: 1.0.0
description: "Deploy a grid trading strategy using layered limit orders to profit from sideways market volatility."
metadata:
  openclaw:
    category: "finance"
  requires:
    bins: ["bybit"]
---

# Recipe: Grid Trading

Grid trading involves placing a "grid" of buy and sell orders at regular intervals around a base price. It is most effective in sideways (ranging) markets.

## 1. Define the Grid Parameters

*   **Range High:** $70,000 (Top of the range)
*   **Range Low:** $60,000 (Bottom of the range)
*   **Number of Grids:** 10
*   **Quantity per Grid:** 0.01 BTC

## 2. Calculate the Grid Spacing

Calculate the price interval between grids:
`Spacing = (Range High - Range Low) / Number of Grids`
Example: `(70000 - 60000) / 10 = 1000`

## 3. Place Initial Batch Buy Orders

Place buy orders at intervals below the current price ($66,000):

```bash
bybit trade batch-place --category linear --orders '[
  {"symbol":"BTCUSDT","side":"Buy","orderType":"Limit","qty":"0.01","price":"65000","timeInForce":"GTC"},
  {"symbol":"BTCUSDT","side":"Buy","orderType":"Limit","qty":"0.01","price":"64000","timeInForce":"GTC"},
  {"symbol":"BTCUSDT","side":"Buy","orderType":"Limit","qty":"0.01","price":"63000","timeInForce":"GTC"},
  {"symbol":"BTCUSDT","side":"Buy","orderType":"Limit","qty":"0.01","price":"62000","timeInForce":"GTC"},
  {"symbol":"BTCUSDT","side":"Buy","orderType":"Limit","qty":"0.01","price":"61000","timeInForce":"GTC"}
]'
```

## 4. Place Initial Batch Sell Orders

Place sell orders (take-profits) at intervals above the current price:

```bash
bybit trade batch-place --category linear --orders '[
  {"symbol":"BTCUSDT","side":"Sell","orderType":"Limit","qty":"0.01","price":"67000","timeInForce":"GTC"},
  {"symbol":"BTCUSDT","side":"Sell","orderType":"Limit","qty":"0.01","price":"68000","timeInForce":"GTC"},
  {"symbol":"BTCUSDT","side":"Sell","orderType":"Limit","qty":"0.01","price":"69000","timeInForce":"GTC"},
  {"symbol":"BTCUSDT","side":"Sell","orderType":"Limit","qty":"0.01","price":"70000","timeInForce":"GTC"}
]'
```

## 5. Monitoring & Re-gridding

The agent should monitor for fills via WebSocket:

```bash
bybit ws executions
```

**Logic for the Agent:**
1.  If a **Buy Order** at $65,000 is filled:
    *   Immediately place a new **Sell Order** at $66,000 (one grid spacing above).
2.  If a **Sell Order** at $67,000 is filled:
    *   Immediately place a new **Buy Order** at $66,000 (one grid spacing below).

## Safety Rules

*   **Stop Loss:** Always define a "Hard Stop" outside your grid range. If the price hits $59,000, use `bybit position flatten --category linear -y`.
*   **Inventory Risk:** Grid trading can leave you with a directional position if the market trends hard in one direction. Monitor your `bybit position list`.
*   **Fees:** Ensure your grid spacing is larger than your round-trip trading fees (`bybit account fee-rate`).
