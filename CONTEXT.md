# bybit-cli Runtime Context

This file is for AI agents and tool-calling LLMs. Load it at session start to avoid repeated discovery calls.

## What you are using

`bybit-cli` — an unofficial community-maintained terminal CLI for the [Bybit V5 API](https://bybit-exchange.github.io/docs/v5/intro).

- Full Bybit V5 REST and WebSocket coverage
- Built-in paper trading with live prices
- Native MCP server for agent tool-calling
- Single binary, no runtime dependencies

## Binary invocation

```bash
bybit <command> [subcommand] [flags] -o json
```

Always pass `-o json` for programmatic use. Capture stderr as well, because failures are emitted there as JSON.

## Output contract

- **stdout** — valid JSON on success
- **stderr** — JSON error envelope on failure, plus any human-readable diagnostics
- **exit code 0** — success; **non-zero** — failure

## Error envelope

```json
{
  "error": "api|auth|network|rate_limit|paper|validation|config|websocket|io|parse",
  "message": "human-readable description",
  "ret_code": 10003,
  "retryable": false
}
```

`ret_code` is present on every error envelope. It is `null` when no Bybit retCode applies.

Route on `error`, not `message`. The `message` field is not stable.

## Authentication

```bash
export BYBIT_API_KEY="your-key"
export BYBIT_API_SECRET="your-secret"
```

Credential resolution order: CLI flags > environment variables > platform config file (for example `~/.config/bybit/config.toml` on Linux)

For local development, `bybit-cli` also loads `.env` from the current working directory or any parent directory. Already-exported environment variables keep precedence.

Public market data and paper trading require no credentials.

## Asset categories

| Flag value | Description |
|------------|-------------|
| `linear` | USDT/USDC perpetual contracts (default) |
| `spot` | Spot trading |
| `inverse` | Coin-margined inverse contracts |
| `option` | Options contracts |

## Safe commands (no credentials, no side effects)

```bash
bybit market server-time
bybit market tickers --category linear --symbol BTCUSDT
bybit market spread --symbol BTCUSDT
bybit market orderbook --category linear --symbol BTCUSDT --limit 5
bybit market kline --category linear --symbol BTCUSDT --interval 60
bybit market funding-rate --category linear --symbol BTCUSDT
bybit paper status
```

## Dangerous commands (require confirmation or --yes)

- `trade buy` / `trade sell` — place real orders (supports Iceberg, Post-Only, TP/SL limits)
- `trade cancel` / `trade cancel-all` — cancel orders
- `trade batch-place` — up to 20 orders in one call
- `trade cancel-after` — dead man's switch
- `position set-leverage` / `position set-tpsl` — modify positions
- `position flatten` — EMERGENCY close all trades and cancel all orders
- `earn stake` / `earn unstake` — manage savings/staking
- `asset transfer` / `asset withdraw` — move funds

Use `--validate` on buy/sell for dry-run. Pass `-y` to skip confirmation in automation.

## Paper trading (safe sandbox)

```bash
bybit paper init --usdt 10000
bybit paper buy --symbol BTCUSDT --qty 0.1
bybit paper sell --symbol BTCUSDT --qty 0.05
bybit paper orders    # show open limit orders (checks fills)
bybit paper status    # P&L summary with live prices
bybit paper reset
```

## Testnet

```bash
bybit --testnet market tickers --category linear --symbol BTCUSDT
```

Or set `BYBIT_TESTNET=1`. Testnet credentials differ from mainnet.

## Key commands quick-reference

```bash
# Market data
bybit market spread --symbol BTCUSDT -o json
bybit market risk-limit --symbol BTCUSDT -o json

# Trading (always --validate first)
bybit trade buy --symbol BTCUSDT --qty 0.01 --price 50000 --validate
bybit trade buy --symbol BTCUSDT --qty 0.01 --post-only --display-qty 0.001 -y

# Dead man's switch
bybit trade cancel-after 60   # cancel all orders in 60s if not refreshed

# Risk Management
bybit position flatten --category linear -y
bybit account adl-alert --symbol BTCUSDT

# Account & Earn
bybit account volume --days 30
bybit earn products --coin USDT
bybit auth permissions
```

## Rate limits

The CLI retries transient errors (HTTP 5xx, network) up to 3 times with exponential backoff. API rate limit errors (retCode 10006/10018) are surfaced immediately — back off before retrying.

## Local state persistence

Saved credentials, the paper journal, shell history, and the anonymous instance ID persist across normal CLI and MCP sessions until reset or deleted.

## Full documentation

- Commands: `bybit --help`, `bybit <group> --help`
- Agent/MCP tool catalog: `agents/tool-catalog.json`
- Error catalog: `agents/error-catalog.json`
- Skills: `skills/INDEX.md`
- Integration guide: `AGENTS.md`
- API reference: https://bybit-exchange.github.io/docs/v5/intro
