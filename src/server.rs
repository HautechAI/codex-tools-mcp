use codex_apply_patch::apply_patch as run_apply_patch;
use log::{debug, error, info, warn};
use serde_json::{json, Value};
use std::io::{self, BufRead, Write};

use crate::tools::{
    apply_patch_tool_schema, update_plan_tool_schema, INVALID_PARAMS, INVALID_REQUEST,
    JSONRPC_VERSION, MCP_PROTOCOL_VERSION, METHOD_NOT_FOUND, PARSE_ERROR,
};

pub fn run_server() -> io::Result<()> {
    let stdin = io::stdin();
    for line_result in stdin.lock().lines() {
        let line = match line_result {
            Ok(line) => {
                if line.trim().is_empty() {
                    continue;
                }
                debug!("stdin: {line}");
                line
            }
            Err(err) => {
                error!("failed to read stdin: {err}");
                break;
            }
        };

        let message: Value = match serde_json::from_str(&line) {
            Ok(value) => value,
            Err(err) => {
                warn!("Malformed JSON from client: {err}");
                if let Err(send_err) =
                    send_error(None, PARSE_ERROR, format!("Malformed JSON: {err}"))
                {
                    error!("failed to send parse error response: {send_err}");
                }
                continue;
            }
        };

        if !message.is_object() {
            warn!("Received non-object message");
            if let Err(err) = send_error(None, INVALID_REQUEST, "Request must be a JSON object") {
                error!("failed to send invalid request error: {err}");
            }
            continue;
        }

        if let Err(err) = handle_message(message) {
            error!("internal error while processing message: {err}");
        }
    }

    Ok(())
}

fn handle_message(message: Value) -> io::Result<()> {
    let method = message
        .get("method")
        .and_then(Value::as_str)
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "missing method"))?;
    let request_id = message.get("id").cloned();
    let params = message.get("params").cloned();

    debug!("dispatching method: {method}");

    match method {
        "initialize" => handle_initialize(request_id, params),
        "tools/list" => handle_tools_list(request_id),
        "tools/call" => handle_tools_call(request_id, params),
        "ping" => handle_ping(request_id),
        _ => send_error(
            request_id,
            METHOD_NOT_FOUND,
            format!("Unknown method: {method}"),
        ),
    }
}

fn handle_initialize(request_id: Option<Value>, params: Option<Value>) -> io::Result<()> {
    if request_id.is_none() {
        return send_error(None, INVALID_REQUEST, "initialize must include an id");
    }

    if !params.as_ref().is_some_and(Value::is_object) {
        return send_error(
            request_id,
            INVALID_PARAMS,
            "initialize params must be object",
        );
    }

    let result = json!({
        "protocolVersion": MCP_PROTOCOL_VERSION,
        "serverInfo": {
            "name": "codex-tools-mcp",
            "version": env!("CARGO_PKG_VERSION"),
        },
        "capabilities": {
            "tools": json!({}),
        }
    });
    send_result(request_id.clone(), result)?;

    debug!("sent initialize response");

    send_json(json!({
        "jsonrpc": JSONRPC_VERSION,
        "method": "notifications/initialized",
        "params": Value::Null,
    }))
}

fn handle_tools_list(request_id: Option<Value>) -> io::Result<()> {
    if request_id.is_none() {
        return send_error(None, INVALID_REQUEST, "tools/list must include an id");
    }

    let tools = vec![update_plan_tool_schema(), apply_patch_tool_schema()];
    debug!("advertising {} tools", tools.len());
    let result = json!({ "tools": tools });
    send_result(request_id, result)
}

fn handle_tools_call(request_id: Option<Value>, params: Option<Value>) -> io::Result<()> {
    if request_id.is_none() {
        return send_error(None, INVALID_REQUEST, "tools/call must include an id");
    }

    let params_obj = match params {
        Some(Value::Object(map)) => map,
        _ => {
            return send_error(
                request_id,
                INVALID_PARAMS,
                "tools/call params must be object",
            )
        }
    };

    match params_obj.get("name").and_then(Value::as_str) {
        Some("update_plan") => {
            info!("received update_plan call");
            let result = json!({
                "content": [
                    {
                        "type": "text",
                        "text": "Plan updated",
                    }
                ]
            });
            send_result(request_id, result)
        }
        Some("apply_patch") => handle_apply_patch_tool(request_id, &params_obj),
        Some(other) => {
            warn!("unknown tool requested: {other}");
            send_error(
                request_id,
                METHOD_NOT_FOUND,
                format!("Unknown tool: {other}"),
            )
        }
        None => send_error(request_id, INVALID_PARAMS, "tools/call params missing name"),
    }
}

fn handle_apply_patch_tool(
    request_id: Option<Value>,
    params_obj: &serde_json::Map<String, Value>,
) -> io::Result<()> {
    let arguments = match params_obj.get("arguments") {
        Some(Value::Object(arguments)) => arguments,
        Some(_) => {
            return send_error(
                request_id,
                INVALID_PARAMS,
                "apply_patch arguments must be an object",
            )
        }
        None => return send_error(request_id, INVALID_PARAMS, "apply_patch requires arguments"),
    };

    let patch = match arguments.get("input").and_then(Value::as_str) {
        Some(patch) => patch.to_string(),
        None => {
            return send_error(
                request_id,
                INVALID_PARAMS,
                "apply_patch input must be provided as a string",
            )
        }
    };

    info!("running apply_patch ({} bytes)", patch.len());

    let mut stdout_buf = Vec::new();
    let mut stderr_buf = Vec::new();
    let apply_result = run_apply_patch(&patch, &mut stdout_buf, &mut stderr_buf);
    let stdout_text = String::from_utf8_lossy(&stdout_buf).to_string();
    let stderr_text = String::from_utf8_lossy(&stderr_buf).to_string();

    match apply_result {
        Ok(()) => {
            let mut blocks = Vec::new();
            if !stdout_text.trim().is_empty() {
                blocks.push(json!({
                    "type": "text",
                    "text": stdout_text.trim_end_matches('\n'),
                }));
            }
            if !stderr_text.trim().is_empty() {
                blocks.push(json!({
                    "type": "text",
                    "text": format!("stderr:\n{}", stderr_text.trim_end_matches('\n')),
                }));
            }
            if blocks.is_empty() {
                blocks.push(json!({
                    "type": "text",
                    "text": "Patch applied",
                }));
            }

            info!("apply_patch completed successfully");
            let result = json!({ "content": blocks });
            send_result(request_id, result)
        }
        Err(err) => {
            warn!("apply_patch failed: {err}");
            let mut blocks = Vec::new();
            let error_message = err.to_string();
            blocks.push(json!({
                "type": "text",
                "text": format!("apply_patch failed: {error_message}"),
            }));

            if !stdout_text.trim().is_empty() {
                blocks.push(json!({
                    "type": "text",
                    "text": stdout_text.trim_end_matches('\n'),
                }));
            }
            if !stderr_text.trim().is_empty() {
                blocks.push(json!({
                    "type": "text",
                    "text": format!("stderr:\n{}", stderr_text.trim_end_matches('\n')),
                }));
            }

            let result = json!({
                "content": blocks,
                "isError": true,
            });
            send_result(request_id, result)
        }
    }
}

fn handle_ping(request_id: Option<Value>) -> io::Result<()> {
    if request_id.is_none() {
        return send_error(None, INVALID_REQUEST, "ping must include an id");
    }

    send_result(request_id, json!({}))
}

fn send_result(request_id: Option<Value>, result: Value) -> io::Result<()> {
    let mut message = serde_json::Map::new();
    message.insert(
        "jsonrpc".to_string(),
        Value::String(JSONRPC_VERSION.to_string()),
    );
    message.insert("result".to_string(), result);

    match request_id {
        Some(id) => {
            message.insert("id".to_string(), id);
        }
        None => {
            message.insert("id".to_string(), Value::Null);
        }
    }

    send_json(Value::Object(message))
}

fn send_error(request_id: Option<Value>, code: i64, message: impl Into<String>) -> io::Result<()> {
    let mut payload = serde_json::Map::new();
    payload.insert(
        "jsonrpc".to_string(),
        Value::String(JSONRPC_VERSION.to_string()),
    );
    payload.insert(
        "error".to_string(),
        json!({
            "code": code,
            "message": message.into(),
        }),
    );

    match request_id {
        Some(id) => {
            payload.insert("id".to_string(), id);
        }
        None => {
            payload.insert("id".to_string(), Value::Null);
        }
    }

    send_json(Value::Object(payload))
}

fn send_json(value: Value) -> io::Result<()> {
    let mut stdout = io::stdout().lock();
    serde_json::to_writer(&mut stdout, &value)?;
    stdout.write_all(b"\n")?;
    stdout.flush()
}
