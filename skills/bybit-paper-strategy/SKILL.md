---
name: bybit-paper-strategy
version: 1.0.0
description: "Test spot strategy logic on Bybit paper trading before touching live funds."
metadata:
  openclaw:
    category: "finance"
  requires:
    bins: ["bybit"]
---

# Skill: Bybit Spot Paper Strategy

Use this skill for:

- validating spot entry and exit logic on live market data
- testing position sizing and simple rebalance loops
- rehearsing order handling without account risk

For leveraged perpetual workflows, use `bybit futures paper`. This skill covers spot paper trading only.

## Limitations

Spot paper trading runs locally against live Bybit public prices and keeps its own journal on disk.

- **Fees are modeled.** Market fills use the configured taker fee and limit fills use the configured maker fee.
- **Slippage is modeled for market orders.** The default market slippage is configurable with `--slippage-bps`.
- **No partial fills or exchange-side rejects.** Orders either fill locally under the simulator rules or remain/cancel as local paper orders.

When presenting results to the user, note that live execution can still differ because exchange matching, latency, and real liquidity constraints are not fully modeled.

## Baseline Workflow

```bash
bybit paper init --usdt 10000 -o json
bybit paper buy --symbol BTCUSDT --qty 0.01 -o json
bybit paper status -o json
bybit paper sell --symbol BTCUSDT --qty 0.005 --price 70000 -o json
bybit paper orders -o json
bybit paper history -o json
```

## Reset Between Runs

```bash
bybit paper reset -o json
```

To reseed with different paper assumptions:

```bash
bybit paper reset --balance 5000 --settle-coin USDC --taker-fee-bps 8 --maker-fee-bps 2 --slippage-bps 3 -o json
```

## Migration Rule

Only move a spot strategy to live trading after:

1. repeated paper runs with stable behavior
2. explicit user sign-off
3. `--validate` checks pass for the live `bybit trade` order payloads
