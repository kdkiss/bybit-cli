# bybit-cli

*Thank you to the creators of [krakenfx/kraken-cli](https://github.com/krakenfx/kraken-cli) for providing the idea and framework for this project. This tool is inspired by their CLI, designed as a mirror for Bybit.*

![version](https://img.shields.io/github/v/release/kdkiss/bybit-cli?color=blue)
![license](https://img.shields.io/badge/license-MIT-green)
![platform](https://img.shields.io/badge/platform-macOS%20%7C%20Linux%20%7C%20Windows-lightgrey)
![rust](https://img.shields.io/badge/built_with-Rust-orange)

**DISCLAIMER: This is an UNOFFICIAL community-maintained CLI and is NOT affiliated with Bybit. Use at your own risk. Trading involves significant risk of loss.**

The AI-native CLI for trading crypto on Bybit. Full Bybit V5 API access. Built-in MCP server. Live and paper trading. Single binary.

Works with Claude, Cursor, Codex, Copilot, Gemini, and any MCP-compatible agent.

Try these with your AI agent:

> *"Build me a morning market brief for BTC, ETH, and SOL with trend, volatility, funding, and key levels."*

> *"Watch BTCUSDT and alert me if price breaks above 68,000, drops below 66,000, or gets within 0.25% of either level."*

> *"Create a trade plan for ETHUSDT with an entry, stop, and target, risk only 0.5% of my account, and show me the validate command before placing anything."*

> *"Check my open positions and set a stop-loss at 5% below entry on each one."*

---

> [!CAUTION]
> Unofficial software. Interacts with the live Bybit exchange and can execute real financial transactions. Read [DISCLAIMER.md](DISCLAIMER.md) before using with real funds or AI agents.

## Contents

- [Installation](#installation)
- [What You Can Trade](#what-you-can-trade)
- [For AI Agents](#for-ai-agents)
- [Verifying Binaries](#verifying-binaries)
- [Quick Start](#quick-start)
- [API Keys & Configuration](#api-keys--configuration)
- [MCP Server](#mcp-server)
- [Paper Trading](#paper-trading)
- [Commands](#commands)
- [Examples](#examples)
- [Agent Skills](#agent-skills)
- [Development](#development)
- [Contributing](#contributing)
- [License & Disclaimer](#license--disclaimer)

## Installation

Single binary, no runtime dependencies.

### One-liner (macOS / Linux)

```bash
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/kdkiss/bybit-cli/releases/latest/download/bybit-cli-installer.sh | sh
```

### One-liner (Windows)

```powershell
irm https://github.com/kdkiss/bybit-cli/releases/latest/download/bybit-cli-installer.ps1 | iex
```

Detects your OS and architecture, downloads the right archive, verifies checksums, and installs it. Prebuilt installers are generated for macOS (Apple Silicon and Intel), Linux (`x86_64`), and Windows (`x86_64`).

Verify it works:

```bash
bybit market server-time && bybit market tickers --category linear --symbol BTCUSDT
```

Pre-built archives and installer scripts are available on the [GitHub Releases](https://github.com/kdkiss/bybit-cli/releases) page.

Tagged releases are built with `cargo-dist`, attested with GitHub build provenance, and published with per-artifact `.sha256` files, a unified `sha256.sum`, `SHA256SUMS.txt`, and `minisig` signatures. Maintainers must configure `MINISIGN_SECRET_KEY` and publish the matching minisign public key before the first public release.

See [Verifying Binaries](#verifying-binaries) for minisign verification.

<details>
<summary>Build from source</summary>

Requires [Rust](https://rustup.rs/).

```bash
git clone https://github.com/kdkiss/bybit-cli
cd bybit-cli
cargo install --path . --locked
```

Or just build:

```bash
cargo build --release
cp target/release/bybit ~/.local/bin/
```

</details>

## What You Can Trade

One binary covers Bybit spot, derivatives, earn workflows, streaming, and paper trading.

| Area | What it covers | Flag / namespace | Example |
|---|---|---|---|
| **Spot** | Public market data, balances, transfers, and spot orders | `--category spot` | `bybit market tickers --category spot --symbol BTCUSDT` |
| **Linear derivatives** | USDT/USDC-margined perpetuals and related account/position flows | `--category linear` | `bybit futures buy --symbol BTCUSDT --qty 0.01 --price 50000` |
| **Inverse derivatives** | Coin-margined contracts and historical market/account views | `--category inverse` | `bybit market instruments --category inverse` |
| **Options** | Option market data plus account greeks and volatility views | `--category option` | `bybit market instruments --category option` |
| **Earn** | Flexible saving / earn products, staking, redeeming, and yield history | `earn` namespace | `bybit earn products --coin BTC` |
| **Paper trading** | Local simulation with live public prices and no API keys | `paper` namespace | `bybit paper buy --symbol BTCUSDT --qty 0.01` |

Product availability and permissions vary by jurisdiction, account type, and API key scope.

## For AI Agents

If you're an AI agent or building one, start here:

| Resource | Description |
|----------|-------------|
| [CONTEXT.md](CONTEXT.md) | Runtime context — load this at session start |
| [AGENTS.md](AGENTS.md) | Full integration guide: auth, invocation, errors, rate limits |
| [agents/tool-catalog.json](agents/tool-catalog.json) | Canonical agent/MCP tool catalog with parameter schemas, auth requirements, examples, and safety flags |
| [agents/error-catalog.json](agents/error-catalog.json) | Error categories with retry guidance and remediation |
| [agents/examples/README.md](agents/examples/README.md) | Curated shell-script examples for agent and automation workflows |
| [skills/INDEX.md](skills/INDEX.md) | Goal-oriented workflow packages |
| [CLAUDE.md](CLAUDE.md) | Claude-specific integration guidance |
| [gemini-extension.json](gemini-extension.json) | Gemini CLI extension manifest for auto-starting the MCP server |

Core invocation pattern:

```bash
bybit <command> [args...] -o json
```

- stdout is valid JSON on success.
- Exit code 0 means success. Non-zero means failure.
- On failure, the CLI prints a JSON error envelope to stderr.
- stderr may also contain human-oriented diagnostics. Do not discard it if you need machine-readable errors.

<details>
<summary>Why agent-first?</summary>

Most CLIs are built for humans at a terminal. This one is built for LLM-based agents, MCP tool servers, and automated pipelines that need to call Bybit reliably without custom API clients.

- **Structured output by default.** Every command supports `-o json`. No screen-scraping.
- **Consistent error envelopes.** Errors are JSON objects with a stable `error` field. Agents route on `error` without parsing human sentences.
- **Predictable exit codes.** Success is 0, failure is non-zero. Agents detect and classify failures programmatically.
- **Paper trading for safe iteration.** Test strategies against live prices with `bybit paper` commands. No API keys, no real money, and a close simulation of live market and limit order flows.
- **Full API surface.** 100+ commands covering market data, trading, account, funding, reporting, positions, assets, subaccounts, futures, and WebSocket streaming.
- **Built-in MCP server.** Native Model Context Protocol support over stdio with guarded dangerous-tool handling.
- **Rate-limit aware.** When Bybit rejects a request, the CLI returns an enriched error with `suggestion`, `retryable`, and `docs_url` fields so agents can adapt their strategy.

</details>

## Verifying Binaries

Release binaries are signed with [minisign](https://jedisct1.github.io/minisign/). Published artifacts on the [GitHub Releases](https://github.com/kdkiss/bybit-cli/releases) page include checksums and minisign signatures.

**Public key:**

```text
untrusted comment: minisign public key 9A81EBFCA673CEDE
RWTeznOm/OuBmlyv8EeOQxZOog4NsO014QzO/aS3/+1woRbSPGUy3eEF
```

**Verify a downloaded archive:**

```bash
minisign -Vm bybit-cli-x86_64-unknown-linux-gnu.tar.gz -P RWTeznOm/OuBmlyv8EeOQxZOog4NsO014QzO/aS3/+1woRbSPGUy3eEF
```

Install minisign with `brew install minisign` (macOS) or your Linux package manager.

## Quick Start

### Getting Started for Humans

If you just want to configure the CLI and start using it interactively:

```bash
bybit setup
bybit shell
```

`bybit setup` walks through credentials and defaults. `bybit shell` starts the interactive REPL with history and tab-completion.

### Quick Checks

Public market data requires no credentials:

```bash
bybit market tickers --category linear --symbol BTCUSDT -o json
bybit market orderbook --category linear --symbol BTCUSDT --limit 10
bybit market kline --category linear --symbol BTCUSDT --interval 60
```

With authentication:

```bash
export BYBIT_API_KEY="your-key"
export BYBIT_API_SECRET="your-secret"

bybit account balance -o json
bybit trade open-orders --category linear -o json
bybit trade buy --symbol BTCUSDT --qty 0.01 --price 50000 --validate
```

For humans (table output, interactive setup):

```bash
bybit setup
bybit market tickers --category linear --symbol BTCUSDT
bybit account balance
bybit shell
```

## API Keys & Configuration

Authenticated commands require a Bybit API key pair. Public market data and paper trading work without credentials.

### Getting API keys

Visit [Bybit API Management](https://www.bybit.com/app/user/api-management). Grant minimum required permissions. For read-only monitoring, "Read" is sufficient.

### Environment variables (recommended for agents)

```bash
export BYBIT_API_KEY="your-key"
export BYBIT_API_SECRET="your-secret"
export BYBIT_TESTNET="true"        # optional: use testnet
export BYBIT_API_URL="https://..."  # optional: override base URL
```

For local development, `bybit-cli` also loads `.env` from the current working directory (or its parents) at startup. Already-exported environment variables keep precedence.

### Config file (for humans)

Stored in your platform config directory:

- Linux: `~/.config/bybit/config.toml`
- macOS: `~/Library/Application Support/bybit/config.toml`
- Windows: `%APPDATA%\bybit\config.toml`

Example:

```toml
[auth]
api_key = "your-api-key"
api_secret = "your-api-secret"

[settings]
default_category = "linear"
output = "table"
recv_window = 5000
testnet = false
```

Run `bybit setup` for interactive configuration.

### Credential resolution

Highest precedence first:

1. CLI flags (`--api-key`, `--api-secret`)
2. Environment variables (`BYBIT_API_KEY`, `BYBIT_API_SECRET`)
3. Config file (platform config path, for example `~/.config/bybit/config.toml` on Linux)

### Security

- Config file is created with `0600` permissions (owner read/write only) on Unix.
- Secrets are never logged, printed, or included in error messages.
- Use `--api-secret-stdin` instead of `--api-secret` to avoid secrets in process listings.
- For automation, prefer environment variables over command-line flags.

## MCP Server

> Built-in MCP server is available via `bybit mcp`. It exposes command groups over stdio for MCP-compatible agents.

```json
{
  "mcpServers": {
    "bybit": {
      "command": "bybit",
      "args": ["mcp", "-s", "all"]
    }
  }
}
```

Gemini CLI users can use the included [gemini-extension.json](gemini-extension.json) manifest from the repo root to register the same MCP server configuration.

```bash
bybit mcp                          # read-only (market, account, paper)
bybit mcp -s all                   # all services, dangerous calls require acknowledged=true
bybit mcp -s all --allow-dangerous # all services, no per-call confirmation
bybit mcp -s market,trade,paper    # specific services
bybit mcp -s funding,reports,futures,subaccount
```

Available service groups include `market`, `account`, `trade`, `position`, `asset`, `funding`, `reports`, `subaccount`, `futures`, `paper`, and `auth`.
The server expects the standard MCP `initialize` plus `notifications/initialized` handshake; normal MCP clients handle that automatically.

Persisted local state is shared with normal CLI mode: saved credentials, the paper journal, shell history, and the anonymous instance ID persist across MCP tool calls and server restarts until reset or deleted.

## Paper Trading

Paper trading provides a safe sandbox for testing trading logic against live Bybit prices. No API keys, no real money. It supports market and limit `buy` / `sell` flows with live pricing, local journal state, fees, and slippage.

**Market orders (fill immediately at live price):**

```bash
bybit paper init --usdt 10000
bybit paper buy --symbol BTCUSDT --qty 0.01
bybit paper sell --symbol BTCUSDT --qty 0.005
bybit paper status
```

**Limit orders (fill when market price crosses):**

```bash
bybit paper buy --symbol BTCUSDT --qty 0.01 --price 50000   # fills when price ≤ 50000
bybit paper sell --symbol BTCUSDT --qty 0.01 --price 60000  # fills when price ≥ 60000
bybit paper orders     # check open limit orders (checks fills at current prices)
bybit paper cancel 1   # cancel order by ID
bybit paper cancel-all # cancel all pending limit orders
```

**Full session:**

```bash
bybit paper init --usdt 10000 -o json
bybit paper buy --symbol BTCUSDT --qty 0.01 -o json
bybit paper positions -o json
bybit paper history -o json
bybit paper status -o json
bybit paper reset -o json
```

`bybit paper reset` can also re-seed the simulator in one step, for example:

```bash
bybit paper reset --balance 2500 --settle-coin USDC --taker-fee-bps 10 --maker-fee-bps 2 --slippage-bps 0
```

| Command | Description |
|---------|-------------|
| `bybit paper init [--usdt 10000]` | Initialize paper account |
| `bybit paper buy --symbol SYM --qty Q [--price P]` | Paper buy (market or limit) |
| `bybit paper sell --symbol SYM --qty Q [--price P]` | Paper sell (market or limit) |
| `bybit paper orders` | Open limit orders (checks fills) |
| `bybit paper cancel <ORDER_ID>` | Cancel a limit order |
| `bybit paper cancel-all` | Cancel all limit orders |
| `bybit paper balance` | Coin balances with total, reserved, and available |
| `bybit paper positions` | Open positions |
| `bybit paper history` | Filled trade history |
| `bybit paper status` | Portfolio value, realized/unrealized P&L, fees, and valuation status |
| `bybit paper reset` | Reinitialize the paper account, optionally overriding balance, settle coin, fees, or slippage |

All output includes `"mode": "paper"` in JSON. Limit buys reserve quote balance plus maker fees, limit sells reserve base asset quantity, and pending orders are reconciled when paper read commands run (`balance`, `positions`, `history`, `cancelled`, `orders`, or `status`).

## Commands

The CLI exposes 18 top-level command groups. For the machine-readable agent/MCP tool surface, load [agents/tool-catalog.json](agents/tool-catalog.json).

| Group | Auth | Dangerous | Description |
|-------|------|-----------|-------------|
| market | No | No | Tickers, orderbook, klines, funding, open interest |
| trade | Yes | Yes | Order placement, amendment, cancellation, batch ops |
| account | Yes | No | Balances, info, fee rates, transaction log |
| position | Yes | Yes | List, leverage, TP/SL, margin, closed P&L |
| asset | Yes | Yes | Balances, transfers, deposits, withdrawals |
| convert | Yes | Mixed | Coin conversion quote, execute, status, and history |
| margin | Mixed | Mixed | Spot margin status, VIP data, mode toggle, and leverage |
| funding | Yes | Yes | Wallet balances, deposits, withdrawals, and transfers |
| subaccount | Yes | Yes | Master-account subaccount management |
| earn | Mixed | Mixed | Bybit Earn products, positions, stake/redeem, and yield history |
| futures | Mixed | Mixed | Derivatives-focused market data, trading, positions, and streaming |
| ws | Optional | No | Real-time WebSocket streams |
| paper | No | No | Paper trading simulation |
| reports | Yes | No | Histories plus Bybit tax export request/status/retrieve workflows |
| auth | Mixed | No | Credential management |
| utility | No | No | Setup wizard, interactive shell |

<details>
<summary>Full command reference</summary>

### Market Data (Public)

| Command | Description |
|---------|-------------|
| `bybit market server-time` | Server time |
| `bybit market instruments --category linear` | List instruments |
| `bybit market orderbook --category linear --symbol BTCUSDT` | L2 order book |
| `bybit market tickers --category linear [--symbol SYM]` | Ticker data |
| `bybit market kline --category linear --symbol SYM --interval 60` | OHLCV candles |
| `bybit market mark-price-kline ...` | Mark price kline |
| `bybit market index-price-kline ...` | Index price kline |
| `bybit market premium-index-kline ...` | Premium index kline |
| `bybit market funding-rate --category linear --symbol SYM` | Funding rate history |
| `bybit market trades --category linear --symbol SYM` | Recent trades |
| `bybit market open-interest --category linear --symbol SYM --interval 1h` | Open interest |
| `bybit market volatility [--currency BTC]` | Historical volatility (options) |
| `bybit market insurance [--coin USDT]` | Insurance fund |
| `bybit market risk-limit --category linear --symbol SYM` | Risk limits |
| `bybit market delivery-price --category linear` | Delivery price |
| `bybit market ls-ratio --category linear --symbol SYM --period 5min` | Long/short ratio |

### Trading

| Command | Description |
|---------|-------------|
| `bybit trade buy --symbol SYM --qty Q [--price P] [--validate]` | Buy order |
| `bybit trade sell --symbol SYM --qty Q [--price P]` | Sell order |
| `bybit trade amend --symbol SYM --order-id ID [--price P] [--qty Q]` | Amend open order |
| `bybit trade cancel --symbol SYM --order-id ID` | Cancel order |
| `bybit trade cancel-all [--symbol SYM]` | Cancel all orders |
| `bybit trade cancel-after <SECS>` | Dead man's switch (0 = disable) |
| `bybit trade dcp-info` | Show current DCP (Disconnect Cancel All) configuration |
| `bybit trade open-orders [--symbol SYM]` | Open orders |
| `bybit trade history [--symbol SYM] [--limit N]` | Order history |
| `bybit trade fills [--symbol SYM] [--limit N]` | Execution history |
| `bybit trade batch-place --orders '[...]'` | Batch place (up to 20) |
| `bybit trade batch-amend --orders '[...]'` | Batch amend (up to 20) |
| `bybit trade batch-cancel --orders '[...]'` | Batch cancel (up to 20) |

### Account

| Command | Description |
|---------|-------------|
| `bybit account balance [--account-type UNIFIED] [--coin USDT]` | Wallet balance |
| `bybit account extended-balance [--account-type UNIFIED] [--coin USDT]` | Per-coin balance view across the account |
| `bybit account info` | Account info (UID, margin mode) |
| `bybit account fee-rate [--category linear] [--symbol SYM]` | Fee rates |
| `bybit account transaction-log [--limit N]` | Transaction log |
| `bybit account borrow-history` | Borrow history |
| `bybit account collateral-info` | Collateral info |
| `bybit account greeks` | Options greeks |
| `bybit account volume [--category linear] [--days 30]` | Approximate executed trading volume over a lookback window |
| `bybit account set-margin-mode --margin-mode REGULAR_MARGIN` | Set margin mode |
| `bybit account set-spot-hedging --mode ON` | Set spot hedging |
| `bybit account set-usdc-settlement --coin USDC` | Set UTA settlement coin for USDC products |
| `bybit account borrow --coin USDT --amount AMT` | Manually borrow funds |
| `bybit account repay --coin USDT --amount AMT` | Manually repay borrowed funds |
| `bybit account quick-repay [--coin USDT]` | Auto-select and repay liabilities |

### Convert

| Command | Description |
|---------|-------------|
| `bybit convert coins [--account-type UNIFIED] [--coin BTC] [--side 1]` | List coins and supported conversion directions |
| `bybit convert quote --from-coin BTC --to-coin USDT (--from-amount AMT \| --to-amount AMT)` | Request a conversion quote |
| `bybit convert quote ... --dry-run` | Preview the quote request without calling the API |
| `bybit convert execute --quote-tx-id ID` | Execute a previously obtained quote |
| `bybit convert status --quote-tx-id ID [--account-type UNIFIED]` | Check conversion status |
| `bybit convert history [--account-type UNIFIED] [--coin BTC] [--start MS] [--end MS] [--index N] [--limit N]` | Conversion history |

Notes:
The Convert API requires the Bybit API key permission `Exchange`. If that permission is missing, Bybit may reject coin-list and quote requests even though the CLI command itself is correct.

### Spot Margin

| Command | Description |
|---------|-------------|
| `bybit margin vip-data [--vip-level "No VIP"] [--currency BTC]` | Public VIP borrow/leverage data for spot margin |
| `bybit margin status` | Current spot margin state and leverage for the unified account |
| `bybit margin toggle --mode on\|off` | Enable or disable unified account spot margin |
| `bybit margin set-leverage --leverage N [--currency BTC]` | Set spot margin leverage globally or for a coin |

Notes:
Spot margin activation and leverage changes can be rejected by Bybit until the account completes the required margin-trading setup or quiz in the Bybit UI.

### Earn

| Command | Description |
|---------|-------------|
| `bybit earn products [--category FlexibleSaving] [--coin BTC]` | List available Bybit Earn products |
| `bybit earn positions [--category FlexibleSaving] [--coin BTC]` | List active Earn positions |
| `bybit earn stake --product-id ID --coin BTC --amount AMT` | Stake into an Earn product |
| `bybit earn redeem --product-id ID --coin BTC --amount AMT` | Redeem from an Earn product |
| `bybit earn history [--category FlexibleSaving] [--order-id ID]` | Stake/redeem order history or status |
| `bybit earn yield [--category FlexibleSaving]` | Yield distribution history |
| `bybit earn hourly-yield [--category FlexibleSaving]` | Hourly yield history |

### Positions

| Command | Description |
|---------|-------------|
| `bybit position list [--symbol SYM]` | Open positions |
| `bybit position set-leverage --symbol SYM --buy-leverage N --sell-leverage N` | Set leverage |
| `bybit position switch-mode --symbol SYM --mode 0` | Switch one-way/hedge |
| `bybit position set-tpsl --symbol SYM [--take-profit P] [--stop-loss P]` | Set TP/SL |
| `bybit position set-risk-limit --symbol SYM --risk-id N` | Set risk limit |
| `bybit position add-margin --symbol SYM --margin AMT` | Add/reduce margin |
| `bybit position closed-pnl [--category linear] [--symbol SYM] [--limit N]` | Closed P&L history for supported contract categories |
| `bybit position move --from-uid UID --to-uid UID --positions '[...]'` | Move positions |
| `bybit position move-history` | Move position history |

### Assets

| Command | Description |
|---------|-------------|
| `bybit asset coin-info [--coin BTC]` | Coin info (networks, limits) |
| `bybit asset balance [--account-type UNIFIED]` | Asset balance |
| `bybit asset all-balance [--account-type UNIFIED]` | All coins balance |
| `bybit asset account-balance --coin USDT` | Single coin balance |
| `bybit asset transferable --from-account-type FUND --to-account-type UNIFIED` | Transferable coins |
| `bybit asset transfer --coin USDT --amount AMT --from-account-type F --to-account-type T` | Internal transfer |
| `bybit asset transfer-history` | Transfer history |
| `bybit asset sub-transfer ...` | Universal (cross-UID) transfer |
| `bybit asset sub-transfer-history` | Universal transfer history |
| `bybit asset deposit-address --coin BTC [--chain-type BTC]` | Deposit address |
| `bybit asset deposit-history` | Deposit history |
| `bybit asset withdraw --coin USDT --chain TRX --address ADDR --amount AMT` | Withdraw |
| `bybit asset withdraw-history` | Withdrawal history |
| `bybit asset cancel-withdraw --id ID` | Cancel pending withdrawal |

### Funding

| Command | Description |
|---------|-------------|
| `bybit funding coin-info [--coin BTC]` | Coin funding metadata |
| `bybit funding balance [--account-type UNIFIED] [--coin USDT]` | Funding balance by wallet type |
| `bybit funding all-balance [--account-type UNIFIED]` | All funding balances |
| `bybit funding account-balance --coin USDT` | Single funding coin balance |
| `bybit funding transferable --from-account-type FUND --to-account-type UNIFIED` | Coins transferable between wallets |
| `bybit funding transfer --coin USDT --amount AMT --from-account-type F --to-account-type T` | Internal wallet transfer |
| `bybit funding transfer-history` | Internal transfer history |
| `bybit funding sub-transfer ...` | Universal transfer across UIDs |
| `bybit funding sub-transfer-history` | Universal transfer history |
| `bybit funding deposit-address --coin BTC [--chain-type BTC]` | Deposit address |
| `bybit funding deposit-history` | Deposit history |
| `bybit funding withdraw --coin USDT --chain TRX --address ADDR --amount AMT` | Withdraw |
| `bybit funding withdraw-history` | Withdrawal history |
| `bybit funding cancel-withdraw --id ID` | Cancel pending withdrawal |

### Subaccounts

| Command | Description |
|---------|-------------|
| `bybit subaccount list` | List up to 10k subaccounts |
| `bybit subaccount list-all [--page-size N] [--next-cursor CURSOR]` | Paginated subaccount listing |
| `bybit subaccount wallet-types [--member-ids UID1,UID2]` | Show master/subaccount wallet types |
| `bybit subaccount api-keys --sub-member-id UID [--limit N]` | List API keys for a subaccount |
| `bybit subaccount create --username NAME [--member-type 1] [--quick-login]` | Create a subaccount |
| `bybit subaccount delete --sub-member-id UID` | Delete a subaccount |
| `bybit subaccount freeze --sub-member-id UID` | Freeze a subaccount |
| `bybit subaccount unfreeze --sub-member-id UID` | Unfreeze a subaccount |

### Futures

| Command | Description |
|---------|-------------|
| `bybit futures instruments [--category linear] [--symbol SYM]` | Derivatives instrument metadata |
| `bybit futures tickers [--category linear] [--symbol SYM]` | Futures ticker / 24h stats |
| `bybit futures orderbook --symbol SYM [--limit 50]` | Futures L2 order book |
| `bybit futures funding-rate --symbol SYM` | Funding rate history |
| `bybit futures open-interest --symbol SYM --interval-time 1h` | Open interest |
| `bybit futures positions [--symbol SYM]` | Open futures positions |
| `bybit futures open-orders [--symbol SYM]` | Open futures orders |
| `bybit futures history [--symbol SYM] [--limit N]` | Futures order history |
| `bybit futures fills [--symbol SYM] [--limit N]` | Futures execution history |
| `bybit futures buy --symbol SYM --qty Q [--price P] [--validate]` | Place a futures buy order |
| `bybit futures sell --symbol SYM --qty Q [--price P]` | Place a futures sell order |
| `bybit futures cancel --symbol SYM --order-id ID` | Cancel a futures order |
| `bybit futures cancel-all [--symbol SYM]` | Cancel all futures orders |
| `bybit futures set-leverage --symbol SYM --buy-leverage N --sell-leverage N` | Set futures leverage |
| `bybit futures ws orderbook --symbol SYM [--depth 50]` | Futures order book stream |
| `bybit futures ws orders` | Private futures order updates |

### Reports

| Command | Description |
|---------|-------------|
| `bybit reports transactions [--account-type UNIFIED] [--currency USDT]` | Account transaction log |
| `bybit reports borrow-history [--currency USDT]` | Borrow history |
| `bybit reports orders [--category linear] [--symbol SYM]` | Order history |
| `bybit reports fills [--category linear] [--symbol SYM]` | Execution history |
| `bybit reports closed-pnl [--category linear] [--symbol SYM]` | Closed P&L history for supported contract categories |
| `bybit reports moves [--category linear] [--symbol SYM]` | Position move history |
| `bybit reports deposits [--coin BTC]` | Deposit history |
| `bybit reports withdrawals [--coin USDT]` | Withdrawal history |
| `bybit reports transfers [--coin USDT]` | Internal transfer history |
| `bybit reports sub-transfers [--coin USDT]` | Universal transfer history |
| `bybit reports register-time` | Bybit Tax API register date for the current account |
| `bybit reports export-request --report-type TRADE --report-number 2 --start ... --end ...` | Request a Bybit tax export job |
| `bybit reports export-status --query-id ID` | Check tax export job status |
| `bybit reports export-retrieve --query-id ID [--download-dir DIR]` | Retrieve tax export URLs and optionally download the files |

### WebSocket Streaming

| Command | Description |
|---------|-------------|
| `bybit ws orderbook --symbol SYM [--depth 50]` | L2 order book stream |
| `bybit ws ticker --symbol SYM` | Ticker stream |
| `bybit ws trades --symbol SYM` | Public trades stream |
| `bybit ws kline --symbol SYM --interval 1` | Kline/OHLCV stream |
| `bybit ws liquidation --symbol SYM` | Liquidation stream |
| `bybit ws greeks --base-coin BTC` | Options greeks stream |
| `bybit ws lt-kline --symbol SYM --interval 1` | Leveraged token kline stream |
| `bybit ws lt-ticker --symbol SYM` | Leveraged token ticker stream |
| `bybit ws orders` | Private order updates (auth) |
| `bybit ws positions` | Private position updates (auth) |
| `bybit ws executions` | Private execution updates (auth) |
| `bybit ws wallet` | Private wallet updates (auth) |
| `bybit ws notifications` | All private streams combined (auth) |
| `bybit ws dcp` | Disconnection-cut-position events (auth) |

Press Ctrl+C to stop. Auto-reconnects on disconnect (exponential backoff, up to 12 attempts).

### Auth

| Command | Description |
|---------|-------------|
| `bybit auth set --api-key KEY [--api-secret TEXT]` | Save credentials to config |
| `bybit auth sign [--payload TEXT]` | Sign test payload, print HMAC-SHA256 |
| `bybit auth test` | Test credentials against /v5/account/info |
| `bybit auth show` | Show current credential source and masked key |
| `bybit auth permissions` | Show active API key permissions and scopes |
| `bybit auth reset` | Remove credentials from config file |

### Utility

| Command | Description |
|---------|-------------|
| `bybit setup` | Interactive setup wizard |
| `bybit shell` | Interactive REPL with tab-completion and history |

</details>

## Examples

### Conditional order based on live price

```bash
PRICE=$(bybit market tickers --category linear --symbol BTCUSDT -o json \
  | jq -r '.list[0].lastPrice')
bybit trade buy --symbol BTCUSDT --qty 0.001 --price "$PRICE" --validate
```

### Real-time price stream

```bash
bybit ws ticker --category linear --symbol BTCUSDT -o json | while read -r line; do
  LAST=$(echo "$line" | jq -r '.data.lastPrice // empty')
  [ -n "$LAST" ] && echo "BTC: $LAST"
done
```

### Portfolio snapshot

```bash
bybit account balance -o json
bybit position list --category linear -o json | jq '[.list[] | {symbol, side, size, unrealisedPnl}]'
```

### Dead man's switch

```bash
bybit trade cancel-after 60
```

Cancels all open orders if not refreshed within 60 seconds. Pass `0` to disable.

### Morning brief

```bash
#!/bin/bash
echo "=== Balance ===" && bybit account balance
echo "=== Positions ===" && bybit position list --category linear
echo "=== BTC ===" && bybit market tickers --category linear --symbol BTCUSDT
```

## Agent Skills

Ships with a growing agent skills library. See the full [Skills Index](skills/INDEX.md).

Examples:

- `account-snapshot` for a quick balance and positions overview
- `bybit-recipe-morning-brief` for a reusable multi-market briefing workflow
- `bybit-recipe-emergency-flatten` for guarded position-exit playbooks

<details>
<summary>Troubleshooting</summary>

**"API key is invalid" (auth error)**

- Verify `BYBIT_API_KEY` and `BYBIT_API_SECRET` are set correctly (case-sensitive).
- Run `bybit auth test` to test the current credentials.
- Check that the API key has the required permissions for the operation.
- Ensure your system clock is accurate — timestamp drift causes auth failures.

**Rate limit errors**

The CLI does not pre-throttle requests. When Bybit returns a rate limit error (retCode 10006/10018), the CLI surfaces it immediately with a `suggestion` field. Read the suggestion and adjust request frequency. For high-frequency data, prefer WebSocket streaming over REST polling.

**Mainnet vs testnet confusion**

- `BYBIT_TESTNET=true` or `--testnet` switches the CLI to Bybit testnet.
- Testnet keys are separate from mainnet keys.
- If a command unexpectedly shows empty balances or auth failures, confirm you are pointing at the intended environment.

**"Permission denied" / retCode 10005**

- Your API key is valid, but missing the specific permission required by that command.
- Run `bybit auth permissions -o json` to inspect the active key scopes.
- Asset, funding, subaccount, transfer, and some reporting commands typically require wallet-related permissions beyond basic read/trade access.

**"Symbol not found" or category mismatch**

- Check that the symbol matches the selected category, for example `BTCUSDT` with `--category linear` or `BTCUSDT` spot with `--category spot`.
- If a derivatives list command returns parameter errors without a symbol, try leaving the default `linear` category in place or explicitly provide `--settle-coin USDT`.
- Use `bybit market instruments --category <category>` to confirm the exact symbol spelling.

**Config file not found**

- This is normal on first run if you have only set environment variables.
- Run `bybit setup` to create the config file in your platform config directory, or continue using `BYBIT_API_KEY` / `BYBIT_API_SECRET` directly.

**"No paper journal" error**

Run `bybit paper init` to initialize the paper trading account before using other paper commands.

**WebSocket disconnects**

The CLI reconnects automatically with exponential backoff (up to 12 attempts). If reconnects fail, check network connectivity and Bybit's [status page](https://status.bybit.com).

**"MCP tool missing"**

Check the service selection passed to `bybit mcp -s ...`. The default service set is `market,account,paper`; use `-s all` or include the specific group you need.

For machine-readable remediation guidance, see [agents/error-catalog.json](agents/error-catalog.json).

</details>

<details>
<summary>Architecture</summary>

```
src/
  main.rs         — CLI entry point, clap parsing, exit codes
  lib.rs          — AppContext, Command enum, dispatch, apply_default_category
  auth.rs         — HMAC-SHA256 signing, timestamp, AuthHeaders
  config.rs       — Config file I/O, credential resolution, SecretValue wrapper
  client.rs       — HTTP client with retry, envelope parsing, rustls TLS
  errors.rs       — Unified error types with JSON envelopes
  paper.rs        — Paper trading state machine (market + limit orders)
  shell.rs        — Interactive REPL with rustyline
  telemetry.rs    — Instance ID, agent detection, request metadata
  commands/
    market.rs     — 17 public market data endpoints
    trade.rs      — Order management + batch ops + cancel-after
    account.rs    — Account info, balances, settings
    position.rs   — Position management
    asset.rs      — Asset transfers, deposits, withdrawals
    funding.rs    — Funding and wallet workflow aliases
    subaccount.rs — Master-account subaccount management
    futures.rs    — Derivatives-focused command namespace
    reports.rs    — Read-only reporting and history aliases
    websocket.rs  — WebSocket streaming with reconnect
    paper.rs      — Paper trading commands
    auth.rs       — Credential management (set/sign/test/show/reset)
    utility.rs    — Setup wizard
    helpers.rs    — Shared confirm(), build_params()
  mcp/            — MCP server, registry, and schema helpers
  output/
    json.rs       — JSON output
    table.rs      — comfy-table rendering
```

</details>

## Development

```bash
cargo build                         # dev build
cargo build --release               # optimized build
cargo test                          # all tests
cargo clippy -- -D warnings         # lint
cargo audit -D warnings             # security advisories
cargo fmt                           # format
```

Live smoke tests are opt-in and do not run during normal `cargo test`.

```bash
BYBIT_RUN_LIVE_PUBLIC=1 cargo test --test live_smoke
BYBIT_RUN_LIVE_PUBLIC=1 BYBIT_RUN_LIVE_WS=1 cargo test --test live_smoke
BYBIT_RUN_LIVE_TESTNET=1 BYBIT_TESTNET_API_KEY=... BYBIT_TESTNET_API_SECRET=... cargo test --test live_smoke
BYBIT_RUN_LIVE_TESTNET=1 BYBIT_RUN_LIVE_WS=1 BYBIT_TESTNET_API_KEY=... BYBIT_TESTNET_API_SECRET=... cargo test --test live_smoke
BYBIT_RUN_LIVE_EARN=1 BYBIT_TESTNET_API_KEY=... BYBIT_TESTNET_API_SECRET=... cargo test --test live_smoke live_testnet_earn_positions_smoke
```

On PowerShell:

```powershell
$env:BYBIT_RUN_LIVE_PUBLIC = "1"
cargo test --test live_smoke

$env:BYBIT_RUN_LIVE_TESTNET = "1"
$env:BYBIT_TESTNET_API_KEY = "..."
$env:BYBIT_TESTNET_API_SECRET = "..."
cargo test --test live_smoke
```

## Contributing

Bug reports, feature requests, and pull requests are welcome. See [CONTRIBUTING.md](CONTRIBUTING.md) for contribution guidelines and repo conventions.

## License & Disclaimer

MIT. See [LICENSE](LICENSE).

This is unofficial community-maintained software. It interacts with the live Bybit exchange and can result in real financial transactions. Orders, withdrawals, and transfers are irreversible once processed. See [DISCLAIMER.md](DISCLAIMER.md) for full terms and risk disclosure.
