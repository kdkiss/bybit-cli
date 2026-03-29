//! Interactive REPL shell for bybit-cli.
//!
//! Launched via `bybit shell`. Provides a persistent prompt with command
//! history, basic tab-completion, and full access to all CLI commands.

use rustyline::completion::{Completer, FilenameCompleter, Pair};
use rustyline::error::ReadlineError;
use rustyline::highlight::Highlighter;
use rustyline::hint::Hinter;
use rustyline::validate::Validator;
use rustyline::{CompletionType, Config as RlConfig, Context, Editor};
use rustyline_derive::Helper;

use clap::Parser;

use crate::config::history_path;
use crate::errors::BybitResult;
use crate::{has_option_flag, AppContext};

// ---------------------------------------------------------------------------
// Top-level commands the completer knows about
// ---------------------------------------------------------------------------

const TOP_COMMANDS: &[&str] = &[
    "market",
    "trade",
    "account",
    "position",
    "asset",
    "funding",
    "subaccount",
    "earn",
    "futures",
    "ws",
    "paper",
    "reports",
    "auth",
    "setup",
    "help",
    "exit",
    "quit",
];

const SUBCOMMANDS: &[(&str, &[&str])] = &[
    (
        "market",
        &[
            "server-time",
            "instruments",
            "orderbook",
            "tickers",
            "kline",
            "mark-price-kline",
            "index-price-kline",
            "premium-index-kline",
            "funding-rate",
            "trades",
            "open-interest",
            "volatility",
            "insurance",
            "risk-limit",
            "delivery-price",
            "ls-ratio",
            "adl-alert",
            "spread",
        ],
    ),
    (
        "trade",
        &[
            "buy",
            "sell",
            "amend",
            "cancel",
            "cancel-all",
            "open-orders",
            "history",
            "fills",
            "batch-place",
            "batch-amend",
            "batch-cancel",
            "cancel-after",
        ],
    ),
    (
        "account",
        &[
            "balance",
            "extended-balance",
            "info",
            "fee-rate",
            "transaction-log",
            "borrow-history",
            "collateral-info",
            "greeks",
            "set-margin-mode",
            "set-spot-hedging",
            "set-usdc-settlement",
            "volume",
        ],
    ),
    (
        "position",
        &[
            "list",
            "set-leverage",
            "switch-mode",
            "set-tpsl",
            "trailing-stop",
            "set-risk-limit",
            "add-margin",
            "closed-pnl",
            "move",
            "move-history",
            "flatten",
        ],
    ),
    (
        "asset",
        &[
            "coin-info",
            "balance",
            "all-balance",
            "transferable",
            "transfer",
            "transfer-history",
            "sub-transfer",
            "deposit-address",
            "withdrawal-methods",
            "deposit-history",
            "withdraw",
            "withdraw-history",
            "cancel-withdraw",
            "account-balance",
            "sub-transfer-history",
        ],
    ),
    (
        "funding",
        &[
            "coin-info",
            "balance",
            "all-balance",
            "account-balance",
            "transferable",
            "transfer",
            "transfer-history",
            "sub-transfer",
            "sub-transfer-history",
            "deposit-address",
            "deposit-history",
            "withdraw",
            "withdraw-history",
            "cancel-withdraw",
        ],
    ),
    (
        "subaccount",
        &[
            "list",
            "list-all",
            "wallet-types",
            "api-keys",
            "create",
            "delete",
            "freeze",
            "unfreeze",
        ],
    ),
    (
        "earn",
        &[
            "products",
            "positions",
            "stake",
            "redeem",
            "history",
            "yield",
            "hourly-yield",
        ],
    ),
    (
        "futures",
        &[
            "instruments",
            "tickers",
            "orderbook",
            "funding-rate",
            "adl-alert",
            "risk-limit",
            "open-interest",
            "positions",
            "open-orders",
            "history",
            "fills",
            "buy",
            "sell",
            "cancel",
            "cancel-all",
            "set-leverage",
            "ws",
        ],
    ),
    (
        "ws",
        &[
            "orderbook",
            "ticker",
            "trades",
            "kline",
            "liquidation",
            "orders",
            "positions",
            "executions",
            "wallet",
            "notifications",
            "lt-kline",
            "lt-ticker",
            "greeks",
            "dcp",
        ],
    ),
    (
        "paper",
        &[
            "init",
            "buy",
            "sell",
            "balance",
            "history",
            "cancelled",
            "positions",
            "orders",
            "cancel",
            "cancel-all",
            "status",
            "reset",
        ],
    ),
    (
        "reports",
        &[
            "transactions",
            "borrow-history",
            "orders",
            "fills",
            "closed-pnl",
            "moves",
            "deposits",
            "withdrawals",
            "transfers",
            "sub-transfers",
            "register-time",
            "export-request",
            "export-status",
            "export-retrieve",
        ],
    ),
    ("auth", &["set", "sign", "test", "verify", "show", "reset"]),
];

// ---------------------------------------------------------------------------
// rustyline Helper
// ---------------------------------------------------------------------------

#[derive(Helper)]
struct ShellHelper {
    #[allow(dead_code)]
    file_completer: FilenameCompleter,
}

impl Completer for ShellHelper {
    type Candidate = Pair;

    fn complete(
        &self,
        line: &str,
        pos: usize,
        _ctx: &Context<'_>,
    ) -> rustyline::Result<(usize, Vec<Pair>)> {
        let line_up_to_cursor = &line[..pos];
        let tokens: Vec<&str> = line_up_to_cursor.split_whitespace().collect();

        // Determine the word being completed
        let (start, word) = if line_up_to_cursor.ends_with(char::is_whitespace) {
            (pos, "")
        } else {
            let word = tokens.last().copied().unwrap_or("");
            (pos - word.len(), word)
        };

        let candidates: Vec<&str> = match tokens.len() {
            // Completing the top-level command
            0 | 1 if !line_up_to_cursor.ends_with(char::is_whitespace) => TOP_COMMANDS.to_vec(),
            // Completing a subcommand
            1 => {
                let cmd = tokens[0];
                SUBCOMMANDS
                    .iter()
                    .find(|(c, _)| *c == cmd)
                    .map(|(_, subs)| subs.to_vec())
                    .unwrap_or_default()
            }
            _ => {
                let cmd = tokens[0];
                SUBCOMMANDS
                    .iter()
                    .find(|(c, _)| *c == cmd)
                    .map(|(_, subs)| subs.to_vec())
                    .unwrap_or_default()
            }
        };

        let pairs: Vec<Pair> = candidates
            .into_iter()
            .filter(|c| c.starts_with(word))
            .map(|c| Pair {
                display: c.to_string(),
                replacement: c.to_string(),
            })
            .collect();

        Ok((start, pairs))
    }
}

impl Hinter for ShellHelper {
    type Hint = String;
    fn hint(&self, _line: &str, _pos: usize, _ctx: &Context<'_>) -> Option<String> {
        None
    }
}

impl Highlighter for ShellHelper {}
impl Validator for ShellHelper {}

// ---------------------------------------------------------------------------
// Shell entry point
// ---------------------------------------------------------------------------

/// Public entry point — boxed to break the async recursion cycle with dispatch().
pub fn run_shell(
    ctx: AppContext,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = BybitResult<()>> + Send>> {
    Box::pin(run_shell_impl(ctx))
}

async fn run_shell_impl(ctx: AppContext) -> BybitResult<()> {
    use colored::Colorize;

    let rl_config = RlConfig::builder()
        .completion_type(CompletionType::List)
        .max_history_size(1000)
        .expect("valid history size")
        .build();

    let helper = ShellHelper {
        file_completer: FilenameCompleter::new(),
    };

    let mut rl = Editor::with_config(rl_config)
        .map_err(|e| crate::errors::BybitError::Io(std::io::Error::other(e.to_string())))?;
    rl.set_helper(Some(helper));

    // Load history
    let history_file = history_path().ok();
    if let Some(ref path) = history_file {
        let _ = rl.load_history(path);
    }

    println!(
        "{} Type a command or {}. Tab-completes command names.",
        "bybit shell".bold().cyan(),
        "exit".bold(),
    );
    println!(
        "Example: {} {}",
        "market tickers --category linear --symbol BTCUSDT".dimmed(),
        "".normal()
    );
    println!();

    loop {
        let readline = rl.readline("bybit> ");
        match readline {
            Ok(line) => {
                let line = line.trim().to_string();
                if line.is_empty() {
                    continue;
                }

                rl.add_history_entry(&line).map_err(|e| {
                    crate::errors::BybitError::Io(std::io::Error::other(e.to_string()))
                })?;

                match line.as_str() {
                    "exit" | "quit" => break,
                    "help" => print_help(),
                    _ => {
                        // Prepend "bybit" and parse via the CLI parser
                        let full = format!("bybit {line}");
                        let args = match shell_words::split(&full) {
                            Ok(a) => a,
                            Err(e) => {
                                eprintln!("{}: {e}", "parse error".red());
                                continue;
                            }
                        };

                        let output_flag_present = has_option_flag(&args, Some('o'), "--output");
                        let category_flag_present = has_option_flag(&args, None, "--category");

                        match crate::Cli::try_parse_from(args) {
                            Ok(cli) => {
                                if cli.api_secret_stdin {
                                    let err = crate::errors::BybitError::Validation(
                                        "--api-secret-stdin is not supported inside `bybit shell`."
                                            .to_string(),
                                    );
                                    eprintln!(
                                        "{}",
                                        serde_json::to_string_pretty(&err.to_json())
                                            .unwrap_or_else(|_| err.to_string())
                                            .red()
                                    );
                                    continue;
                                }

                                let api_secret = match crate::resolve_cli_api_secret(
                                    cli.api_secret,
                                    false,
                                    cli.api_secret_file.as_deref(),
                                ) {
                                    Ok(secret) => secret,
                                    Err(e) => {
                                        eprintln!(
                                            "{}",
                                            serde_json::to_string_pretty(&e.to_json())
                                                .unwrap_or_else(|_| e.to_string())
                                                .red()
                                        );
                                        continue;
                                    }
                                };
                                let api_secret_from_input = api_secret.is_some();

                                let mut command = cli.command;
                                if !category_flag_present {
                                    command.apply_default_category(&ctx.default_category);
                                }

                                // Merge shell-level context with any per-command flags
                                let cmd_ctx = AppContext {
                                    api_key: cli.api_key.or_else(|| ctx.api_key.clone()),
                                    api_secret: api_secret.or_else(|| ctx.api_secret.clone()),
                                    api_secret_from_input: api_secret_from_input
                                        || ctx.api_secret_from_input,
                                    api_url: cli.api_url.or_else(|| ctx.api_url.clone()),
                                    format: if output_flag_present {
                                        cli.output
                                    } else {
                                        ctx.format
                                    },
                                    verbose: cli.verbose || ctx.verbose,
                                    default_category: ctx.default_category.clone(),
                                    recv_window: cli.recv_window.or(ctx.recv_window),
                                    testnet: cli.testnet || ctx.testnet,
                                    force: cli.yes || ctx.force,
                                    mcp_mode: false,
                                };
                                if let Err(e) = crate::dispatch(cmd_ctx, command).await {
                                    eprintln!(
                                        "{}",
                                        serde_json::to_string_pretty(&e.to_json())
                                            .unwrap_or_else(|_| e.to_string())
                                            .red()
                                    );
                                }
                            }
                            Err(e) => {
                                // clap prints its own formatted error; just show it
                                eprintln!("{e}");
                            }
                        }
                    }
                }
            }
            Err(ReadlineError::Interrupted) => {
                // Ctrl+C — clear line, keep going
                continue;
            }
            Err(ReadlineError::Eof) => {
                // Ctrl+D — exit
                break;
            }
            Err(e) => {
                eprintln!("readline error: {e}");
                break;
            }
        }
    }

    // Persist history
    if let Some(ref path) = history_file {
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let _ = rl.save_history(path);
    }

    println!("Goodbye.");
    Ok(())
}

fn print_help() {
    use colored::Colorize;
    println!("{}", "Available command groups:".bold());
    for (cmd, subs) in SUBCOMMANDS {
        println!("  {:<12} {}", cmd.cyan(), subs.join(", ").dimmed());
    }
    println!("  {}", "exit / quit".cyan());
}
