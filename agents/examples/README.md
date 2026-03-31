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
| `morning-brief.sh` | Read-only | Low | Account, positions, orders, and market summary for LLM briefings. |
| `market-regime-monitor.sh` | Read-only | Low | Summarizes ticker, order book depth, and recent candle behavior into a simple market regime label. |
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
- `market-regime-monitor.sh`
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
./market-regime-monitor.sh BTCUSDT linear 60 20 | jq
```

```bash
TESTNET=1 ./risk-guardian.sh 3 1.5 250 linear 60
```

```bash
LIVE=1 TESTNET=0 ./risk-guardian.sh 2 1 0 linear 30
```
