# Contributing to bybit-cli

Thank you for your interest in contributing to `bybit-cli`! As a community-maintained project, we rely on contributors to keep this tool robust and secure.

## 🛡️ Security First

Because this tool manages financial credentials and executes real trades, security is our absolute priority.

1.  **Read the [SecureSDLC](SECURESDLC.md)**: All contributions must adhere to our security lifecycle.
2.  **No Leaks**: Never include API keys, secrets, or personal data in your PRs or issues.
3.  **Audit**: We audit all dependencies. If you add a new crate, please justify why it is necessary and ensure it is widely used and well-maintained.

## 🚀 How to Contribute

### 1. Development Environment
Ensure you have the latest stable Rust toolchain installed.

```bash
cargo build
cargo test
cargo audit -D warnings
```

### 2. Coding Standards
-   **Linting**: Run `cargo clippy --all-targets -- -D warnings` before submitting.
-   **Formatting**: Run `cargo fmt`.
-   **Tests**: All PRs must include tests. Add integration tests for new features in `tests/integration/` and unit tests for internal logic in `src/`. We aim for 100% command and line coverage.

### 3. Submitting a PR
-   Keep PRs focused on a single feature or bug fix.
-   Update `README.md` and `skills/` if you add new commands or workflows.
-   If you add a new CLI command, make sure to also update the MCP registry in `src/mcp/registry.rs`.
-   Run `cargo test --test doc_sync` after command-surface or documentation changes. Use `cargo run --example command_inventory` to inspect the live clap command inventory when updating docs.
-   If you touch the MCP registry or service list, regenerate `agents/mcp-tool-catalog.json` with `cargo run --example mcp_tool_catalog > agents/mcp-tool-catalog.json` and run `cargo test --test mcp_catalog_sync`.

## 📜 Unofficial Status

Remember that this is an **unofficial** tool. Avoid using Bybit's official branding or implying any affiliation with Bybit in your contributions.

## ⚠️ Legal

By contributing, you agree that your code will be licensed under the project's [MIT License](LICENSE).
