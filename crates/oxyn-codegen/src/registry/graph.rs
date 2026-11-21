//! The item graph of one renderer crate (`crates/oxyn-dioxus/src` or `crates/oxyn-gpui/src`):
//! every file module, its top-level items, and what each item names. Names are resolved through
//! `use` trees, re-exports, globs and qualified paths, the way rustc resolves them, closely enough
//! for the code in this repository. The registry builder walks it to find the exact set of items
//! (not whole files) a component needs.

use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::fs;
use std::path::{Path, PathBuf};

use super::scan::{self, Item, ItemKind, UseBinding};

/// A module path from the crate root (`["composer", "send"]`); the root is `[]`.
pub type ModPath = Vec<String>;

/// One top-level item: its module and its index in that module's item list.
pub type Node = (ModPath, usize);

/// One file module.
#[derive(Debug, Clone)]
pub struct Module {
    /// Its path from the crate root.
    pub path: ModPath,
    /// Source file, relative to the crate's `src/`.
    pub file: String,
    /// Full source.
    pub src: String,
    /// Blanked source (same offsets).
    pub code: String,
    /// Top-level items.
    pub items: Vec<Item>,
    /// Names defined here → the item defining them.
    pub defs: BTreeMap<String, usize>,
    /// Child file modules (`mod x;`), in source order.
    pub children: Vec<String>,
    /// Paths and lone identifiers named by each item (empty for `use`s and test items).
    pub item_paths: Vec<Vec<Vec<String>>>,
    /// Files each item includes (`include_str!`), relative to `src/`.
    pub item_includes: Vec<Vec<String>>,
}

impl Module {
    /// Every live `use` binding with its item index.
    pub fn uses(&self) -> impl Iterator<Item = (usize, &UseBinding)> {
        self.items
            .iter()
            .enumerate()
            .filter(|(_, it)| !it.test_only)
            .flat_map(|(i, it)| match &it.kind {
                ItemKind::Use(b) => b.iter().map(move |u| (i, u)).collect::<Vec<_>>(),
                _ => Vec::new(),
            })
    }
    /// The source text of item `i`.
    pub fn text(&self, i: usize) -> &str {
        &self.src[self.items[i].start..self.items[i].end]
    }
    /// The blanked text of item `i`.
    pub fn code_of(&self, i: usize) -> &str {
        &self.code[self.items[i].start..self.items[i].end]
    }
}

/// What a name resolves to.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Target {
    /// A top-level item of a module.
    Item(ModPath, usize),
    /// A module.
    Module(ModPath),
}

/// The item graph of one crate.
#[derive(Debug)]
pub struct Graph {
    /// Every file module by path.
    pub modules: BTreeMap<ModPath, Module>,
    memo: RefCell<HashMap<(ModPath, String), Option<Target>>>,
}

/// Paths rooted in these are the standard library, not crates to depend on.
const STD: [&str; 3] = ["std", "core", "alloc"];

impl Graph {
    /// Loads a crate from its `src/` directory, starting at `lib.rs`.
    pub fn load(src_dir: &Path) -> Result<Graph, String> {
        let mut modules = BTreeMap::new();
        load_module(src_dir, Path::new("lib.rs"), Vec::new(), &mut modules)?;
        Ok(Graph {
            modules,
            memo: RefCell::new(HashMap::new()),
        })
    }

    /// The item that defines `name` at the top level (the shallowest module when several do).
    pub fn find_def(&self, name: &str) -> Option<Node> {
        let mut hits: Vec<(&ModPath, usize)> = self
            .modules
            .values()
            .filter_map(|m| m.defs.get(name).map(|&i| (&m.path, i)))
            .collect();
        hits.sort_by_key(|(p, _)| p.len());
        hits.first().map(|(p, i)| ((*p).clone(), *i))
    }

    /// The parent of a module.
    pub fn parent(p: &ModPath) -> Option<ModPath> {
        if p.is_empty() {
            None
        } else {
            Some(p[..p.len() - 1].to_vec())
        }
    }

    /// Resolves one name in a module's scope (its items, children, `use`s and globs).
    pub fn resolve_name(&self, m: &ModPath, name: &str) -> Option<Target> {
        let key = (m.clone(), name.to_string());
        if let Some(hit) = self.memo.borrow().get(&key) {
            return hit.clone();
        }
        // Cycle guard: a re-entrant lookup of the same key sees `None`.
        self.memo.borrow_mut().insert(key.clone(), None);
        let r = self.resolve_name_uncached(m, name);
        self.memo.borrow_mut().insert(key, r.clone());
        r
    }

    fn resolve_name_uncached(&self, m: &ModPath, name: &str) -> Option<Target> {
        let module = self.modules.get(m)?;
        if let Some(&i) = module.defs.get(name) {
            return Some(Target::Item(m.clone(), i));
        }
        if module.children.iter().any(|c| c == name) {
            let mut p = m.clone();
            p.push(name.to_string());
            return Some(Target::Module(p));
        }
        for (_, u) in module.uses() {
            if u.name.as_deref() == Some(name)
                && let Some(t) = self.resolve_path(m, &u.path)
            {
                return Some(t);
            }
        }
        for (_, u) in module.uses() {
            if u.name.is_none()
                && let Some(Target::Module(g)) = self.resolve_path(m, &u.path)
                && let Some(t) = self.resolve_name(&g, name)
            {
                return Some(t);
            }
        }
        None
    }

    /// Resolves a path (`crate::a::b`, `super::x`, `child::Y`) from module `m`. Stops at an item
    /// (`Enum::Variant` resolves to the enum). `None` for extern and std paths.
    pub fn resolve_path(&self, m: &ModPath, path: &[String]) -> Option<Target> {
        let first = path.first()?;
        let mut rest = &path[1..];
        let mut cur = match first.as_str() {
            "crate" => Target::Module(Vec::new()),
            "self" => Target::Module(m.clone()),
            "super" => {
                let mut p = Self::parent(m)?;
                while rest.first().is_some_and(|s| s == "super") {
                    p = Self::parent(&p)?;
                    rest = &rest[1..];
                }
                Target::Module(p)
            }
            "Self" => return None,
            name => self.resolve_name(m, name)?,
        };
        for seg in rest {
            match &cur {
                Target::Module(mp) => match self.resolve_name(mp, seg) {
                    Some(t) => cur = t,
                    None => return Some(cur),
                },
                Target::Item(..) => break,
            }
        }
        Some(cur)
    }

    /// What item `i` of module `p` names: the items and modules its paths resolve to.
    pub fn item_refs(&self, p: &ModPath, i: usize) -> BTreeSet<Target> {
        let m = &self.modules[p];
        let mut out = BTreeSet::new();
        for path in &m.item_paths[i] {
            let t = if path.len() == 1 {
                self.resolve_name(p, &path[0])
            } else {
                self.resolve_path(p, path)
            };
            if let Some(t) = t
                && t != Target::Item(p.clone(), i)
            {
                out.insert(t);
            }
        }
        out
    }

    /// For an `impl` block: the local items its header names (`impl Trait for Type`). The block is
    /// needed exactly when all of them are.
    pub fn impl_header_refs(&self, p: &ModPath, i: usize) -> Option<BTreeSet<Node>> {
        let m = &self.modules[p];
        let code = m.code_of(i);
        let at = code.find("impl")?;
        let rest = &code[at + 4..];
        if !(rest.starts_with([' ', '<', '\n', '\t'])) {
            return None;
        }
        let header = &rest[..rest.find('{').unwrap_or(rest.len())];
        let mut out = BTreeSet::new();
        for path in scan::paths(header) {
            let t = if path.len() == 1 {
                self.resolve_name(p, &path[0])
            } else {
                self.resolve_path(p, &path)
            };
            if let Some(Target::Item(mp, j)) = t {
                out.insert((mp, j));
            }
        }
        Some(out)
    }

    /// Extern crate names used by the given items of module `p` and its `use`s.
    pub fn externs(&self, p: &ModPath, items: &BTreeSet<usize>) -> BTreeSet<String> {
        let m = &self.modules[p];
        let mut firsts: Vec<&String> = Vec::new();
        for (i, u) in m.uses() {
            let _ = i;
            firsts.push(&u.path[0]);
        }
        for &i in items {
            for path in &m.item_paths[i] {
                if path.len() > 1 {
                    firsts.push(&path[0]);
                }
            }
        }
        let mut out = BTreeSet::new();
        for f in firsts {
            if matches!(f.as_str(), "crate" | "self" | "super" | "Self")
                || STD.contains(&f.as_str())
            {
                continue;
            }
            if !f.chars().next().is_some_and(|c| c.is_ascii_lowercase()) {
                continue;
            }
            if self.resolve_name(p, f).is_some() {
                continue;
            }
            if f.contains('_') || matches!(f.as_str(), "dioxus" | "gpui" | "serde" | "toml") {
                out.insert(f.clone());
            }
        }
        out
    }
}

/// Normalises `a/b/../c.css` to `a/c.css`.
pub fn normalize(p: &Path) -> String {
    let mut parts: Vec<String> = Vec::new();
    for c in p.components() {
        match c {
            std::path::Component::ParentDir => {
                parts.pop();
            }
            std::path::Component::Normal(s) => parts.push(s.to_string_lossy().into_owned()),
            _ => {}
        }
    }
    parts.join("/")
}

fn load_module(
    src_dir: &Path,
    rel: &Path,
    path: ModPath,
    out: &mut BTreeMap<ModPath, Module>,
) -> Result<(), String> {
    let full = src_dir.join(rel);
    let src = fs::read_to_string(&full).map_err(|e| format!("{}: {e}", full.display()))?;
    let code = scan::blank(&src);
    let items = scan::items(&src, &code);
    let mut defs = BTreeMap::new();
    let mut children = Vec::new();
    for (i, it) in items.iter().enumerate().filter(|(_, i)| !i.test_only) {
        match &it.kind {
            ItemKind::Def(names) => {
                for n in names {
                    defs.entry(n.clone()).or_insert(i);
                }
            }
            ItemKind::ModDecl(name) => children.push(name.clone()),
            ItemKind::ModInline(name) => {
                defs.entry(name.clone()).or_insert(i);
            }
            _ => {}
        }
    }
    let dir = rel.parent().unwrap_or(Path::new("")).to_path_buf();
    let mut item_paths = Vec::with_capacity(items.len());
    let mut item_includes = Vec::with_capacity(items.len());
    for it in &items {
        let skip = it.test_only || matches!(it.kind, ItemKind::Use(_) | ItemKind::ModDecl(_));
        if skip {
            item_paths.push(Vec::new());
            item_includes.push(Vec::new());
            continue;
        }
        item_paths.push(scan::paths(&code[it.start..it.end]));
        item_includes.push(
            scan::includes(&src[it.start..it.end], &code[it.start..it.end])
                .into_iter()
                .map(|f| normalize(&dir.join(f)))
                .collect(),
        );
    }
    let file = rel.to_string_lossy().replace('\\', "/");
    let stem = rel
        .file_stem()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_default();
    let child_dir: PathBuf = if stem == "lib" || stem == "mod" {
        dir.clone()
    } else {
        dir.join(&stem)
    };
    out.insert(
        path.clone(),
        Module {
            path: path.clone(),
            file,
            src,
            code,
            items,
            defs,
            children: children.clone(),
            item_paths,
            item_includes,
        },
    );
    for c in children {
        let flat = child_dir.join(format!("{c}.rs"));
        let nested = child_dir.join(&c).join("mod.rs");
        let rel_child = if src_dir.join(&flat).exists() {
            flat
        } else if src_dir.join(&nested).exists() {
            nested
        } else {
            return Err(format!(
                "module {c} declared in {} has no file",
                rel.display()
            ));
        };
        let mut p = path.clone();
        p.push(c);
        load_module(src_dir, &rel_child, p, out)?;
    }
    Ok(())
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    /// A tiny renderer crate: a root with helpers, a composer directory module with children, and
    /// a Pro module that shares a file with nothing free.
    pub(crate) fn fixture() -> tempfile::TempDir {
        let d = tempfile::tempdir().unwrap();
        let w = |p: &str, s: &str| {
            let f = d.path().join(p);
            fs::create_dir_all(f.parent().unwrap()).unwrap();
            fs::write(f, s).unwrap();
        };
        w(
            "lib.rs",
            "//! Root.\nmod brand;\nmod composer;\nmod prem;\npub use composer::{Composer, SendButton};\npub use prem::Fancy;\n/// Colour.\npub fn color_var() -> String { String::new() }\n",
        );
        w(
            "brand.rs",
            "pub fn BrandIcon() {}\npub fn unused_helper() {}\n",
        );
        w(
            "composer/mod.rs",
            "use dioxus::prelude::*;\nmod icons;\nmod send;\npub use send::SendButton;\npub(crate) use icons::Icon;\npub const COMPOSER_CSS: &str = include_str!(\"composer.css\");\n#[component]\npub fn Composer() -> Element { let _ = Icon::Send; rsx! {} }\n#[component]\npub fn ProPicker() -> Element { rsx! {} }\n#[cfg(test)]\nmod tests { use super::*; }\n",
        );
        w("composer/composer.css", ".x{}");
        w(
            "composer/icons.rs",
            "pub enum Icon { Send }\nimpl Icon { pub fn name(&self) -> &str { \"send\" } }\nimpl Default for Icon { fn default() -> Self { Icon::Send } }\n",
        );
        w(
            "composer/send.rs",
            "use crate::brand::BrandIcon;\nuse oxyn_core::ClockMs;\n#[component]\npub fn SendButton() -> Element { BrandIcon(); super::super::color_var(); rsx! {} }\n",
        );
        w(
            "prem.rs",
            "use crate::composer::Icon;\npub fn Fancy() { let _ = Icon::Send; }\n",
        );
        d
    }

    fn p(s: &str) -> ModPath {
        if s.is_empty() {
            vec![]
        } else {
            s.split("::").map(String::from).collect()
        }
    }

    #[test]
    fn resolves_items_through_uses_and_reexports() {
        let d = fixture();
        let g = Graph::load(d.path()).unwrap();
        assert_eq!(g.modules.len(), 6);
        let (m, i) = g.find_def("SendButton").unwrap();
        assert_eq!(m, p("composer::send"));
        assert_eq!(g.find_def("SendButtonProps"), Some((m.clone(), i)));
        let refs = g.item_refs(&m, i);
        let brand = g.find_def("BrandIcon").unwrap();
        let color = g.find_def("color_var").unwrap();
        assert!(refs.contains(&Target::Item(brand.0, brand.1)));
        assert!(refs.contains(&Target::Item(color.0, color.1)));
        // Through a parent's re-export of a child.
        let (pm, pi) = g.find_def("Fancy").unwrap();
        let icon = g.find_def("Icon").unwrap();
        assert!(
            g.item_refs(&pm, pi)
                .contains(&Target::Item(icon.0.clone(), icon.1))
        );
        // Impl headers name their type.
        let m = &g.modules[&p("composer::icons")];
        let impls: Vec<usize> = (0..m.items.len())
            .filter(|&i| g.impl_header_refs(&icon.0, i).is_some())
            .collect();
        assert_eq!(impls.len(), 2);
        for i in impls {
            assert!(g.impl_header_refs(&icon.0, i).unwrap().contains(&icon));
        }
        assert!(
            g.externs(&p("composer::send"), &BTreeSet::new())
                .contains("oxyn_core")
        );
    }
}
