//! Builds the registry from the renderer crates: one item per catalogue entry, each with the files
//! it needs per framework, its Oxyn and Cargo dependencies, and a content hash.
//!
//! An item is the exact set of top-level Rust items its entry reaches (functions, types, consts,
//! the `impl`s of those types, macros), walked through names, `use`s and re-exports, plus the
//! files they include. Each file it touches is shipped as chunks: the items it needs, and the
//! file's `use` lines and `mod` declarations as conditional chunks the CLI writes only when what
//! they name is installed. So a file shared with other components carries only this item's part.
//!
//! Pro never leaks: a free item whose closure reaches the entry of a Pro item is listed but not
//! installable, and no `<slug>.json` is written for it or for any Pro item. Stylesheets are not
//! gated: the site already serves every component's CSS for its previews.

use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::fs;
use std::path::{Path, PathBuf};

use oxyn_registry_schema::{
    CargoDep, Chunk, FileKind, Framework, FrameworkBuild, IndexEntry, Kind, OxynSource,
    RegistryFile, RegistryIndex, RegistryItem, SCHEMA_VERSION, Tier, valid_slug,
};
use serde::Deserialize;

use super::graph::{Graph, ModPath, Node, Target};
use super::scan::ItemKind;

/// One catalogue entry, as the site's generator writes it (`site/src/generated/registry-input.json`).
#[derive(Debug, Clone, Deserialize)]
pub struct MetaItem {
    /// URL-safe name.
    pub slug: String,
    /// Display name.
    pub name: String,
    /// What it is.
    pub kind: Kind,
    /// Site category.
    pub category: String,
    /// Free or Pro (the site's rule: screens, agent apps, blocks and the AI special components are Pro).
    pub tier: Tier,
    /// One-line summary.
    #[serde(default)]
    pub description: String,
    /// Page URL.
    #[serde(default)]
    pub url: String,
    /// Dioxus entry (`Composer`).
    #[serde(default)]
    pub dioxus: Option<String>,
    /// GPUI entry (`ComposerEl`).
    #[serde(default)]
    pub gpui: Option<String>,
}

/// Where to read from and what to stamp.
#[derive(Debug, Clone)]
pub struct BuildConfig {
    /// The Oxyn repository root.
    pub repo_root: PathBuf,
    /// Site origin, for the index (`https://ui.oxyn.workers.dev`).
    pub homepage: String,
    /// Repository URL for git dependencies.
    pub git: String,
    /// Commit the registry is built from.
    pub rev: String,
}

/// The built registry.
#[derive(Debug, Clone)]
pub struct Registry {
    /// `index.json`.
    pub index: RegistryIndex,
    /// Every item with files (free and Pro). Only installable free ones are ever written.
    pub items: Vec<RegistryItem>,
    /// Human-readable notes (skipped entries, unresolved names).
    pub report: Vec<String>,
    /// The repository it was built from (assets are copied from here).
    pub root: PathBuf,
}

/// The stylesheet item every Dioxus item depends on.
pub const THEME_SLUG: &str = "theme";

const CRATES: [(Framework, &str); 2] = [
    (Framework::Dioxus, "crates/oxyn-dioxus"),
    (Framework::Gpui, "crates/oxyn-gpui"),
];

/// FNV-1a 64-bit, hex: a stable content hash for change detection (not a security boundary).
pub fn content_hash(parts: &[&str]) -> String {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for p in parts {
        for b in p.bytes().chain([0xff]) {
            h ^= u64::from(b);
            h = h.wrapping_mul(0x0100_0000_01b3);
        }
    }
    format!("{h:016x}")
}

/// One framework's view of the catalogue.
struct FwBuild {
    builds: BTreeMap<String, FrameworkBuild>,
    /// slug → why it can't be installed.
    blocked: BTreeMap<String, String>,
}

/// Builds the registry.
pub fn build(meta: &[MetaItem], cfg: &BuildConfig) -> Result<Registry, String> {
    let mut report = Vec::new();
    for m in meta {
        if !valid_slug(&m.slug) {
            return Err(format!("invalid slug `{}`", m.slug));
        }
    }
    let ws = read_workspace_deps(&cfg.repo_root)?;
    let mut per_fw = BTreeMap::new();
    for (fw, dir) in CRATES {
        let b = build_framework(fw, &cfg.repo_root.join(dir), dir, meta, &ws, &mut report)?;
        per_fw.insert(fw, b);
    }
    let mut items = Vec::new();
    // The theme (design tokens stylesheet).
    let tokens_css = fs::read_to_string(cfg.repo_root.join("tokens/dist/oxyn.css"))
        .map_err(|e| format!("tokens/dist/oxyn.css: {e}"))?;
    let theme = RegistryItem {
        schema: SCHEMA_VERSION,
        slug: THEME_SLUG.into(),
        name: "Theme".into(),
        kind: Kind::Style,
        category: "theme".into(),
        tier: Tier::Free,
        description: "The Oxyn design tokens as CSS custom properties (--oxyn-*): load it once in every Dioxus app.".into(),
        url: format!("{}/docs/cli/", cfg.homepage),
        hash: content_hash(&[&tokens_css]),
        frameworks: vec![FrameworkBuild {
            framework: Framework::Dioxus,
            entry: String::new(),
            module: String::new(),
            oxyn_deps: vec![],
            cargo_deps: vec![],
            css: vec![],
            files: vec![RegistryFile {
                path: "tokens/dist/oxyn.css".into(),
                target: "oxyn-tokens.css".into(),
                kind: FileKind::Css,
                chunks: vec![],
                content: tokens_css,
                url: None,
            }],
        }],
    };
    items.push(theme);
    for m in meta {
        let mut frameworks = Vec::new();
        let mut notes = Vec::new();
        for fw in Framework::ALL {
            let fb = &per_fw[&fw];
            if let Some(b) = fb.builds.get(&m.slug) {
                frameworks.push(b.clone());
            }
            if let Some(why) = fb.blocked.get(&m.slug) {
                notes.push(format!("{fw}: {why}"));
            }
        }
        if frameworks.is_empty() {
            // Still listed (Pro screens made by macros, for example), just never installable.
            report.push(format!("{}: no entry found in either renderer", m.slug));
        }
        let mut parts: Vec<&str> = Vec::new();
        for b in &frameworks {
            for f in &b.files {
                parts.push(&f.target);
                parts.push(&f.content);
                for c in &f.chunks {
                    parts.push(&c.code);
                }
            }
        }
        let hash = content_hash(&parts);
        items.push(RegistryItem {
            schema: SCHEMA_VERSION,
            slug: m.slug.clone(),
            name: m.name.clone(),
            kind: m.kind,
            category: m.category.clone(),
            tier: m.tier,
            description: m.description.clone(),
            url: m.url.clone(),
            hash,
            frameworks,
        });
        if m.tier == Tier::Free && !notes.is_empty() {
            report.push(format!(
                "{}: not installable ({})",
                m.slug,
                notes.join("; ")
            ));
        }
    }
    // Free items must not depend on a blocked or missing item.
    let mut installable: BTreeSet<String> = items
        .iter()
        .filter(|i| i.tier == Tier::Free)
        .filter(|i| {
            Framework::ALL
                .iter()
                .all(|fw| !per_fw[fw].blocked.contains_key(&i.slug))
        })
        .map(|i| i.slug.clone())
        .collect();
    loop {
        let before = installable.len();
        let snapshot = installable.clone();
        installable.retain(|s| {
            let it = items.iter().find(|i| &i.slug == s).expect("item");
            it.frameworks
                .iter()
                .all(|b| b.oxyn_deps.iter().all(|d| snapshot.contains(d)))
        });
        if installable.len() == before {
            break;
        }
    }
    // Pro items (served by the authenticated Pro registry): installable when they have a build and
    // everything they need is installable, free or Pro.
    let mut pro_ok: BTreeSet<String> = items
        .iter()
        .filter(|i| i.tier == Tier::Pro && !i.frameworks.is_empty())
        .map(|i| i.slug.clone())
        .collect();
    loop {
        let before = pro_ok.len();
        let snapshot = pro_ok.clone();
        pro_ok.retain(|s| {
            let it = items.iter().find(|i| &i.slug == s).expect("item");
            it.frameworks.iter().all(|b| {
                b.oxyn_deps
                    .iter()
                    .all(|d| snapshot.contains(d) || installable.contains(d))
            })
        });
        if pro_ok.len() == before {
            break;
        }
    }
    installable.extend(pro_ok);
    let mut index_items = Vec::new();
    for it in &items {
        let note = if it.tier == Tier::Pro {
            Some(if installable.contains(&it.slug) {
                "Part of Oxyn Pro: `oxyn login`, then `oxyn add`.".to_string()
            } else {
                "Part of Oxyn Pro. Not installable with the CLI yet: use the code on its page."
                    .to_string()
            })
        } else if installable.contains(&it.slug) {
            None
        } else {
            let why: Vec<String> = Framework::ALL
                .iter()
                .filter_map(|fw| {
                    per_fw[fw]
                        .blocked
                        .get(&it.slug)
                        .map(|w| format!("{fw}: {w}"))
                })
                .collect();
            Some(if why.is_empty() {
                "Needs an item that is not installable.".into()
            } else {
                format!(
                    "Not installable from the free registry: {}.",
                    why.join("; ")
                )
            })
        };
        index_items.push(IndexEntry {
            slug: it.slug.clone(),
            name: it.name.clone(),
            kind: it.kind,
            category: it.category.clone(),
            tier: it.tier,
            description: it.description.clone(),
            url: it.url.clone(),
            frameworks: it.framework_list(),
            oxyn_deps: it
                .frameworks
                .iter()
                .map(|b| (b.framework, b.oxyn_deps.clone()))
                .collect(),
            hash: it.hash.clone(),
            installable: installable.contains(&it.slug),
            note,
        });
    }
    index_items.sort_by(|a, b| a.slug.cmp(&b.slug));
    let index = RegistryIndex {
        schema: SCHEMA_VERSION,
        name: "oxyn".into(),
        homepage: cfg.homepage.clone(),
        pricing: format!("{}/pricing/", cfg.homepage),
        source: OxynSource {
            git: cfg.git.clone(),
            rev: cfg.rev.clone(),
        },
        items: index_items,
    };
    Ok(Registry {
        index,
        items,
        report,
        root: cfg.repo_root.clone(),
    })
}

/// Writes `index.json` and one `<slug>.json` per installable free item into `out`, removing any
/// JSON left from an earlier build first. Never writes a Pro item.
pub fn write(reg: &Registry, out: &Path) -> Result<(usize, usize), String> {
    fs::create_dir_all(out).map_err(|e| format!("{}: {e}", out.display()))?;
    for e in fs::read_dir(out).map_err(|e| e.to_string())?.flatten() {
        if e.path().extension().is_some_and(|x| x == "json") {
            fs::remove_file(e.path()).map_err(|e| e.to_string())?;
        }
    }
    let idx = serde_json::to_string(&reg.index).map_err(|e| e.to_string())?;
    fs::write(out.join("index.json"), idx).map_err(|e| e.to_string())?;
    let files_dir = out.join("files");
    if files_dir.exists() {
        fs::remove_dir_all(&files_dir).map_err(|e| e.to_string())?;
    }
    let mut n = 0;
    let mut bytes = 0;
    let mut stored: BTreeSet<String> = BTreeSet::new();
    for it in &reg.items {
        let entry = reg.index.get(&it.slug).expect("indexed");
        if it.tier != Tier::Free || !entry.installable {
            continue;
        }
        // Shared assets of installable items, once each.
        for b in &it.frameworks {
            for f in &b.files {
                let Some(url) = &f.url else { continue };
                if !stored.insert(url.clone()) {
                    continue;
                }
                let crate_dir = if b.framework == Framework::Dioxus {
                    "crates/oxyn-dioxus"
                } else {
                    "crates/oxyn-gpui"
                };
                let src = reg.root.join(crate_dir).join("src").join(&f.target);
                let dst = out.join(url);
                fs::create_dir_all(dst.parent().expect("has parent")).map_err(|e| e.to_string())?;
                let content = fs::read(&src).map_err(|e| format!("{}: {e}", src.display()))?;
                bytes += content.len();
                fs::write(&dst, content).map_err(|e| e.to_string())?;
            }
        }
        let s = serde_json::to_string(it).map_err(|e| e.to_string())?;
        bytes += s.len();
        fs::write(out.join(format!("{}.json", it.slug)), s).map_err(|e| e.to_string())?;
        n += 1;
    }
    Ok((n, bytes))
}

/// Writes `<slug>.json` for every installable Pro item into `out` (the Worker-only Pro registry,
/// never a public directory), removing JSON left from an earlier build first. Their stylesheets are
/// public like every other component's: [`write`] copies shared assets of free items, and this
/// copies the ones only Pro items use into `public_out` (the public registry's `files/`).
pub fn write_pro(reg: &Registry, out: &Path, public_out: &Path) -> Result<(usize, usize), String> {
    fs::create_dir_all(out).map_err(|e| format!("{}: {e}", out.display()))?;
    for e in fs::read_dir(out).map_err(|e| e.to_string())?.flatten() {
        if e.path().extension().is_some_and(|x| x == "json") {
            fs::remove_file(e.path()).map_err(|e| e.to_string())?;
        }
    }
    let mut n = 0;
    let mut bytes = 0;
    for it in &reg.items {
        let entry = reg.index.get(&it.slug).expect("indexed");
        if it.tier != Tier::Pro || !entry.installable {
            continue;
        }
        for b in &it.frameworks {
            for f in &b.files {
                let Some(url) = &f.url else { continue };
                let dst = public_out.join(url);
                if dst.exists() {
                    continue;
                }
                let crate_dir = if b.framework == Framework::Dioxus {
                    "crates/oxyn-dioxus"
                } else {
                    "crates/oxyn-gpui"
                };
                let src = reg.root.join(crate_dir).join("src").join(&f.target);
                fs::create_dir_all(dst.parent().expect("has parent")).map_err(|e| e.to_string())?;
                let content = fs::read(&src).map_err(|e| format!("{}: {e}", src.display()))?;
                fs::write(&dst, content).map_err(|e| e.to_string())?;
            }
        }
        let s = serde_json::to_string(it).map_err(|e| e.to_string())?;
        bytes += s.len();
        fs::write(out.join(format!("{}.json", it.slug)), s).map_err(|e| e.to_string())?;
        n += 1;
    }
    Ok((n, bytes))
}

/// `[workspace.dependencies]` of the repo, for git / version specs of renderer dependencies.
fn read_workspace_deps(root: &Path) -> Result<toml::Table, String> {
    let s = fs::read_to_string(root.join("Cargo.toml")).map_err(|e| format!("Cargo.toml: {e}"))?;
    let t: toml::Table = s.parse().map_err(|e| format!("Cargo.toml: {e}"))?;
    Ok(t.get("workspace")
        .and_then(|w| w.get("dependencies"))
        .and_then(|d| d.as_table())
        .cloned()
        .unwrap_or_default())
}

/// The Cargo dependency for an extern crate name used by a renderer, from its `Cargo.toml`.
fn cargo_dep(crate_toml: &toml::Table, ws: &toml::Table, ident: &str) -> Option<CargoDep> {
    let deps = crate_toml.get("dependencies")?.as_table()?;
    let (name, spec) = deps.iter().find(|(k, _)| k.replace('-', "_") == ident)?;
    let mut dep = CargoDep {
        name: name.clone(),
        version: None,
        git: None,
        rev: None,
        oxyn: false,
        no_default_features: false,
        features: vec![],
    };
    let mut fill = |t: &toml::Value| {
        if let Some(v) = t.as_str() {
            dep.version = Some(v.to_string());
            return;
        }
        if let Some(v) = t.get("version").and_then(|v| v.as_str()) {
            dep.version = Some(v.to_string());
        }
        if let Some(v) = t.get("git").and_then(|v| v.as_str()) {
            dep.git = Some(v.to_string());
        }
        if let Some(v) = t.get("rev").and_then(|v| v.as_str()) {
            dep.rev = Some(v.to_string());
        }
        if t.get("path").is_some() {
            dep.oxyn = true;
        }
        if t.get("default-features").and_then(|v| v.as_bool()) == Some(false) {
            dep.no_default_features = true;
        }
        if let Some(f) = t.get("features").and_then(|v| v.as_array()) {
            dep.features
                .extend(f.iter().filter_map(|x| x.as_str().map(String::from)));
        }
    };
    if spec.get("workspace").and_then(|v| v.as_bool()) == Some(true) {
        fill(ws.get(name)?);
    }
    fill(spec);
    if dep.oxyn {
        dep.version = None;
        dep.git = None;
        dep.rev = None;
    }
    // A Dioxus app picks its own Dioxus features (web, desktop, …); ask only for the version.
    if name == "dioxus" {
        dep.no_default_features = false;
        dep.features.clear();
    }
    Some(dep)
}

fn target_of(file: &str) -> String {
    if file == "lib.rs" {
        "mod.rs".into()
    } else {
        file.to_string()
    }
}

/// Chunk id of the leading `//!` docs of a file.
const HEAD: u32 = 0;
/// Chunk id of item `i` (`use` bindings take the ids after it).
fn chunk_id(i: usize) -> u32 {
    (i as u32 + 1) * 100
}

/// What one item of the catalogue reaches.
struct Closure {
    nodes: BTreeSet<Node>,
    modules: BTreeSet<ModPath>,
    assets: BTreeSet<String>,
}

fn closure(g: &Graph, start: &[Node]) -> Closure {
    let mut nodes: BTreeSet<Node> = BTreeSet::new();
    let mut modules: BTreeSet<ModPath> = BTreeSet::new();
    let mut q: VecDeque<Node> = start.iter().cloned().collect();
    loop {
        while let Some(n) = q.pop_front() {
            if !nodes.insert(n.clone()) {
                continue;
            }
            modules.insert(n.0.clone());
            for t in g.item_refs(&n.0, n.1) {
                match t {
                    Target::Item(m, i) => {
                        if !nodes.contains(&(m.clone(), i)) {
                            q.push_back((m, i));
                        }
                    }
                    Target::Module(m) => {
                        modules.insert(m);
                    }
                }
            }
        }
        // impl blocks of what is included.
        let mut more = Vec::new();
        for mp in &modules {
            let m = &g.modules[mp];
            for i in 0..m.items.len() {
                if m.items[i].test_only || nodes.contains(&(mp.clone(), i)) {
                    continue;
                }
                if let Some(h) = g.impl_header_refs(mp, i)
                    && !h.is_empty()
                    && h.iter().all(|n| nodes.contains(n))
                {
                    more.push((mp.clone(), i));
                }
            }
        }
        if more.is_empty() {
            break;
        }
        q.extend(more);
    }
    // Every ancestor of a present module is present (it declares the child).
    let mut all = BTreeSet::new();
    for m in &modules {
        let mut p = m.clone();
        loop {
            all.insert(p.clone());
            match Graph::parent(&p) {
                Some(pp) => p = pp,
                None => break,
            }
        }
    }
    let assets = nodes
        .iter()
        .flat_map(|(m, i)| g.modules[m].item_includes[*i].iter().cloned())
        .collect();
    Closure {
        nodes,
        modules: all,
        assets,
    }
}

fn stem_css(file: &str) -> String {
    let stem = file
        .strip_suffix("/mod.rs")
        .or_else(|| file.strip_suffix(".rs"))
        .unwrap_or(file);
    format!("{stem}.css")
}

fn build_framework(
    fw: Framework,
    crate_dir: &Path,
    crate_rel: &str,
    meta: &[MetaItem],
    ws: &toml::Table,
    report: &mut Vec<String>,
) -> Result<FwBuild, String> {
    let g = Graph::load(&crate_dir.join("src"))?;
    let crate_toml: toml::Table = fs::read_to_string(crate_dir.join("Cargo.toml"))
        .map_err(|e| e.to_string())?
        .parse()
        .map_err(|e| format!("{crate_rel}/Cargo.toml: {e}"))?;
    let entry_of = |m: &MetaItem| -> Option<String> {
        match fw {
            Framework::Dioxus => m.dioxus.clone(),
            Framework::Gpui => m.gpui.clone(),
        }
        .filter(|e| !e.is_empty())
    };
    // An entry can name several parts (`AttachmentRow, DropZone`); the first is the main one.
    let mut root: BTreeMap<String, Node> = BTreeMap::new();
    let mut extra_roots: BTreeMap<String, Vec<Node>> = BTreeMap::new();
    for m in meta {
        let Some(entry) = entry_of(m) else { continue };
        let names: Vec<&str> = entry
            .split(',')
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .collect();
        // The gallery names some GPUI entries `<Name>El` where the type is `<Name>` or `<Name>View`.
        let find = |n: &str| {
            g.find_def(n).or_else(|| {
                let base = n.strip_suffix("El")?;
                g.find_def(base)
                    .or_else(|| g.find_def(&format!("{base}View")))
            })
        };
        let found: Vec<Option<Node>> = names.iter().map(|n| find(n)).collect();
        if found.iter().any(Option::is_none) {
            report.push(format!("{}: {fw} entry `{entry}` not found", m.slug));
            continue;
        }
        let mut found = found.into_iter().flatten();
        if let Some(first) = found.next() {
            root.insert(m.slug.clone(), first);
            extra_roots.insert(m.slug.clone(), found.collect());
        }
    }
    // Entry node → the items it is the entry of.
    let mut owners: BTreeMap<Node, Vec<&MetaItem>> = BTreeMap::new();
    for m in meta {
        if let Some(n) = root.get(&m.slug) {
            owners.entry(n.clone()).or_default().push(m);
        }
    }
    // *_CSS constants and the stylesheets each includes.
    let mut css_consts: Vec<(Node, String, Vec<String>)> = Vec::new();
    for m in g.modules.values() {
        for (i, it) in m.items.iter().enumerate() {
            if let ItemKind::Def(names) = &it.kind
                && !it.test_only
                && names.len() == 1
                && names[0].ends_with("_CSS")
            {
                css_consts.push((
                    (m.path.clone(), i),
                    names[0].clone(),
                    m.item_includes[i].clone(),
                ));
            }
        }
    }
    let mut builds = BTreeMap::new();
    let mut blocked = BTreeMap::new();
    for m in meta {
        let Some(r) = root.get(&m.slug).cloned() else {
            continue;
        };
        let mut start = vec![r.clone()];
        start.extend(extra_roots.get(&m.slug).cloned().unwrap_or_default());
        let mut css = Vec::new();
        if fw == Framework::Dioxus {
            let own = stem_css(&g.modules[&r.0].file);
            if let Some((n, name, _)) = css_consts.iter().find(|(_, _, inc)| inc.contains(&own)) {
                css.push(name.clone());
                start.push(n.clone());
            } else {
                let mut p = r.0.clone();
                loop {
                    if let Some((n, name, _)) = css_consts.iter().find(|((pp, _), _, _)| *pp == p) {
                        css.push(name.clone());
                        start.push(n.clone());
                        break;
                    }
                    if p.is_empty() {
                        break;
                    }
                    p.pop();
                }
            }
        }
        let c = closure(&g, &start);
        // Other catalogue items this one renders.
        let mut deps: BTreeSet<String> = BTreeSet::new();
        let mut pro_hit: Option<String> = None;
        for n in &c.nodes {
            if *n == r {
                continue;
            }
            if let Some(v) = owners.get(n) {
                for o in v {
                    if o.tier == Tier::Pro && m.tier == Tier::Free && pro_hit.is_none() {
                        pro_hit = Some(o.slug.clone());
                    }
                }
                let mut v: Vec<&&MetaItem> = v.iter().collect();
                v.sort_by_key(|o| (o.tier != Tier::Free, o.slug.clone()));
                if let Some(o) = v.first()
                    && o.slug != m.slug
                {
                    deps.insert(o.slug.clone());
                }
            }
        }
        if let Some(p) = pro_hit {
            blocked.insert(m.slug.clone(), format!("renders the Pro item `{p}`"));
        }
        // Files.
        let mut files = Vec::new();
        let mut externs: BTreeSet<String> = BTreeSet::new();
        for mp in &c.modules {
            let items: BTreeSet<usize> = c
                .nodes
                .iter()
                .filter(|(p, _)| p == mp)
                .map(|(_, i)| *i)
                .collect();
            externs.extend(g.externs(mp, &items));
            files.push(module_file(&g, mp, &items, crate_rel));
        }
        for a in &c.assets {
            if !crate_dir.join("src").join(a).is_file() {
                return Err(format!("{crate_rel}/src/{a}: included but missing"));
            }
            files.push(RegistryFile {
                path: format!("{crate_rel}/src/{a}"),
                target: a.clone(),
                kind: FileKind::Asset,
                chunks: vec![],
                content: String::new(),
                url: Some(format!("files/{fw}/{a}")),
            });
        }
        let mut cargo_deps = Vec::new();
        for e in &externs {
            match cargo_dep(&crate_toml, ws, e) {
                Some(d) => cargo_deps.push(d),
                None => report.push(format!(
                    "{}: {fw}: `{e}` looks like a crate but is not a dependency of {crate_rel}",
                    m.slug
                )),
            }
        }
        cargo_deps.sort();
        cargo_deps.dedup();
        let mut oxyn_deps: Vec<String> = deps.into_iter().collect();
        if fw == Framework::Dioxus {
            oxyn_deps.insert(0, THEME_SLUG.into());
        }
        builds.insert(
            m.slug.clone(),
            FrameworkBuild {
                framework: fw,
                entry: entry_of(m)
                    .unwrap_or_default()
                    .split(',')
                    .next()
                    .unwrap_or("")
                    .trim()
                    .to_string(),
                module: r.0.join("::"),
                oxyn_deps,
                cargo_deps,
                css,
                files,
            },
        );
    }
    Ok(FwBuild { builds, blocked })
}

/// The target of a module's file in the user's project (`lib.rs` becomes `mod.rs`).
fn module_target(g: &Graph, p: &ModPath) -> String {
    target_of(&g.modules[p].file)
}

/// One module as a registry file: the chunks for `items`, the leading docs, inner attributes, and
/// every `use` line and child declaration as conditional chunks.
fn module_file(g: &Graph, p: &ModPath, items: &BTreeSet<usize>, crate_rel: &str) -> RegistryFile {
    let m = &g.modules[p];
    let target = module_target(g, p);
    let mut chunks = Vec::new();
    for (i, it) in m.items.iter().enumerate() {
        if it.test_only {
            continue;
        }
        let mut text = m.text(i).to_string();
        // Leading `//!` docs belong to the file, not to the first item.
        if i == 0 {
            let mut head = String::new();
            let mut rest = text.as_str();
            while rest.starts_with("//!") {
                let nl = rest.find('\n').map_or(rest.len(), |n| n + 1);
                head.push_str(&rest[..nl]);
                rest = &rest[nl..];
            }
            if !head.is_empty() {
                chunks.push(Chunk {
                    id: HEAD,
                    code: head.trim_end().to_string(),
                    requires: None,
                });
                text = rest.trim_start().to_string();
            }
        }
        let id = chunk_id(i);
        match &it.kind {
            ItemKind::ModDecl(child) => {
                let mut cp = p.clone();
                cp.push(child.clone());
                let code: String = text
                    .lines()
                    .filter(|l| !l.trim_start().starts_with("//"))
                    .collect::<Vec<_>>()
                    .join("\n");
                chunks.push(Chunk {
                    id,
                    code: code.trim().to_string(),
                    requires: Some(format!("file:{}", module_target(g, &cp))),
                });
            }
            ItemKind::Use(bindings) => {
                let resolved: Vec<Option<Target>> = bindings
                    .iter()
                    .map(|u| g.resolve_path(p, &u.path))
                    .collect();
                let is_local_path = |u: &super::scan::UseBinding| {
                    matches!(u.path[0].as_str(), "crate" | "self" | "super")
                        || m.children.contains(&u.path[0])
                };
                let all_extern = bindings
                    .iter()
                    .zip(&resolved)
                    .all(|(u, r)| r.is_none() && !is_local_path(u));
                if all_extern {
                    chunks.push(Chunk {
                        id,
                        code: text.trim().to_string(),
                        requires: None,
                    });
                    continue;
                }
                let code = m.code_of(i);
                let use_at = code.find("use ").unwrap_or(0);
                let vis_src = &m.text(i)[..use_at];
                let vis = vis_src.lines().last().unwrap_or("").trim();
                let vis = if vis.is_empty() || vis.starts_with('#') {
                    String::new()
                } else {
                    format!("{vis} ")
                };
                for (k, (u, r)) in bindings.iter().zip(resolved).enumerate() {
                    let path: Vec<&str> = u
                        .path
                        .iter()
                        .map(String::as_str)
                        .filter(|s| *s != "self")
                        .collect();
                    let line = match &u.name {
                        Some(name) => {
                            let alias = if path.last() == Some(&name.as_str()) {
                                String::new()
                            } else {
                                format!(" as {name}")
                            };
                            format!("{vis}use {}{alias};", path.join("::"))
                        }
                        None => format!("{vis}use {}::*;", path.join("::")),
                    };
                    let requires = match r {
                        Some(Target::Item(mp, j)) => {
                            Some(format!("{}#{}", module_target(g, &mp), chunk_id(j)))
                        }
                        Some(Target::Module(mp)) => Some(format!("file:{}", module_target(g, &mp))),
                        None => None,
                    };
                    chunks.push(Chunk {
                        id: id + 1 + k as u32,
                        code: line,
                        requires,
                    });
                }
            }
            ItemKind::Other
                if !m.code_of(i).trim_start().contains("impl")
                    || m.code_of(i).trim_start().starts_with("#!") =>
            {
                // Inner attributes and other file-level lines.
                if m.code_of(i).trim_start().starts_with("#!") {
                    chunks.push(Chunk {
                        id,
                        code: text.trim().to_string(),
                        requires: None,
                    });
                }
            }
            _ => {
                if items.contains(&i) {
                    chunks.push(Chunk {
                        id,
                        code: text.trim_end().to_string(),
                        requires: None,
                    });
                }
            }
        }
    }
    RegistryFile {
        path: format!("{crate_rel}/src/{}", m.file),
        target,
        kind: FileKind::Rust,
        chunks,
        content: String::new(),
        url: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn meta(slug: &str, tier: Tier, dioxus: &str) -> MetaItem {
        MetaItem {
            slug: slug.into(),
            name: slug.into(),
            kind: Kind::Component,
            category: "chat".into(),
            tier,
            description: String::new(),
            url: String::new(),
            dioxus: Some(dioxus.into()),
            gpui: None,
        }
    }

    /// The fixture crate laid out as a repository (renderer crate, workspace manifest, tokens).
    fn repo() -> tempfile::TempDir {
        let src = super::super::graph::tests::fixture();
        let d = tempfile::tempdir().unwrap();
        let dx = d.path().join("crates/oxyn-dioxus");
        fs::create_dir_all(&dx).unwrap();
        copy_dir(src.path(), &dx.join("src"));
        fs::write(
            dx.join("Cargo.toml"),
            "[package]\nname = \"oxyn-dioxus\"\n[dependencies]\ndioxus = { version = \"0.7\", default-features = false, features = [\"macro\"] }\noxyn-core.workspace = true\n",
        )
        .unwrap();
        let gp = d.path().join("crates/oxyn-gpui/src");
        fs::create_dir_all(&gp).unwrap();
        fs::write(gp.join("lib.rs"), "//! GPUI.\n").unwrap();
        fs::write(
            d.path().join("crates/oxyn-gpui/Cargo.toml"),
            "[package]\nname = \"oxyn-gpui\"\n",
        )
        .unwrap();
        fs::write(
            d.path().join("Cargo.toml"),
            "[workspace.dependencies]\noxyn-core = { path = \"crates/oxyn-core\" }\n",
        )
        .unwrap();
        fs::create_dir_all(d.path().join("tokens/dist")).unwrap();
        fs::write(d.path().join("tokens/dist/oxyn.css"), ":root{}").unwrap();
        d
    }

    fn copy_dir(from: &Path, to: &Path) {
        fs::create_dir_all(to).unwrap();
        for e in fs::read_dir(from).unwrap().flatten() {
            let p = e.path();
            if p.is_dir() {
                copy_dir(&p, &to.join(e.file_name()));
            } else {
                fs::copy(&p, to.join(e.file_name())).unwrap();
            }
        }
    }

    fn cfg(root: &Path) -> BuildConfig {
        BuildConfig {
            repo_root: root.into(),
            homepage: "https://x.test".into(),
            git: "https://g.test/o".into(),
            rev: "a".repeat(40),
        }
    }

    #[test]
    fn items_carry_only_their_chunks_and_never_pro_code() {
        let d = repo();
        let meta = vec![
            meta("send-button", Tier::Free, "SendButton"),
            meta("composer", Tier::Free, "Composer"),
            meta("pro-picker", Tier::Pro, "ProPicker"),
            meta("fancy", Tier::Pro, "Fancy"),
        ];
        let reg = build(&meta, &cfg(d.path())).unwrap();
        let send = reg.items.iter().find(|i| i.slug == "send-button").unwrap();
        let b = send.build(Framework::Dioxus).unwrap();
        assert_eq!(b.oxyn_deps, vec!["theme".to_string()]);
        assert_eq!(b.css, vec!["COMPOSER_CSS".to_string()]);
        assert!(b.cargo_deps.iter().any(|c| c.name == "oxyn-core" && c.oxyn));
        assert!(
            b.cargo_deps
                .iter()
                .any(|c| c.name == "dioxus" && c.features.is_empty())
        );
        let all: String = b
            .files
            .iter()
            .flat_map(|f| f.chunks.iter().map(|c| c.code.clone()))
            .collect::<Vec<_>>()
            .join("\n");
        assert!(all.contains("pub fn SendButton"));
        assert!(all.contains("pub fn BrandIcon"));
        assert!(all.contains("pub fn color_var"));
        // Not reached: other components, unused helpers, Pro code, tests.
        assert!(!all.contains("fn ProPicker"));
        assert!(!all.contains("fn Fancy"));
        assert!(!all.contains("unused_helper"));
        assert!(!all.contains("mod tests"));
        // Child declarations and re-exports are conditional.
        let root = b.files.iter().find(|f| f.target == "mod.rs").unwrap();
        let decl = root
            .chunks
            .iter()
            .find(|c| c.code == "mod composer;")
            .unwrap();
        assert_eq!(decl.requires.as_deref(), Some("file:composer/mod.rs"));
        let reexport = root
            .chunks
            .iter()
            .find(|c| c.code == "pub use composer::SendButton;")
            .unwrap();
        assert!(
            reexport
                .requires
                .as_deref()
                .unwrap()
                .starts_with("composer/send.rs#")
        );
        assert_eq!(root.chunks[0].code, "//! Root.");
        // The stylesheet is shared, by URL.
        let css = b.files.iter().find(|f| f.kind == FileKind::Asset).unwrap();
        assert_eq!(
            css.url.as_deref(),
            Some("files/dioxus/composer/composer.css")
        );
        // The composer includes the impls of Icon.
        let comp = reg
            .items
            .iter()
            .find(|i| i.slug == "composer")
            .unwrap()
            .build(Framework::Dioxus)
            .unwrap();
        let icons = comp
            .files
            .iter()
            .find(|f| f.target == "composer/icons.rs")
            .unwrap();
        assert_eq!(icons.chunks.len(), 3);
    }

    #[test]
    fn index_lists_pro_and_write_skips_it() {
        let d = repo();
        let meta = vec![
            meta("send-button", Tier::Free, "SendButton"),
            meta("fancy", Tier::Pro, "Fancy"),
            meta("composer", Tier::Free, "Composer"),
        ];
        let reg = build(&meta, &cfg(d.path())).unwrap();
        assert_eq!(reg.index.items.len(), 4); // + theme
        let fancy = reg.index.get("fancy").unwrap();
        // Installable through the authenticated Pro registry, never written to the public one.
        assert!(fancy.installable);
        assert_eq!(fancy.tier, Tier::Pro);
        let out = d.path().join("out");
        fs::create_dir_all(&out).unwrap();
        fs::write(out.join("stale.json"), "{}").unwrap();
        let (n, _) = write(&reg, &out).unwrap();
        assert_eq!(n, 3);
        assert!(out.join("send-button.json").exists());
        assert!(out.join("theme.json").exists());
        assert!(!out.join("fancy.json").exists());
        assert!(!out.join("stale.json").exists());
        assert!(out.join("files/dioxus/composer/composer.css").exists());
        let pro = d.path().join("pro");
        let (np, _) = write_pro(&reg, &pro, &out).unwrap();
        assert_eq!(np, 1);
        assert!(pro.join("fancy.json").exists());
        assert!(!pro.join("send-button.json").exists());
        assert!(!out.join("fancy.json").exists());
    }

    #[test]
    fn a_free_item_that_renders_a_pro_item_is_not_installable() {
        let d = repo();
        let meta = vec![
            meta("composer", Tier::Free, "Composer"),
            meta("icon", Tier::Pro, "Icon"),
        ];
        let reg = build(&meta, &cfg(d.path())).unwrap();
        let c = reg.index.get("composer").unwrap();
        assert!(!c.installable);
        assert!(c.note.is_some());
        let out = d.path().join("out");
        write(&reg, &out).unwrap();
        assert!(!out.join("composer.json").exists());
    }

    #[test]
    fn hash_is_stable() {
        assert_eq!(content_hash(&["a", "b"]), content_hash(&["a", "b"]));
        assert_ne!(content_hash(&["ab"]), content_hash(&["a", "b"]));
    }
}
