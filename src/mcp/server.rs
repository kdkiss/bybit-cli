use std::{
    sync::{Arc, Mutex},
    time::Instant,
};

use rmcp::{
    handler::server::ServerHandler,
    model::{
        CallToolRequestParams, CallToolResult, Content, Implementation, InitializeRequestParams,
        InitializeResult, ListToolsResult, PaginatedRequestParams, ServerCapabilities, Tool,
    },
    service::{RequestContext, RoleServer, ServiceExt},
};
use serde_json::{json, Value};

use crate::{
    errors::{BybitError, BybitResult},
    AppContext,
};

use super::{
    parse_services,
    registry::{all_tools, tool_to_args, McpTool},
    schema::inject_dangerous_confirmation,
};

const MCP_TRANSPORT: &str = "stdio";
const DANGEROUS_GATE_ERROR: &str =
    "This operation modifies account state. Set \"acknowledged\": true to proceed, or start the server with --allow-dangerous.";

#[derive(Debug, Default, Clone)]
struct SessionMetadata {
    client_name: Option<String>,
    client_version: Option<String>,
}

pub async fn run_mcp_server(
    ctx: AppContext,
    services: &str,
    allow_dangerous: bool,
) -> BybitResult<()> {
    let enabled_services = parse_services(services)?;
    let server = BybitMcpServer::new(ctx, enabled_services, allow_dangerous);
    let mode_label = if allow_dangerous {
        "autonomous"
    } else {
        "guarded"
    };

    eprintln!(
        "bybit-cli MCP server v{} starting on stdio ({} tools, mode: {})",
        env!("CARGO_PKG_VERSION"),
        server.tool_count(),
        mode_label,
    );

    let transport = rmcp::transport::io::stdio();
    let service = server
        .serve(transport)
        .await
        .map_err(|e| BybitError::Config(format!("Failed to start MCP server: {e}")))?;

    service
        .waiting()
        .await
        .map_err(|e| BybitError::Config(format!("MCP server error: {e}")))?;

    Ok(())
}

struct BybitMcpServer {
    ctx: Arc<AppContext>,
    enabled_services: Vec<String>,
    allow_dangerous: bool,
    instructions: String,
    session: Mutex<SessionMetadata>,
}

impl BybitMcpServer {
    fn new(ctx: AppContext, enabled_services: Vec<String>, allow_dangerous: bool) -> Self {
        let instructions = build_instructions(&enabled_services, allow_dangerous);
        Self {
            ctx: Arc::new(ctx),
            enabled_services,
            allow_dangerous,
            instructions,
            session: Mutex::new(SessionMetadata::default()),
        }
    }

    fn tool_count(&self) -> usize {
        self.filtered_tools().count()
    }

    fn filtered_tools(&self) -> impl Iterator<Item = McpTool> + '_ {
        all_tools().into_iter().filter(|tool| {
            self.enabled_services
                .iter()
                .any(|service| service == tool.service)
        })
    }

    fn build_tool_definition(&self, tool: &McpTool) -> Tool {
        let mut input_schema = tool.input_schema.clone();
        if tool.dangerous && !self.allow_dangerous {
            inject_dangerous_confirmation(&mut input_schema);
        }

        let schema_obj: serde_json::Map<String, Value> =
            serde_json::from_value(input_schema).unwrap_or_default();

        Tool::new(
            tool.name.to_string(),
            tool.description.to_string(),
            Arc::new(schema_obj),
        )
    }

    fn tool_definitions(&self) -> Vec<Tool> {
        self.filtered_tools()
            .map(|tool| self.build_tool_definition(&tool))
            .collect()
    }

    fn find_tool(&self, name: &str) -> Option<McpTool> {
        self.filtered_tools().find(|tool| tool.name == name)
    }

    fn session_snapshot(&self) -> SessionMetadata {
        self.session
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .clone()
    }

    fn update_session_from_initialize(&self, request: &InitializeRequestParams) {
        let request_value = serde_json::to_value(request).unwrap_or(Value::Null);
        let session = SessionMetadata::from_initialize(&request_value);
        *self.session.lock().unwrap_or_else(|e| e.into_inner()) = session;
    }

    async fn execute_tool_call(
        &self,
        tool_name: &str,
        arguments: &Value,
    ) -> Result<String, (&'static str, String)> {
        let tool = self
            .find_tool(tool_name)
            .ok_or_else(|| ("unknown_tool", format!("Unknown tool: {tool_name}")))?;

        enforce_dangerous_gate(tool.dangerous, self.allow_dangerous, arguments)
            .map_err(|msg| ("dangerous_confirmation_required", msg.to_string()))?;

        let args = tool_to_args(tool_name, arguments).ok_or_else(|| {
            (
                "argv_build_failed",
                format!("Cannot build CLI args for tool '{tool_name}'"),
            )
        })?;

        execute_subprocess(&self.ctx, &args, tool.dangerous)
            .await
            .map_err(|msg| ("subprocess_failed", msg))
    }
}

impl ServerHandler for BybitMcpServer {
    async fn initialize(
        &self,
        request: InitializeRequestParams,
        _context: RequestContext<RoleServer>,
    ) -> Result<InitializeResult, rmcp::model::ErrorData> {
        self.update_session_from_initialize(&request);
        let session = self.session_snapshot();

        audit_log(&json!({
            "ts": now_iso(),
            "event": "session_start",
            "server_version": env!("CARGO_PKG_VERSION"),
            "services": self.enabled_services,
            "mode": if self.allow_dangerous { "autonomous" } else { "guarded" },
            "caller": caller_metadata(&session),
        }));

        Ok(
            InitializeResult::new(ServerCapabilities::builder().enable_tools().build())
                .with_protocol_version(rmcp::model::ProtocolVersion::V_2024_11_05)
                .with_server_info(
                    Implementation::new("bybit-cli", env!("CARGO_PKG_VERSION")).with_description(
                        "Bybit exchange CLI tools. Use service filtering to control which command groups are available.",
                    ),
                )
                .with_instructions(&self.instructions),
        )
    }

    async fn list_tools(
        &self,
        _request: Option<PaginatedRequestParams>,
        _context: RequestContext<RoleServer>,
    ) -> Result<ListToolsResult, rmcp::model::ErrorData> {
        Ok(ListToolsResult {
            tools: self.tool_definitions(),
            ..Default::default()
        })
    }

    fn get_tool(&self, name: &str) -> Option<Tool> {
        self.find_tool(name)
            .map(|tool| self.build_tool_definition(&tool))
    }

    async fn call_tool(
        &self,
        request: CallToolRequestParams,
        _context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, rmcp::model::ErrorData> {
        let tool_name = request.name.to_string();
        let arguments_map = request.arguments.clone();
        let arguments = Value::Object(arguments_map.clone().unwrap_or_default());
        let started = Instant::now();
        let session = self.session_snapshot();

        audit_log(&json!({
            "ts": now_iso(),
            "event": "tool_call",
            "tool": tool_name,
            "arg_keys": argument_keys(&arguments_map),
            "arg_count": arguments_map.as_ref().map(|args| args.len()).unwrap_or(0),
            "caller": caller_metadata(&session),
        }));

        let (response, error_code, status) =
            match self.execute_tool_call(&tool_name, &arguments).await {
                Ok(output) => {
                    let text = if output.trim().is_empty() {
                        r#"{"ok":true}"#.to_string()
                    } else {
                        output
                    };
                    (
                        CallToolResult::success(vec![Content::text(text)]),
                        None,
                        "executed",
                    )
                }
                Err((code, msg)) => (
                    CallToolResult::error(vec![Content::text(msg)]),
                    Some(code),
                    "rejected",
                ),
            };

        audit_log(&json!({
            "ts": now_iso(),
            "event": "tool_result",
            "tool": tool_name,
            "status": status,
            "error_code": error_code,
            "duration_ms": started.elapsed().as_millis() as u64,
            "caller": caller_metadata(&session),
        }));

        Ok(response)
    }
}

fn build_instructions(active_services: &[String], allow_dangerous: bool) -> String {
    let svc_list = active_services.join(", ");
    let mode = if allow_dangerous {
        "autonomous"
    } else {
        "guarded"
    };

    let mut text = format!("Bybit exchange CLI tools. Active services: {svc_list}. Mode: {mode}.");

    let missing: Vec<&str> = super::VALID_SERVICES
        .iter()
        .copied()
        .filter(|service| !active_services.iter().any(|active| active == service))
        .collect();

    if !missing.is_empty() {
        text.push_str(&format!(
            " Services not loaded: {}. Restart the MCP server with -s all or include the missing groups explicitly.",
            missing.join(", "),
        ));
    }

    if !allow_dangerous {
        text.push_str(
            " Dangerous tools stay visible but require \"acknowledged\": true per call. Start the server with --allow-dangerous to skip per-call confirmation.",
        );
    }

    text
}

fn argument_keys(arguments: &Option<serde_json::Map<String, Value>>) -> Vec<String> {
    let Some(arguments) = arguments else {
        return Vec::new();
    };

    let mut keys: Vec<String> = arguments.keys().cloned().collect();
    keys.sort();
    keys
}

fn caller_metadata(session: &SessionMetadata) -> Value {
    let mut caller = serde_json::Map::new();
    caller.insert("agent".into(), json!(crate::telemetry::agent_client()));
    caller.insert("instance_id".into(), json!(crate::telemetry::instance_id()));
    caller.insert("pid".into(), json!(std::process::id()));
    caller.insert("transport".into(), json!(MCP_TRANSPORT));

    if let Some(name) = &session.client_name {
        caller.insert("client_name".into(), json!(name));
    }
    if let Some(version) = &session.client_version {
        caller.insert("client_version".into(), json!(version));
    }

    Value::Object(caller)
}

fn audit_log(event: &Value) {
    eprintln!(
        "[mcp audit] {}",
        serde_json::to_string(event).unwrap_or_default()
    );
}

fn now_iso() -> String {
    chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true)
}

fn enforce_dangerous_gate(
    dangerous: bool,
    allow_dangerous: bool,
    arguments: &Value,
) -> Result<(), &'static str> {
    if !dangerous || allow_dangerous {
        return Ok(());
    }

    let confirmed = arguments
        .as_object()
        .and_then(|obj| obj.get("acknowledged"))
        .and_then(Value::as_bool)
        .unwrap_or(false);

    if confirmed {
        Ok(())
    } else {
        Err(DANGEROUS_GATE_ERROR)
    }
}

impl SessionMetadata {
    fn from_initialize(params: &Value) -> Self {
        let client_info = params.get("clientInfo").and_then(Value::as_object);
        Self {
            client_name: client_info
                .and_then(|info| info.get("name"))
                .and_then(Value::as_str)
                .map(str::to_string),
            client_version: client_info
                .and_then(|info| info.get("version"))
                .and_then(Value::as_str)
                .map(str::to_string),
        }
    }
}

async fn execute_subprocess(
    ctx: &AppContext,
    args: &[String],
    needs_confirm_skip: bool,
) -> Result<String, String> {
    use tokio::process::Command;

    let exe = std::env::current_exe().map_err(|e| e.to_string())?;

    let mut cmd = Command::new(&exe);
    cmd.args(args);
    cmd.arg("-o").arg("json");

    if needs_confirm_skip {
        cmd.arg("-y");
    }

    if let Some(key) = &ctx.api_key {
        cmd.env("BYBIT_API_KEY", key);
    }
    if let Some(secret) = &ctx.api_secret {
        cmd.env("BYBIT_API_SECRET", secret);
    }
    if ctx.testnet {
        cmd.env("BYBIT_TESTNET", "true");
    } else {
        cmd.env_remove("BYBIT_TESTNET");
    }
    if let Some(url) = &ctx.api_url {
        cmd.env("BYBIT_API_URL", url);
    }
    if let Some(rw) = ctx.recv_window {
        cmd.arg("--recv-window").arg(rw.to_string());
    }

    cmd.stdin(std::process::Stdio::null());
    cmd.stdout(std::process::Stdio::piped());
    cmd.stderr(std::process::Stdio::null());

    let output = cmd.output().await.map_err(|e| e.to_string())?;
    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    let stderr_str = String::from_utf8_lossy(&output.stderr).into_owned();

    if !output.status.success() && stdout.trim().is_empty() {
        let msg = if stderr_str.trim().is_empty() {
            format!("Command exited with status {}", output.status)
        } else {
            stderr_str.trim().to_string()
        };
        return Err(msg);
    }

    Ok(stdout)
}

#[cfg(test)]
mod tests {
    use serde_json::{json, Value};

    use super::{
        argument_keys, build_instructions, caller_metadata, enforce_dangerous_gate, BybitMcpServer,
        SessionMetadata, DANGEROUS_GATE_ERROR,
    };
    use crate::{output::OutputFormat, AppContext};

    fn test_server() -> BybitMcpServer {
        BybitMcpServer::new(
            AppContext {
                format: OutputFormat::Json,
                verbose: false,
                api_url: None,
                api_key: None,
                api_secret: None,
                api_secret_from_input: false,
                default_category: "linear".to_string(),
                recv_window: None,
                testnet: false,
                force: true,
                mcp_mode: true,
            },
            vec!["trade".to_string()],
            false,
        )
    }

    #[test]
    fn dangerous_gate_allows_non_dangerous() {
        assert!(enforce_dangerous_gate(false, false, &json!({})).is_ok());
    }

    #[test]
    fn dangerous_gate_rejects_guarded_without_ack() {
        let err = enforce_dangerous_gate(true, false, &json!({})).unwrap_err();
        assert_eq!(err, DANGEROUS_GATE_ERROR);
    }

    #[test]
    fn dangerous_gate_accepts_guarded_with_ack() {
        assert!(enforce_dangerous_gate(true, false, &json!({"acknowledged": true})).is_ok());
    }

    #[test]
    fn guarded_tools_list_injects_acknowledged_into_dangerous_schema() {
        let tools = test_server().tool_definitions();
        let trade_buy = tools
            .into_iter()
            .find(|tool| {
                serde_json::to_value(tool)
                    .ok()
                    .and_then(|value| value.get("name").cloned())
                    == Some(json!("trade_buy"))
            })
            .expect("trade_buy tool present");

        let tool_json = serde_json::to_value(trade_buy).expect("tool JSON");
        let props = tool_json
            .get("inputSchema")
            .and_then(|schema| schema.get("properties"))
            .and_then(Value::as_object)
            .expect("properties object");
        assert!(props.contains_key("acknowledged"));
    }

    #[test]
    fn build_instructions_mentions_guarded_confirmation() {
        let text = build_instructions(&["market".into(), "trade".into()], false);
        assert!(text.contains("Mode: guarded"));
        assert!(text.contains("acknowledged"));
        assert!(text.contains("--allow-dangerous"));
    }

    #[test]
    fn argument_keys_are_sorted() {
        let keys = argument_keys(&Some(serde_json::Map::from_iter([
            ("z".to_string(), json!(1)),
            ("a".to_string(), json!(2)),
        ])));
        assert_eq!(keys, vec!["a".to_string(), "z".to_string()]);
    }

    #[test]
    fn caller_metadata_includes_client_info_when_present() {
        let caller = caller_metadata(&SessionMetadata {
            client_name: Some("codex".to_string()),
            client_version: Some("1.2.3".to_string()),
        });
        assert_eq!(caller.get("client_name"), Some(&json!("codex")));
        assert_eq!(caller.get("client_version"), Some(&json!("1.2.3")));
    }
}
