//! The `oxyn` command line as a library: the registry client, the dependency resolver and the
//! project writer behind `oxyn init | list | search | view | add | diff`. The MCP server (P2) and
//! the Pro registry client (P3) reuse these pieces.
//!
//! - [`client`]: reads `index.json` and `<slug>.json` over HTTP(S) or from a directory; holds the
//!   `OXYN_TOKEN` seam for the Pro registry.
//! - [`resolve`]: slugs to items in install order, Pro / unknown / framework errors, search.
//! - [`install`]: composes files from chunks, rewrites `crate::` paths, edits `Cargo.toml`, writes
//!   with `oxyn.lock` protection, diffs.
//! - [`config`]: `oxyn.toml` and `oxyn.lock`.

pub mod client;
pub mod config;
pub mod install;
pub mod resolve;

use std::fmt;
use std::path::PathBuf;

use oxyn_registry_schema::Framework;

pub use client::{Client, DEFAULT_REGISTRY, PRICING_URL, auth_header, registry_base};
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
    /// A Pro item: needs Oxyn Pro.
    ProRequired(String),
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
    /// Process exit code: 2 for "needs Pro", 1 otherwise.
    pub fn exit_code(&self) -> i32 {
        if matches!(self, Error::ProRequired(_)) {
            2
        } else {
            1
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
                "`{s}` is part of Oxyn Pro. The preview is free on the site; the code needs Pro: {PRICING_URL}/\n(Pro installs from the CLI arrive with sign-in in the next release; OXYN_TOKEN will carry your licence.)"
            ),
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
    use oxyn_registry_schema::{Kind, Tier};

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
        let client = Client::new(r.join("r").to_str().unwrap());
        (d, client)
    }
}
