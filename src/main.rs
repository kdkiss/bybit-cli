use clap::{CommandFactory, Parser};

use bybit_cli::{
    dispatch, env_flag, has_option_flag, has_switch_flag, resolve_cli_api_secret, AppContext, Cli,
};

const CLI_MAIN_STACK_SIZE: usize = 32 * 1024 * 1024;

fn exit_with_error(e: bybit_cli::errors::BybitError) -> ! {
    let json = e.to_json();
    eprintln!(
        "{}",
        serde_json::to_string_pretty(&json).unwrap_or_else(|_| e.to_string())
    );
    std::process::exit(1);
}

fn main() {
    let handle = std::thread::Builder::new()
        .name("bybit-main".to_string())
        .stack_size(CLI_MAIN_STACK_SIZE)
        .spawn(run)
        .unwrap_or_else(|error| {
            exit_with_error(bybit_cli::errors::BybitError::Io(std::io::Error::other(
                format!("Failed to start CLI thread: {error}"),
            )))
        });

    match handle.join() {
        Ok(Ok(())) => {}
        Ok(Err(error)) => exit_with_error(error),
        Err(_) => exit_with_error(bybit_cli::errors::BybitError::Io(std::io::Error::other(
            "CLI thread panicked",
        ))),
    }
}

fn run() -> Result<(), bybit_cli::errors::BybitError> {
    // Load .env from the current directory or any parent directory.
    // Already-exported shell variables keep precedence (dotenv() does not override).
    dotenvy::dotenv().ok();

    let raw_args: Vec<String> = std::env::args().collect();
    if raw_args.len() == 1 {
        let mut cmd = Cli::command();
        let _ = cmd.print_help();
        println!();
        return Ok(());
    }

    let cli = Cli::parse();
    let config = bybit_cli::config::load_config().unwrap_or_default();
    let output_flag_present = has_option_flag(&raw_args, Some('o'), "--output");
    let recv_window_flag_present = has_option_flag(&raw_args, None, "--recv-window");
    let category_flag_present = has_option_flag(&raw_args, None, "--category");
    let testnet_flag_present = has_switch_flag(&raw_args, "--testnet");

    // Read secret from stdin if requested
    let api_secret = match resolve_cli_api_secret(
        cli.api_secret,
        cli.api_secret_stdin,
        cli.api_secret_file.as_deref(),
    ) {
        Ok(secret) => secret,
        Err(e) => return Err(e),
    };
    let api_secret_from_input = api_secret.is_some();

    // Merge flag/env credentials with config file fallback
    let (api_key, api_secret) = if cli.api_key.is_none() || api_secret.is_none() {
        match bybit_cli::config::resolve_credentials(None, None) {
            Ok(Some(creds)) => {
                let key = cli.api_key.or(Some(creds.api_key));
                let secret = api_secret.or(Some(creds.api_secret.expose().to_string()));
                (key, secret)
            }
            _ => (cli.api_key, api_secret),
        }
    } else {
        (cli.api_key, api_secret)
    };

    let format = if output_flag_present {
        cli.output
    } else if let Ok(env_val) = std::env::var("BYBIT_OUTPUT") {
        bybit_cli::output::OutputFormat::from_setting(&env_val)
    } else {
        bybit_cli::output::OutputFormat::from_setting(&config.settings.output)
    };
    let recv_window = if recv_window_flag_present {
        cli.recv_window
    } else {
        Some(config.settings.recv_window)
    };
    let testnet = if testnet_flag_present {
        true
    } else if let Some(value) = env_flag("BYBIT_TESTNET") {
        value
    } else {
        config.settings.testnet
    };

    let effective_default_category = std::env::var("BYBIT_DEFAULT_CATEGORY")
        .ok()
        .unwrap_or(config.settings.default_category);
    let mut command = cli.command;
    if !category_flag_present {
        command.apply_default_category(&effective_default_category);
    }

    let ctx = AppContext {
        format,
        verbose: cli.verbose,
        api_url: cli.api_url,
        api_key,
        api_secret,
        api_secret_from_input,
        default_category: effective_default_category,
        recv_window,
        testnet,
        force: cli.yes,
        mcp_mode: false,
    };

    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap_or_else(|error| {
            exit_with_error(bybit_cli::errors::BybitError::Io(std::io::Error::other(
                format!("Failed to build async runtime: {error}"),
            )))
        });

    runtime.block_on(dispatch(ctx, command))?;
    Ok(())
}
