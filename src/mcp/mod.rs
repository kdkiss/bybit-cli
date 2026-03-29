pub mod registry;
pub mod schema;
pub mod server;

use crate::errors::{BybitError, BybitResult};

pub const VALID_SERVICES: &[&str] = &[
    "market",
    "account",
    "trade",
    "position",
    "asset",
    "funding",
    "reports",
    "subaccount",
    "futures",
    "paper",
    "auth",
];

pub const DEFAULT_SERVICES: &str = "market,account,paper";

pub fn parse_services(input: &str) -> BybitResult<Vec<String>> {
    let trimmed = input.trim();
    if trimmed.eq_ignore_ascii_case("all") {
        return Ok(VALID_SERVICES
            .iter()
            .map(|service| service.to_string())
            .collect());
    }

    let tokens: Vec<String> = trimmed
        .split(',')
        .map(str::trim)
        .filter(|token| !token.is_empty())
        .map(|token| token.to_ascii_lowercase())
        .collect();

    if tokens.is_empty() {
        return Err(BybitError::Validation(
            "At least one MCP service must be specified.".to_string(),
        ));
    }

    for token in &tokens {
        if !VALID_SERVICES.contains(&token.as_str()) {
            return Err(BybitError::Validation(format!(
                "Unknown MCP service: '{token}'. Valid services: {}",
                VALID_SERVICES.join(", ")
            )));
        }
    }

    Ok(tokens)
}

#[cfg(test)]
mod tests {
    use super::{parse_services, VALID_SERVICES};

    #[test]
    fn parse_all_returns_all_valid_services() {
        let result = parse_services("all").unwrap();
        assert_eq!(result.len(), VALID_SERVICES.len());
        for service in VALID_SERVICES {
            assert!(result.contains(&service.to_string()));
        }
    }

    #[test]
    fn parse_explicit_list_is_normalized() {
        let result = parse_services(" Market , TRADE ").unwrap();
        assert_eq!(result, vec!["market", "trade"]);
    }

    #[test]
    fn parse_rejects_unknown_service() {
        let err = parse_services("market,bogus").unwrap_err().to_string();
        assert!(err.contains("Unknown MCP service"));
    }

    #[test]
    fn parse_rejects_empty_input() {
        let err = parse_services(" ").unwrap_err().to_string();
        assert!(err.contains("At least one MCP service"));
    }
}
