use std::fs;
use std::path::Path;
use std::sync::OnceLock;

use uuid::Uuid;

use crate::config::instance_id_path;

pub const CLIENT_NAME: &str = "bybit-cli";
pub const CLIENT_VERSION: &str = env!("CARGO_PKG_VERSION");
const BASE_USER_AGENT: &str = concat!("bybit-cli/", env!("CARGO_PKG_VERSION"));

const AGENT_CLIENT_ENV: &str = "BYBIT_AGENT_CLIENT";
const INSTANCE_ID_ENV: &str = "BYBIT_INSTANCE_ID";

static AGENT_CLIENT: OnceLock<String> = OnceLock::new();
static INSTANCE_ID: OnceLock<String> = OnceLock::new();
static USER_AGENT: OnceLock<String> = OnceLock::new();

pub fn agent_client() -> &'static str {
    AGENT_CLIENT.get_or_init(resolve_agent_client).as_str()
}

pub fn instance_id() -> &'static str {
    INSTANCE_ID.get_or_init(resolve_instance_id).as_str()
}

pub fn user_agent() -> &'static str {
    USER_AGENT
        .get_or_init(|| build_structured_user_agent(agent_client()))
        .as_str()
}

fn build_structured_user_agent(agent: &str) -> String {
    format!("{BASE_USER_AGENT} ({agent})")
}

fn resolve_agent_client() -> String {
    if let Ok(raw) = std::env::var(AGENT_CLIENT_ENV) {
        let normalized = normalize_agent_client(&raw);
        return if normalized == "other" {
            sanitize_agent_value(&raw).unwrap_or_else(|| "other".to_string())
        } else {
            normalized.to_string()
        };
    }

    detect_from_environment()
}

fn normalize_agent_client(raw: &str) -> &'static str {
    let normalized = raw.trim().to_ascii_lowercase();
    match normalized.as_str() {
        "cursor" | "cursor-ide" | "cursor-agent" => "cursor",
        "claude" | "claude-code" | "claude_code" | "claudecode" => "claude",
        "openclaw" | "open-claw" => "openclaw",
        "codex" | "openai-codex" => "codex",
        "goose" | "block-goose" => "goose",
        "gemini" | "gemini-cli" => "gemini",
        _ => "other",
    }
}

fn detect_from_environment() -> String {
    if env_present("CURSOR_AGENT") || env_present("CURSOR_TRACE_ID") {
        return "cursor".into();
    }
    if env_present("CLAUDECODE") {
        return "claude".into();
    }
    if env_present("OPENCLAW_SHELL") {
        return "openclaw".into();
    }
    if env_present("CODEX_SANDBOX") {
        return "codex".into();
    }
    if env_present("GOOSE_TERMINAL") {
        return "goose".into();
    }
    if env_present("GEMINI_CLI") {
        return "gemini".into();
    }

    if let Some(agent) = std::env::var("AGENT")
        .ok()
        .and_then(|value| sanitize_agent_value(&value))
    {
        let normalized = normalize_agent_client(&agent);
        return if normalized == "other" {
            agent
        } else {
            normalized.to_string()
        };
    }

    if env_present("VSCODE_PID") || env_present("VSCODE_CLI") {
        return "vscode".into();
    }

    "direct".into()
}

fn sanitize_agent_value(raw: &str) -> Option<String> {
    let value = raw
        .trim()
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || *c == '-')
        .take(32)
        .collect::<String>()
        .to_ascii_lowercase();
    if value.is_empty() {
        None
    } else {
        Some(value)
    }
}

fn env_present(name: &str) -> bool {
    std::env::var_os(name).is_some_and(|value| !value.is_empty())
}

fn resolve_instance_id() -> String {
    if let Ok(value) = std::env::var(INSTANCE_ID_ENV) {
        let trimmed = value.trim();
        if is_uuid_like(trimmed) {
            return trimmed.to_string();
        }
    }

    if let Ok(path) = instance_id_path() {
        if let Some(existing) = read_instance_id(&path) {
            return existing;
        }

        let generated = Uuid::new_v4().to_string();
        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        let _ = fs::write(&path, &generated);
        return generated;
    }

    Uuid::new_v4().to_string()
}

fn read_instance_id(path: &Path) -> Option<String> {
    let content = fs::read_to_string(path).ok()?;
    let trimmed = content.trim();
    if is_uuid_like(trimmed) {
        Some(trimmed.to_string())
    } else {
        None
    }
}

fn is_uuid_like(value: &str) -> bool {
    let bytes = value.as_bytes();
    if bytes.len() != 36 {
        return false;
    }

    for (index, byte) in bytes.iter().enumerate() {
        match index {
            8 | 13 | 18 | 23 => {
                if *byte != b'-' {
                    return false;
                }
            }
            _ => {
                if !byte.is_ascii_hexdigit() {
                    return false;
                }
            }
        }
    }

    true
}

#[cfg(test)]
mod tests {
    use super::{
        build_structured_user_agent, is_uuid_like, normalize_agent_client, sanitize_agent_value,
    };

    #[test]
    fn user_agent_includes_agent_label() {
        let user_agent = build_structured_user_agent("codex");
        assert!(user_agent.starts_with("bybit-cli/"));
        assert!(user_agent.ends_with("(codex)"));
    }

    #[test]
    fn normalize_agent_client_maps_known_aliases() {
        assert_eq!(normalize_agent_client("cursor-agent"), "cursor");
        assert_eq!(normalize_agent_client("claude_code"), "claude");
        assert_eq!(normalize_agent_client("openai-codex"), "codex");
    }

    #[test]
    fn sanitize_agent_value_keeps_safe_ascii() {
        assert_eq!(
            sanitize_agent_value("  Cursor IDE!! "),
            Some("cursoride".to_string())
        );
        assert_eq!(sanitize_agent_value(""), None);
    }

    #[test]
    fn uuid_validator_accepts_valid_shape() {
        assert!(is_uuid_like("550e8400-e29b-41d4-a716-446655440000"));
        assert!(!is_uuid_like("not-a-uuid"));
    }
}
