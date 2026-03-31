# Example Scripts

Curated shell-script examples for agents and automation built on top of `bybit-cli`.

These scripts are examples, not a framework. Review them before use, especially anything that places, amends, or cancels real orders.

## Safety

- Prefer `--testnet` for all first runs and validation.
- Read-only scripts are safest to publish and reuse.
- State-changing scripts can place, amend, or cancel orders and may use `-y` for unattended execution.
- `risk-guardian.sh` defaults to testnet and refuses mainnet unless you pass `LIVE=1 TESTNET=0`.
- Bybit testnet may reject `trade cancel-after` with `HTTP 403` on some accounts or environments; this affects `dead-mans-switch.sh` and the optional timer step in `risk-guardian.sh`.
- The scripts in this directory assume current CLI JSON output shapes, not the old wrapped `.result...` format.

## Dependencies

Most scripts assume:

- `bybit` on `PATH`
- `jq`
- `bash`

`earn-brief.sh` and `earn-cycle.sh` also accept `BYBIT_BIN=/path/to/bybit` and, when run from the repo, prefer `target/debug/bybit(.exe)` automatically so local development can use the current build instead of an older installed release.

`earn-cycle.sh` is intentionally interactive because Earn currently has no dry-run flag.

Some scripts also require:

- `python3`
- `sqlite3`

## Scripts

| Script | Type | Risk | Notes |
|---|---|---|---|
| `earn-brief.sh` | Read-only | Low | Summarizes available Earn products, positions, order history, and yield history for a coin/category. Defaults to testnet. |
| `earn-cycle.sh` | Interactive Earn | High | Prompts for action, coin, product, and amount, then runs a real `earn stake` or `earn redeem`. Defaults to testnet and asks for its own final confirmation before executing with `-y`. |
| `breakout-detector.sh` | Read-only | Low | Detects simple volatility-expansion breakout setups from kline range structure, ATR compression/expansion, volume spike, and orderbook imbalance. |
| `price-level-alert.sh` | Read-only | Low | Watches a symbol for simple price/level conditions like above, below, near, or live crossing events, with one-shot and watch modes. |
| `risk-snapshot.sh` | Read-only | Low | AI-facing summary of account balance, positions, open orders, margin state, and options greeks with best-effort section-level error handling. |
| `trade-plan-builder.sh` | Read-only | Low | Turns a thesis, entry, stop, and target into a sized trade plan with reward/risk, exchange minimum checks, and `--validate` preview commands. |
| `signal-watch.sh` | Read-only | Low | Combines breakout and regime analysis into a compact signal object for one-shot use or watch-mode alerting. |
| `journal-capture.sh` | Local journal | Low | Appends a local JSONL journal entry with note/thesis plus live risk snapshot, signal watch, and optional trade plan context. `SIGNAL_INTERVAL` and `SIGNAL_LIMIT` let you align the embedded signal snapshot with the trade timeframe. |
| `morning-brief.sh` | Read-only | Low | Account, positions, orders, and market summary for LLM briefings. |
| `market-regime-monitor.sh` | Read-only | Low | Summarizes ticker, order book depth, and recent candle behavior into a simple market regime label. |
| `options-opportunity-scanner.sh` | Read-only | Low | Scans BTC/ETH option chains, scores contracts, and emits structured option candidates by strategy type. Defaults to mainnet data and supports view/risk/DTE filters. |
| `options-strategy-advisor.sh` | Read-only | Low | Opinionated layer on top of the scanner that returns the best single-leg idea, best spread, best hedge, and contracts to avoid for a view/risk/holding profile. |
| `options-protective-put.sh` | Read-only | Low | Suggests protective put candidates for BTC/ETH exposure, with estimated hedge notional and premium cost. Supports manual sizing or best-effort auto exposure discovery from wallet balance plus linear `BASEUSDT` positions. |
| `options-covered-call-finder.sh` | Read-only | Low | Ranks OTM call candidates for covered-call income on BTC/ETH spot exposure, with assignment-risk and premium-yield context. Supports manual sizing or best-effort auto exposure discovery. |
| `options-iv-skew-monitor.sh` | Read-only | Low | Compares call/put IV across comparable absolute-delta buckets within each expiry to highlight call-rich and put-rich skew distortions with basic liquidity and spread filters. |
| `options-greeks-risk-report.sh` | Read-only | Low | Estimates single-leg option greek impact and, when auth is available, projects current portfolio greeks before and after the proposed trade. Supports advisor-driven selection or `SYMBOL=...` override. |
| `paper-session.sh` | Paper trading | Low | End-to-end paper session: init, buy, limit order, cancel, sell, report. |
| `conditional-order.sh` | Trading | High | Polls live price and submits a market buy when a threshold is hit. |
| `dca-buy.sh` | Trading | High | Places repeated market buys on a timer. |
| `batch-limit-ladder.sh` | Trading | High | Builds and submits a batch of limit buy orders below market. |
| `dead-mans-switch.sh` | Risk control | Medium | Repeatedly refreshes `trade cancel-after` until stopped. |
| `close-all-positions.sh` | Trading | Very high | Cancels orders and closes all open positions in a category. |
| `risk-guardian.sh` | Risk control | High | Applies TP/SL and optional trailing stops to open positions, then arms `cancel-after`. Defaults to testnet. |

## Suggested Use

For public examples in docs or demos, prefer:

- `earn-brief.sh`
- `breakout-detector.sh`
- `price-level-alert.sh`
- `risk-snapshot.sh`
- `trade-plan-builder.sh`
- `signal-watch.sh`
- `journal-capture.sh`
- `market-regime-monitor.sh`
- `options-opportunity-scanner.sh`
- `options-strategy-advisor.sh`
- `options-protective-put.sh`
- `options-covered-call-finder.sh`
- `options-iv-skew-monitor.sh`
- `options-greeks-risk-report.sh`
- `morning-brief.sh`
- `paper-session.sh`

For advanced operator workflows, keep these clearly marked as dangerous:

- `earn-cycle.sh`
- `conditional-order.sh`
- `dca-buy.sh`
- `batch-limit-ladder.sh`
- `close-all-positions.sh`
- `risk-guardian.sh`

## Examples

```bash
./earn-brief.sh USDT FlexibleSaving 5
```

```bash
./earn-cycle.sh
```

```bash
./breakout-detector.sh BTCUSDT linear 60 48 | jq
```

```bash
./breakout-detector.sh ETHUSDT linear 15 64 | jq
```

```bash
./price-level-alert.sh BTCUSDT 66000 above linear | jq
```

```bash
MODE=watch POLL_SECONDS=15 PRICE_SOURCE=mark ./price-level-alert.sh ETHUSDT 2050 cross_below linear
```

```bash
TOLERANCE_PCT=0.2 ./price-level-alert.sh BTCUSDT 66500 near linear | jq
```

```bash
./risk-snapshot.sh | jq
```

```bash
./risk-snapshot.sh USDT BTC,ETH | jq
```

```bash
./trade-plan-builder.sh BTCUSDT buy market 65000 70000 50 linear | jq
```

```bash
RISK_PCT=0.5 THESIS="Fade resistance breakout failure" ./trade-plan-builder.sh ETHUSDT sell 2050 2100 1950 0 linear | jq
```

```bash
./signal-watch.sh BTCUSDT linear 60 48 | jq
```

```bash
MODE=watch POLL_SECONDS=60 ./signal-watch.sh ETHUSDT linear 15 64
```

```bash
NOTE="Watching for reclaim follow-through" ./journal-capture.sh note BTCUSDT linear | jq
```

```bash
PLAN_SIDE=buy PLAN_ENTRY=market PLAN_STOP=65000 PLAN_TARGET=70000 PLAN_RISK_USD=50 THESIS="Breakout retest" ./journal-capture.sh planned BTCUSDT linear | jq
```

```bash
SIGNAL_INTERVAL=15 SIGNAL_LIMIT=64 NOTE="Lower-timeframe continuation watch" ./journal-capture.sh note ETHUSDT linear | jq
```

```bash
./market-regime-monitor.sh BTCUSDT linear 60 20 | jq
```

```bash
./options-opportunity-scanner.sh BTC bullish defined_risk 7 45 3 | jq
```

```bash
TESTNET=true ./options-opportunity-scanner.sh ETH neutral income 14 60 5 | jq
```

```bash
./options-strategy-advisor.sh BTC bullish defined_risk none 7 45 3 | jq
```

```bash
./options-strategy-advisor.sh ETH neutral income spot 14 60 3 | jq
```

```bash
./options-protective-put.sh BTC 1 7 45 3 | jq
```

```bash
./options-protective-put.sh ETH auto 14 60 3 | jq
```

```bash
./options-covered-call-finder.sh BTC 1 7 45 3 | jq
```

```bash
./options-covered-call-finder.sh ETH auto 14 60 3 | jq
```

```bash
./options-iv-skew-monitor.sh BTC 7 45 5 | jq
```

```bash
./options-iv-skew-monitor.sh ETH 14 60 8 | jq
```

```bash
./options-greeks-risk-report.sh BTC bullish defined_risk none 1 buy 7 45 | jq
```

```bash
SYMBOL='<OPTION_SYMBOL>' ./options-greeks-risk-report.sh ETH bearish hedge spot 2 buy 14 60 | jq
```

```bash
TESTNET=true ./risk-guardian.sh 3 1.5 250 linear 60
```

```bash
LIVE=1 TESTNET=0 ./risk-guardian.sh 2 1 0 linear 30
```
