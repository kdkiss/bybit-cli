use dialoguer::Confirm;

use crate::errors::{BybitError, BybitResult};

/// Prompt the user for confirmation before a dangerous action.
/// Returns Ok(()) if confirmed or `force` is true, Err otherwise.
pub fn confirm(prompt: &str, force: bool) -> BybitResult<()> {
    if force {
        return Ok(());
    }
    let confirmed = Confirm::new()
        .with_prompt(prompt)
        .default(false)
        .interact()
        .map_err(|e| BybitError::Io(std::io::Error::other(e.to_string())))?;

    if confirmed {
        Ok(())
    } else {
        Err(BybitError::Validation(
            "Operation cancelled by user.".to_string(),
        ))
    }
}

/// Convert an optional string param to a query pair only if Some.
pub fn optional_param<'a>(key: &'a str, value: &'a Option<String>) -> Option<(&'a str, &'a str)> {
    value.as_deref().map(|v| (key, v))
}

/// Build a query params vec from an iterator of optional pairs, filtering out None values.
pub fn build_params<'a>(
    pairs: impl IntoIterator<Item = Option<(&'a str, &'a str)>>,
) -> Vec<(&'a str, &'a str)> {
    pairs.into_iter().flatten().collect()
}

/// Bybit's linear list endpoints reject completely unfiltered requests on some
/// accounts unless `settleCoin` is supplied. Use the common USDT market as the
/// safe default for read-only queries when no other scope is present.
pub fn should_default_linear_settle_coin(
    category: &str,
    symbol: &Option<String>,
    base_coin: &Option<String>,
    settle_coin: &Option<String>,
) -> bool {
    category == "linear" && symbol.is_none() && base_coin.is_none() && settle_coin.is_none()
}
