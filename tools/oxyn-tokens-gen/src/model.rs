//! Loading, alias resolution and structural validation of a DTCG file.

use std::collections::{BTreeSet, HashMap};

use serde_json::Value;

const TYPES: &[&str] = &[
    "color",
    "dimension",
    "fontFamily",
    "fontWeight",
    "duration",
    "cubicBezier",
    "number",
    "strokeStyle",
    "border",
    "transition",
    "shadow",
    "gradient",
    "typography",
];

/// Composite types whose parts may alias tokens of other types.
const COMPOSITES: &[&str] = &["typography", "border", "shadow"];

/// One token as written in the file.
#[derive(Clone, Debug)]
pub struct Token {
    pub path: String,
    pub ty: String,
    pub value: Value,
    pub description: Option<String>,
    pub deprecated: Option<String>,
}

/// All tokens of one file, in file order.
pub struct TokenSet {
    pub tokens: Vec<Token>,
    index: HashMap<String, usize>,
    pub structural: Vec<String>,
    root: Value,
}

fn alias(v: &str) -> Option<&str> {
    let inner = v.strip_prefix('{')?.strip_suffix('}')?;
    (!inner.contains(['{', '}'])).then_some(inner)
}

fn refs_in(v: &Value, out: &mut Vec<String>) {
    match v {
        Value::String(s) => {
            if let Some(a) = alias(s) {
                out.push(a.to_owned());
            }
        }
        Value::Object(m) => m.values().for_each(|x| refs_in(x, out)),
        Value::Array(a) => a.iter().for_each(|x| refs_in(x, out)),
        _ => {}
    }
}

impl TokenSet {
    pub fn load(root: &Value) -> Self {
        let mut set = TokenSet {
            tokens: Vec::new(),
            index: HashMap::new(),
            structural: Vec::new(),
            root: root.clone(),
        };
        set.walk(root, "", None);
        set.index = set
            .tokens
            .iter()
            .enumerate()
            .map(|(i, t)| (t.path.clone(), i))
            .collect();
        set
    }

    fn walk(&mut self, node: &Value, path: &str, inherited: Option<&str>) {
        let Some(obj) = node.as_object() else {
            self.structural.push(format!("{path}: not an object"));
            return;
        };
        let ty = obj.get("$type").and_then(Value::as_str).or(inherited);
        if let Some(value) = obj.get("$value") {
            match ty {
                Some(t) if TYPES.contains(&t) => {}
                other => self
                    .structural
                    .push(format!("{path}: missing or unknown $type {other:?}")),
            }
            if let Some(child) = obj.keys().find(|k| !k.starts_with('$')) {
                self.structural
                    .push(format!("{path}: token has child {child}"));
            }
            self.tokens.push(Token {
                path: path.to_owned(),
                ty: ty.unwrap_or_default().to_owned(),
                value: value.clone(),
                description: obj
                    .get("$description")
                    .and_then(Value::as_str)
                    .map(str::to_owned),
                deprecated: obj.get("$deprecated").map(|d| match d {
                    Value::String(s) => s.clone(),
                    _ => String::from("deprecated token"),
                }),
            });
            return;
        }
        for (k, v) in obj {
            if k.starts_with('$') {
                continue;
            }
            if k.contains(['.', '{', '}']) {
                self.structural.push(format!("bad name {path}.{k}"));
            }
            let child = if path.is_empty() {
                k.clone()
            } else {
                format!("{path}.{k}")
            };
            self.walk(v, &child, ty);
        }
    }

    pub fn len(&self) -> usize {
        self.tokens.len()
    }

    pub fn alias_count(&self) -> usize {
        self.tokens
            .iter()
            .map(|t| {
                let mut r = Vec::new();
                refs_in(&t.value, &mut r);
                r.len()
            })
            .sum()
    }

    pub fn get(&self, path: &str) -> Option<&Token> {
        self.index.get(path).map(|&i| &self.tokens[i])
    }

    /// The fully resolved value of a token (aliases replaced, recursively).
    pub fn resolved(&self, path: &str) -> Result<Value, String> {
        let t = self
            .get(path)
            .ok_or_else(|| format!("unknown token {path}"))?;
        self.deep(&t.value, &mut vec![path.to_owned()])
    }

    fn deep(&self, v: &Value, seen: &mut Vec<String>) -> Result<Value, String> {
        Ok(match v {
            Value::String(s) => match alias(s) {
                Some(target) => {
                    if seen.iter().any(|p| p == target) {
                        return Err(format!("cycle: {} -> {target}", seen.join(" -> ")));
                    }
                    let t = self.get(target).ok_or_else(|| {
                        format!("{} -> {{{target}}} does not exist", seen.last().unwrap())
                    })?;
                    seen.push(target.to_owned());
                    let out = self.deep(&t.value, seen)?;
                    seen.pop();
                    out
                }
                None => v.clone(),
            },
            Value::Object(m) => Value::Object(
                m.iter()
                    .map(|(k, x)| Ok((k.clone(), self.deep(x, seen)?)))
                    .collect::<Result<_, String>>()?,
            ),
            Value::Array(a) => Value::Array(
                a.iter()
                    .map(|x| self.deep(x, seen))
                    .collect::<Result<_, _>>()?,
            ),
            other => other.clone(),
        })
    }

    /// Checks shared by every file: structure, aliases resolve, alias types match, extension refs.
    pub fn validate_common(&self) -> Vec<String> {
        let mut errors = self.structural.clone();
        for t in &self.tokens {
            let mut refs = Vec::new();
            refs_in(&t.value, &mut refs);
            for r in &refs {
                if let Some(target) = self.get(r)
                    && target.ty != t.ty
                    && !COMPOSITES.contains(&t.ty.as_str())
                {
                    errors.push(format!(
                        "type mismatch {} ({}) -> {r} ({})",
                        t.path, t.ty, target.ty
                    ));
                }
            }
            match self.resolved(&t.path) {
                Err(e) => errors.push(e),
                Ok(v) if t.ty == "cubicBezier" => {
                    let ok = v.as_array().is_some_and(|a| {
                        a.len() == 4
                            && [0, 2]
                                .iter()
                                .all(|&i| a[i].as_f64().is_some_and(|x| (0.0..=1.0).contains(&x)))
                    });
                    if !ok {
                        errors.push(format!(
                            "{}: cubicBezier needs 4 numbers with x1, x2 in [0, 1]",
                            t.path
                        ));
                    }
                }
                Ok(v) if t.ty == "duration" => {
                    if !matches!(v.get("unit").and_then(Value::as_str), Some("ms" | "s")) {
                        errors.push(format!("{}: duration unit must be ms or s", t.path));
                    }
                }
                Ok(_) => {}
            }
        }
        self.extension_refs(&self.root.clone(), "", &mut errors);
        errors
    }

    fn extension_refs(&self, node: &Value, path: &str, errors: &mut Vec<String>) {
        let Some(obj) = node.as_object() else { return };
        for (k, v) in obj {
            if k == "$extensions" {
                let text = v.to_string();
                let mut rest = text.as_str();
                while let Some(start) = rest.find('{') {
                    let after = &rest[start + 1..];
                    let Some(end) = after.find('}') else { break };
                    let inner = &after[..end];
                    let looks_like_ref = inner.starts_with(|c: char| c.is_ascii_lowercase())
                        && inner.chars().all(|c| {
                            c.is_ascii_lowercase() || c.is_ascii_digit() || c == '.' || c == '-'
                        });
                    if looks_like_ref && self.get(inner).is_none() {
                        errors.push(format!("{path}: $extensions ref {{{inner}}} missing"));
                    }
                    rest = &after[end + 1..];
                }
            } else if v.is_object() {
                let child = if path.is_empty() {
                    k.clone()
                } else {
                    format!("{path}.{k}")
                };
                self.extension_refs(v, &child, errors);
            }
        }
    }

    /// Theme names under `theme`, in file order.
    pub fn themes(&self) -> Vec<String> {
        let mut out: Vec<String> = Vec::new();
        for t in &self.tokens {
            if let Some(rest) = t.path.strip_prefix("theme.") {
                let name = rest.split('.').next().unwrap().to_owned();
                if !out.contains(&name) {
                    out.push(name);
                }
            }
        }
        out
    }

    /// Paths under `prefix.` with the prefix removed, in file order.
    pub fn under<'a>(
        &'a self,
        prefix: &str,
    ) -> impl Iterator<Item = (&'a str, &'a Token)> + use<'a> {
        let prefix = format!("{prefix}.");
        self.tokens
            .iter()
            .filter_map(move |t| t.path.strip_prefix(prefix.as_str()).map(|rest| (rest, t)))
    }

    /// The common checks plus the tier and theme rules of `oxyn.tokens.json`.
    pub fn validate_tokens_file(&self) -> Vec<String> {
        let mut errors = self.validate_common();
        let themes = self.themes();
        let keys = |th: &str| -> BTreeSet<&str> {
            self.under(&format!("theme.{th}")).map(|(k, _)| k).collect()
        };
        if let Some(first) = themes.first() {
            let base = keys(first);
            for th in &themes[1..] {
                let other = keys(th);
                if other != base {
                    let missing: Vec<_> = base.difference(&other).take(5).collect();
                    let extra: Vec<_> = other.difference(&base).take(5).collect();
                    errors.push(format!(
                        "theme {th} key mismatch: missing {missing:?} extra {extra:?}"
                    ));
                }
            }
        }
        for t in &self.tokens {
            let mut refs = Vec::new();
            refs_in(&t.value, &mut refs);
            let tier = t.path.split('.').next().unwrap_or_default();
            for r in refs {
                match tier {
                    "primitive" if !r.starts_with("primitive.") => {
                        errors.push(format!("primitive {} references {r}", t.path));
                    }
                    "semantic" if r.starts_with("theme.") => {
                        errors.push(format!("semantic {} references a theme token {r}", t.path));
                    }
                    "theme" => {
                        let own = t.path.split('.').nth(1).unwrap_or_default();
                        if r.starts_with("theme.") && r.split('.').nth(1) != Some(own) {
                            errors.push(format!("{} references another theme: {r}", t.path));
                        }
                    }
                    _ => {}
                }
            }
        }
        errors
    }
}
