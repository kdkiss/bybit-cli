# bybit-cli — Claude Agent Integration Guide

## What is bybit-cli?

`bybit-cli` is a command-line interface for trading, querying, and managing your Bybit account from the terminal. It mirrors the architecture and conventions of [krakenfx/kraken-cli](https://github.com/krakenfx/kraken-cli) but targets the Bybit V5 REST and WebSocket APIs.

Successful commands return structured JSON on stdout. Failures return a JSON error envelope on stderr. The CLI is designed to be used by AI agents, shell scripts, and automated pipelines.

---

## Invocation

```bash
bybit <command> [subcommand] [flags] -o json
```

For scripting and AI agents, always pass `-o json` and capture stderr as well, because failures are emitted there as JSON.

---

## Authentication

Bybit V5 uses HMAC-SHA256 signing via request headers (not body).

Required headers on private endpoints:
- `X-BAPI-API-KEY` — API key
- `X-BAPI-TIMESTAMP` — UTC millisecond timestamp
- `X-BAPI-SIGN` — HMAC-SHA256 hex signature
- `X-BAPI-RECV-WINDOW` — Request validity window (default 5000ms)

**Signature construction:**
- GET: `timestamp + api_key + recv_window + queryString`
- POST: `timestamp + api_key + recv_window + jsonBody`

**Credential resolution order (highest priority first):**
1. CLI flags: `--api-key`, `--api-secret`
2. Environment variables: `BYBIT_API_KEY`, `BYBIT_API_SECRET`
3. Platform config file: for example `~/.config/bybit/config.toml` on Linux, `~/Library/Application Support/bybit/config.toml` on macOS, or `%APPDATA%\\bybit\\config.toml` on Windows

Public market data commands require no authentication.

---

## Key Conventions

- **stdout** is valid JSON on success
- **stderr** carries the JSON error envelope on failure, plus any human-readable progress output
- **Exit code 0** = success, non-zero = failure
- **WebSocket commands** emit NDJSON (one JSON object per line)
- **Paper trading** uses live Bybit prices but no real money or authentication
- **`--validate`** dry-runs order commands without submitting them
- **`-o json`** or `-o table` selects output format (table is default)

### Error Envelope

All errors return a JSON object with stable `error` category field:

```json
{
  "error": "rate_limit",
  "message": "Too many requests",
  "ret_code": 10006,
  "retryable": true,
  "suggestion": "Wait for the rate limit window to reset",
  "docs_url": "https://bybit-exchange.github.io/docs/v5/rate-limit"
}
```

Error categories: `api`, `auth`, `network`, `rate_limit`, `paper`, `validation`, `config`, `websocket`, `io`, `parse`

---

## Asset Categories

Bybit V5 uses a `--category` flag across many commands:

| Category | Description |
|----------|-------------|
| `spot` | Spot trading |
| `linear` | USDT/USDC perpetual contracts |
| `inverse` | Inverse contracts (coin-margined) |
| `option` | Options contracts |

---

## Command Groups

| Group | Auth Required | Dangerous |
|-------|--------------|-----------|
| `market` | No | No |
| `trade` | Yes | Yes |
| `account` | Yes | No |
| `position` | Yes | Yes |
| `asset` | Yes | Yes |
| `funding` | Yes | Yes |
| `subaccount` | Yes | Yes |
| `earn` | Yes | Yes |
| `reports` | Yes | No |
| `ws` | Mixed | Some private calls |
| `futures` | Mixed | Some trading calls |
| `paper` | No | No |
| `auth` | Mixed | No |
| `setup` | No | No |
| `shell` | No | No |
| `mcp` | No | Exposes selected services |

---

## Safety Rules for Agents

- **Never** place, cancel, or amend orders without explicit user confirmation
- **Never** execute withdrawals, transfers, or position changes without user approval
- Use `--validate` to dry-run all order commands before executing
- Gate all `trade`, `funding`, `asset`, and position-modifying operations behind user approval
- Use paper trading (`bybit paper ...`) for strategy testing
- Public market commands (`bybit market ...`) are always safe to call

---

## MCP Integration

The CLI includes a built-in MCP server for LLM tool use:

```bash
# Read-only mode (market + account + paper)
bybit mcp

# All services enabled (dangerous calls require acknowledged=true)
bybit mcp -s all

# Autonomous mode (no per-call confirmation prompt)
bybit mcp -s all --allow-dangerous
```

Persisted local state is shared with normal CLI usage: saved credentials, the paper journal, shell history, and the anonymous instance ID persist across MCP tool calls and server restarts until reset or deleted.

---

## API Reference

- **Bybit V5 API Docs:** https://bybit-exchange.github.io/docs/v5/intro
- **Base URL (mainnet):** `https://api.bybit.com`
- **Base URL (testnet):** `https://api-testnet.bybit.com`
- **WebSocket (public):** `wss://stream.bybit.com/v5/public/{category}`
- **WebSocket (private):** `wss://stream.bybit.com/v5/private`

---

## Configuration File

Location depends on platform:

- Linux: `~/.config/bybit/config.toml`
- macOS: `~/Library/Application Support/bybit/config.toml`
- Windows: `%APPDATA%\\bybit\\config.toml`

```toml
[auth]
api_key = "..."
api_secret = "..."

[settings]
default_category = "linear"
output = "table"
recv_window = 5000
```

File is saved with `0600` permissions. Secrets are never logged or printed.

For local development, `bybit-cli` also loads `.env` from the current working directory or any parent directory. Already-exported environment variables keep precedence.

---

## Environment Variables

| Variable | Description |
|----------|-------------|
| `BYBIT_API_KEY` | API key for private endpoints |
| `BYBIT_API_SECRET` | API secret for signing |
| `BYBIT_API_URL` | Override mainnet base URL |
| `BYBIT_TESTNET` | Set to `1` to use testnet URLs |
| `BYBIT_DEFAULT_CATEGORY` | Default market category when `--category` is omitted |
| `BYBIT_OUTPUT` | Default output format (`table` or `json`) |
| `BYBIT_AGENT_CLIENT` | Optional agent label for telemetry headers and user-agent |
| `BYBIT_INSTANCE_ID` | Optional override for the persisted anonymous instance ID |

---

## Source Layout Reference

```
src/
  main.rs           — CLI entry point
  lib.rs            — AppContext, Cli parser, command dispatch
  auth.rs           — HMAC-SHA256 signing
  client.rs         — HTTP client (reqwest + rustls)
  config.rs         — Config file management
  errors.rs         — Error types and categories
  paper.rs          — Paper trading state machine
  shell.rs          — Interactive REPL
  telemetry.rs      — Anonymous request-identification headers (never API keys, secrets, or HMAC signatures)
  commands/
    mod.rs          — Command module registry
    market.rs       — Public market data
    trade.rs        — Order placement and management
    account.rs      — Account and wallet data
    position.rs     — Position management
    funding.rs      — Deposits, withdrawals, transfers
    asset.rs        — Asset info, coin balances
    websocket.rs    — WebSocket streaming
    paper.rs        — Paper trading commands
    auth.rs         — Credential management
    utility.rs      — Shell, setup, mcp
    helpers.rs      — Shared helpers
  output/
    mod.rs
    json.rs
    table.rs
  mcp/
    mod.rs
    server.rs
    registry.rs
    schema.rs
```
