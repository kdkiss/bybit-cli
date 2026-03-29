---
name: bybit-recipe-drawdown-circuit-breaker
version: 1.0.0
description: "Automatically stop trading and flatten positions if portfolio drawdown exceeds a threshold."
metadata:
  openclaw:
    category: "finance"
  requires:
    bins: ["bybit"]
---

# Recipe: Drawdown Circuit Breaker

Protect your capital by defining a "Max Drawdown" limit. If your account equity falls below this level, the agent will automatically exit all trades and stop.

## 1. Define High-Water Mark (HWM)

Record your starting balance or highest previous balance:

```bash
START_BALANCE=$(bybit account balance -o json | jq -r '.list[0].totalWalletBalance')
echo "High-Water Mark: $START_BALANCE"
```

## 2. Set Drawdown Threshold

Define how much you are willing to lose from the HWM (e.g., 5%):

```bash
MAX_LOSS_PCT=0.05
CIRCUIT_BREAKER_LEVEL=$(echo "$START_BALANCE * (1 - $MAX_LOSS_PCT)" | bc -l)
echo "Circuit Breaker Level: $CIRCUIT_BREAKER_LEVEL"
```

## 3. Monitor Equity

The agent should check the `totalMarginBalance` (which includes unrealized P&L) every 30-60 seconds:

```bash
while true; do
  CURRENT_EQUITY=$(bybit account balance -o json | jq -r '.list[0].totalMarginBalance')
  echo "Current Equity: $CURRENT_EQUITY"
  
  if (( $(echo "$CURRENT_EQUITY < $CIRCUIT_BREAKER_LEVEL" | bc -l) )); then
    echo "CIRCUIT BREAKER TRIGGERED! Drawdown exceeded."
    bybit position flatten --category linear -y
    bybit position flatten --category spot -y
    break
  fi
  sleep 60
done
```

## 4. Resetting the Breaker

Only resume trading after performing a manual review of your strategy. Update the High-Water Mark to your new balance before restarting the guard.

## Safety Rules

*   **Inclusive Valuation:** Use `totalMarginBalance` rather than `totalWalletBalance` to account for open trade losses.
*   **Subaccount Isolation:** If running multiple strategies, use separate subaccounts and run a dedicated circuit breaker for each UID.
*   **Emergency Contact:** If the breaker triggers, the agent should immediately notify the user (e.g., via Slack/Discord if integrated, or local terminal alert).
