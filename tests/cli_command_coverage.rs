use assert_cmd::Command;

#[test]
fn asset_and_funding_command_help_paths_parse() {
    Command::cargo_bin("bybit")
        .unwrap()
        .args(["account", "set-spot-hedging", "--help"])
        .assert()
        .success();

    Command::cargo_bin("bybit")
        .unwrap()
        .args(["asset", "account-balance", "--help"])
        .assert()
        .success();

    Command::cargo_bin("bybit")
        .unwrap()
        .args(["asset", "all-balance", "--help"])
        .assert()
        .success();

    Command::cargo_bin("bybit")
        .unwrap()
        .args(["asset", "cancel-withdraw", "--help"])
        .assert()
        .success();

    Command::cargo_bin("bybit")
        .unwrap()
        .args(["asset", "deposit-address", "--help"])
        .assert()
        .success();

    Command::cargo_bin("bybit")
        .unwrap()
        .args(["asset", "sub-transfer", "--help"])
        .assert()
        .success();

    Command::cargo_bin("bybit")
        .unwrap()
        .args(["asset", "sub-transfer-history", "--help"])
        .assert()
        .success();

    Command::cargo_bin("bybit")
        .unwrap()
        .args(["asset", "transferable", "--help"])
        .assert()
        .success();

    Command::cargo_bin("bybit")
        .unwrap()
        .args(["asset", "withdraw", "--help"])
        .assert()
        .success();

    Command::cargo_bin("bybit")
        .unwrap()
        .args(["funding", "account-balance", "--help"])
        .assert()
        .success();

    Command::cargo_bin("bybit")
        .unwrap()
        .args(["funding", "all-balance", "--help"])
        .assert()
        .success();

    Command::cargo_bin("bybit")
        .unwrap()
        .args(["funding", "cancel-withdraw", "--help"])
        .assert()
        .success();

    Command::cargo_bin("bybit")
        .unwrap()
        .args(["funding", "coin-info", "--help"])
        .assert()
        .success();

    Command::cargo_bin("bybit")
        .unwrap()
        .args(["funding", "deposit-address", "--help"])
        .assert()
        .success();

    Command::cargo_bin("bybit")
        .unwrap()
        .args(["funding", "sub-transfer", "--help"])
        .assert()
        .success();

    Command::cargo_bin("bybit")
        .unwrap()
        .args(["funding", "sub-transfer-history", "--help"])
        .assert()
        .success();

    Command::cargo_bin("bybit")
        .unwrap()
        .args(["funding", "transfer-history", "--help"])
        .assert()
        .success();

    Command::cargo_bin("bybit")
        .unwrap()
        .args(["funding", "transferable", "--help"])
        .assert()
        .success();

    Command::cargo_bin("bybit")
        .unwrap()
        .args(["funding", "withdraw", "--help"])
        .assert()
        .success();

    Command::cargo_bin("bybit")
        .unwrap()
        .args(["funding", "withdraw-history", "--help"])
        .assert()
        .success();
}

#[test]
fn futures_and_futures_paper_command_help_paths_parse() {
    Command::cargo_bin("bybit")
        .unwrap()
        .args(["futures", "buy", "--help"])
        .assert()
        .success();

    Command::cargo_bin("bybit")
        .unwrap()
        .args(["futures", "cancel", "--help"])
        .assert()
        .success();

    Command::cargo_bin("bybit")
        .unwrap()
        .args(["futures", "cancel-all", "--help"])
        .assert()
        .success();

    Command::cargo_bin("bybit")
        .unwrap()
        .args(["futures", "fills", "--help"])
        .assert()
        .success();

    Command::cargo_bin("bybit")
        .unwrap()
        .args(["futures", "funding-rate", "--help"])
        .assert()
        .success();

    Command::cargo_bin("bybit")
        .unwrap()
        .args(["futures", "history", "--help"])
        .assert()
        .success();

    Command::cargo_bin("bybit")
        .unwrap()
        .args(["futures", "open-interest", "--help"])
        .assert()
        .success();

    Command::cargo_bin("bybit")
        .unwrap()
        .args(["futures", "open-orders", "--help"])
        .assert()
        .success();

    Command::cargo_bin("bybit")
        .unwrap()
        .args(["futures", "orderbook", "--help"])
        .assert()
        .success();

    Command::cargo_bin("bybit")
        .unwrap()
        .args(["futures", "sell", "--help"])
        .assert()
        .success();

    Command::cargo_bin("bybit")
        .unwrap()
        .args(["futures", "tickers", "--help"])
        .assert()
        .success();

    Command::cargo_bin("bybit")
        .unwrap()
        .args(["futures", "paper", "balance", "--help"])
        .assert()
        .success();

    Command::cargo_bin("bybit")
        .unwrap()
        .args(["futures", "paper", "batch-order", "--help"])
        .assert()
        .success();

    Command::cargo_bin("bybit")
        .unwrap()
        .args(["futures", "paper", "buy", "--help"])
        .assert()
        .success();

    Command::cargo_bin("bybit")
        .unwrap()
        .args(["futures", "paper", "cancel", "--help"])
        .assert()
        .success();

    Command::cargo_bin("bybit")
        .unwrap()
        .args(["futures", "paper", "cancel-all", "--help"])
        .assert()
        .success();

    Command::cargo_bin("bybit")
        .unwrap()
        .args(["futures", "paper", "edit-order", "--help"])
        .assert()
        .success();

    Command::cargo_bin("bybit")
        .unwrap()
        .args(["futures", "paper", "fills", "--help"])
        .assert()
        .success();

    Command::cargo_bin("bybit")
        .unwrap()
        .args(["futures", "paper", "history", "--help"])
        .assert()
        .success();

    Command::cargo_bin("bybit")
        .unwrap()
        .args(["futures", "paper", "init", "--help"])
        .assert()
        .success();

    Command::cargo_bin("bybit")
        .unwrap()
        .args(["futures", "paper", "leverage", "--help"])
        .assert()
        .success();

    Command::cargo_bin("bybit")
        .unwrap()
        .args(["futures", "paper", "order-status", "--help"])
        .assert()
        .success();

    Command::cargo_bin("bybit")
        .unwrap()
        .args(["futures", "paper", "orders", "--help"])
        .assert()
        .success();

    Command::cargo_bin("bybit")
        .unwrap()
        .args(["futures", "paper", "positions", "--help"])
        .assert()
        .success();

    Command::cargo_bin("bybit")
        .unwrap()
        .args(["futures", "paper", "reset", "--help"])
        .assert()
        .success();

    Command::cargo_bin("bybit")
        .unwrap()
        .args(["futures", "paper", "sell", "--help"])
        .assert()
        .success();

    Command::cargo_bin("bybit")
        .unwrap()
        .args(["futures", "paper", "set-leverage", "--help"])
        .assert()
        .success();

    Command::cargo_bin("bybit")
        .unwrap()
        .args(["futures", "paper", "status", "--help"])
        .assert()
        .success();
}

#[test]
fn websocket_command_help_paths_parse() {
    Command::cargo_bin("bybit")
        .unwrap()
        .args(["futures", "ws", "executions", "--help"])
        .assert()
        .success();

    Command::cargo_bin("bybit")
        .unwrap()
        .args(["futures", "ws", "kline", "--help"])
        .assert()
        .success();

    Command::cargo_bin("bybit")
        .unwrap()
        .args(["futures", "ws", "liquidation", "--help"])
        .assert()
        .success();

    Command::cargo_bin("bybit")
        .unwrap()
        .args(["futures", "ws", "orderbook", "--help"])
        .assert()
        .success();

    Command::cargo_bin("bybit")
        .unwrap()
        .args(["futures", "ws", "orders", "--help"])
        .assert()
        .success();

    Command::cargo_bin("bybit")
        .unwrap()
        .args(["futures", "ws", "positions", "--help"])
        .assert()
        .success();

    Command::cargo_bin("bybit")
        .unwrap()
        .args(["futures", "ws", "ticker", "--help"])
        .assert()
        .success();

    Command::cargo_bin("bybit")
        .unwrap()
        .args(["futures", "ws", "trades", "--help"])
        .assert()
        .success();

    Command::cargo_bin("bybit")
        .unwrap()
        .args(["futures", "ws", "wallet", "--help"])
        .assert()
        .success();

    Command::cargo_bin("bybit")
        .unwrap()
        .args(["ws", "dcp", "--help"])
        .assert()
        .success();

    Command::cargo_bin("bybit")
        .unwrap()
        .args(["ws", "executions", "--help"])
        .assert()
        .success();

    Command::cargo_bin("bybit")
        .unwrap()
        .args(["ws", "kline", "--help"])
        .assert()
        .success();

    Command::cargo_bin("bybit")
        .unwrap()
        .args(["ws", "liquidation", "--help"])
        .assert()
        .success();

    Command::cargo_bin("bybit")
        .unwrap()
        .args(["ws", "lt-kline", "--help"])
        .assert()
        .success();

    Command::cargo_bin("bybit")
        .unwrap()
        .args(["ws", "lt-ticker", "--help"])
        .assert()
        .success();

    Command::cargo_bin("bybit")
        .unwrap()
        .args(["ws", "notifications", "--help"])
        .assert()
        .success();

    Command::cargo_bin("bybit")
        .unwrap()
        .args(["ws", "orderbook", "--help"])
        .assert()
        .success();

    Command::cargo_bin("bybit")
        .unwrap()
        .args(["ws", "orders", "--help"])
        .assert()
        .success();

    Command::cargo_bin("bybit")
        .unwrap()
        .args(["ws", "positions", "--help"])
        .assert()
        .success();

    Command::cargo_bin("bybit")
        .unwrap()
        .args(["ws", "ticker", "--help"])
        .assert()
        .success();

    Command::cargo_bin("bybit")
        .unwrap()
        .args(["ws", "trades", "--help"])
        .assert()
        .success();

    Command::cargo_bin("bybit")
        .unwrap()
        .args(["ws", "wallet", "--help"])
        .assert()
        .success();
}

#[test]
fn market_position_reports_and_trade_help_paths_parse() {
    Command::cargo_bin("bybit")
        .unwrap()
        .args(["earn", "redeem", "--help"])
        .assert()
        .success();

    Command::cargo_bin("bybit")
        .unwrap()
        .args(["market", "delivery-price", "--help"])
        .assert()
        .success();

    Command::cargo_bin("bybit")
        .unwrap()
        .args(["market", "index-price-kline", "--help"])
        .assert()
        .success();

    Command::cargo_bin("bybit")
        .unwrap()
        .args(["market", "insurance", "--help"])
        .assert()
        .success();

    Command::cargo_bin("bybit")
        .unwrap()
        .args(["market", "mark-price-kline", "--help"])
        .assert()
        .success();

    Command::cargo_bin("bybit")
        .unwrap()
        .args(["market", "premium-index-kline", "--help"])
        .assert()
        .success();

    Command::cargo_bin("bybit")
        .unwrap()
        .args(["market", "volatility", "--help"])
        .assert()
        .success();

    Command::cargo_bin("bybit")
        .unwrap()
        .args(["paper", "positions", "--help"])
        .assert()
        .success();

    Command::cargo_bin("bybit")
        .unwrap()
        .args(["position", "add-margin", "--help"])
        .assert()
        .success();

    Command::cargo_bin("bybit")
        .unwrap()
        .args(["position", "move", "--help"])
        .assert()
        .success();

    Command::cargo_bin("bybit")
        .unwrap()
        .args(["position", "move-history", "--help"])
        .assert()
        .success();

    Command::cargo_bin("bybit")
        .unwrap()
        .args(["position", "set-risk-limit", "--help"])
        .assert()
        .success();

    Command::cargo_bin("bybit")
        .unwrap()
        .args(["reports", "borrow-history", "--help"])
        .assert()
        .success();

    Command::cargo_bin("bybit")
        .unwrap()
        .args(["reports", "deposits", "--help"])
        .assert()
        .success();

    Command::cargo_bin("bybit")
        .unwrap()
        .args(["reports", "fills", "--help"])
        .assert()
        .success();

    Command::cargo_bin("bybit")
        .unwrap()
        .args(["reports", "moves", "--help"])
        .assert()
        .success();

    Command::cargo_bin("bybit")
        .unwrap()
        .args(["reports", "sub-transfers", "--help"])
        .assert()
        .success();

    Command::cargo_bin("bybit")
        .unwrap()
        .args(["reports", "transfers", "--help"])
        .assert()
        .success();

    Command::cargo_bin("bybit")
        .unwrap()
        .args(["reports", "withdrawals", "--help"])
        .assert()
        .success();

    Command::cargo_bin("bybit")
        .unwrap()
        .args(["subaccount", "list-all", "--help"])
        .assert()
        .success();

    Command::cargo_bin("bybit")
        .unwrap()
        .args(["trade", "batch-amend", "--help"])
        .assert()
        .success();

    Command::cargo_bin("bybit")
        .unwrap()
        .args(["trade", "batch-cancel", "--help"])
        .assert()
        .success();
}
