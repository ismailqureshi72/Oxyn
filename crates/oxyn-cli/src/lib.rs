//! The `oxyn` command line as a library: the registry client, the dependency resolver and the
//! project writer behind `oxyn init | list | search | view | add | diff`. The MCP server (P2) and
//! the Pro registry client (P3) reuse these pieces.
//!
//! - [`client`]: reads `index.json` and `<slug>.json` over HTTP(S) or from a directory, and Pro
//!   items from the authenticated Pro registry (`<api>/r/<slug>.json`).
//! - [`auth`]: `oxyn login` (browser or `--token`), the saved token, `OXYN_TOKEN`, `whoami`.
//! - [`resolve`]: slugs to items in install order, Pro / unknown / framework errors, search.
//! - [`install`]: composes files from chunks, rewrites `crate::` paths, edits `Cargo.toml`, writes
//!   with `oxyn.lock` protection, diffs.
//! - [`config`]: `oxyn.toml` and `oxyn.lock`.

pub mod auth;
pub mod client;
pub mod config;
pub mod install;
pub mod resolve;

use std::fmt;
use std::path::PathBuf;

use oxyn_registry_schema::Framework;

pub use client::{Client, DEFAULT_REGISTRY, PRICING_URL, api_base, auth_header, registry_base};
pub use config::{Config, Lock, Source};

/// Everything that can go wrong, with a message meant for the person at the terminal.
#[derive(Debug)]
pub enum Error {
    /// No `Cargo.toml` here or above.
    NoCargoProject(PathBuf),
    /// No `oxyn.toml` yet.
    NotInitialised(PathBuf),
    /// No item with that name; close matches.
    UnknownItem(String, Vec<String>),
    /// A Pro item and an account (or registry) without Pro.
    ProRequired(String),
    /// A Pro item and no valid token: `oxyn login`.
    SignInRequired(String),
    /// The month's Pro items are used up; the server's message (with the reset date).
    Quota(String),
    /// The item has no build for the project's framework.
    NoFramework(String, Framework, Vec<Framework>),
    /// The registry could not be reached.
    Network(String, String),
    /// Reading or writing a file failed.
    Io(PathBuf, std::io::Error),
    /// Anything else, already phrased.
    Invalid(String),
}

impl Error {
    /// Process exit code: 2 for "needs Pro" or "sign in", 3 for "monthly Pro limit reached", 1
    /// otherwise.
    pub fn exit_code(&self) -> i32 {
        match self {
            Error::ProRequired(_) | Error::SignInRequired(_) => 2,
            Error::Quota(_) => 3,
            _ => 1,
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::NoCargoProject(p) => write!(
                f,
                "no Cargo.toml in {} or above: run oxyn inside a Rust project (cargo new my-app)",
                p.display()
            ),
            Error::NotInitialised(p) => {
                write!(f, "no oxyn.toml in {}: run `oxyn init` first", p.display())
            }
            Error::UnknownItem(s, sug) if sug.is_empty() => write!(
                f,
                "there is no item `{s}` (see `oxyn list` or `oxyn search <words>`)"
            ),
            Error::UnknownItem(s, sug) => write!(
                f,
                "there is no item `{s}`; did you mean {}?",
                sug.iter()
                    .map(|x| format!("`{x}`"))
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            Error::ProRequired(s) => write!(
                f,
                "`{s}` is part of Oxyn Pro. The preview is free on the site; the code needs a Pro plan: {PRICING_URL}/\n(Signed in already? `oxyn whoami` shows your plan.)"
            ),
            Error::SignInRequired(s) => write!(
                f,
                "`{s}` is part of Oxyn Pro: sign in first with `oxyn login` (or set OXYN_TOKEN to a token from your account page). No plan yet? {PRICING_URL}/"
            ),
            Error::Quota(m) => write!(f, "{m}\n(`oxyn whoami` shows this month's count.)"),
            Error::NoFramework(s, fw, has) => write!(
                f,
                "`{s}` has no {fw} build (it ships for {})",
                if has.is_empty() {
                    "no framework yet".to_string()
                } else {
                    has.iter()
                        .map(|x| x.as_str())
                        .collect::<Vec<_>>()
                        .join(", ")
                }
            ),
            Error::Network(url, why) => write!(f, "could not read {url}: {why}"),
            Error::Io(p, e) => write!(f, "{}: {e}", p.display()),
            Error::Invalid(m) => f.write_str(m),
        }
    }
}

impl std::error::Error for Error {}

#[cfg(test)]
pub(crate) mod tests {
    use std::fs;
    use std::path::Path;

    use oxyn_codegen::registry::{self, BuildConfig, MetaItem};
    use oxyn_registry_schema::{Framework, Kind, Tier};

    use crate::Error;

    use crate::client::Client;

    fn w(root: &Path, p: &str, s: &str) {
        let f = root.join(p);
        fs::create_dir_all(f.parent().unwrap()).unwrap();
        fs::write(f, s).unwrap();
    }

    /// A real registry built by `oxyn-codegen` from a tiny renderer crate: `send-button` and
    /// `composer` are free, `fancy-screen` is Pro.
    pub(crate) fn fake_registry() -> (tempfile::TempDir, Client) {
        let d = tempfile::tempdir().unwrap();
        let r = d.path();
        let dx = "crates/oxyn-dioxus/src";
        w(
            r,
            &format!("{dx}/lib.rs"),
            "//! Root.\nmod brand;\nmod composer;\nmod prem;\npub use composer::{Composer, SendButton, COMPOSER_CSS};\npub use prem::Fancy;\n/// Colour.\npub fn color_var() -> String { String::new() }\n",
        );
        w(
            r,
            &format!("{dx}/brand.rs"),
            "pub fn BrandIcon() {}\npub fn unused_helper() {}\n",
        );
        w(
            r,
            &format!("{dx}/composer/mod.rs"),
            "use dioxus::prelude::*;\nmod icons;\nmod send;\npub use send::SendButton;\npub(crate) use icons::Icon;\npub const COMPOSER_CSS: &str = include_str!(\"composer.css\");\n#[component]\npub fn Composer() -> Element { let _ = Icon::Send; rsx! {} }\n",
        );
        w(r, &format!("{dx}/composer/composer.css"), ".x{}");
        w(
            r,
            &format!("{dx}/composer/icons.rs"),
            "pub enum Icon { Send }\n",
        );
        w(
            r,
            &format!("{dx}/composer/send.rs"),
            "use crate::brand::BrandIcon;\nuse oxyn_core::ClockMs;\n#[component]\npub fn SendButton() -> Element { BrandIcon(); crate::color_var(); rsx! {} }\n",
        );
        w(
            r,
            &format!("{dx}/prem.rs"),
            "use crate::composer::Icon;\npub fn Fancy() { let _ = Icon::Send; }\n",
        );
        w(
            r,
            "crates/oxyn-dioxus/Cargo.toml",
            "[package]\nname = \"oxyn-dioxus\"\n[dependencies]\ndioxus = { version = \"0.7\", default-features = false }\noxyn-core.workspace = true\n",
        );
        w(r, "crates/oxyn-gpui/src/lib.rs", "//! GPUI.\n");
        w(
            r,
            "crates/oxyn-gpui/Cargo.toml",
            "[package]\nname = \"oxyn-gpui\"\n",
        );
        w(
            r,
            "Cargo.toml",
            "[workspace.dependencies]\noxyn-core = { path = \"crates/oxyn-core\" }\n",
        );
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
        registry::write_pro(&reg, &r.join("_pro/r"), &r.join("r")).unwrap();
        let client = Client::new(r.join("r").to_str().unwrap()).with_token(None);
        (d, client)
    }

    /// A tiny HTTP server for the Pro registry: `/r/*` from the fake registry, `/api/r/<slug>.json`
    /// from its `_pro/r` behind `Bearer oxyn_good` (`oxyn_capped` answers the monthly limit),
    /// `/api/cli/whoami` and the login endpoints (approved on the second poll). Returns the base
    /// URL and the request log (`METHOD path auth`).
    pub(crate) fn mock_server(
        root: std::path::PathBuf,
    ) -> (String, std::sync::Arc<std::sync::Mutex<Vec<String>>>) {
        use std::io::{BufRead, BufReader, Read, Write};
        use std::sync::{Arc, Mutex};
        let l = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let base = format!("http://{}", l.local_addr().unwrap());
        let log = Arc::new(Mutex::new(Vec::new()));
        let log2 = log.clone();
        std::thread::spawn(move || {
            let mut polls = 0;
            let mut unlocked: Vec<String> = Vec::new();
            for stream in l.incoming() {
                let Ok(mut stream) = stream else { continue };
                let mut r = BufReader::new(stream.try_clone().unwrap());
                let mut line = String::new();
                r.read_line(&mut line).unwrap();
                let mut auth = String::new();
                let mut len = 0usize;
                loop {
                    let mut h = String::new();
                    r.read_line(&mut h).unwrap();
                    let h = h.trim_end();
                    if h.is_empty() {
                        break;
                    }
                    let (k, v) = h.split_once(':').unwrap_or((h, ""));
                    match k.to_ascii_lowercase().as_str() {
                        "authorization" => auth = v.trim().to_string(),
                        "content-length" => len = v.trim().parse().unwrap_or(0),
                        _ => {}
                    }
                }
                let mut body = vec![0; len];
                r.read_exact(&mut body).unwrap();
                let mut parts = line.split_whitespace();
                let method = parts.next().unwrap_or("").to_string();
                let path = parts.next().unwrap_or("").to_string();
                log2.lock()
                    .unwrap()
                    .push(format!("{method} {path} {auth}").trim().to_string());
                let usage = format!(
                    "{} of 20 components, 0 of 8 screens, resets 2026-12-05",
                    unlocked.len()
                );
                let (status, text, extra): (u16, String, String) = if let Some(rel) =
                    path.strip_prefix("/r/")
                {
                    match fs::read_to_string(root.join("r").join(rel)) {
                        Ok(t) => (200, t, String::new()),
                        Err(_) => (404, "{}".into(), String::new()),
                    }
                } else if let Some(rest) = path.strip_prefix("/api/r/") {
                    let slug = rest.split(['.', '?']).next().unwrap_or("").to_string();
                    match auth.as_str() {
                        "Bearer oxyn_good" => match fs::read_to_string(root.join("_pro/r").join(format!("{slug}.json"))) {
                            Ok(t) => {
                                let counted = !unlocked.contains(&slug) && !path.contains("?for=");
                                if counted {
                                    unlocked.push(slug.clone());
                                }
                                let usage = format!("{} of 20 components, 0 of 8 screens, resets 2026-12-05", unlocked.len());
                                (200, t, format!("x-oxyn-counted: {}\r\nx-oxyn-usage: {usage}\r\n", if counted { 1 } else { 0 }))
                            }
                            Err(_) => (404, r#"{"ok":false,"error":{"code":"not_found","message":"no such item"}}"#.into(), String::new()),
                        },
                        "Bearer oxyn_capped" => (403, r#"{"ok":false,"error":{"code":"quota_exceeded","message":"You have opened 20 of 20 Pro components this billing month. The count resets on 2026-12-05."}}"#.into(), String::new()),
                        "Bearer oxyn_free" => (403, r#"{"ok":false,"error":{"code":"pro_required","message":"This item is part of Oxyn Pro."}}"#.into(), String::new()),
                        _ => (401, r#"{"ok":false,"error":{"code":"unauthorized","message":"Sign in first"}}"#.into(), String::new()),
                    }
                } else if path == "/api/cli/whoami" {
                    if auth == "Bearer oxyn_good" {
                        (
                            200,
                            format!(
                                r#"{{"ok":true,"email":"dev@example.com","tier":"pro","pro":true,"admin":false,"usage":{{"summary":"{usage}"}}}}"#
                            ),
                            String::new(),
                        )
                    } else {
                        (
                            401,
                            r#"{"ok":false,"error":{"code":"unauthorized","message":"no"}}"#.into(),
                            String::new(),
                        )
                    }
                } else if path == "/api/cli/auth/start" {
                    (
                        200,
                        format!(
                            r#"{{"ok":true,"device_code":"dev_x","user_code":"BCDF-GHJK","verification_url":"{}/cli/auth/?code=BCDF-GHJK","interval":1,"expires_in":30}}"#,
                            "http://mock"
                        ),
                        String::new(),
                    )
                } else if path == "/api/cli/auth/poll" {
                    polls += 1;
                    if polls < 2 {
                        (
                            200,
                            r#"{"ok":true,"status":"pending"}"#.into(),
                            String::new(),
                        )
                    } else {
                        (200, r#"{"ok":true,"status":"approved","token":"oxyn_good","email":"dev@example.com"}"#.into(), String::new())
                    }
                } else {
                    (404, "{}".into(), String::new())
                };
                let _ = write!(
                    stream,
                    "HTTP/1.1 {status} X\r\ncontent-type: application/json\r\ncontent-length: {}\r\nconnection: close\r\n{extra}\r\n{text}",
                    text.len()
                );
            }
        });
        (base, log)
    }

    #[test]
    fn pro_items_install_with_a_token_and_dependencies_name_their_parent() {
        let (r, _) = fake_registry();
        let (base, log) = mock_server(r.path().to_path_buf());
        let client = Client::new(&format!("{base}/r")).with_token(Some("oxyn_good".into()));
        assert_eq!(client.api(), Some(format!("{base}/api").as_str()));
        let idx = client.index().unwrap();
        assert!(idx.get("fancy-screen").unwrap().installable);
        let items =
            crate::resolve::resolve(&client, &idx, &["fancy-screen".into()], Framework::Dioxus)
                .unwrap();
        let slugs: Vec<_> = items.iter().map(|i| i.slug.as_str()).collect();
        assert_eq!(slugs.last(), Some(&"fancy-screen"));
        assert!(slugs.contains(&"theme"), "{slugs:?}");
        assert_eq!(client.counted(), vec!["fancy-screen".to_string()]);
        assert_eq!(
            client.last_usage().as_deref(),
            Some("1 of 20 components, 0 of 8 screens, resets 2026-12-05")
        );
        let log = log.lock().unwrap().clone();
        assert!(
            log.iter()
                .any(|l| l == "GET /api/r/fancy-screen.json Bearer oxyn_good"),
            "{log:?}"
        );
        // Free items never carry the token.
        assert!(log.iter().any(|l| l == "GET /r/theme.json"), "{log:?}");
        // Cached: item() returns the Pro item without another request.
        assert!(client.item("fancy-screen").unwrap().is_some());
    }

    #[test]
    fn pro_errors_sign_in_pro_and_monthly_limit() {
        let (r, _) = fake_registry();
        let (base, _) = mock_server(r.path().to_path_buf());
        let c = |t: Option<&str>| Client::new(&format!("{base}/r")).with_token(t.map(String::from));
        let none = c(None).fetch_pro("fancy-screen", None).unwrap_err();
        assert!(matches!(none, Error::SignInRequired(_)));
        assert_eq!(none.exit_code(), 2);
        assert!(none.to_string().contains("oxyn login"));
        assert!(matches!(
            c(Some("oxyn_revoked")).fetch_pro("fancy-screen", None),
            Err(Error::SignInRequired(_))
        ));
        let free = c(Some("oxyn_free"))
            .fetch_pro("fancy-screen", None)
            .unwrap_err();
        assert!(matches!(free, Error::ProRequired(_)));
        assert_eq!(free.exit_code(), 2);
        let capped = c(Some("oxyn_capped"))
            .fetch_pro("fancy-screen", None)
            .unwrap_err();
        assert_eq!(capped.exit_code(), 3);
        assert!(
            capped.to_string().contains("resets on 2026-12-05"),
            "{capped}"
        );
    }

    #[test]
    fn whoami_and_browser_login() {
        let (r, _) = fake_registry();
        let (base, _) = mock_server(r.path().to_path_buf());
        let good = Client::new(&format!("{base}/r")).with_token(Some("oxyn_good".into()));
        let v = crate::auth::whoami(&good).unwrap();
        assert_eq!(v["email"], "dev@example.com");
        assert!(
            crate::auth::usage_line(&v)
                .unwrap()
                .contains("of 20 components")
        );
        let anon = Client::new(&format!("{base}/r")).with_token(None);
        assert!(crate::auth::whoami(&anon).is_err());
        let d = crate::auth::start_login(&anon, "laptop").unwrap();
        assert_eq!(d.user_code, "BCDF-GHJK");
        let (t, email) = crate::auth::wait_login(&anon, &d).unwrap();
        assert_eq!(
            (t.as_str(), email.as_deref()),
            ("oxyn_good", Some("dev@example.com"))
        );
    }
}
