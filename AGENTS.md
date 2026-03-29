# bybit-cli Agent Integration Guide

**UNOFFICIAL community-maintained software.**

This document describes how AI agents and LLM-based tools can integrate with `bybit-cli`.

## Overview

`bybit-cli` is designed to be agent-friendly:

- All output is available in JSON (`-o json`) for easy parsing
- All errors are printed to stderr as JSON objects
- Dangerous operations require explicit confirmation (bypassable with `-y`)
- The `--validate` flag enables dry-run order placement
- Machine-readable tool and error catalogs live in `agents/`

## Artifacts

| File | Purpose |
|------|---------|
| `agents/tool-catalog.json` | Command catalog with parameters, auth requirements, and examples |
| `agents/error-catalog.json` | Error taxonomy with ret-codes and remediation guidance |
| `CLAUDE.md` | Context and safety rules for Claude-based agents |

## Invoking bybit-cli from an Agent

### Basic invocation pattern

```python
import subprocess, json

def bybit(args: list[str]) -> dict:
    result = subprocess.run(
        ["bybit", "-o", "json"] + args,
        capture_output=True, text=True
    )
    if result.returncode != 0:
        error = json.loads(result.stderr)
        raise RuntimeError(f"bybit error: {error}")
    return json.loads(result.stdout)

# Example: get BTC price
ticker = bybit(["market", "tickers", "--category", "linear", "--symbol", "BTCUSDT"])
price = ticker["list"][0]["lastPrice"]
```

### Error handling

```python
try:
    result = bybit(["trade", "buy", "--symbol", "BTCUSDT", "--qty", "0.01", "--price", "50000", "-y"])
except RuntimeError as e:
    error = json.loads(str(e).split("bybit error: ")[1])
    if error["error"] == "rate_limit":
        time.sleep(60)  # back off
    elif error["error"] == "auth":
        # credentials issue
        pass
```

## Safety Checklist

Before placing real orders, agents should:

1. **Verify credentials**: `bybit auth test`
2. **Check balance**: `bybit account balance -o json`
3. **Dry-run the order**: `bybit trade buy ... --validate`
4. **Get current price**: `bybit market tickers --symbol <SYM> -o json`
5. **Confirm with user** before using `-y` on withdrawal/transfer commands

## MCP Integration

The built-in MCP server is available over stdio:

```bash
bybit mcp
bybit mcp -s all
bybit mcp -s all --allow-dangerous
```

This exposes Bybit command groups as structured MCP tools over stdio. In guarded mode, dangerous tools stay visible but require `acknowledged=true` per call unless the server is started with `--allow-dangerous`.

## Credential Handling

Agents should never hardcode credentials. Resolution order:

1. `--api-key` / `--api-secret` CLI flags
2. `BYBIT_API_KEY` / `BYBIT_API_SECRET` environment variables
3. `~/.config/bybit/config.toml`

For automated agents, use environment variables injected at runtime.

## Rate Limit Awareness

The CLI automatically retries transient errors (HTTP 5xx, network failures) up to 3 times with exponential backoff (500ms → 1s → 2s). Rate limit errors from Bybit (retCode 10006/10018) are surfaced immediately and not retried automatically — agents should implement their own back-off strategy when hitting rate limits.

## Testnet

Always use `--testnet` for agent testing:

```bash
bybit --testnet market tickers --category linear --symbol BTCUSDT
bybit --testnet trade buy --symbol BTCUSDT --qty 0.01 --price 50000 --validate
```

Testnet credentials are separate from mainnet and can be obtained at https://testnet.bybit.com.
