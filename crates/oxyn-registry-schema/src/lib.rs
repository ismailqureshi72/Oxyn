//! Schema for the Oxyn registry: the shapes of `r/index.json` and `r/<slug>.json`.
//!
//! The registry is static JSON, shadcn style. `index.json` lists every item (free and Pro) without
//! file contents; `<slug>.json` holds one free item with the source of each file, per framework.
//! The registry builder (`oxyn-codegen`, run in CI before the site build) writes it, and the `oxyn`
//! CLI (and the MCP server after it) reads it. Pro items are listed in the index but never get a
//! public JSON; their source comes from the authenticated Pro registry.
//!
//! Every file of an item is source copied into the user's project under one module directory
//! (`src/oxyn/` by default), keeping its path relative to the renderer crate's `src/`. So
//! `crates/oxyn-dioxus/src/composer/send.rs` lands at `src/oxyn/composer/send.rs`, the renderer's
//! `lib.rs` becomes `src/oxyn/mod.rs`, and `crate::` paths are rewritten to the module path.
//! Rust files travel as [`Chunk`]s (top-level items): an item carries only the chunks it needs, so
//! a file shared with Pro items never ships their code, and the CLI merges the chunks of every
//! installed item.

use std::collections::BTreeMap;
use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

/// Version of this schema. Bump it on a breaking change; clients refuse a newer one.
pub const SCHEMA_VERSION: u32 = 1;

/// Crate version, from `Cargo.toml`.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Free or Pro.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Tier {
    /// Core components and primitives: public JSON, MIT source.
    Free,
    /// Screens, agent-app templates, blocks, the AI special components: Oxyn Pro.
    Pro,
}

impl Tier {
    /// The lowercase name used in JSON and on the command line.
    pub fn as_str(self) -> &'static str {
        match self {
            Tier::Free => "free",
            Tier::Pro => "pro",
        }
    }
}

impl fmt::Display for Tier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for Tier {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, String> {
        match s.to_ascii_lowercase().as_str() {
            "free" => Ok(Tier::Free),
            "pro" => Ok(Tier::Pro),
            _ => Err(format!("unknown tier `{s}` (use free or pro)")),
        }
    }
}

/// What an item is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Kind {
    /// One component or primitive (a page under /components/).
    Component,
    /// A whole app frame (a page under /screens/).
    Screen,
    /// A screen of one of the agent-app templates.
    App,
    /// A larger composition (a page under /blocks/).
    Block,
    /// A stylesheet (the design tokens).
    Style,
}

impl Kind {
    /// The lowercase name used in JSON.
    pub fn as_str(self) -> &'static str {
        match self {
            Kind::Component => "component",
            Kind::Screen => "screen",
            Kind::App => "app",
            Kind::Block => "block",
            Kind::Style => "style",
        }
    }
}

impl fmt::Display for Kind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A renderer an item ships for.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Framework {
    /// Dioxus 0.7 (web, desktop, mobile).
    Dioxus,
    /// GPUI (Zed's UI framework), pinned to the revision in the Oxyn workspace.
    Gpui,
}

impl Framework {
    /// The lowercase name used in JSON, `oxyn.toml` and on the command line.
    pub fn as_str(self) -> &'static str {
        match self {
            Framework::Dioxus => "dioxus",
            Framework::Gpui => "gpui",
        }
    }
    /// Every framework, in a stable order.
    pub const ALL: [Framework; 2] = [Framework::Dioxus, Framework::Gpui];
}

impl fmt::Display for Framework {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for Framework {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, String> {
        match s.to_ascii_lowercase().as_str() {
            "dioxus" => Ok(Framework::Dioxus),
            "gpui" => Ok(Framework::Gpui),
            _ => Err(format!("unknown framework `{s}` (use dioxus or gpui)")),
        }
    }
}

/// How a file is written into the user's project.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, Default,
)]
#[serde(rename_all = "lowercase")]
pub enum FileKind {
    /// Rust source under the module directory; `crate::` is rewritten to the module path.
    #[default]
    Rust,
    /// A file a Rust file includes (`include_str!`), copied verbatim next to it.
    Asset,
    /// A stylesheet the app loads itself, written to the CSS directory (`assets/oxyn/`).
    Css,
}

/// One top-level piece of a Rust file: an item (`fn`, `struct`, `impl`, …), a `use` line or a
/// child `mod` declaration, in source order.
///
/// Files are shipped as chunks so that two items sharing a source file each carry only what they
/// need, and the CLI writes the union of the chunks of every installed item. A chunk with
/// `requires` is conditional (a `use` line, a `mod child;`): the CLI writes it only when the chunk
/// or file it names is installed too.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Chunk {
    /// Position in the source file; stable for one registry build, and the key two items' copies
    /// of a file are merged by.
    pub id: u32,
    /// The source text, comments and attributes included.
    pub code: String,
    /// What must be installed for this chunk to compile: `"<target>#<id>"` for a chunk,
    /// `"file:<target>"` for a whole file (a `mod` declaration, a glob re-export).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub requires: Option<String>,
}

/// One file of an item.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RegistryFile {
    /// Where the file lives in the Oxyn repository (`crates/oxyn-dioxus/src/composer/send.rs`).
    pub path: String,
    /// Where it goes, relative to the module directory (Rust, assets) or the CSS directory.
    pub target: String,
    /// How it is written.
    #[serde(default)]
    pub kind: FileKind,
    /// Rust files: the chunks this item needs (unconditional) plus every conditional chunk.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub chunks: Vec<Chunk>,
    /// Assets and stylesheets: the whole content, unless `url` is set.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub content: String,
    /// Assets shared by many items are stored once: the content is at this path, relative to the
    /// registry base URL (`files/dioxus/kit/run.css`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
}

impl RegistryFile {
    /// The key a chunk of this file is required by.
    pub fn chunk_key(&self, id: u32) -> String {
        format!("{}#{id}", self.target)
    }
    /// The key this whole file is required by.
    pub fn file_key(&self) -> String {
        format!("file:{}", self.target)
    }
}

/// A Cargo dependency an item needs.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct CargoDep {
    /// Package name (`oxyn-core`, `dioxus`, `gpui`).
    pub name: String,
    /// A crates.io version requirement, for crates.io packages.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    /// A git repository, for git packages (GPUI).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub git: Option<String>,
    /// The pinned git revision.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rev: Option<String>,
    /// True for a crate of the Oxyn repository: its source (git rev or a local checkout) comes
    /// from `oxyn.toml`, defaulting to the commit the registry was built from.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub oxyn: bool,
    /// `default-features = false` when set.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub no_default_features: bool,
    /// Features to enable.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub features: Vec<String>,
}

/// What an item ships for one framework.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FrameworkBuild {
    /// The renderer.
    pub framework: Framework,
    /// The item's main type or function (`Composer`, `ComposerEl`).
    pub entry: String,
    /// The module path of the entry, relative to the Oxyn module (`composer`), for `use` lines.
    #[serde(default)]
    pub module: String,
    /// Other items this one needs (installed first, recursively).
    #[serde(default)]
    pub oxyn_deps: Vec<String>,
    /// Cargo dependencies.
    #[serde(default)]
    pub cargo_deps: Vec<CargoDep>,
    /// Stylesheet constants to put in the page once (`COMPOSER_CSS`), Dioxus only.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub css: Vec<String>,
    /// The files. Empty in the index.
    #[serde(default)]
    pub files: Vec<RegistryFile>,
}

/// One registry item with its files: `r/<slug>.json`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RegistryItem {
    /// [`SCHEMA_VERSION`] when written.
    pub schema: u32,
    /// URL-safe name (`send-button`).
    pub slug: String,
    /// Display name (`Send button`).
    pub name: String,
    /// What it is.
    pub kind: Kind,
    /// Site category (`chat`, `inputs`, `agents`, ...).
    pub category: String,
    /// Free or Pro.
    pub tier: Tier,
    /// One-line summary.
    #[serde(default)]
    pub description: String,
    /// Its page on the site.
    #[serde(default)]
    pub url: String,
    /// Content hash of every file of every framework (hex SHA-256 prefix); changes when any file does.
    #[serde(default)]
    pub hash: String,
    /// The frameworks it ships for, with their files.
    pub frameworks: Vec<FrameworkBuild>,
}

impl RegistryItem {
    /// The build for one framework, if the item ships it.
    pub fn build(&self, fw: Framework) -> Option<&FrameworkBuild> {
        self.frameworks.iter().find(|b| b.framework == fw)
    }
    /// Every framework the item ships for.
    pub fn framework_list(&self) -> Vec<Framework> {
        self.frameworks.iter().map(|b| b.framework).collect()
    }
}

/// One line of `r/index.json`: an item without file contents.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IndexEntry {
    /// URL-safe name.
    pub slug: String,
    /// Display name.
    pub name: String,
    /// What it is.
    pub kind: Kind,
    /// Site category.
    pub category: String,
    /// Free or Pro.
    pub tier: Tier,
    /// One-line summary.
    #[serde(default)]
    pub description: String,
    /// Its page on the site.
    #[serde(default)]
    pub url: String,
    /// Frameworks it ships for.
    #[serde(default)]
    pub frameworks: Vec<Framework>,
    /// Direct Oxyn dependencies per framework.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub oxyn_deps: BTreeMap<Framework, Vec<String>>,
    /// Content hash (same as the item's).
    #[serde(default)]
    pub hash: String,
    /// True when `r/<slug>.json` exists (a free item the CLI can add). Pro items are false here;
    /// so is a free item that shares a source file with a Pro item (see `note`).
    #[serde(default)]
    pub installable: bool,
    /// Why an item is not installable, when it is not.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
    /// Older slugs of this item (it was renamed); `oxyn add <old>` still finds it.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub aliases: Vec<String>,
}

/// Where the Oxyn crates come from by default.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OxynSource {
    /// The repository (`https://github.com/ismailqureshi72/Oxyn`).
    pub git: String,
    /// The commit the registry was built from; copied code matches `oxyn-core` at this commit.
    pub rev: String,
}

/// `r/index.json`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RegistryIndex {
    /// [`SCHEMA_VERSION`] when written.
    pub schema: u32,
    /// `oxyn`.
    pub name: String,
    /// The site.
    pub homepage: String,
    /// Where Pro is sold.
    pub pricing: String,
    /// Where the Oxyn crates come from.
    pub source: OxynSource,
    /// Every item, free and Pro, sorted by slug.
    pub items: Vec<IndexEntry>,
}

impl RegistryIndex {
    /// The entry for a slug, or for an older slug of a renamed item (see [`IndexEntry::aliases`]).
    /// The entry's own `slug` is the current one.
    pub fn get(&self, slug: &str) -> Option<&IndexEntry> {
        self.items.iter().find(|e| e.slug == slug).or_else(|| {
            self.items
                .iter()
                .find(|e| e.aliases.iter().any(|a| a == slug))
        })
    }
}

/// Checks a slug: 1 to 80 of `[a-z0-9-]`, not starting or ending with `-`.
pub fn valid_slug(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 80
        && !s.starts_with('-')
        && !s.ends_with('-')
        && s.bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
}

#[cfg(test)]
mod tests {
    use super::*;

    fn item() -> RegistryItem {
        RegistryItem {
            schema: SCHEMA_VERSION,
            slug: "send-button".into(),
            name: "Send button".into(),
            kind: Kind::Component,
            category: "chat".into(),
            tier: Tier::Free,
            description: "Send, stop, queue.".into(),
            url: "https://ui.oxyn.workers.dev/components/chat/send-button/".into(),
            hash: "abc".into(),
            frameworks: vec![FrameworkBuild {
                framework: Framework::Dioxus,
                entry: "SendButton".into(),
                module: "composer".into(),
                oxyn_deps: vec!["theme".into()],
                cargo_deps: vec![CargoDep {
                    name: "oxyn-core".into(),
                    version: None,
                    git: None,
                    rev: None,
                    oxyn: true,
                    no_default_features: false,
                    features: vec![],
                }],
                css: vec!["COMPOSER_CSS".into()],
                files: vec![RegistryFile {
                    path: "crates/oxyn-dioxus/src/composer/mod.rs".into(),
                    target: "composer/mod.rs".into(),
                    kind: FileKind::Rust,
                    chunks: vec![
                        Chunk {
                            id: 0,
                            code: "use dioxus::prelude::*;".into(),
                            requires: None,
                        },
                        Chunk {
                            id: 1,
                            code: "mod send;".into(),
                            requires: Some("file:composer/send.rs".into()),
                        },
                    ],
                    content: String::new(),
                    url: None,
                }],
            }],
        }
    }

    #[test]
    fn item_round_trips_through_json() {
        let it = item();
        let s = serde_json::to_string(&it).unwrap();
        let back: RegistryItem = serde_json::from_str(&s).unwrap();
        assert_eq!(back, it);
        assert!(s.contains("\"tier\":\"free\""));
        assert!(s.contains("\"kind\":\"component\""));
        assert!(s.contains("\"framework\":\"dioxus\""));
        // Empty optional fields stay out of the JSON.
        assert!(!s.contains("\"git\""));
        assert!(!s.contains("no_default_features"));
    }

    #[test]
    fn file_kind_defaults_to_rust_and_keys_are_stable() {
        let f: RegistryFile =
            serde_json::from_str(r#"{"path":"a","target":"composer/mod.rs"}"#).unwrap();
        assert_eq!(f.kind, FileKind::Rust);
        assert!(f.chunks.is_empty());
        assert_eq!(f.chunk_key(3), "composer/mod.rs#3");
        assert_eq!(f.file_key(), "file:composer/mod.rs");
    }

    #[test]
    fn tiers_and_frameworks_parse() {
        assert_eq!("Pro".parse::<Tier>().unwrap(), Tier::Pro);
        assert_eq!("free".parse::<Tier>().unwrap(), Tier::Free);
        assert!("gold".parse::<Tier>().is_err());
        assert_eq!("gpui".parse::<Framework>().unwrap(), Framework::Gpui);
        assert!("react".parse::<Framework>().is_err());
        assert_eq!(Tier::Pro.to_string(), "pro");
    }

    #[test]
    fn index_entries_tolerate_missing_optional_fields() {
        let e: IndexEntry = serde_json::from_str(
            r#"{"slug":"x","name":"X","kind":"screen","category":"screens","tier":"pro"}"#,
        )
        .unwrap();
        assert!(!e.installable);
        assert!(e.frameworks.is_empty());
        assert_eq!(e.tier, Tier::Pro);
    }

    #[test]
    fn index_finds_items_by_an_older_slug() {
        let e: IndexEntry = serde_json::from_str(
            r#"{"slug":"crew-tree","name":"Crew tree","kind":"component","category":"agents","tier":"free","aliases":["agent-crew"]}"#,
        )
        .unwrap();
        let idx = RegistryIndex {
            schema: SCHEMA_VERSION,
            name: "oxyn".into(),
            homepage: String::new(),
            pricing: String::new(),
            source: OxynSource {
                git: String::new(),
                rev: String::new(),
            },
            items: vec![e],
        };
        assert_eq!(idx.get("agent-crew").unwrap().slug, "crew-tree");
        assert_eq!(idx.get("crew-tree").unwrap().slug, "crew-tree");
        assert!(idx.get("agent").is_none());
    }

    #[test]
    fn slugs_are_checked() {
        assert!(valid_slug("send-button"));
        assert!(valid_slug("a1"));
        assert!(!valid_slug(""));
        assert!(!valid_slug("-a"));
        assert!(!valid_slug("Send"));
        assert!(!valid_slug("../etc"));
    }

    #[test]
    fn version_is_set() {
        assert!(!VERSION.is_empty());
    }
}
