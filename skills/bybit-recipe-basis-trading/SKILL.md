---
name: bybit-recipe-basis-trading
version: 1.0.0
description: "Capture the price difference between Spot and Futures markets while maintaining a market-neutral position."
metadata:
  openclaw:
    category: "finance"
  requires:
    bins: ["bybit"]
---

# Recipe: Basis Trading

Basis trading involves capturing the spread between the **Spot Price** and the **Futures/Perpetual Price**. The goal is to profit from the spread narrowing (basis convergence) or from collecting funding payments on perpetuals.

## 1. Identify the Basis (Spread)

Calculate the difference between the Linear Perpetual price and the Spot price:

```bash
# Get Perpetual Price
PERP=$(bybit market tickers --category linear --symbol BTCUSDT -o json | jq -r '.list[0].lastPrice')

# Get Spot Price
SPOT=$(bybit market tickers --category spot --symbol BTCUSDT -o json | jq -r '.list[0].lastPrice')

# Calculate Basis %
BASIS=$(echo "scale=4; ($PERP - $SPOT) / $SPOT * 100" | bc)
echo "Current Basis: $BASIS %"
```

*   **Positive Basis (Contango):** Futures > Spot. Strategy: Buy Spot, Sell Futures.
*   **Negative Basis (Backwardation):** Spot > Futures. Strategy: Sell Spot (if held), Buy Futures.

## 2. Evaluate Funding Rate (for Perpetuals)

On Bybit, if you are short a perpetual with a **positive funding rate**, you collect payments from long holders every 8 hours.

```bash
bybit market tickers --category linear --symbol BTCUSDT -o json | jq -r '.list[0].fundingRate'
```

## 3. Enter the Trade (Cash & Carry)

To capture a positive basis:
1.  **Buy Spot BTC** (e.g., 0.1 BTC)
2.  **Short Linear Perpetual BTC** (0.1 BTC)

```bash
# Leg 1: Buy Spot
bybit trade buy --category spot --symbol BTCUSDT --qty 0.1 --order-type Market

# Leg 2: Short Perpetual
bybit trade sell --category linear --symbol BTCUSDT --qty 0.1 --order-type Market
```

## 4. Monitor the Position

The total "Delta" of your account for BTC should be near zero. Check your positions and balances:

```bash
# Check Spot Balance
bybit account balance --account-type UNIFIED --coin BTC

# Check Perpetual Position
bybit position list --category linear --symbol BTCUSDT
```

## 5. Exit the Trade

Exit when the basis narrows or funding becomes unfavorable:

```bash
# Leg 1: Sell Spot
bybit trade sell --category spot --symbol BTCUSDT --qty 0.1 --order-type Market

# Leg 2: Close Short (Buy back)
bybit trade buy --category linear --symbol BTCUSDT --qty 0.1 --order-type Market --reduce-only
```

## Safety Rules

*   **Execution Risk:** Prices can move between Leg 1 and Leg 2. Use Market orders for simultaneous entry or use the `batch-place` system if available for cross-category (Note: Bybit batch API is usually per-category).
*   **Liquidation Risk:** Even if the trade is "neutral," your Perpetual Short can still be liquidated if the price of BTC rallies significantly. Ensure you have enough USDT margin in your Unified account.
*   **Leg Risk:** Always verify both legs filled. If only one fills, use `bybit position flatten` or manual trade to neutralize.
