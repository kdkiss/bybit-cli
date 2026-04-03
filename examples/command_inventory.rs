use serde_json::json;

fn main() {
    let payload = json!({
        "top_level_commands": bybit_cli::command_inventory::top_level_commands(),
        "leaf_commands": bybit_cli::command_inventory::leaf_commands(),
    });

    println!("{}", serde_json::to_string_pretty(&payload).unwrap());
}
