//! A component's props, read from its code so they always match what `add_component` writes.
//!
//! - Dioxus: the parameters of the `#[component] fn Entry(...)`, with their `#[props(default)]`
//!   and doc comments.
//! - GPUI: the element's builder, `impl EntryEl { pub fn new(..) -> Self; pub fn size(mut self, ..) }`:
//!   the constructor and each builder method.

use serde_json::{Value, json};

/// One prop (Dioxus) or builder method (GPUI).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Prop {
    /// Parameter or method name.
    pub name: String,
    /// Its type (Dioxus) or its parameters (GPUI builder method).
    pub ty: String,
    /// The default, when the prop is optional (`#[props(default)]` gives `Default`).
    pub default: Option<String>,
    /// First sentence of the doc comment.
    pub doc: Option<String>,
}

impl Prop {
    /// Compact JSON: absent fields are left out.
    pub fn to_json(&self) -> Value {
        let mut v = json!({ "name": self.name, "type": self.ty });
        if let Some(d) = &self.default {
            v["default"] = json!(d);
        }
        if let Some(d) = &self.doc {
            v["doc"] = json!(d);
        }
        v
    }
}

/// The text between the `(` at `open` and its matching `)`.
fn balanced(src: &str, open: usize) -> Option<&str> {
    let mut depth = 0;
    let mut in_str = false;
    let mut esc = false;
    for (i, c) in src[open..].char_indices() {
        if in_str {
            if esc {
                esc = false;
            } else if c == '\\' {
                esc = true;
            } else if c == '"' {
                in_str = false;
            }
            continue;
        }
        match c {
            '"' => in_str = true,
            '(' => depth += 1,
            ')' => {
                depth -= 1;
                if depth == 0 {
                    return Some(&src[open + 1..open + i]);
                }
            }
            _ => {}
        }
    }
    None
}

/// Splits a leading `#[...]` off `s`.
fn split_attr(s: &str) -> Option<(&str, &str)> {
    if !s.starts_with("#[") {
        return None;
    }
    let mut depth = 0;
    for (i, c) in s.char_indices() {
        match c {
            '[' | '(' => depth += 1,
            ']' | ')' => {
                depth -= 1;
                if depth == 0 {
                    return Some((&s[..=i], s[i + 1..].trim_start()));
                }
            }
            _ => {}
        }
    }
    None
}

fn first_sentence(docs: &[String]) -> Option<String> {
    let all = docs.join(" ");
    let all = all.trim();
    if all.is_empty() {
        return None;
    }
    let end = all.find(". ").map(|i| i + 1).unwrap_or(all.len()).min(200);
    let mut cut = end;
    while !all.is_char_boundary(cut) {
        cut -= 1;
    }
    Some(all[..cut].to_string())
}

fn squash(s: &str) -> String {
    s.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn default_of(attrs: &[String]) -> Option<String> {
    let a = attrs.iter().find(|a| a.contains("default"))?;
    match a.find("default =") {
        Some(i) => {
            let rest = a[i + "default =".len()..].trim();
            // `#[props(default = X)]`: drop the closing `)]`.
            let rest = rest.strip_suffix(']').unwrap_or(rest).trim_end();
            let rest = rest.strip_suffix(')').unwrap_or(rest).trim_end();
            Some(squash(rest))
        }
        None => Some("Default".into()),
    }
}

/// Parses a parameter list (`name: Type, ...` with doc comments and attributes).
pub fn parse_params(params: &str) -> Vec<Prop> {
    let mut out = Vec::new();
    let mut docs: Vec<String> = Vec::new();
    let mut attrs: Vec<String> = Vec::new();
    let mut cur = String::new();
    let mut depth = 0;
    let mut finish = |cur: &mut String, docs: &mut Vec<String>, attrs: &mut Vec<String>| {
        let mut p = cur.trim().trim_end_matches(',').trim().to_string();
        while let Some((a, rest)) = split_attr(&p) {
            attrs.push(a.to_string());
            p = rest.to_string();
        }
        if !p.is_empty()
            && let Some((name, ty)) = p.split_once(':')
        {
            let name = name.trim().trim_start_matches("mut ").to_string();
            if name != "self" && !name.ends_with("self") {
                out.push(Prop {
                    name,
                    ty: squash(ty),
                    default: default_of(attrs),
                    doc: first_sentence(docs),
                });
            }
        }
        cur.clear();
        docs.clear();
        attrs.clear();
    };
    for line in params.lines() {
        let t = line.trim();
        if cur.trim().is_empty() && depth == 0 {
            if let Some(d) = t.strip_prefix("///") {
                docs.push(d.trim().to_string());
                continue;
            }
            if t.starts_with("//") || t.is_empty() {
                continue;
            }
        }
        // Walk the line, cutting at depth-0 commas (one-line lists too).
        let mut in_str = false;
        let mut esc = false;
        let mut prev = ' ';
        for c in t.chars() {
            if !in_str && c == ',' && depth == 0 {
                finish(&mut cur, &mut docs, &mut attrs);
                prev = c;
                continue;
            }
            cur.push(c);
            if in_str {
                if esc {
                    esc = false;
                } else if c == '\\' {
                    esc = true;
                } else if c == '"' {
                    in_str = false;
                }
            } else {
                match c {
                    '"' => in_str = true,
                    '(' | '[' | '{' | '<' => depth += 1,
                    ')' | ']' | '}' => depth -= 1,
                    '>' if prev != '-' && prev != '=' => depth -= 1,
                    _ => {}
                }
            }
            prev = c;
        }
        cur.push(' ');
        // An attribute alone on its line belongs to the next parameter.
        if depth == 0 {
            let p = cur.trim().to_string();
            if let Some((a, rest)) = split_attr(&p)
                && rest.is_empty()
            {
                attrs.push(a.to_string());
                cur.clear();
            }
        }
    }
    if !cur.trim().is_empty() {
        finish(&mut cur, &mut docs, &mut attrs);
    }
    out
}

/// Dioxus: the props of `fn <entry>(...)` in any of the chunks.
pub fn dioxus_props<'a>(
    chunks: impl IntoIterator<Item = &'a str>,
    entry: &str,
) -> Option<Vec<Prop>> {
    let needle = format!("fn {entry}(");
    for code in chunks {
        if let Some(at) = code.find(&needle) {
            let open = at + needle.len() - 1;
            return balanced(code, open).map(parse_params);
        }
    }
    None
}

/// GPUI: the constructor (`EntryEl::new(..)`) and builder methods of `impl <entry> { .. }`.
pub fn gpui_builder<'a>(
    chunks: impl IntoIterator<Item = &'a str>,
    entry: &str,
) -> Option<(Vec<String>, Vec<Prop>)> {
    let head = format!("impl {entry} {{");
    let code = chunks.into_iter().find(|c| c.contains(&head))?;
    let body = &code[code.find(&head)? + head.len()..];
    let mut ctors = Vec::new();
    let mut methods = Vec::new();
    let mut docs: Vec<String> = Vec::new();
    let mut offset = 0;
    for line in body.split_inclusive('\n') {
        let start = offset;
        offset += line.len();
        let t = line.trim();
        if let Some(d) = t.strip_prefix("///") {
            docs.push(d.trim().to_string());
            continue;
        }
        let Some(rest) = t.strip_prefix("pub fn ") else {
            if !t.starts_with("#[") {
                docs.clear();
            }
            continue;
        };
        let Some(paren) = rest.find('(') else {
            docs.clear();
            continue;
        };
        let name = rest[..paren].trim().to_string();
        let abs = start + line.find("pub fn ").unwrap_or(0) + "pub fn ".len() + paren;
        let Some(params) = balanced(body, abs) else {
            docs.clear();
            continue;
        };
        let takes_self = params
            .split(',')
            .next()
            .is_some_and(|p| p.trim().ends_with("self"));
        let args = squash(
            &params
                .split(',')
                .filter(|p| !p.trim().ends_with("self"))
                .collect::<Vec<_>>()
                .join(","),
        );
        if takes_self {
            methods.push(Prop {
                name,
                ty: args.trim_start_matches(',').trim().to_string(),
                default: None,
                doc: first_sentence(&docs),
            });
        } else {
            ctors.push(format!("{entry}::{name}({args})"));
        }
        docs.clear();
    }
    Some((ctors, methods))
}

#[cfg(test)]
mod tests {
    use super::*;

    const DX: &str = r#"/// A button.
#[component]
pub fn Button(
    /// Label.
    label: String,
    /// `primary` · `ghost`, or `link`.
    #[props(default = "primary".to_string())]
    variant: String,
    /// Busy: keeps its width. Ignores presses.
    #[props(default)]
    loading: bool,
    #[props(default)] on_press: Option<EventHandler<(u32, String)>>,
    render: Option<Box<dyn Fn(u32) -> Element>>,
) -> Element {
    rsx! {}
}"#;

    #[test]
    fn dioxus_params_with_docs_and_defaults() {
        let p = dioxus_props([DX], "Button").unwrap();
        let names: Vec<&str> = p.iter().map(|p| p.name.as_str()).collect();
        assert_eq!(names, ["label", "variant", "loading", "on_press", "render"]);
        assert_eq!(p[0].ty, "String");
        assert_eq!(p[0].default, None);
        assert_eq!(p[0].doc.as_deref(), Some("Label."));
        assert_eq!(p[1].default.as_deref(), Some("\"primary\".to_string()"));
        assert_eq!(p[2].default.as_deref(), Some("Default"));
        assert_eq!(p[2].doc.as_deref(), Some("Busy: keeps its width."));
        assert_eq!(p[3].ty, "Option<EventHandler<(u32, String)>>");
        assert_eq!(p[3].default.as_deref(), Some("Default"));
        assert_eq!(p[4].ty, "Option<Box<dyn Fn(u32) -> Element>>");
        assert!(dioxus_props([DX], "Missing").is_none());
        assert_eq!(p[1].to_json()["default"], "\"primary\".to_string()");
        assert!(p[4].to_json().get("doc").is_none());
    }

    #[test]
    fn one_line_params() {
        let p = parse_params("a: u32, b: Vec<(u8, u8)>, mut c: bool");
        assert_eq!(
            p.iter()
                .map(|p| (p.name.as_str(), p.ty.as_str()))
                .collect::<Vec<_>>(),
            [("a", "u32"), ("b", "Vec<(u8, u8)>"), ("c", "bool")]
        );
    }

    #[test]
    fn gpui_constructor_and_builder() {
        let code = "#[derive(IntoElement)]\npub struct BadgeEl { a: u8 }\n\nimpl BadgeEl {\n    /// A badge.\n    pub fn new(badge: Badge, id: impl Into<ElementId>) -> Self {\n        Self { a: 0 }\n    }\n    /// Size (`sm` default). More.\n    pub fn size(mut self, size: BadgeSize) -> Self {\n        self\n    }\n    /// Loading.\n    pub fn loading(mut self, progress: Option<f32>, now: ClockMs) -> Self {\n        self\n    }\n}\n";
        let (ctors, m) = gpui_builder([code], "BadgeEl").unwrap();
        assert_eq!(
            ctors,
            ["BadgeEl::new(badge: Badge, id: impl Into<ElementId>)"]
        );
        assert_eq!(m.len(), 2);
        assert_eq!(m[0].name, "size");
        assert_eq!(m[0].ty, "size: BadgeSize");
        assert_eq!(m[0].doc.as_deref(), Some("Size (`sm` default)."));
        assert_eq!(m[1].ty, "progress: Option<f32>, now: ClockMs");
        assert!(gpui_builder([code], "Nope").is_none());
    }
}
