---
name: bybit-recipe-trailing-stop-runner
version: 1.0.0
description: "Enter a position and attach a dynamic trailing stop to lock in profits during a trend."
metadata:
  openclaw:
    category: "finance"
  requires:
    bins: ["bybit"]
---

# Recipe: Trailing Stop Runner

A trailing stop automatically follows the price as it moves in your favor, maintaining a set distance. If the price reverses by that distance, the position is closed.

## 1. Enter a Position

First, enter a market or limit position:

```bash
# Example: Long 0.1 BTC
bybit trade buy --symbol BTCUSDT --qty 0.1 --order-type Market
```

## 2. Attach the Trailing Stop

Use the `position trailing-stop` command to set the retracement distance.

```bash
# Trail by $500. If price hits $67,000, stop moves to $66,500.
bybit position trailing-stop --symbol BTCUSDT --trailing-stop 500
```

## 3. Optional: Add Activation Price

You can set the trailing stop to only "activate" once the price reaches a certain profit level:

```bash
# Only start trailing after price hits $68,000
bybit position trailing-stop --symbol BTCUSDT --trailing-stop 500 --active-price 68000
```

## 4. Monitor the Stop

You can see the current trailing stop status in your position list:

```bash
bybit position list --symbol BTCUSDT -o json | jq '.list[0] | {symbol, size, avgPrice, trailingStop}'
```

## 5. Automation Pattern (Agent Logic)

The agent can also simulate a trailing stop manually by updating a standard Stop-Loss as the price hits "high-water marks":

1.  **Monitor Price:** `bybit ws ticker --symbol BTCUSDT`
2.  **Update High-Water Mark:** If `current_price > high_water_mark`, then `high_water_mark = current_price`.
3.  **Update Stop-Loss:** If `high_water_mark - current_price > trail_distance`, then `bybit position flatten`. (Or update the `stop-loss` parameter).

## Safety Rules

*   **One-Way vs Hedge:** Ensure you use the correct `--position-idx` (0 for one-way, 1 for long-hedge, 2 for short-hedge).
*   **Market Exit:** Trailing stops on Bybit always trigger **Market Orders** to ensure an exit. Be aware of slippage in low-liquidity markets.
*   **Removal:** To remove a trailing stop, set it to "0": `bybit position trailing-stop --symbol BTCUSDT --trailing-stop 0`.
