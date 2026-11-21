//! Spawns the `oxyn-mcp` binary and talks MCP to it over stdio, like an agent would.
//!
//! `fake_registry_session` builds a tiny registry with `oxyn-codegen`. `real_registry_session`
//! runs only when `OXYN_MCP_E2E_REGISTRY` names a registry built from this commit (CI's mcp step
//! sets it, with `OXYN_MCP_E2E_PROJECT` pointing at an `oxyn init`-ed Dioxus project to add into).

#![allow(missing_docs)]

use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::path::Path;
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};

use oxyn_codegen::registry::{self, BuildConfig, MetaItem};
use oxyn_registry_schema::{Kind, Tier};
use serde_json::{Value, json};

struct Session {
    child: Child,
    stdin: ChildStdin,
    stdout: BufReader<ChildStdout>,
    next: u64,
}

impl Session {
    fn start(registry: &str, cwd: &Path) -> Session {
        let mut child = Command::new(env!("CARGO_BIN_EXE_oxyn-mcp"))
            .args(["--registry", registry])
            .current_dir(cwd)
            .env_remove("OXYN_REGISTRY")
            .env_remove("OXYN_TOKEN")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .spawn()
            .expect("spawn oxyn-mcp");
        let stdin = child.stdin.take().unwrap();
        let stdout = BufReader::new(child.stdout.take().unwrap());
        Session {
            child,
            stdin,
            stdout,
            next: 1,
        }
    }

    fn send(&mut self, msg: &Value) {
        writeln!(self.stdin, "{msg}").unwrap();
        self.stdin.flush().unwrap();
    }

    fn request(&mut self, method: &str, params: Value) -> Value {
        let id = self.next;
        self.next += 1;
        self.send(&json!({ "jsonrpc": "2.0", "id": id, "method": method, "params": params }));
        let mut line = String::new();
        self.stdout.read_line(&mut line).unwrap();
        let v: Value = serde_json::from_str(&line).expect("one JSON reply per line");
        assert_eq!(v["jsonrpc"], "2.0");
        assert_eq!(v["id"], id, "replies come in order, ids echoed");
        v
    }

    /// Calls a tool; returns (isError, parsed JSON or the text as a string).
    fn tool(&mut self, name: &str, args: Value) -> (bool, Value) {
        let r = self.request("tools/call", json!({ "name": name, "arguments": args }));
        let res = &r["result"];
        let text = res["content"][0]["text"].as_str().expect("text content");
        let body = serde_json::from_str(text).unwrap_or_else(|_| Value::String(text.into()));
        (res["isError"].as_bool().unwrap(), body)
    }

    fn handshake(&mut self) {
        let r = self.request(
            "initialize",
            json!({ "protocolVersion": "2025-06-18", "capabilities": {}, "clientInfo": { "name": "test", "version": "0" } }),
        );
        assert_eq!(r["result"]["protocolVersion"], "2025-06-18");
        assert_eq!(r["result"]["serverInfo"]["name"], "oxyn");
        self.send(&json!({ "jsonrpc": "2.0", "method": "notifications/initialized" }));
        let tools = self.request("tools/list", json!({}));
        assert_eq!(tools["result"]["tools"].as_array().unwrap().len(), 5);
    }

    fn finish(mut self) {
        drop(self.stdin);
        let status = self.child.wait().unwrap();
        assert!(status.success(), "exits cleanly when stdin closes");
    }
}

fn w(root: &Path, p: &str, s: &str) {
    let f = root.join(p);
    fs::create_dir_all(f.parent().unwrap()).unwrap();
    fs::write(f, s).unwrap();
}

/// A registry built by `oxyn-codegen` from a tiny renderer crate: `send-button` and `composer`
/// are free, `fancy-screen` is Pro.
fn fake_registry(r: &Path) {
    let dx = "crates/oxyn-dioxus/src";
    w(
        r,
        &format!("{dx}/lib.rs"),
        "//! Root.\nmod brand;\nmod composer;\nmod prem;\npub use composer::{Composer, SendButton, COMPOSER_CSS};\npub use prem::Fancy;\n",
    );
    w(r, &format!("{dx}/brand.rs"), "pub fn BrandIcon() {}\n");
    w(
        r,
        &format!("{dx}/composer/mod.rs"),
        "use dioxus::prelude::*;\nmod send;\npub use send::SendButton;\npub const COMPOSER_CSS: &str = include_str!(\"composer.css\");\n/// The composer.\n#[component]\npub fn Composer(\n    /// Placeholder text.\n    #[props(default = \"Ask\".to_string())]\n    placeholder: String,\n    /// Busy.\n    #[props(default)]\n    busy: bool,\n) -> Element { rsx! {} }\n",
    );
    w(r, &format!("{dx}/composer/composer.css"), ".x{}");
    w(
        r,
        &format!("{dx}/composer/send.rs"),
        "use crate::brand::BrandIcon;\n#[component]\npub fn SendButton(label: String) -> Element { BrandIcon(); rsx! {} }\n",
    );
    w(r, &format!("{dx}/prem.rs"), "pub fn Fancy() {}\n");
    w(
        r,
        "crates/oxyn-dioxus/Cargo.toml",
        "[package]\nname = \"oxyn-dioxus\"\n[dependencies]\ndioxus = { version = \"0.7\", default-features = false }\n",
    );
    w(r, "crates/oxyn-gpui/src/lib.rs", "//! GPUI.\n");
    w(
        r,
        "crates/oxyn-gpui/Cargo.toml",
        "[package]\nname = \"oxyn-gpui\"\n",
    );
    w(r, "Cargo.toml", "[workspace.dependencies]\n");
    w(r, "tokens/dist/oxyn.css", ":root{}");
    let m = |slug: &str, tier: Tier, kind: Kind, entry: &str| MetaItem {
        slug: slug.into(),
        name: slug.replace('-', " "),
        kind,
        category: "chat".into(),
        tier,
        description: format!("The {slug}."),
        url: format!("https://x.test/{slug}/"),
        dioxus: Some(entry.into()),
        gpui: None,
    };
    let meta = vec![
        m("send-button", Tier::Free, Kind::Component, "SendButton"),
        m("composer", Tier::Free, Kind::Component, "Composer"),
        m("fancy-screen", Tier::Pro, Kind::Screen, "Fancy"),
    ];
    let cfg = BuildConfig {
        repo_root: r.into(),
        homepage: "https://x.test".into(),
        git: "https://github.com/ismailqureshi72/Oxyn".into(),
        rev: "a".repeat(40),
    };
    let reg = registry::build(&meta, &cfg).unwrap();
    registry::write(&reg, &r.join("r")).unwrap();
}

#[test]
fn fake_registry_session() {
    let d = tempfile::tempdir().unwrap();
    fake_registry(d.path());
    let app = d.path().join("app");
    w(
        &app,
        "Cargo.toml",
        "[package]\nname = \"app\"\nversion = \"0.1.0\"\nedition = \"2024\"\n\n[dependencies]\ndioxus = \"0.7\"\n",
    );
    w(&app, "src/main.rs", "fn main() {}\n");
    let reg = d.path().join("r");
    let mut s = Session::start(reg.to_str().unwrap(), &app);
    s.handshake();

    let (err, list) = s.tool("list_components", json!({}));
    assert!(!err);
    assert_eq!(list["total"], 3);
    assert_eq!(list["free"], 2);
    let (_, free) = s.tool("list_components", json!({ "tier": "pro" }));
    assert_eq!(free["items"][0]["slug"], "fancy-screen");

    let (_, hits) = s.tool("search_components", json!({ "query": "composer" }));
    assert_eq!(hits["items"][0]["slug"], "composer");

    let (err, comp) = s.tool("get_component", json!({ "slug": "composer" }));
    assert!(!err, "{comp}");
    let props = &comp["builds"][0]["props"];
    assert_eq!(props[0]["name"], "placeholder");
    assert_eq!(props[0]["default"], "\"Ask\".to_string()");
    assert_eq!(props[1]["default"], "Default");
    assert_eq!(comp["install"]["cli"], "oxyn add composer");
    let files = comp["code"]["files"].as_array().unwrap();
    assert_eq!(
        files[0]["path"], "src/oxyn/composer/mod.rs",
        "entry file first"
    );
    assert!(
        files[0]["content"]
            .as_str()
            .unwrap()
            .contains("pub fn Composer")
    );

    let (err, pro) = s.tool("get_component", json!({ "slug": "fancy-screen" }));
    assert!(!err);
    assert!(pro["pro"].as_str().unwrap().contains("/pricing"));
    assert!(pro.get("code").is_none(), "no Pro code");

    let (err, msg) = s.tool("get_component", json!({ "slug": "composr" }));
    assert!(err);
    assert!(msg.as_str().unwrap().contains("`composer`"));

    let (err, added) = s.tool(
        "add_component",
        json!({ "slugs": ["composer"], "framework": "dioxus" }),
    );
    assert!(!err, "{added}");
    assert!(
        added["added"]
            .as_array()
            .unwrap()
            .iter()
            .any(|x| x == "composer")
    );
    assert!(added["setup"].is_array(), "first add sets the project up");
    assert!(app.join("oxyn.toml").is_file());
    assert!(app.join("src/oxyn/composer/mod.rs").is_file());
    let main = fs::read_to_string(app.join("src/main.rs")).unwrap();
    assert!(main.contains("mod oxyn;"));

    let (err, pro) = s.tool("add_component", json!({ "slugs": ["fancy-screen"] }));
    assert!(err);
    assert!(pro.as_str().unwrap().contains("Oxyn Pro"));

    let (err, again) = s.tool("add_component", json!({ "slugs": ["composer"] }));
    assert!(!err);
    assert_eq!(again["written"], json!([]));

    let (err, text) = s.tool("get_install_instructions", json!({ "framework": "dioxus" }));
    assert!(!err);
    assert!(text.as_str().unwrap().contains("oxyn-tokens.css"));
    s.finish();
}

#[test]
fn real_registry_session() {
    let Ok(reg) = std::env::var("OXYN_MCP_E2E_REGISTRY") else {
        eprintln!("skipped: set OXYN_MCP_E2E_REGISTRY to a built registry");
        return;
    };
    let project = std::env::var("OXYN_MCP_E2E_PROJECT").ok();
    let cwd = std::env::temp_dir();
    let mut s = Session::start(&reg, &cwd);
    s.handshake();
    let (err, free) = s.tool("list_components", json!({ "tier": "free", "limit": 500 }));
    assert!(!err, "{free}");
    let n = free["total"].as_u64().unwrap();
    assert!((72..=75).contains(&n), "{n} free items");
    let (err, comp) = s.tool("get_component", json!({ "slug": "composer" }));
    assert!(!err, "{comp}");
    assert!(comp["code"]["files"].as_array().unwrap().len() > 1);
    let (err, pro) = s.tool(
        "add_component",
        json!({ "slugs": ["memory-panel"], "project_path": cwd }),
    );
    assert!(err, "a Pro item is refused: {pro}");
    if let Some(p) = project {
        let (err, added) = s.tool(
            "add_component",
            json!({ "slugs": ["composer", "approval-card", "tag-input"], "project_path": p }),
        );
        assert!(!err, "{added}");
        eprintln!("{added}");
    }
    s.finish();
}
