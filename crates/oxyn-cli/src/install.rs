//! Writing items into a project: composing files from chunks, rewriting `crate::` paths, editing
//! `Cargo.toml`, declaring the module, and diffing against what is on disk.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use oxyn_registry_schema::{
    CargoDep, Chunk, FileKind, Framework, RegistryIndex, RegistryItem, Tier,
};
use toml_edit::{DocumentMut, InlineTable, Item, Value};

use crate::Error;
use crate::client::Client;
use crate::config::{self, Config, Lock, Source};

/// First lines of the generated root module.
pub const ROOT_HEADER: &str = "// Oxyn components, added with `oxyn add` (https://oxynui.com/docs/cli/).\n// The CLI rewrites this file as you add components; the other files are yours to edit.\n#![allow(dead_code, unused_imports, unused_macros)]\n";

/// What `init` found and did.
#[derive(Debug, Default)]
pub struct InitReport {
    /// The framework chosen.
    pub framework: Option<Framework>,
    /// Lines describing each change.
    pub changes: Vec<String>,
}

/// Which renderers a `Cargo.toml` depends on.
pub fn detect_frameworks(cargo_toml: &str) -> Vec<Framework> {
    let Ok(doc) = cargo_toml.parse::<DocumentMut>() else {
        return Vec::new();
    };
    let mut out = Vec::new();
    let deps = doc.get("dependencies").and_then(Item::as_table_like);
    for fw in Framework::ALL {
        if deps.is_some_and(|d| d.contains_key(fw.as_str())) {
            out.push(fw);
        }
    }
    out
}

/// `oxyn init`: writes `oxyn.toml`, the root module, `mod oxyn;` and the `oxyn-core` dependency.
pub fn init(
    project: &Path,
    framework: Option<Framework>,
    path: Option<&str>,
    source: Source,
    force: bool,
) -> Result<InitReport, Error> {
    let cargo = project.join("Cargo.toml");
    let manifest =
        fs::read_to_string(&cargo).map_err(|_| Error::NoCargoProject(project.to_path_buf()))?;
    if project.join(config::CONFIG_FILE).exists() && !force {
        return Err(Error::Invalid(format!(
            "{} already exists (use --force to write it again)",
            config::CONFIG_FILE
        )));
    }
    let found = detect_frameworks(&manifest);
    let fw = match (framework, found.as_slice()) {
        (Some(f), _) => f,
        (None, [one]) => *one,
        (None, []) => {
            return Err(Error::Invalid(
                "no dioxus or gpui dependency in Cargo.toml: add one (for example `cargo add dioxus --features web`) or pass --framework dioxus|gpui".into(),
            ));
        }
        (None, _) => {
            return Err(Error::Invalid(
                "Cargo.toml depends on both dioxus and gpui: pass --framework dioxus|gpui".into(),
            ));
        }
    };
    let mut cfg = Config::new(fw, source);
    if let Some(p) = path {
        cfg.path = p.trim_end_matches('/').to_string();
    }
    let module = cfg.module_path()?;
    let mut rep = InitReport {
        framework: Some(fw),
        changes: Vec::new(),
    };
    let mut lock = Lock::load(project);
    // The root module, so `mod oxyn;` compiles before the first `oxyn add`.
    let root = project.join(&cfg.path).join("mod.rs");
    if !root.exists() {
        let stub = format!("{ROOT_HEADER}//! Components go here as you run `oxyn add <name>`.\n");
        config::write(&root, &stub)?;
        lock.files.insert(rel(project, &root), config::hash(&stub));
        rep.changes.push(format!("created {}", rel(project, &root)));
    }
    if let Some(line) = declare_module(project, &module)? {
        rep.changes.push(line);
    }
    let core = CargoDep {
        name: "oxyn-core".into(),
        version: None,
        git: None,
        rev: None,
        oxyn: true,
        no_default_features: false,
        features: vec![],
    };
    for added in add_cargo_deps(project, &[core], &cfg.oxyn)? {
        rep.changes.push(format!("added {added} to Cargo.toml"));
    }
    cfg.save(project)?;
    lock.save(project)?;
    rep.changes.push(format!(
        "wrote {} (framework {fw}, components in {})",
        config::CONFIG_FILE,
        cfg.path
    ));
    Ok(rep)
}

/// Adds `mod <name>;` to `src/lib.rs` (or `src/main.rs`) for a top-level module path like
/// `crate::oxyn`. Returns what it did, or `None` when the declaration is already there. Nested paths
/// are left to the user (the parent module is theirs).
pub fn declare_module(project: &Path, module: &str) -> Result<Option<String>, Error> {
    let parts: Vec<&str> = module.trim_start_matches("crate::").split("::").collect();
    if parts.len() != 1 {
        return Ok(Some(format!(
            "declare `mod {};` in src/{}.rs yourself (nested component paths are not edited)",
            parts[parts.len() - 1],
            parts[..parts.len() - 1].join("/")
        )));
    }
    let name = parts[0];
    let lib = project.join("src/lib.rs");
    let main = project.join("src/main.rs");
    let target = if lib.exists() {
        lib
    } else if main.exists() {
        main
    } else {
        return Ok(Some(format!("add `mod {name};` to your crate root")));
    };
    let src = fs::read_to_string(&target).map_err(|e| Error::Io(target.clone(), e))?;
    let has = src.lines().any(|l| {
        let t = l.trim();
        t == format!("mod {name};")
            || t == format!("pub mod {name};")
            || t == format!("pub(crate) mod {name};")
    });
    if has {
        return Ok(None);
    }
    // After the leading inner docs and attributes.
    let mut at = 0;
    for line in src.split_inclusive('\n') {
        let t = line.trim_start();
        if t.starts_with("//!") || t.starts_with("#![") || t.trim().is_empty() {
            at += line.len();
        } else {
            break;
        }
    }
    let new = format!(
        "{}mod {name};\n{}{}",
        &src[..at],
        if at == 0 { "\n" } else { "" },
        &src[at..]
    );
    fs::write(&target, new).map_err(|e| Error::Io(target.clone(), e))?;
    Ok(Some(format!(
        "declared `mod {name};` in {}",
        rel(project, &target)
    )))
}

/// Adds the Cargo dependencies that are not there yet; returns their names. Existing entries are
/// never changed.
pub fn add_cargo_deps(
    project: &Path,
    deps: &[CargoDep],
    source: &Source,
) -> Result<Vec<String>, Error> {
    let p = project.join("Cargo.toml");
    let s = fs::read_to_string(&p).map_err(|e| Error::Io(p.clone(), e))?;
    let mut doc: DocumentMut = s
        .parse()
        .map_err(|e| Error::Invalid(format!("Cargo.toml: {e}")))?;
    if doc.get("dependencies").is_none() {
        doc["dependencies"] = toml_edit::table();
    }
    let table = doc["dependencies"]
        .as_table_like_mut()
        .ok_or_else(|| Error::Invalid("Cargo.toml: [dependencies] is not a table".into()))?;
    let mut added = Vec::new();
    let mut seen = BTreeSet::new();
    for d in deps {
        if !seen.insert(d.name.clone()) || table.contains_key(&d.name) {
            continue;
        }
        let mut t = InlineTable::new();
        if d.oxyn {
            if let Some(path) = &source.path {
                t.insert(
                    "path",
                    Value::from(format!("{}/crates/{}", path.trim_end_matches('/'), d.name)),
                );
            } else {
                t.insert(
                    "git",
                    Value::from(
                        source
                            .git
                            .clone()
                            .unwrap_or_else(|| "https://github.com/ismailqureshi72/Oxyn".into()),
                    ),
                );
                if let Some(rev) = &source.rev {
                    t.insert("rev", Value::from(rev.clone()));
                }
            }
        } else {
            if let Some(v) = &d.version {
                if d.git.is_none() && !d.no_default_features && d.features.is_empty() {
                    table.insert(&d.name, toml_edit::value(v.clone()));
                    added.push(d.name.clone());
                    continue;
                }
                t.insert("version", Value::from(v.clone()));
            }
            if let Some(g) = &d.git {
                t.insert("git", Value::from(g.clone()));
            }
            if let Some(r) = &d.rev {
                t.insert("rev", Value::from(r.clone()));
            }
            if d.no_default_features {
                t.insert("default-features", Value::from(false));
            }
            if !d.features.is_empty() {
                let mut a = toml_edit::Array::new();
                for f in &d.features {
                    a.push(f.as_str());
                }
                t.insert("features", Value::Array(a));
            }
        }
        table.insert(&d.name, Item::Value(Value::InlineTable(t)));
        added.push(d.name.clone());
    }
    if !added.is_empty() {
        fs::write(&p, doc.to_string()).map_err(|e| Error::Io(p.clone(), e))?;
    }
    Ok(added)
}

/// One file the CLI would write.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Planned {
    /// Path relative to the project.
    pub path: String,
    /// Full content.
    pub content: String,
}

/// Rewrites `crate::` to the components module (`crate::oxyn::`).
pub fn rewrite_crate_paths(src: &str, module: &str) -> String {
    if module == "crate" {
        return src.to_string();
    }
    let b = src.as_bytes();
    let mut out = String::with_capacity(src.len() + 64);
    let mut last = 0;
    let mut i = 0;
    while let Some(p) = src[i..].find("crate::") {
        let at = i + p;
        let prev = if at == 0 { b' ' } else { b[at - 1] };
        let boundary = !(prev.is_ascii_alphanumeric() || prev == b'_' || prev == b'$');
        if boundary {
            out.push_str(&src[last..at]);
            out.push_str(module);
            out.push_str("::");
            last = at + "crate::".len();
        }
        i = at + "crate::".len();
    }
    out.push_str(&src[last..]);
    out
}

fn single_line_decl(c: &str) -> bool {
    !c.contains('\n') && (c.contains("use ") || c.contains("mod ")) && c.ends_with(';')
}

/// Composes every file of a set of items (all installed items plus the new ones) for one framework.
pub fn compose(
    items: &[RegistryItem],
    fw: Framework,
    cfg: &Config,
    client: &Client,
) -> Result<Vec<Planned>, Error> {
    let module = cfg.module_path()?;
    let mut rust: BTreeMap<String, BTreeMap<u32, Chunk>> = BTreeMap::new();
    let mut assets: BTreeMap<String, (FileKind, Option<String>, String)> = BTreeMap::new();
    for it in items {
        let Some(b) = it.build(fw) else { continue };
        for f in &b.files {
            if f.target.contains("..") || f.target.starts_with('/') {
                return Err(Error::Invalid(format!(
                    "{}: unsafe file target `{}`",
                    it.slug, f.target
                )));
            }
            match f.kind {
                FileKind::Rust => {
                    let e = rust.entry(f.target.clone()).or_default();
                    for c in &f.chunks {
                        e.entry(c.id).or_insert_with(|| c.clone());
                    }
                }
                k => {
                    assets
                        .entry(f.target.clone())
                        .or_insert_with(|| (k, f.url.clone(), f.content.clone()));
                }
            }
        }
    }
    let mut present: BTreeSet<String> = BTreeSet::new();
    for (target, chunks) in &rust {
        present.insert(format!("file:{target}"));
        for c in chunks.values().filter(|c| c.requires.is_none()) {
            present.insert(format!("{target}#{}", c.id));
        }
    }
    let mut out = Vec::new();
    for (target, chunks) in &rust {
        let mut text = String::new();
        if target == "mod.rs" {
            text.push_str(ROOT_HEADER);
        }
        let mut prev_decl = false;
        for c in chunks.values() {
            if let Some(r) = &c.requires
                && !present.contains(r)
            {
                continue;
            }
            let decl = single_line_decl(&c.code);
            if !text.is_empty() {
                text.push_str(if decl && prev_decl { "\n" } else { "\n\n" });
            }
            text.push_str(&c.code);
            prev_decl = decl;
        }
        text.push('\n');
        out.push(Planned {
            path: format!("{}/{target}", cfg.path),
            content: rewrite_crate_paths(&text, &module),
        });
    }
    for (target, (kind, url, content)) in assets {
        let content = match url {
            Some(u) => client
                .get(&u)?
                .ok_or_else(|| Error::Invalid(format!("registry file {u} is missing")))?,
            None => content,
        };
        let dir = if kind == FileKind::Css {
            &cfg.css
        } else {
            &cfg.path
        };
        out.push(Planned {
            path: format!("{dir}/{target}"),
            content,
        });
    }
    Ok(out)
}

/// What happened to each file.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct WriteReport {
    /// Written (new or updated).
    pub written: Vec<String>,
    /// Already identical.
    pub unchanged: Vec<String>,
    /// Left alone: changed locally since the CLI wrote it (or never written by it).
    pub skipped: Vec<String>,
}

/// Writes planned files. A file that exists and was edited since the CLI wrote it (or that the CLI
/// never wrote) is skipped unless `overwrite`.
pub fn write_files(
    project: &Path,
    planned: &[Planned],
    overwrite: bool,
    lock: &mut Lock,
) -> Result<WriteReport, Error> {
    let mut rep = WriteReport::default();
    for p in planned {
        let abs = project.join(&p.path);
        let current = fs::read_to_string(&abs).ok();
        if current.as_deref() == Some(p.content.as_str()) {
            lock.files.insert(p.path.clone(), config::hash(&p.content));
            rep.unchanged.push(p.path.clone());
            continue;
        }
        if let Some(cur) = &current {
            let ours = lock
                .files
                .get(&p.path)
                .is_some_and(|h| *h == config::hash(cur));
            if !ours && !overwrite {
                rep.skipped.push(p.path.clone());
                continue;
            }
        }
        config::write(&abs, &p.content)?;
        lock.files.insert(p.path.clone(), config::hash(&p.content));
        rep.written.push(p.path.clone());
    }
    Ok(rep)
}

/// The result of `oxyn add`.
#[derive(Debug, Default)]
pub struct AddReport {
    /// Items installed, in order (dependencies first).
    pub items: Vec<String>,
    /// File outcomes.
    pub files: WriteReport,
    /// Cargo dependencies added.
    pub cargo: Vec<String>,
    /// What to do next.
    pub next: Vec<String>,
}

/// `oxyn add`: resolves, composes with what is installed, writes, edits `Cargo.toml`, records.
pub fn add(
    project: &Path,
    cfg: &mut Config,
    client: &Client,
    index: &RegistryIndex,
    slugs: &[String],
    overwrite: bool,
) -> Result<AddReport, Error> {
    // An older slug of a renamed item installs the item under its current name.
    let slugs: Vec<String> = slugs.iter().map(|s| canonical(index, s)).collect();
    let slugs = slugs.as_slice();
    let fw = cfg.framework;
    let new = crate::resolve::resolve(client, index, slugs, fw)?;
    let others: Vec<String> = cfg
        .installed
        .iter()
        .filter(|s| !new.iter().any(|i| &i.slug == *s))
        .cloned()
        .collect();
    prime_pro(project, client, index, &others);
    let mut all: Vec<RegistryItem> = Vec::new();
    for s in &cfg.installed {
        if new.iter().any(|i| &i.slug == s) {
            continue;
        }
        match client.item(s)? {
            Some(it) if it.build(fw).is_some() => all.push(it),
            _ => eprintln!(
                "warning: `{s}` is in oxyn.toml but no longer in the registry; its files stay as they are"
            ),
        }
    }
    all.extend(new.iter().cloned());
    let planned = compose(&all, fw, cfg, client)?;
    let mut lock = Lock::load(project);
    let files = write_files(project, &planned, overwrite, &mut lock)?;
    let deps: Vec<CargoDep> = all
        .iter()
        .filter_map(|i| i.build(fw))
        .flat_map(|b| b.cargo_deps.clone())
        .collect();
    let cargo = add_cargo_deps(project, &deps, &cfg.oxyn)?;
    let module = cfg.module_path()?;
    if let Some(line) = declare_module(project, &module)?
        && (line.starts_with("declare") || line.starts_with("add"))
    {
        eprintln!("note: {line}");
    }
    for it in &new {
        cfg.mark_installed(&it.slug);
    }
    save_pro(project, &new)?;
    cfg.save(project)?;
    lock.save(project)?;
    let next = next_steps(&new, slugs, fw, cfg, &module);
    Ok(AddReport {
        items: new.iter().map(|i| i.slug.clone()).collect(),
        files,
        cargo,
        next,
    })
}

/// The current slug for `s`: itself, or the item it is an older name of (see `IndexEntry::aliases`).
pub fn canonical(index: &RegistryIndex, s: &str) -> String {
    index
        .get(s)
        .map_or_else(|| s.to_string(), |e| e.slug.clone())
}

/// Where the registry JSON of installed Pro items is kept, inside the project. It holds the same
/// code `oxyn add` wrote into `src/`, so later adds and diffs need not download them again (a new
/// download would count toward the month's Pro items).
pub const PRO_CACHE: &str = ".oxyn/pro";

fn prime_pro(project: &Path, client: &Client, index: &RegistryIndex, slugs: &[String]) {
    for s in slugs {
        if index.get(s).is_some_and(|e| e.tier == Tier::Pro)
            && let Ok(t) = fs::read_to_string(project.join(PRO_CACHE).join(format!("{s}.json")))
        {
            client.prime(s, t);
        }
    }
}

fn save_pro(project: &Path, items: &[RegistryItem]) -> Result<(), Error> {
    let dir = project.join(PRO_CACHE);
    for it in items.iter().filter(|i| i.tier == Tier::Pro) {
        fs::create_dir_all(&dir).map_err(|e| Error::Io(dir.clone(), e))?;
        let p = dir.join(format!("{}.json", it.slug));
        let s = serde_json::to_string(it).map_err(|e| Error::Invalid(e.to_string()))?;
        fs::write(&p, s).map_err(|e| Error::Io(p, e))?;
    }
    Ok(())
}

/// Usage hints for the items the user asked for.
pub fn next_steps(
    items: &[RegistryItem],
    asked: &[String],
    fw: Framework,
    cfg: &Config,
    module: &str,
) -> Vec<String> {
    let mut out = Vec::new();
    let mut css: Vec<String> = Vec::new();
    for it in items.iter().filter(|i| asked.contains(&i.slug)) {
        let Some(b) = it.build(fw) else { continue };
        if b.entry.is_empty() {
            continue;
        }
        out.push(format!(
            "use {module}::{};   // {} ({})",
            b.entry, it.name, it.url
        ));
        for c in &b.css {
            if !css.contains(c) {
                css.push(c.clone());
            }
        }
    }
    if fw == Framework::Dioxus {
        if items.iter().any(|i| i.slug == "theme") || cfg.installed.iter().any(|s| s == "theme") {
            out.push(format!("load the tokens once: document::Stylesheet {{ href: asset!(\"/{}/oxyn-tokens.css\") }}", cfg.css));
        }
        for c in css {
            out.push(format!(
                "put the component styles in the page once: style {{ {{{module}::{c}}} }}"
            ));
        }
        out.push("dark theme: set data-oxyn-theme=\"dark\" on an ancestor element".into());
    }
    out
}

/// One file's difference between the project and the registry.
#[derive(Debug, PartialEq, Eq)]
pub enum FileDiff {
    /// Same content.
    Same(String),
    /// Not in the project.
    Missing(String),
    /// Different: a unified diff.
    Changed(String, String),
}

/// `oxyn diff <slug>`: the item's files as the CLI would write them now (with everything installed)
/// against the project.
pub fn diff(
    project: &Path,
    cfg: &Config,
    client: &Client,
    index: &RegistryIndex,
    slug: &str,
) -> Result<Vec<FileDiff>, Error> {
    let fw = cfg.framework;
    // Pro items compare with the version that was installed (kept in .oxyn/pro/), so a diff never
    // counts toward the month's Pro items.
    prime_pro(project, client, index, &cfg.installed);
    let wanted = crate::resolve::resolve(client, index, &[slug.to_string()], fw)?;
    let mut all = wanted.clone();
    for s in &cfg.installed {
        if all.iter().any(|i| &i.slug == s) {
            continue;
        }
        if let Some(it) = client.item(s)? {
            all.push(it);
        }
    }
    let planned = compose(&all, fw, cfg, client)?;
    let item = wanted.iter().find(|i| i.slug == slug).expect("resolved");
    let mine: BTreeSet<String> = item
        .build(fw)
        .map(|b| {
            b.files
                .iter()
                .map(|f| {
                    format!(
                        "{}/{}",
                        if f.kind == FileKind::Css {
                            &cfg.css
                        } else {
                            &cfg.path
                        },
                        f.target
                    )
                })
                .collect()
        })
        .unwrap_or_default();
    let mut out = Vec::new();
    for p in planned.iter().filter(|p| mine.contains(&p.path)) {
        let abs: PathBuf = project.join(&p.path);
        match fs::read_to_string(&abs) {
            Err(_) => out.push(FileDiff::Missing(p.path.clone())),
            Ok(cur) if cur == p.content => out.push(FileDiff::Same(p.path.clone())),
            Ok(cur) => {
                let d = similar::TextDiff::from_lines(&cur, &p.content);
                let u = d
                    .unified_diff()
                    .context_radius(3)
                    .header(
                        &format!("{} (local)", p.path),
                        &format!("{} (registry)", p.path),
                    )
                    .to_string();
                out.push(FileDiff::Changed(p.path.clone(), u));
            }
        }
    }
    Ok(out)
}

fn rel(project: &Path, p: &Path) -> String {
    p.strip_prefix(project)
        .unwrap_or(p)
        .to_string_lossy()
        .replace('\\', "/")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::fake_registry;

    fn project(fw_dep: &str) -> tempfile::TempDir {
        let d = tempfile::tempdir().unwrap();
        fs::write(d.path().join("Cargo.toml"), format!("[package]\nname = \"app\"\nversion = \"0.1.0\"\nedition = \"2024\"\n\n[dependencies]\n{fw_dep}\n")).unwrap();
        fs::create_dir_all(d.path().join("src")).unwrap();
        fs::write(
            d.path().join("src/main.rs"),
            "//! App.\nuse dioxus::prelude::*;\nfn main() {}\n",
        )
        .unwrap();
        d
    }

    fn src() -> Source {
        Source {
            git: Some("https://github.com/ismailqureshi72/Oxyn".into()),
            rev: Some("a".repeat(40)),
            path: None,
        }
    }

    #[test]
    fn detects_frameworks() {
        assert_eq!(
            detect_frameworks("[dependencies]\ndioxus = \"0.7\"\n"),
            vec![Framework::Dioxus]
        );
        assert_eq!(
            detect_frameworks("[dependencies]\ngpui = { git = \"x\" }\n"),
            vec![Framework::Gpui]
        );
        assert!(detect_frameworks("[package]\n").is_empty());
    }

    #[test]
    fn init_writes_config_module_and_dependency() {
        let d = project("dioxus = { version = \"0.7\", features = [\"web\"] }");
        let rep = init(d.path(), None, None, src(), false).unwrap();
        assert_eq!(rep.framework, Some(Framework::Dioxus));
        let cfg = Config::load(d.path()).unwrap();
        assert_eq!(cfg.path, "src/oxyn");
        let main = fs::read_to_string(d.path().join("src/main.rs")).unwrap();
        assert!(
            main.starts_with("//! App.\nmod oxyn;\nuse dioxus"),
            "{main}"
        );
        let cargo = fs::read_to_string(d.path().join("Cargo.toml")).unwrap();
        assert!(
            cargo.contains(
                "oxyn-core = { git = \"https://github.com/ismailqureshi72/Oxyn\", rev = \"aaaa"
            ),
            "{cargo}"
        );
        assert!(
            cargo.contains("features = [\"web\"]"),
            "keeps existing deps: {cargo}"
        );
        assert!(d.path().join("src/oxyn/mod.rs").exists());
        // Twice: refused without --force; the declaration is not doubled with it.
        assert!(init(d.path(), None, None, src(), false).is_err());
        init(d.path(), None, None, src(), true).unwrap();
        assert_eq!(
            fs::read_to_string(d.path().join("src/main.rs"))
                .unwrap()
                .matches("mod oxyn;")
                .count(),
            1
        );
    }

    #[test]
    fn init_needs_a_framework() {
        let d = project("");
        assert!(
            matches!(init(d.path(), None, None, src(), false), Err(Error::Invalid(m)) if m.contains("--framework"))
        );
        let none = tempfile::tempdir().unwrap();
        assert!(matches!(
            init(none.path(), None, None, src(), false),
            Err(Error::NoCargoProject(_))
        ));
        let p = project("");
        let local = Source {
            git: None,
            rev: None,
            path: Some("/repo/oxyn".into()),
        };
        init(
            p.path(),
            Some(Framework::Gpui),
            Some("src/ui"),
            local,
            false,
        )
        .unwrap();
        let cargo = fs::read_to_string(p.path().join("Cargo.toml")).unwrap();
        assert!(cargo.contains("oxyn-core = { path = \"/repo/oxyn/crates/oxyn-core\" }"));
        assert!(
            fs::read_to_string(p.path().join("src/main.rs"))
                .unwrap()
                .contains("mod ui;")
        );
    }

    #[test]
    fn rewrites_crate_paths_only_at_boundaries() {
        let s = "use crate::brand::X;\npub(crate) fn f() { crate::color_var(); $crate::m!(); my_crate::y(); }";
        let r = rewrite_crate_paths(s, "crate::oxyn");
        assert!(r.contains("use crate::oxyn::brand::X;"));
        assert!(r.contains("crate::oxyn::color_var()"));
        assert!(r.contains("pub(crate) fn"));
        assert!(r.contains("$crate::m!"));
        assert!(r.contains("my_crate::y"));
    }

    #[test]
    fn add_writes_merges_and_respects_local_edits() {
        let (_r, client) = fake_registry();
        let idx = client.index().unwrap();
        let d = project("dioxus = \"0.7\"");
        init(d.path(), None, None, src(), false).unwrap();
        let mut cfg = Config::load(d.path()).unwrap();
        let rep = add(
            d.path(),
            &mut cfg,
            &client,
            &idx,
            &["send-button".into()],
            false,
        )
        .unwrap();
        assert_eq!(
            rep.items,
            vec!["theme".to_string(), "send-button".to_string()]
        );
        let root = fs::read_to_string(d.path().join("src/oxyn/mod.rs")).unwrap();
        assert!(root.starts_with(ROOT_HEADER));
        assert!(
            root.contains("mod composer;\npub use composer::SendButton;"),
            "{root}"
        );
        assert!(!root.contains("mod prem;"), "not installed: {root}");
        let send = fs::read_to_string(d.path().join("src/oxyn/composer/send.rs")).unwrap();
        assert!(send.contains("use crate::oxyn::brand::BrandIcon;"));
        assert_eq!(
            fs::read_to_string(d.path().join("assets/oxyn/oxyn-tokens.css")).unwrap(),
            ":root{}"
        );
        assert_eq!(
            fs::read_to_string(d.path().join("src/oxyn/composer/composer.css")).unwrap(),
            ".x{}"
        );
        assert!(
            Config::load(d.path())
                .unwrap()
                .installed
                .contains(&"send-button".to_string())
        );
        assert!(
            rep.next
                .iter()
                .any(|l| l.contains("use crate::oxyn::SendButton;"))
        );
        // A second item merges into the same files.
        let mut cfg = Config::load(d.path()).unwrap();
        add(
            d.path(),
            &mut cfg,
            &client,
            &idx,
            &["composer".into()],
            false,
        )
        .unwrap();
        let comp = fs::read_to_string(d.path().join("src/oxyn/composer/mod.rs")).unwrap();
        assert!(comp.contains("pub fn Composer"));
        assert!(comp.contains("mod send;"), "send stays declared: {comp}");
        let root = fs::read_to_string(d.path().join("src/oxyn/mod.rs")).unwrap();
        assert!(root.contains("pub use composer::Composer;"));
        // A local edit is kept, unless --overwrite.
        let send_path = d.path().join("src/oxyn/composer/send.rs");
        fs::write(&send_path, "// mine\n").unwrap();
        let mut cfg = Config::load(d.path()).unwrap();
        let rep = add(
            d.path(),
            &mut cfg,
            &client,
            &idx,
            &["send-button".into()],
            false,
        )
        .unwrap();
        assert!(
            rep.files
                .skipped
                .contains(&"src/oxyn/composer/send.rs".to_string())
        );
        assert_eq!(fs::read_to_string(&send_path).unwrap(), "// mine\n");
        let diffs = diff(d.path(), &cfg, &client, &idx, "send-button").unwrap();
        assert!(diffs.iter().any(|f| matches!(f, FileDiff::Changed(p, u) if p.ends_with("send.rs") && u.contains("-// mine"))));
        let rep = add(
            d.path(),
            &mut cfg,
            &client,
            &idx,
            &["send-button".into()],
            true,
        )
        .unwrap();
        assert!(
            rep.files
                .written
                .contains(&"src/oxyn/composer/send.rs".to_string())
        );
        assert!(
            diff(d.path(), &cfg, &client, &idx, "send-button")
                .unwrap()
                .iter()
                .all(|f| matches!(f, FileDiff::Same(_)))
        );
    }

    #[test]
    fn add_refuses_pro_items_without_writing() {
        let (_r, client) = fake_registry();
        let idx = client.index().unwrap();
        let d = project("dioxus = \"0.7\"");
        init(d.path(), None, None, src(), false).unwrap();
        let mut cfg = Config::load(d.path()).unwrap();
        let before = fs::read_to_string(d.path().join("src/oxyn/mod.rs")).unwrap();
        assert!(matches!(
            add(
                d.path(),
                &mut cfg,
                &client,
                &idx,
                &["fancy-screen".into()],
                false
            ),
            Err(Error::ProRequired(_))
        ));
        assert_eq!(
            fs::read_to_string(d.path().join("src/oxyn/mod.rs")).unwrap(),
            before
        );
    }
}
