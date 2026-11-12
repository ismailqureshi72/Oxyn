//! Writes `crates/oxyn-tokens/src/generated.rs`.

use std::{collections::BTreeMap, fmt::Write};

use serde_json::Value;

use crate::{contrast::hex_alpha, model::TokenSet};

const KEYWORDS: &[&str] = &[
    "as", "break", "const", "continue", "crate", "else", "enum", "extern", "false", "fn", "for",
    "if", "impl", "in", "let", "loop", "match", "mod", "move", "mut", "pub", "ref", "return",
    "self", "static", "struct", "super", "trait", "true", "type", "unsafe", "use", "where",
    "while", "async", "await", "dyn", "abstract", "become", "box", "do", "final", "macro",
    "override", "priv", "typeof", "unsized", "virtual", "yield", "try", "gen",
];

fn module_name(seg: &str) -> String {
    let mut s = seg.replace('-', "_");
    if s.starts_with(|c: char| c.is_ascii_digit()) {
        s = format!("n{s}");
    }
    if KEYWORDS.contains(&s.as_str()) {
        s.push('_');
    }
    s
}

fn const_name(seg: &str) -> String {
    let mut s = seg.replace('-', "_").to_ascii_uppercase();
    if s.starts_with(|c: char| c.is_ascii_digit()) {
        s = format!("N{s}");
    }
    s
}

fn camel(path: &str) -> String {
    path.split(['.', '-'])
        .map(|p| {
            let mut c = p.chars();
            c.next()
                .map(|f| f.to_ascii_uppercase().to_string() + c.as_str())
                .unwrap_or_default()
        })
        .collect()
}

fn doc_text(s: &str) -> String {
    s.replace('\n', " ")
}

pub fn color(v: &Value) -> String {
    let (c, a) = hex_alpha(v);
    let [r, g, b] = c.0;
    if a < 1.0 {
        format!("Color::rgba({r}, {g}, {b}, {})", float(a))
    } else {
        format!("Color::rgb({r}, {g}, {b})")
    }
}

fn float(x: f64) -> String {
    let s = format!("{x}");
    if s.contains(['.', 'e']) {
        s
    } else {
        format!("{s}.0")
    }
}

fn dimension(v: &Value) -> String {
    let value = v
        .get("value")
        .and_then(Value::as_f64)
        .expect("dimension value");
    let unit = match v.get("unit").and_then(Value::as_str) {
        Some("px") => "Px",
        Some("rem") => "Rem",
        other => panic!("unsupported unit {other:?}"),
    };
    format!(
        "Dimension {{ value: {}, unit: Unit::{unit} }}",
        float(value)
    )
}

fn families(v: &Value) -> String {
    let list: Vec<String> = match v {
        Value::Array(a) => a
            .iter()
            .map(|x| format!("{:?}", x.as_str().unwrap()))
            .collect(),
        Value::String(s) => vec![format!("{s:?}")],
        _ => panic!("font family"),
    };
    format!("&[{}]", list.join(", "))
}

/// `(rust type, rust expression)` for a resolved value.
fn typed(ty: &str, v: &Value) -> (String, String) {
    match ty {
        "color" => ("Color".into(), color(v)),
        "dimension" => ("Dimension".into(), dimension(v)),
        "number" => ("f32".into(), float(v.as_f64().expect("number"))),
        "fontWeight" => ("u16".into(), v.as_u64().expect("font weight").to_string()),
        "fontFamily" => ("&[&str]".into(), families(v)),
        "duration" => {
            let raw = v.get("value").and_then(Value::as_f64).expect("duration");
            let ms = match v.get("unit").and_then(Value::as_str) {
                Some("s") => raw * 1000.0,
                _ => raw,
            };
            let expr = if ms.fract() == 0.0 {
                format!("Duration::from_millis({})", ms as u64)
            } else {
                format!("Duration::from_micros({})", (ms * 1000.0).round() as u64)
            };
            ("Duration".into(), expr)
        }
        "cubicBezier" => {
            let a: Vec<String> = v
                .as_array()
                .expect("bezier")
                .iter()
                .map(|x| float(x.as_f64().unwrap()))
                .collect();
            (
                "CubicBezier".into(),
                format!("CubicBezier([{}])", a.join(", ")),
            )
        }
        "typography" => (
            "Typography".into(),
            format!(
                "Typography {{ font_family: {}, font_size: {}, font_weight: {}, letter_spacing: {}, line_height: {} }}",
                families(&v["fontFamily"]),
                dimension(&v["fontSize"]),
                v["fontWeight"].as_u64().expect("weight"),
                dimension(&v["letterSpacing"]),
                float(v["lineHeight"].as_f64().expect("line height")),
            ),
        ),
        "shadow" => ("Shadow".into(), shadow(v)),
        "border" => ("Border".into(), border(v)),
        other => panic!("no Rust mapping for $type {other}"),
    }
}

fn shadow(v: &Value) -> String {
    format!(
        "Shadow {{ color: {}, offset_x: {}, offset_y: {}, blur: {}, spread: {} }}",
        color(&v["color"]),
        dimension(&v["offsetX"]),
        dimension(&v["offsetY"]),
        dimension(&v["blur"]),
        dimension(&v["spread"]),
    )
}

fn border(v: &Value) -> String {
    format!(
        "Border {{ color: {}, width: {} }}",
        color(&v["color"]),
        dimension(&v["width"])
    )
}

#[derive(Default)]
struct Tree {
    leaves: Vec<(String, String, String, String, Option<String>)>, // const, type, expr, doc, deprecated
    children: BTreeMap<String, (String, Tree)>,                    // module -> (dotted path, tree)
}

impl Tree {
    fn insert(
        &mut self,
        segs: &[&str],
        prefix: &str,
        leaf: (String, String, String, Option<String>),
    ) {
        if segs.len() == 1 {
            let name = const_name(segs[0]);
            assert!(
                self.leaves.iter().all(|l| l.0 != name),
                "duplicate const {name} under {prefix}"
            );
            self.leaves.push((name, leaf.0, leaf.1, leaf.2, leaf.3));
            return;
        }
        let path = if prefix.is_empty() {
            segs[0].to_owned()
        } else {
            format!("{prefix}.{}", segs[0])
        };
        let entry = self
            .children
            .entry(module_name(segs[0]))
            .or_insert_with(|| (path.clone(), Tree::default()));
        entry.1.insert(&segs[1..], &path, leaf);
    }

    fn write(&self, out: &mut String, depth: usize) {
        let pad = "    ".repeat(depth);
        for (name, ty, expr, doc, deprecated) in &self.leaves {
            writeln!(out, "{pad}/// {doc}").unwrap();
            if let Some(note) = deprecated {
                writeln!(out, "{pad}#[deprecated(note = {note:?})]").unwrap();
            }
            writeln!(out, "{pad}pub const {name}: {ty} = {expr};").unwrap();
        }
        for (module, (path, tree)) in &self.children {
            writeln!(out, "{pad}/// `{path}` tokens.").unwrap();
            writeln!(out, "{pad}pub mod {module} {{").unwrap();
            writeln!(out, "{pad}    #[allow(unused_imports)]").unwrap();
            writeln!(out, "{pad}    use crate::types::*;").unwrap();
            writeln!(out, "{pad}    #[allow(unused_imports)]").unwrap();
            writeln!(out, "{pad}    use std::time::Duration;").unwrap();
            tree.write(out, depth + 1);
            writeln!(out, "{pad}}}").unwrap();
        }
    }
}

fn tree_for<'a>(
    set: &TokenSet,
    items: impl Iterator<Item = (&'a str, &'a crate::model::Token)>,
    root: &str,
) -> Tree {
    let mut tree = Tree::default();
    for (rel, tok) in items {
        let value = set.resolved(&tok.path).expect("validated");
        let (ty, expr) = typed(&tok.ty, &value);
        let doc = match &tok.description {
            Some(d) => format!("`{}`: {}", tok.path, doc_text(d)),
            None => format!("`{}`.", tok.path),
        };
        let segs: Vec<&str> = rel.split('.').collect();
        tree.insert(&segs, root, (ty, expr, doc, tok.deprecated.clone()));
    }
    tree
}

pub fn generate(tokens: &TokenSet, motion: &TokenSet) -> String {
    let themes = tokens.themes();
    let first = &themes[0];
    let color_roles: Vec<&str> = tokens
        .under(&format!("theme.{first}.color"))
        .map(|(k, _)| k)
        .collect();
    let elevation_roles: Vec<&str> = tokens
        .under(&format!("theme.{first}.elevation"))
        .map(|(k, _)| k)
        .collect();

    let mut o = String::new();
    o.push_str(
        "// @generated by `cargo run -p oxyn-tokens-gen` from tokens/oxyn.tokens.json and\n",
    );
    o.push_str(
        "// tokens/oxyn.motion.json. Do not edit by hand; CI fails if this file is stale.\n\n",
    );
    o.push_str(
        "//! Generated tokens: colour roles, themes, primitives, semantic values and motion.\n\n",
    );
    o.push_str("#![allow(clippy::excessive_precision, clippy::unreadable_literal)]\n\n");
    o.push_str("use crate::types::*;\n\n");

    for (name, roles, what) in [
        (
            "ColorRole",
            &color_roles,
            "A themed colour role (`theme.<name>.color.<path>`).",
        ),
        (
            "ElevationRole",
            &elevation_roles,
            "A themed elevation (`theme.<name>.elevation.<path>`).",
        ),
    ] {
        let mut seen = std::collections::HashSet::new();
        writeln!(o, "/// {what}").unwrap();
        o.push_str("#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]\n#[repr(u16)]\n");
        writeln!(o, "pub enum {name} {{").unwrap();
        for r in roles.iter() {
            let v = camel(r);
            assert!(seen.insert(v.clone()), "duplicate role variant {v}");
            writeln!(o, "    /// `{r}`\n    {v},").unwrap();
        }
        o.push_str("}\n\n");
        writeln!(o, "impl {name} {{").unwrap();
        writeln!(
            o,
            "    /// Number of roles.\n    pub const COUNT: usize = {};",
            roles.len()
        )
        .unwrap();
        writeln!(
            o,
            "    /// Every role, in token-file order.\n    pub const ALL: [{name}; {}] = [",
            roles.len()
        )
        .unwrap();
        for r in roles.iter() {
            writeln!(o, "        {name}::{},", camel(r)).unwrap();
        }
        o.push_str("    ];\n");
        o.push_str("    /// The token path without the theme prefix, e.g. `fg.muted`.\n");
        o.push_str("    pub const fn path(self) -> &'static str {\n        match self {\n");
        for r in roles.iter() {
            writeln!(o, "            {name}::{} => {r:?},", camel(r)).unwrap();
        }
        o.push_str("        }\n    }\n");
        o.push_str("    /// Looks a role up by [`Self::path`].\n");
        o.push_str(
            "    pub fn from_path(path: &str) -> Option<Self> {\n        Some(match path {\n",
        );
        for r in roles.iter() {
            writeln!(o, "            {r:?} => {name}::{},", camel(r)).unwrap();
        }
        o.push_str("            _ => return None,\n        })\n    }\n}\n\n");
    }

    o.push_str("impl Theme {\n");
    for th in &themes {
        let cname = const_name(th);
        writeln!(o, "    /// The `{th}` theme.").unwrap();
        writeln!(o, "    pub const {cname}: Theme = Theme {{").unwrap();
        writeln!(o, "        name: {th:?},").unwrap();
        o.push_str("        colors: [\n");
        for r in &color_roles {
            let v = tokens
                .resolved(&format!("theme.{th}.color.{r}"))
                .expect("validated");
            writeln!(o, "            {}, // {r}", color(&v)).unwrap();
        }
        o.push_str("        ],\n        elevation: [\n");
        for r in &elevation_roles {
            let v = tokens
                .resolved(&format!("theme.{th}.elevation.{r}"))
                .expect("validated");
            writeln!(o, "            {}, // {r}", shadow(&v)).unwrap();
        }
        o.push_str("        ],\n");
        let ring = tokens
            .resolved(&format!("theme.{th}.focus.ring"))
            .expect("focus ring");
        writeln!(o, "        focus_ring: {},", border(&ring)).unwrap();
        o.push_str("    };\n");
    }
    o.push_str("}\n\n");

    let mut prim = String::new();
    tree_for(tokens, tokens.under("primitive"), "primitive").write(&mut prim, 1);
    o.push_str(
        "/// Raw palette and scales. Components should read [`semantic`] and [`Theme`] instead.\n",
    );
    o.push_str("pub mod primitive {\n    #[allow(unused_imports)]\n    use crate::types::*;\n");
    o.push_str(&prim);
    o.push_str("}\n\n");

    let mut sem = String::new();
    tree_for(tokens, tokens.under("semantic"), "semantic").write(&mut sem, 1);
    o.push_str("/// Theme-independent roles: type, space, radius, density, z, opacity and more.\n");
    o.push_str("pub mod semantic {\n    #[allow(unused_imports)]\n    use crate::types::*;\n");
    o.push_str(&sem);
    o.push_str("}\n\n");

    // Motion: the file's own `motion.*` group is flattened into the module root.
    let items = motion.tokens.iter().map(|t| {
        let rel = t.path.strip_prefix("motion.").unwrap_or(&t.path);
        (rel, t)
    });
    let mut mot = String::new();
    tree_for(motion, items, "").write(&mut mot, 1);
    o.push_str("/// Durations, easings, loops and frame budgets from `tokens/oxyn.motion.json`.\n");
    o.push_str("pub mod motion {\n    #[allow(unused_imports)]\n    use crate::types::*;\n    #[allow(unused_imports)]\n    use std::time::Duration;\n");
    o.push_str(&mot);
    o.push_str("}\n");
    o
}
