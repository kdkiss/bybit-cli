use bybit_cli::commands::{
    futures::{FuturesArgs, FuturesCommand},
    market::{MarketArgs, MarketCommand},
    paper::{PaperArgs, PaperCommand},
    reports::{ReportsArgs, ReportsCommand},
};
use bybit_cli::Command;

#[test]
fn apply_default_category_updates_linear_defaults() {
    let mut command = Command::Market(MarketArgs {
        command: MarketCommand::Orderbook {
            category: "linear".to_string(),
            symbol: "BTCUSDT".to_string(),
            limit: 50,
        },
    });

    command.apply_default_category("spot");

    match command {
        Command::Market(MarketArgs {
            command: MarketCommand::Orderbook { category, .. },
        }) => assert_eq!(category, "spot"),
        _ => panic!("unexpected command variant"),
    }
}

#[test]
fn apply_default_category_leaves_specialized_defaults_intact() {
    let mut market_command = Command::Market(MarketArgs {
        command: MarketCommand::Volatility {
            category: "option".to_string(),
            base_coin: None,
            period: None,
            start: None,
            end: None,
        },
    });
    let mut paper_command = Command::Paper(PaperArgs {
        command: PaperCommand::Buy {
            category: "linear".to_string(),
            symbol: "BTCUSDT".to_string(),
            qty: 1.0,
            price: None,
        },
    });

    market_command.apply_default_category("spot");
    paper_command.apply_default_category("spot");

    match market_command {
        Command::Market(MarketArgs {
            command: MarketCommand::Volatility { category, .. },
        }) => assert_eq!(category, "option"),
        _ => panic!("unexpected market command variant"),
    }

    match paper_command {
        Command::Paper(PaperArgs {
            command: PaperCommand::Buy { category, .. },
        }) => assert_eq!(category, "spot"),
        _ => panic!("unexpected paper command variant"),
    }
}

#[test]
fn apply_default_category_updates_futures_for_derivatives_defaults() {
    let mut command = Command::Futures(FuturesArgs {
        command: FuturesCommand::Positions {
            category: "linear".to_string(),
            symbol: Some("BTCUSDT".to_string()),
            base_coin: None,
            settle_coin: None,
            limit: None,
            cursor: None,
        },
    });

    command.apply_default_category("inverse");

    match command {
        Command::Futures(FuturesArgs {
            command: FuturesCommand::Positions { category, .. },
        }) => assert_eq!(category, "inverse"),
        _ => panic!("unexpected command variant"),
    }
}

#[test]
fn apply_default_category_does_not_force_spot_into_futures_namespace() {
    let mut command = Command::Futures(FuturesArgs {
        command: FuturesCommand::Tickers {
            category: "linear".to_string(),
            symbol: Some("BTCUSDT".to_string()),
            base_coin: None,
        },
    });

    command.apply_default_category("spot");

    match command {
        Command::Futures(FuturesArgs {
            command: FuturesCommand::Tickers { category, .. },
        }) => assert_eq!(category, "linear"),
        _ => panic!("unexpected command variant"),
    }
}

#[test]
fn apply_default_category_updates_reports_defaults() {
    let mut command = Command::Reports(ReportsArgs {
        command: ReportsCommand::Orders {
            category: "linear".to_string(),
            symbol: Some("BTCUSDT".to_string()),
            order_id: None,
            order_status: None,
            start: None,
            end: None,
            limit: None,
            cursor: None,
        },
    });

    command.apply_default_category("spot");

    match command {
        Command::Reports(ReportsArgs {
            command: ReportsCommand::Orders { category, .. },
        }) => assert_eq!(category, "spot"),
        _ => panic!("unexpected command variant"),
    }
}
