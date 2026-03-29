---
name: bybit-recipe-twap-execution
version: 1.0.0
description: "Execute large orders as time-weighted slices to reduce market impact and slippage."
metadata:
  openclaw:
    category: "finance"
  requires:
    bins: ["bybit"]
---

# Recipe: TWAP Execution

Time-Weighted Average Price (TWAP) splits a large order into N equal slices executed at regular intervals. This reduces the impact a single large order would have on the order book.

## 1. Plan the TWAP

*   **Total Volume:** 1.0 BTC
*   **Slices:** 10
*   **Interval:** 5 minutes (300 seconds)
*   **Slice Volume:** 0.1 BTC per slice

## 2. Execute with Market Orders (Simple)

The agent runs a loop to place market orders:

```bash
# Example logic for 10 slices of 0.1 BTC every 300s
for i in {1..10}; do
  echo "Executing slice $i of 10..."
  bybit trade buy --symbol BTCUSDT --qty 0.1 --order-type Market -y
  
  if [ $i -lt 10 ]; then
    echo "Waiting 300 seconds..."
    sleep 300
  fi
done
```

## 3. Execute with Limit Orders (Optimized)

For better fills, use limit orders at the current best bid/ask:

1.  **Get current price:**
    ```bash
    PRICE=$(bybit market tickers --symbol BTCUSDT -o json | jq -r '.list[0].bid1Price')
    ```
2.  **Place limit order:**
    ```bash
    bybit trade buy --symbol BTCUSDT --qty 0.1 --price $PRICE --post-only -y
    ```
3.  **Wait and check:** If not filled by the next slice interval, cancel and market-fill the remainder.

## 4. Track Fills and Average Price

After execution, calculate the VWAP (Volume-Weighted Average Price) from the fills:

```bash
bybit trade fills --symbol BTCUSDT --limit 20 -o json | jq '[.list[] | {qty: .execQty, price: .execPrice}]'
```

## 5. Paper Test TWAP

Always test your TWAP logic in paper mode first:

```bash
bybit paper init --usdt 100000
bybit paper buy --symbol BTCUSDT --qty 0.1
# wait...
bybit paper buy --symbol BTCUSDT --qty 0.1
bybit paper status
```

## Safety Rules

*   **Human Approval:** In live trading, each slice should typically be approved by a human unless the agent has high autonomy.
*   **Max Slippage:** Define a maximum price deviation. If the price moves >2% from the start of the TWAP, pause and alert the user.
*   **Connectivity:** Always run `bybit trade cancel-after 600` before starting a long TWAP loop to ensure unfilled limit orders are cleared if the script crashes.
