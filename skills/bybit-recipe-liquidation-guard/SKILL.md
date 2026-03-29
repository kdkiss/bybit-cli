---
name: bybit-recipe-liquidation-guard
version: 1.0.0
description: "Monitor account margin and automatically flatten positions if risk exceeds a threshold."
metadata:
  openclaw:
    category: "finance"
  requires:
    bins: ["bybit"]
---

# Recipe: Liquidation Guard

Protect your account from catastrophic loss by monitoring the Maintenance Margin (MM) ratio and executing an emergency exit if risk thresholds are breached.

## 1. Monitor Risk Level

The most important metric in a Bybit Unified Trading Account (UTA) is the `accountMMRate` (Maintenance Margin Rate).

*   **MM Rate < 50%**: Healthy.
*   **MM Rate 50% - 80%**: Elevated risk.
*   **MM Rate > 90%**: High risk. Liquidation is imminent.

### Check current risk

```bash
bybit account balance --account-type UNIFIED -o json | jq -r '.list[0].accountMMRate'
```

## 2. Setting the Circuit Breaker

The agent should check this value periodically. If it exceeds your defined threshold (e.g., `0.80`), trigger the flatten procedure.

## 3. Emergency Flatten (The "Panic Button")

If the threshold is hit, execute the `flatten` command immediately. This command:
1.  Cancels all open orders in the category.
2.  Fetches all open positions.
3.  Places Market orders to close every position.

```bash
# Flatten all linear perpetual positions
bybit position flatten --category linear -y
```

## 4. Automation Pattern

Use a simple loop to check risk and exit:

```bash
while true; do
  MM_RATE=$(bybit account balance -o json | jq -r '.list[0].accountMMRate')
  echo "Current MM Rate: $MM_RATE"
  
  # Trigger if MM Rate > 0.85
  if (( $(echo "$MM_RATE > 0.85" | bc -l) )); then
    echo "THRESHOLD BREACHED! Flattening..."
    bybit position flatten --category linear -y
    break
  fi
  sleep 30
done
```

## Safety Rules

*   **Always use `-y`** in the flatten command for automated guards to avoid waiting for human confirmation during a crash.
*   **Monitor specific symbols** if you only want to protect one part of your portfolio: `bybit position flatten --symbol BTCUSDT -y`.
