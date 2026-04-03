use std::fs;
use std::path::PathBuf;

use serde_json::Value;

#[test]
fn agents_mcp_tool_catalog_matches_runtime_registry() {
    let documented: Value = serde_json::from_str(
        &fs::read_to_string(repo_root().join("agents").join("mcp-tool-catalog.json"))
            .expect("failed to read agents/mcp-tool-catalog.json"),
    )
    .expect("agents/mcp-tool-catalog.json should be valid JSON");

    assert_eq!(
        documented,
        bybit_cli::mcp::registry::runtime_tool_catalog(),
        "agents/mcp-tool-catalog.json drifted from the runtime MCP registry"
    );
}

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}
