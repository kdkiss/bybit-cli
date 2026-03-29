use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ErrorCategory {
    Api,
    Auth,
    Network,
    RateLimit,
    Paper,
    Validation,
    Config,
    WebSocket,
    Io,
    Parse,
}

#[derive(Debug, Error)]
pub enum BybitError {
    #[error("{message}")]
    Api {
        category: ErrorCategory,
        message: String,
        ret_code: i64,
    },

    #[error("Authentication error: {0}")]
    Auth(String),

    #[error("Network error: {0}")]
    Network(String),

    #[error("{message}")]
    RateLimit {
        message: String,
        suggestion: String,
        retryable: bool,
        docs_url: &'static str,
        ret_code: Option<i64>,
    },

    #[error("Paper trading error: {0}")]
    Paper(String),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Config error: {0}")]
    Config(String),

    #[error("WebSocket error: {0}")]
    WebSocket(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Parse error: {0}")]
    Parse(String),
}

impl BybitError {
    pub fn category(&self) -> ErrorCategory {
        match self {
            Self::Api { category, .. } => *category,
            Self::Auth(_) => ErrorCategory::Auth,
            Self::Network(_) => ErrorCategory::Network,
            Self::RateLimit { .. } => ErrorCategory::RateLimit,
            Self::Paper(_) => ErrorCategory::Paper,
            Self::Validation(_) => ErrorCategory::Validation,
            Self::Config(_) => ErrorCategory::Config,
            Self::WebSocket(_) => ErrorCategory::WebSocket,
            Self::Io(_) => ErrorCategory::Io,
            Self::Parse(_) => ErrorCategory::Parse,
        }
    }

    pub fn is_retryable(&self) -> bool {
        match self {
            Self::Network(_) => true,
            Self::RateLimit { retryable, .. } => *retryable,
            _ => false,
        }
    }

    pub fn to_json(&self) -> serde_json::Value {
        match self {
            Self::Api {
                category,
                message,
                ret_code,
            } => serde_json::json!({
                "error": category,
                "message": message,
                "ret_code": ret_code,
                "retryable": false,
            }),
            Self::RateLimit {
                message,
                suggestion,
                retryable,
                docs_url,
                ret_code,
            } => serde_json::json!({
                "error": "rate_limit",
                "message": message,
                "ret_code": ret_code,
                "suggestion": suggestion,
                "retryable": retryable,
                "docs_url": docs_url,
            }),
            other => serde_json::json!({
                "error": other.category(),
                "message": other.to_string(),
                "ret_code": serde_json::Value::Null,
                "retryable": other.is_retryable(),
            }),
        }
    }
}

impl From<reqwest::Error> for BybitError {
    fn from(e: reqwest::Error) -> Self {
        if e.is_timeout() || e.is_connect() {
            Self::Network(e.to_string())
        } else if e.is_decode() || e.is_body() {
            Self::Parse(e.to_string())
        } else {
            Self::Network(e.to_string())
        }
    }
}

impl From<serde_json::Error> for BybitError {
    fn from(e: serde_json::Error) -> Self {
        Self::Parse(e.to_string())
    }
}

impl From<tokio_tungstenite::tungstenite::Error> for BybitError {
    fn from(e: tokio_tungstenite::tungstenite::Error) -> Self {
        Self::WebSocket(e.to_string())
    }
}

pub type BybitResult<T> = Result<T, BybitError>;

#[cfg(test)]
mod tests {
    use super::BybitError;

    #[test]
    fn paper_errors_emit_paper_category_with_null_ret_code() {
        let json = BybitError::Paper("journal missing".into()).to_json();
        assert_eq!(json["error"], "paper");
        assert!(json["ret_code"].is_null());
    }

    #[test]
    fn validation_errors_emit_null_ret_code() {
        let json = BybitError::Validation("bad input".into()).to_json();
        assert_eq!(json["error"], "validation");
        assert!(json["ret_code"].is_null());
    }
}
