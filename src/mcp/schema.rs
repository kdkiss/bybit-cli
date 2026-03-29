// MCP JSON Schema helpers — builds inputSchema objects for tool definitions.

use serde_json::{json, Value};

pub fn str_prop(description: &str) -> Value {
    json!({"type": "string", "description": description})
}

pub fn num_prop(description: &str) -> Value {
    json!({"type": "number", "description": description})
}

pub fn int_prop(description: &str) -> Value {
    json!({"type": "integer", "description": description})
}

pub fn bool_prop(description: &str) -> Value {
    json!({"type": "boolean", "description": description})
}

pub fn enum_prop(description: &str, values: &[&str]) -> Value {
    json!({"type": "string", "description": description, "enum": values})
}

/// Build an `object` JSON Schema from a list of `(name, schema)` pairs.
pub fn object_schema(props: Vec<(&str, Value)>, required: &[&str]) -> Value {
    let properties: serde_json::Map<String, Value> =
        props.into_iter().map(|(k, v)| (k.to_string(), v)).collect();
    json!({
        "type": "object",
        "properties": properties,
        "required": required,
    })
}

/// Inject a required `acknowledged` boolean into a dangerous tool schema.
pub fn inject_dangerous_confirmation(schema: &mut Value) {
    if let Some(props) = schema.get_mut("properties").and_then(Value::as_object_mut) {
        props.insert(
            "acknowledged".into(),
            json!({
                "type": "boolean",
                "description": "Set to true to confirm you intend to execute this state-changing operation."
            }),
        );
    }

    if let Some(req) = schema.get_mut("required").and_then(Value::as_array_mut) {
        req.push(json!("acknowledged"));
    } else if let Some(obj) = schema.as_object_mut() {
        obj.insert("required".into(), json!(["acknowledged"]));
    }
}
