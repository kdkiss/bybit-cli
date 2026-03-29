---
name: bybit-recipe-emergency-flatten
version: 1.0.0
description: "Cancel all orders and close all positions immediately in an emergency."
metadata:
  openclaw:
    category: "finance"
  requires:
    bins: ["bybit"]
---

# Recipe: Emergency Flatten

Use this recipe when market conditions become extreme or an automated strategy malfunctions. This procedure rapidly neutralizes all market exposure.

## 1. Fast Execute

Run the `flatten` command for the most active categories. This command handles both order cancellation and position closing in one atomic step.

```bash
# Flatten all linear perpetuals (most common)
bybit position flatten --category linear -y

# Flatten spot positions (requires caution as it sells assets)
bybit position flatten --category spot -y
```

## 2. Verify Neutral State

Ensure no open positions or orders remain:

```bash
# Check for any remaining linear positions
bybit position list --category linear -o json | jq '.list | length'

# Check for any remaining orders
bybit trade open-orders --category linear -o json | jq '.list | length'
```

## 3. Disconnect Automation

If you are running an automated agent, stop the process or use the "Dead Man's Switch":

```bash
# Set all orders to cancel if not refreshed within 10 seconds
bybit trade cancel-after 10
```

## 4. Move to Safety

Transfer remaining USDT/USDC to the **Funding Account** to prevent accidental margin usage or to prepare for withdrawal:

```bash
bybit asset transfer \
  --coin USDT \
  --amount <ALL> \
  --from-account-type UNIFIED \
  --to-account-type FUND -y
```

## Hard Rules

*   **Atomic Action:** Prioritize `position flatten` over individual `cancel-all` and `close` commands.
*   **Confirmation Bypass:** Always use `-y` in a true emergency to save critical seconds.
*   **Post-Mortem:** After flattening, check your `trade fills` to document the exit prices and realized P&L.
