use clap::{Args as _, Command as ClapCommand, CommandFactory, Parser, Subcommand as _};

use bybit_cli::commands::{
    account::AccountArgs, earn::EarnArgs, market::MarketArgs, reports::ReportsArgs,
};
use bybit_cli::{Cli, Command};

#[test]
fn market_args_command_builds() {
    let _ = MarketArgs::augment_args(ClapCommand::new("market"));
}

#[test]
fn account_args_command_builds() {
    let _ = AccountArgs::augment_args(ClapCommand::new("account"));
}

#[test]
fn earn_args_command_builds() {
    let _ = EarnArgs::augment_args(ClapCommand::new("earn"));
}

#[test]
fn reports_args_command_builds() {
    let _ = ReportsArgs::augment_args(ClapCommand::new("reports"));
}

#[test]
fn command_enum_builds() {
    let _ = Command::augment_subcommands(ClapCommand::new("bybit"));
}

#[test]
fn cli_command_builds() {
    let _ = Cli::command();
}

#[test]
fn cli_parse_help_builds() {
    let _ = Cli::try_parse_from(["bybit", "--help"]);
}

#[test]
fn cli_help_renders() {
    let mut cmd = Cli::command();
    let _ = cmd.render_help().to_string();
}
