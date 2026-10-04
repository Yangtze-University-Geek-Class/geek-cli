use anyhow::Result;

use crate::api::{ApiError, UsageError};
use comfy_table::{presets::UTF8_BORDERS_ONLY, ContentArrangement, Table};
use serde::Serialize;
use serde_json::Value;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, clap::ValueEnum)]
pub enum Format {
    #[default]
    Json,
    Pretty,
    Table,
}

pub fn emit<T: Serialize>(value: &T, fmt: Format) -> Result<()> {
    match fmt {
        Format::Json => {
            let s = serde_json::to_string(value)?;
            println!("{s}");
        }
        Format::Pretty => {
            let s = serde_json::to_string_pretty(value)?;
            println!("{s}");
        }
        Format::Table => {
            let v: Value = serde_json::to_value(value)?;
            print_table(&v);
        }
    }
    Ok(())
}

fn print_table(v: &Value) {
    if let Value::Array(arr) = v {
        if arr.is_empty() {
            println!("(empty)");
            return;
        }
        let mut headers: Vec<String> = Vec::new();
        for item in arr {
            if let Value::Object(map) = item {
                for k in map.keys() {
                    if !headers.contains(k) {
                        headers.push(k.clone());
                    }
                }
            }
        }
        let mut table = Table::new();
        table
            .load_preset(UTF8_BORDERS_ONLY)
            .set_content_arrangement(ContentArrangement::Dynamic);
        table.set_header(headers.clone());
        for item in arr {
            if let Value::Object(map) = item {
                let row: Vec<String> = headers
                    .iter()
                    .map(|h| map.get(h).map(value_to_cell).unwrap_or_default())
                    .collect();
                table.add_row(row);
            } else {
                table.add_row(vec![item.to_string()]);
            }
        }
        println!("{table}");
    } else if let Value::Object(map) = v {
        let mut table = Table::new();
        table
            .load_preset(UTF8_BORDERS_ONLY)
            .set_content_arrangement(ContentArrangement::Dynamic);
        table.set_header(vec!["field", "value"]);
        for (k, v) in map {
            table.add_row(vec![k.clone(), value_to_cell(v)]);
        }
        println!("{table}");
    } else {
        println!("{v}");
    }
}

fn value_to_cell(v: &Value) -> String {
    match v {
        Value::Null => "—".to_string(),
        Value::String(s) => s.clone(),
        Value::Bool(b) => b.to_string(),
        Value::Number(n) => n.to_string(),
        Value::Array(_) | Value::Object(_) => {
            let s = serde_json::to_string(v).unwrap_or_default();
            if s.len() > 60 {
                format!("{}…", &s[..60])
            } else {
                s
            }
        }
    }
}

pub fn print_message(msg: &str) {
    eprintln!("{msg}");
}

/// 统一的错误呈现与进程退出：`ApiError` → 契约退出码（0–9）；`UsageError` → 2；其余 → 1。
pub fn report_error(err: &anyhow::Error, format: Format) -> ! {
    if let Some(api) = err.downcast_ref::<ApiError>() {
        match format {
            Format::Json | Format::Pretty => {
                let line = serde_json::to_string(api).unwrap_or_else(|_| api.to_string());
                eprintln!("{line}");
            }
            Format::Table => eprintln!("{api}"),
        }
        std::process::exit(api.exit_code());
    }
    if let Some(usage) = err.downcast_ref::<UsageError>() {
        eprintln!("{usage}");
        std::process::exit(2);
    }
    eprintln!("{err:#}");
    std::process::exit(1);
}
