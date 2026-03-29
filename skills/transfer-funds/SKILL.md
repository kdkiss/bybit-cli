---
name: transfer-funds
version: 1.0.0
description: "Move assets between Bybit account types or UIDs."
metadata:
  openclaw:
    category: "finance"
  requires:
    bins: ["bybit"]
---

# Skill: Transfer Funds

Move assets between Bybit account types or UIDs.

## Account types

| Type | Description |
|------|-------------|
| `UNIFIED` | Unified Trading Account (default for trading) |
| `FUND` | Funding account (for deposits/withdrawals) |
| `SPOT` | Spot account (legacy) |
| `CONTRACT` | Derivatives account (legacy) |

## Check transferable coins

```bash
bybit asset transferable --from-account-type FUND --to-account-type UNIFIED
```

## Internal transfer (same UID)

```bash
# Move USDT from Funding to Unified
bybit asset transfer \
  --coin USDT \
  --amount 1000 \
  --from-account-type FUND \
  --to-account-type UNIFIED

# Skip confirmation
bybit asset transfer --coin USDT --amount 1000 \
  --from-account-type FUND --to-account-type UNIFIED -y
```

## Check transfer history

```bash
bybit asset transfer-history --coin USDT --limit 10

# Filter by status
bybit asset transfer-history --status SUCCESS --limit 20
```

## Check balances

```bash
# Funding account
bybit asset balance --account-type FUND --coin USDT

# Unified account
bybit account balance --account-type UNIFIED --coin USDT

# All balances
bybit asset all-balance --account-type UNIFIED
```

## Deposit address

```bash
# Get BTC deposit address on the BTC network
bybit asset deposit-address --coin BTC --chain-type BTC

# USDT on TRC20
bybit asset deposit-address --coin USDT --chain-type TRC20
```

## Withdrawal

> ⚠️ Withdrawal requires API key with "Withdraw" permission enabled. Use with extreme caution.

```bash
# Always dry-run mentally first: verify address, chain, and amount

bybit asset withdraw \
  --coin USDT \
  --chain TRX \
  --address Txxxxxxxxxxxxxxxxxxxxxxxxxxxx \
  --amount 100

# Check withdrawal history
bybit asset withdraw-history --coin USDT --limit 10

# Cancel a pending withdrawal
bybit asset cancel-withdraw --id <withdrawId>
```
