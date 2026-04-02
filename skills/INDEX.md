# bybit-cli Skills Index

Goal-oriented workflow packages for AI agents and human operators.

## Core Skills

| Skill | Description |
|-------|-------------|
| [check-price](check-price/SKILL.md) | Get current price, 24h stats, and funding rate for any symbol |
| [place-limit-order](place-limit-order/SKILL.md) | Place a limit buy or sell with optional TP/SL |
| [manage-position](manage-position/SKILL.md) | View, adjust leverage, and set TP/SL on a position |
| [account-snapshot](account-snapshot/SKILL.md) | Full account balance and position snapshot |
| [monitor-funding](monitor-funding/SKILL.md) | Check and stream funding rates |
| [cancel-all-orders](cancel-all-orders/SKILL.md) | Safely cancel all open orders |
| [bybit-futures-trading](bybit-futures-trading/SKILL.md) | Manage the live and paper futures order lifecycle |
| [bybit-paper-strategy](bybit-paper-strategy/SKILL.md) | Test spot strategies on paper trading before going live |
| [bybit-paper-to-live](bybit-paper-to-live/SKILL.md) | Promote validated paper workflows to live trading with guardrails |
| [paper-trading-session](paper-trading-session/SKILL.md) | Run a complete paper trading session |
| [stream-orderbook](stream-orderbook/SKILL.md) | Stream real-time order book data |
| [transfer-funds](transfer-funds/SKILL.md) | Transfer assets between account types |
| [verify-setup](verify-setup/SKILL.md) | Verify credentials and connectivity |

## Recipes

Multi-step workflows combining several commands:

| Recipe | Description |
|--------|-------------|
| [bybit-recipe-basis-trading](bybit-recipe-basis-trading/SKILL.md) | Capture the Spot-Futures basis premium |
| [bybit-recipe-batch-limit-ladder](bybit-recipe-batch-limit-ladder/SKILL.md) | Place a ladder of limit orders in one call |
| [bybit-recipe-close-position](bybit-recipe-close-position/SKILL.md) | Fully close an open position at market or limit |
| [bybit-recipe-dca-buy](bybit-recipe-dca-buy/SKILL.md) | Dollar-cost averaging entry strategy |
| [bybit-recipe-drawdown-circuit-breaker](bybit-recipe-drawdown-circuit-breaker/SKILL.md) | Stop trading if losses hit a threshold |
| [bybit-recipe-emergency-flatten](bybit-recipe-emergency-flatten/SKILL.md) | Nuclear option: close all trades immediately |
| [bybit-recipe-grid-trading](bybit-recipe-grid-trading/SKILL.md) | Deploy a grid trading strategy for volatility |
| [bybit-recipe-liquidation-guard](bybit-recipe-liquidation-guard/SKILL.md) | Automatically flatten if margin is at risk |
| [bybit-recipe-morning-brief](bybit-recipe-morning-brief/SKILL.md) | Daily account and market summary |
| [bybit-recipe-paper-backtest](bybit-recipe-paper-backtest/SKILL.md) | Simulate a strategy with paper trading |
| [bybit-recipe-set-breakeven](bybit-recipe-set-breakeven/SKILL.md) | Move stop-loss to break-even after entry |
| [bybit-recipe-trailing-stop-runner](bybit-recipe-trailing-stop-runner/SKILL.md) | Dynamic trailing stop to lock in profits |
| [bybit-recipe-twap-execution](bybit-recipe-twap-execution/SKILL.md) | Time-weighted average price order execution |

## Usage

All commands support `-o json` for programmatic output and `--testnet` for safe testing.

### Pattern for agents

```bash
# 1. Always check current state first
bybit market tickers --category linear --symbol BTCUSDT -o json

# 2. Dry-run before placing orders
bybit trade buy --symbol BTCUSDT --qty 0.01 --price 50000 --validate

# 3. Use paper trading for strategy testing
bybit paper init && bybit paper buy --symbol BTCUSDT --qty 0.1
bybit paper status

# 3b. Use futures paper trading for leveraged strategy testing
bybit futures paper init --balance 10000
bybit futures paper buy BTCUSDT 0.01 --leverage 10 --type market
bybit futures paper status

# 4. Gate dangerous operations behind explicit confirmation
bybit trade buy --symbol BTCUSDT --qty 0.01 --price 50000   # prompts user
bybit trade buy --symbol BTCUSDT --qty 0.01 --price 50000 -y  # skip prompt (automation only)
```

### Safety rules

1. **Never use `-y` on withdrawal or transfer commands** without explicit user approval
2. **Always `--validate`** before placing real orders
3. **Check balances first**: `bybit account balance` / `bybit asset balance`
4. **Use paper trading** for strategy development: `bybit paper ...` for spot, `bybit futures paper ...` for perpetual futures
5. **Use `--testnet`** for integration testing
6. **Set a dead man's switch** when running automated strategies: `bybit trade cancel-after 300`
