---
name: bybit-expert
description: Bybit-CLI machine-readable artifacts and integration guide for AI agents.
---
# bybit-cli Agent Integration

This directory contains machine-readable artifacts for AI agents integrating with `bybit-cli`.

## Files

| File | Purpose |
|------|---------|
| `tool-catalog.json` | Canonical CLI/agent command catalog with parameters, auth requirements, safety flags, and examples |
| `mcp-tool-catalog.json` | Runtime MCP tool catalog generated from the live stdio registry |
| `error-catalog.json` | All error types with ret-codes, retry guidance, and remediation steps |
| `examples/README.md` | Curated shell-script examples for read-only, paper, and state-changing workflows |
| `../gemini-extension.json` | Gemini CLI extension manifest |

## Quick Start for Agents

### Discover available commands
```bash
bybit --help
bybit market --help
bybit trade --help
```

### Check authentication
```bash
bybit auth test
```

### Safe read-only commands (no credentials needed)
```bash
bybit market tickers --category linear --symbol BTCUSDT
bybit market orderbook --category linear --symbol BTCUSDT
bybit market kline --category linear --symbol BTCUSDT --interval 60
```

### Output formats
```bash
# JSON output for programmatic consumption
bybit market tickers --category linear -o json

# Table output for human display (default)
bybit market tickers --category linear -o table
```

### Confirmation prompts
Dangerous commands (place order, cancel, transfer, withdraw) prompt for confirmation.
Pass `-y` / `--yes` to skip in automated contexts:
```bash
bybit trade buy --symbol BTCUSDT --qty 0.01 --price 50000 -y
```

## Safety Rules for Agents

1. **Never use `--yes` without explicit user approval** for withdrawal or transfer commands.
2. **Always use `--validate` for dry-run checks** before placing real orders.
3. **Check balances before trading**: `bybit account balance` / `bybit asset balance`.
4. **Prefer paper trading** for strategy testing: `bybit paper ...` for spot and `bybit futures paper ...` for perpetual futures.
5. **Rate limits**: If you see `error: rate_limit`, wait before retrying. The CLI only retries transient network and HTTP 5xx failures automatically, not Bybit rate-limit responses.
6. **Testnet**: Use `--testnet` flag for all testing. Testnet credentials differ from mainnet.

## Error Handling

All errors are printed as JSON to stderr with this structure:
```json
{
  "error": "auth|rate_limit|api|network|config|parse|validation|paper|websocket|io",
  "message": "human-readable description",
  "ret_code": null
}
```

`ret_code` is present on every error envelope. It is `null` when the failure did not come from a Bybit API retCode.

See `error-catalog.json` for the full error taxonomy and remediation steps.

## MCP Server

> Available via `bybit mcp`. Use `bybit mcp -s all` for the full MCP-visible tool set.
> Dangerous tools remain visible in guarded mode and require `acknowledged=true` unless started with `--allow-dangerous` for autonomous mode.

Use `mcp-tool-catalog.json` for the runtime MCP surface and `tool-catalog.json` for the broader CLI/agent command catalog.

The default service set is `market,account,paper,futures-paper`.

Persisted local state is shared across CLI and MCP usage: saved credentials, the spot paper journal, the futures paper state, shell history, and the anonymous instance ID survive across tool calls and server restarts until reset or deleted.

## Configuration

Preferred secret input methods:
1. `--api-secret-stdin` or `--api-secret-file` with `--api-key` for local/manual runs
2. `BYBIT_API_KEY` / `BYBIT_API_SECRET` for injected automation
3. Platform config file for interactive local use

Examples:
```bash
printf '%s\n' 'your-secret' | bybit --api-key your-key --api-secret-stdin auth test
bybit --api-key your-key --api-secret-file ~/.config/bybit/api-secret.txt auth test
```

The CLI resolves credentials in priority order:
1. CLI flags (`--api-key`, `--api-secret`)
2. Environment variables (`BYBIT_API_KEY`, `BYBIT_API_SECRET`)
3. Platform config file (`~/.config/bybit/config.toml` on Linux, `~/Library/Application Support/bybit/config.toml` on macOS, `%APPDATA%\\bybit\\config.toml` on Windows)

Run `bybit setup` to configure interactively.

For local development, `bybit-cli` also loads `.env` from the current working directory or any parent directory. Already-exported environment variables keep precedence.
Avoid `--api-secret` on shared systems because it can appear in process listings.
