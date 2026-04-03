use clap::CommandFactory;

use crate::Cli;

const COMMAND_INVENTORY_STACK_SIZE: usize = 32 * 1024 * 1024;

pub fn top_level_commands() -> Vec<String> {
    with_inventory_stack(|| {
        let mut commands: Vec<String> = Cli::command()
            .get_subcommands()
            .map(|command| command.get_name().to_string())
            .collect();
        commands.sort();
        commands
    })
}

pub fn leaf_commands() -> Vec<String> {
    with_inventory_stack(|| {
        let root = Cli::command();
        let mut commands = Vec::new();
        let mut prefix = Vec::new();
        collect_leaf_commands(&root, &mut prefix, &mut commands);
        commands.sort();
        commands
    })
}

fn collect_leaf_commands(
    command: &clap::Command,
    prefix: &mut Vec<String>,
    commands: &mut Vec<String>,
) {
    let mut subcommands = command.get_subcommands().peekable();
    if subcommands.peek().is_none() {
        if !prefix.is_empty() {
            commands.push(prefix.join(" "));
        }
        return;
    }

    for subcommand in command.get_subcommands() {
        prefix.push(subcommand.get_name().to_string());
        collect_leaf_commands(subcommand, prefix, commands);
        prefix.pop();
    }
}

fn with_inventory_stack<T>(f: impl FnOnce() -> T + Send + 'static) -> T
where
    T: Send + 'static,
{
    let handle = std::thread::Builder::new()
        .name("bybit-command-inventory".to_string())
        .stack_size(COMMAND_INVENTORY_STACK_SIZE)
        .spawn(f)
        .expect("failed to spawn command inventory thread");

    handle.join().expect("command inventory thread panicked")
}
