use comfy_table::{presets::UTF8_FULL, Cell, Color, ContentArrangement, Table};
use serde_json::Value;

/// Render a JSON value as a table when possible, otherwise fall back to JSON.
pub fn print(value: &Value) {
    match value {
        Value::Array(rows) if !rows.is_empty() => {
            if let Some(Value::Object(_)) = rows.first() {
                print_array_of_objects(rows);
            } else {
                print_array_of_scalars(rows);
            }
        }
        Value::Object(map) => {
            print_object(map);
        }
        other => {
            println!(
                "{}",
                serde_json::to_string_pretty(other).unwrap_or_default()
            );
        }
    }
}

fn print_array_of_objects(rows: &[Value]) {
    // Collect headers from first object
    let headers: Vec<String> = if let Some(Value::Object(first)) = rows.first() {
        first.keys().cloned().collect()
    } else {
        return;
    };

    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_header(headers.iter().map(|h| Cell::new(h).fg(Color::Cyan)));

    for row in rows {
        if let Value::Object(map) = row {
            table.add_row(headers.iter().map(|h| {
                let v = map.get(h).unwrap_or(&Value::Null);
                Cell::new(value_to_display(v))
            }));
        }
    }

    println!("{table}");
}

fn print_array_of_scalars(rows: &[Value]) {
    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_header(vec![Cell::new("value").fg(Color::Cyan)]);

    for v in rows {
        table.add_row(vec![Cell::new(value_to_display(v))]);
    }
    println!("{table}");
}

fn print_object(map: &serde_json::Map<String, Value>) {
    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_header(vec![
            Cell::new("key").fg(Color::Cyan),
            Cell::new("value").fg(Color::Cyan),
        ]);

    for (k, v) in map {
        table.add_row(vec![
            Cell::new(k).fg(Color::Green),
            Cell::new(value_to_display(v)),
        ]);
    }
    println!("{table}");
}

fn value_to_display(v: &Value) -> String {
    match v {
        Value::String(s) => s.clone(),
        Value::Null => String::new(),
        other => other.to_string(),
    }
}
