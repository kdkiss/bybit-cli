fn main() {
    let payload = bybit_cli::mcp::registry::runtime_tool_catalog();
    println!("{}", serde_json::to_string_pretty(&payload).unwrap());
}
