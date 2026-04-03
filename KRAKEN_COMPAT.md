# Kraken Compatibility Notes

`bybit-cli` is inspired by `kraken-cli`, but it is not a drop-in replacement. The command style is intentionally similar, while exchange-specific behavior, symbols, permissions, and product categories differ.

## Conceptual Mapping

| Workflow | bybit-cli surface | Notes |
|----------|-------------------|-------|
| Public market data | `bybit market ...` | Bybit uses `--category` heavily across market queries |
| Spot and derivatives order entry | `bybit trade ...` | Spot, linear, inverse, and option workflows depend on category and endpoint support |
| Contract-centric futures flows | `bybit futures ...` | Separate namespace for perpetual/futures-specific views and actions |
| Position management | `bybit position ...` | Leverage, TP/SL, margin, mode switching, flattening |
| Funding and asset movement | `bybit asset ...`, `bybit funding ...` | Transfers, withdrawals, deposit history, wallet-specific views |
| Reports and exports | `bybit reports ...` | Moves, fills, closed PnL, transfers, withdrawals |
| WebSocket streaming | `bybit ws ...` | Public and private topics exposed as streaming commands |
| Paper trading | `bybit paper ...`, `bybit futures paper ...` | Separate spot and futures simulators backed by live public data |
| Auth and local setup | `bybit auth ...`, `bybit setup`, `bybit shell`, `bybit mcp` | Local credential management, REPL, and MCP integration |

## Migration Notes

- Treat the mapping as conceptual, not command-for-command compatibility.
- Replace Kraken symbols and market assumptions with Bybit symbols and `--category` values such as `spot`, `linear`, `inverse`, and `option`.
- Update automation to read JSON from stdout and JSON error envelopes from stderr.
- Replace Kraken-specific environment variables with `BYBIT_API_KEY` and `BYBIT_API_SECRET`.
- For local or shared systems, prefer `--api-secret-stdin` or `--api-secret-file` over `--api-secret`.
- Use `--testnet`, `bybit paper ...`, or `bybit futures paper ...` when porting automation before touching live endpoints.
- Review permissions carefully. Bybit separates capabilities across account type, category, and API key scope.
