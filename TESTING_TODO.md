# bybit-cli Final Verification Checklist

This document tracks the manual and automated verification of all `bybit-cli` features before public release.

## 1. Market Data (Public)
- [ ] `bybit market server-time`: Verify server synchronization.
- [ ] `bybit market tickers`: Test for both `spot` and `linear` categories.
- [ ] `bybit market orderbook`: Verify depth limits (e.g., `--limit 5`).
- [ ] `bybit market spread`: Verify calculation of mid-price and spread %.
- [ ] `bybit market kline`: Test various intervals (`1`, `D`, `W`).
- [ ] `bybit market funding-rate`: Check historical records.
- [ ] `bybit market open-interest`: Verify 5min/1h intervals.
- [ ] `bybit market risk-limit`: Verify tier data for high-leverage symbols.
- [ ] `bybit account adl-alert`: Check for system-wide alerts.
- [ ] `bybit market ls-ratio`: Verify long/short sentiment data.

## 2. Account & Asset (Private)
- [ ] `bybit auth test`: Confirm credentials and UTA status.
- [ ] `bybit auth permissions`: Verify active API key permissions and IP whitelist.
- [ ] `bybit account balance`: Verify standard view.
- [ ] `bybit account balance --extended`: Verify detailed margin fields.
- [ ] `bybit account extended-balance`: Verify per-coin margin breakdown.
- [ ] `bybit account fee-rate`: Check taker/maker tiers.
- [ ] `bybit account volume`: Verify 30-day calculation logic.
- [ ] `bybit asset withdrawal-methods`: Verify network and fee table.
- [ ] `bybit asset transferable`: Check move-limits between FUND and UNIFIED.

## 3. Trading & Positions (Private)
- [ ] `bybit trade buy/sell --validate`: Verify dry-run logic for all new flags.
- [ ] `bybit trade buy --display-qty`: Test Iceberg order parameters.
- [ ] `bybit trade buy --post-only`: Verify TIF is set to PostOnly.
- [ ] `bybit trade buy --tp-limit-price`: Test Limit-style TP/SL attachments.
- [ ] `bybit trade buy --trigger-price`: Verify conditional order entry.
- [ ] `bybit trade batch-place`: Verify JSON array submission.
- [ ] `bybit trade cancel-after`: Verify "Dead Man's Switch" activation.
- [ ] `bybit position list`: Verify `adlRankIndicator` visibility.
- [ ] `bybit position trailing-stop`: Verify retracement distance setting.
- [ ] `bybit position flatten`: **CRITICAL** - Verify atomic cancel + close logic.

## 4. Earn / Staking (Private)
- [ ] `bybit earn products`: List available savings plans.
- [ ] `bybit earn positions`: Check active staking balances.
- [ ] `bybit earn history`: Verify purchase/redemption logs.

## 5. Subaccount Management (Master Only)
- [ ] `bybit subaccount list`: View child accounts.
- [ ] `bybit subaccount freeze/unfreeze`: Verify trading suspension toggle.

## 6. WebSocket Streaming
- [ ] `bybit ws orderbook`: Verify real-time snapshots and deltas.
- [ ] `bybit ws notifications`: Verify combined private stream (Orders/Wallet/Pos).

## 7. Paper Trading (Simulated)
- [ ] `bybit paper init`: Verify journal creation.
- [ ] `bybit paper buy/sell`: Verify live-price execution.
- [ ] `bybit paper status`: Verify P&L calculation and balance tracking.

## 8. Technical & Release
- [ ] `cargo build --release`: Verify optimized compilation.
- [ ] `cargo dist plan`: Verify release orchestration (if installed).
- [ ] `bybit --help`: Verify all new subcommands appear in help menus.
