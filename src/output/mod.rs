use serde_json::Value;

pub mod json;
pub mod table;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, clap::ValueEnum)]
pub enum OutputFormat {
    Json,
    #[default]
    Table,
}

impl OutputFormat {
    pub fn from_setting(value: &str) -> Self {
        match value.trim().to_ascii_lowercase().as_str() {
            "json" => Self::Json,
            _ => Self::Table,
        }
    }
}

/// Print a result value to stdout in the requested format.
pub fn print_output(value: &Value, format: OutputFormat) {
    match format {
        OutputFormat::Json => json::print(value),
        OutputFormat::Table => table::print(value),
    }
}

/// Print an error envelope as JSON.
pub fn print_error(value: &Value) {
    json::print(value);
}
