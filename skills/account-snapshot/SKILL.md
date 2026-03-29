---
name: account-snapshot
version: 1.0.0
description: "Get a complete view of your account: balance, positions, and recent activity."
metadata:
  openclaw:
    category: "finance"
  requires:
    bins: ["bybit"]
---

# Skill: Account Snapshot

Get a complete view of your account: balance, positions, and recent activity.

## Commands

```bash
# Wallet balance (Unified account)
bybit account balance --account-type UNIFIED

# Specific coin balance
bybit account balance --account-type UNIFIED --coin USDT

# Account info (UID, margin mode, status)
bybit account info

# Fee rates
bybit account fee-rate --category linear

# Recent transactions (deposits, withdrawals, P&L, fees)
bybit account transaction-log --account-type UNIFIED --limit 50

# Open positions
bybit position list --category linear

# Open orders
bybit trade open-orders --category linear

# Asset balances across all account types
bybit asset all-balance --account-type UNIFIED
```

## JSON snapshot script

```bash
#!/bin/bash
echo "=== Balance ===" && bybit account balance -o json
echo "=== Positions ===" && bybit position list --category linear -o json
echo "=== Open Orders ===" && bybit trade open-orders --category linear -o json
```

## Key balance fields

| Field | Description |
|-------|-------------|
| `totalWalletBalance` | Total wallet balance in USD |
| `totalAvailableBalance` | Available (unreserved) balance |
| `totalUnrealisedPnl` | Total unrealized P&L across positions |
| `totalMarginBalance` | Total margin balance |
