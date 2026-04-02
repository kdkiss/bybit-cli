# bybit-cli Skills Library

Ready-to-use skill templates for common Bybit trading workflows. Each skill documents a specific task with step-by-step commands.

## Core Skills

| Skill | Description |
|-------|-------------|
| [check-price](check-price/SKILL.md) | Get the current price and 24h stats for any symbol |
| [place-limit-order](place-limit-order/SKILL.md) | Place a limit buy or sell order with TP/SL |
| [manage-position](manage-position/SKILL.md) | View, adjust leverage, and set TP/SL on a position |
| [account-snapshot](account-snapshot/SKILL.md) | Full account balance and position snapshot |
| [monitor-funding](monitor-funding/SKILL.md) | Check and stream funding rates |
| [cancel-all-orders](cancel-all-orders/SKILL.md) | Safely cancel all open orders |
| [bybit-futures-trading](bybit-futures-trading/SKILL.md) | Manage the live and paper futures order lifecycle |
| [bybit-paper-strategy](bybit-paper-strategy/SKILL.md) | Test spot strategies on paper trading before going live |
| [bybit-paper-to-live](bybit-paper-to-live/SKILL.md) | Promote validated paper workflows to live trading with guardrails |
| [paper-trading-session](paper-trading-session/SKILL.md) | Run a complete paper trading session |
| [stream-orderbook](stream-orderbook/SKILL.md) | Stream real-time order book data |
| [transfer-funds](transfer-funds/SKILL.md) | Transfer between account types |
| [verify-setup](verify-setup/SKILL.md) | Verify credentials and connectivity |

## Recipes

Multi-step workflows combining several commands:

| Recipe | Description |
|--------|-------------|
| [bybit-recipe-basis-trading](bybit-recipe-basis-trading/SKILL.md) | Capture the Spot-Futures basis premium |
| [bybit-recipe-batch-limit-ladder](bybit-recipe-batch-limit-ladder/SKILL.md) | Place a ladder of limit orders |
| [bybit-recipe-close-position](bybit-recipe-close-position/SKILL.md) | Fully close an open position |
| [bybit-recipe-dca-buy](bybit-recipe-dca-buy/SKILL.md) | Dollar-cost averaging entry strategy |
| [bybit-recipe-drawdown-circuit-breaker](bybit-recipe-drawdown-circuit-breaker/SKILL.md) | Portfolio drawdown protection |
| [bybit-recipe-emergency-flatten](bybit-recipe-emergency-flatten/SKILL.md) | Close all trades immediately |
| [bybit-recipe-grid-trading](bybit-recipe-grid-trading/SKILL.md) | Deploy a grid trading strategy |
| [bybit-recipe-liquidation-guard](bybit-recipe-liquidation-guard/SKILL.md) | Account liquidation protection |
| [bybit-recipe-morning-brief](bybit-recipe-morning-brief/SKILL.md) | Daily account and market summary |
| [bybit-recipe-paper-backtest](bybit-recipe-paper-backtest/SKILL.md) | Simulate a strategy with paper trading |
| [bybit-recipe-set-breakeven](bybit-recipe-set-breakeven/SKILL.md) | Move stop-loss to break-even |
| [bybit-recipe-trailing-stop-runner](bybit-recipe-trailing-stop-runner/SKILL.md) | Trailing stop to lock in profits |
| [bybit-recipe-twap-execution](bybit-recipe-twap-execution/SKILL.md) | Time-weighted order execution |

## Usage

Skills are documentation — copy the commands and adapt to your symbols and parameters.

All commands support `-o json` for programmatic use and `--testnet` for safe testing.
