//! `oxyn-mcp`: a Model Context Protocol server for the Oxyn registry, so AI coding agents (Claude
//! Code, Cursor, Codex, Zed) can browse Oxyn components and add them to a Dioxus or GPUI project.
//!
//! The transport is stdio: one JSON-RPC 2.0 message per line on stdin, one reply per line on
//! stdout, logs on stderr. The server is a small synchronous loop (no async runtime): every tool
//! is a registry read or a file write through the `oxyn` CLI's library, so the agent gets exactly
//! what `oxyn add` would write.
//!
//! - [`Server::handle`]: one parsed message in, an optional reply out (notifications get none).
//! - [`tools`]: the tool list and each tool.
//! - [`props`]: a component's props, read from the entry function's signature.

pub mod props;
pub mod tools;

use std::io::{BufRead, Write};
use std::path::PathBuf;

use oxyn_cli::Client;
use serde_json::{Value, json};

/// The protocol revisions this server speaks, newest first. A client asking for one of these gets
/// it back; any other request gets the newest.
pub const PROTOCOL_VERSIONS: [&str; 4] = ["2025-11-25", "2025-06-18", "2025-03-26", "2024-11-05"];

/// Short guidance the client may show the model.
pub const INSTRUCTIONS: &str = "Oxyn is a Rust UI kit for agent apps (Dioxus and GPUI). Find components with list_components or search_components, read one with get_component, and install with add_component (pass project_path: the directory with the app's Cargo.toml). Code is copied into src/oxyn/ and is the user's to edit. Free items install directly; Pro items need Oxyn Pro (https://oxynui.com/pricing).";

/// JSON-RPC error codes used here.
pub mod code {
    /// Not valid JSON.
    pub const PARSE: i64 = -32700;
    /// Not a valid request object.
    pub const INVALID_REQUEST: i64 = -32600;
    /// No such method.
    pub const METHOD_NOT_FOUND: i64 = -32601;
    /// Bad parameters (including an unknown tool name).
    pub const INVALID_PARAMS: i64 = -32602;
}

/// The server state: one registry client and the directory relative paths resolve against.
pub struct Server {
    /// The registry.
    pub client: Client,
    /// Where `project_path` is resolved from when it is relative or missing (the server's cwd).
    pub cwd: PathBuf,
}

impl Server {
    /// A server for a registry base (URL or directory).
    pub fn new(registry: &str, cwd: PathBuf) -> Server {
        Server {
            client: Client::new(registry).with_via("mcp"),
            cwd,
        }
    }

    /// Handles one raw line. `None` when nothing should be written back.
    pub fn handle_line(&self, line: &str) -> Option<String> {
        let line = line.trim();
        if line.is_empty() {
            return None;
        }
        let reply = match serde_json::from_str::<Value>(line) {
            Err(e) => Some(error(
                Value::Null,
                code::PARSE,
                &format!("parse error: {e}"),
            )),
            // Batches (2025-03-26) are still answered, as an array.
            Ok(Value::Array(msgs)) => {
                let out: Vec<Value> = msgs.iter().filter_map(|m| self.handle(m)).collect();
                (!out.is_empty()).then_some(Value::Array(out))
            }
            Ok(msg) => self.handle(&msg),
        };
        reply.map(|v| v.to_string())
    }

    /// Handles one message. Requests get a reply; notifications and responses get none.
    pub fn handle(&self, msg: &Value) -> Option<Value> {
        let id = msg.get("id").cloned();
        let Some(method) = msg.get("method").and_then(Value::as_str) else {
            // A response from the client (we never send requests) or garbage.
            return match id {
                Some(id) if msg.get("result").is_none() && msg.get("error").is_none() => Some(
                    error(id, code::INVALID_REQUEST, "invalid request: no method"),
                ),
                _ => None,
            };
        };
        let id = id?; // notifications (initialized, cancelled, ...) need no reply
        let params = msg.get("params").cloned().unwrap_or(Value::Null);
        Some(match method {
            "initialize" => ok(id, initialize(&params)),
            "ping" => ok(id, json!({})),
            "tools/list" => ok(id, json!({ "tools": tools::list() })),
            "tools/call" => match tools::call(self, &params) {
                Ok(result) => ok(id, result),
                Err(e) => error(id, code::INVALID_PARAMS, &e),
            },
            // Not offered in the capabilities, but some clients ask anyway: answer empty.
            "resources/list" => ok(id, json!({ "resources": [] })),
            "resources/templates/list" => ok(id, json!({ "resourceTemplates": [] })),
            "prompts/list" => ok(id, json!({ "prompts": [] })),
            other => error(
                id,
                code::METHOD_NOT_FOUND,
                &format!("method not found: {other}"),
            ),
        })
    }

    /// Reads stdin until it closes, answering on stdout.
    pub fn run_stdio(&self) -> std::io::Result<()> {
        let stdin = std::io::stdin();
        let mut out = std::io::stdout().lock();
        for line in stdin.lock().lines() {
            let line = line?;
            if let Some(reply) = self.handle_line(&line) {
                out.write_all(reply.as_bytes())?;
                out.write_all(b"\n")?;
                out.flush()?;
            }
        }
        Ok(())
    }
}

fn initialize(params: &Value) -> Value {
    let asked = params
        .get("protocolVersion")
        .and_then(Value::as_str)
        .unwrap_or("");
    let version = PROTOCOL_VERSIONS
        .iter()
        .find(|v| **v == asked)
        .copied()
        .unwrap_or(PROTOCOL_VERSIONS[0]);
    json!({
        "protocolVersion": version,
        "capabilities": { "tools": { "listChanged": false } },
        "serverInfo": {
            "name": "oxyn",
            "title": "Oxyn UI",
            "version": env!("CARGO_PKG_VERSION"),
            "websiteUrl": "https://oxynui.com/docs/mcp/"
        },
        "instructions": INSTRUCTIONS
    })
}

fn ok(id: Value, result: Value) -> Value {
    json!({ "jsonrpc": "2.0", "id": id, "result": result })
}

fn error(id: Value, code: i64, message: &str) -> Value {
    json!({ "jsonrpc": "2.0", "id": id, "error": { "code": code, "message": message } })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn server() -> Server {
        Server::new("/nonexistent-registry", std::env::temp_dir())
    }

    #[test]
    fn initialize_negotiates_the_version() {
        let s = server();
        let r = s
            .handle(&json!({"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-06-18","capabilities":{},"clientInfo":{"name":"t","version":"0"}}}))
            .unwrap();
        assert_eq!(r["result"]["protocolVersion"], "2025-06-18");
        assert_eq!(r["result"]["serverInfo"]["name"], "oxyn");
        assert!(r["result"]["capabilities"]["tools"].is_object());
        let r = s
            .handle(&json!({"jsonrpc":"2.0","id":"a","method":"initialize","params":{"protocolVersion":"1999-01-01"}}))
            .unwrap();
        assert_eq!(r["result"]["protocolVersion"], PROTOCOL_VERSIONS[0]);
        assert_eq!(r["id"], "a");
    }

    #[test]
    fn notifications_get_no_reply_and_errors_are_json_rpc() {
        let s = server();
        assert!(
            s.handle(&json!({"jsonrpc":"2.0","method":"notifications/initialized"}))
                .is_none()
        );
        assert!(s.handle_line("   ").is_none());
        let r: Value = serde_json::from_str(&s.handle_line("{nope").unwrap()).unwrap();
        assert_eq!(r["error"]["code"], code::PARSE);
        let r = s
            .handle(&json!({"jsonrpc":"2.0","id":2,"method":"sampling/whatever"}))
            .unwrap();
        assert_eq!(r["error"]["code"], code::METHOD_NOT_FOUND);
        let r = s
            .handle(&json!({"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"nope"}}))
            .unwrap();
        assert_eq!(r["error"]["code"], code::INVALID_PARAMS);
        let r = s
            .handle(&json!({"jsonrpc":"2.0","id":4,"method":"ping"}))
            .unwrap();
        assert_eq!(r["result"], json!({}));
    }

    #[test]
    fn batches_and_tool_list() {
        let s = server();
        let out = s
            .handle_line(r#"[{"jsonrpc":"2.0","id":1,"method":"tools/list"},{"jsonrpc":"2.0","method":"notifications/initialized"}]"#)
            .unwrap();
        let v: Value = serde_json::from_str(&out).unwrap();
        let tools = v[0]["result"]["tools"].as_array().unwrap();
        let names: Vec<&str> = tools.iter().filter_map(|t| t["name"].as_str()).collect();
        assert_eq!(
            names,
            [
                "list_components",
                "search_components",
                "get_component",
                "add_component",
                "get_install_instructions"
            ]
        );
        for t in tools {
            assert_eq!(t["inputSchema"]["type"], "object");
            assert!(!t["description"].as_str().unwrap().is_empty());
        }
    }

    #[test]
    fn a_registry_failure_is_a_tool_error_not_a_crash() {
        let s = server();
        let r = s
            .handle(&json!({"jsonrpc":"2.0","id":5,"method":"tools/call","params":{"name":"list_components","arguments":{}}}))
            .unwrap();
        assert_eq!(r["result"]["isError"], true);
        assert!(
            r["result"]["content"][0]["text"]
                .as_str()
                .unwrap()
                .contains("index.json")
        );
    }
}
