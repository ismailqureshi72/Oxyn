//! A small Rust source scanner: enough of the language to find a file's top-level items, its `use`
//! trees, the paths it names and the files it includes. Not a parser: comments and literals are
//! blanked first, then items are split at depth 0.

/// Replaces the inside of comments, string and char literals with spaces (newlines kept), so the
/// result has the same byte offsets as `src` and only code is left to scan.
pub fn blank(src: &str) -> String {
    let b = src.as_bytes();
    let mut out = b.to_vec();
    let n = b.len();
    let mut i = 0;
    let wipe = |out: &mut Vec<u8>, from: usize, to: usize| {
        for c in out.iter_mut().take(to).skip(from) {
            if *c != b'\n' {
                *c = b' ';
            }
        }
    };
    let ident = |c: u8| c.is_ascii_alphanumeric() || c == b'_';
    while i < n {
        let c = b[i];
        if c == b'/' && i + 1 < n && b[i + 1] == b'/' {
            let end = b[i..].iter().position(|&x| x == b'\n').map_or(n, |p| i + p);
            wipe(&mut out, i, end);
            i = end;
        } else if c == b'/' && i + 1 < n && b[i + 1] == b'*' {
            let mut depth = 1;
            let mut j = i + 2;
            while j < n && depth > 0 {
                if b[j] == b'/' && j + 1 < n && b[j + 1] == b'*' {
                    depth += 1;
                    j += 2;
                } else if b[j] == b'*' && j + 1 < n && b[j + 1] == b'/' {
                    depth -= 1;
                    j += 2;
                } else {
                    j += 1;
                }
            }
            wipe(&mut out, i, j);
            i = j;
        } else if (c == b'r' || (c == b'b' && i + 1 < n && b[i + 1] == b'r'))
            && (i == 0 || !ident(b[i - 1]))
            && {
                let s = if c == b'b' { i + 2 } else { i + 1 };
                let mut k = s;
                while k < n && b[k] == b'#' {
                    k += 1;
                }
                k < n && b[k] == b'"'
            }
        {
            // Raw string: r"…", r#"…"#, br#"…"#.
            let s = if c == b'b' { i + 2 } else { i + 1 };
            let mut k = s;
            while b[k] == b'#' {
                k += 1;
            }
            let hashes = k - s;
            let body = k + 1;
            let mut j = body;
            let mut end = n;
            while j < n {
                if b[j] == b'"'
                    && j + hashes < n
                    && b[j + 1..=j + hashes].iter().all(|&x| x == b'#')
                {
                    end = j;
                    break;
                }
                j += 1;
            }
            wipe(&mut out, body, end);
            i = (end + 1 + hashes).min(n);
        } else if c == b'"' {
            let mut j = i + 1;
            while j < n && b[j] != b'"' {
                if b[j] == b'\\' {
                    j += 1;
                }
                j += 1;
            }
            wipe(&mut out, i + 1, j.min(n));
            i = j + 1;
        } else if c == b'\'' {
            // Char literal or lifetime / label.
            if i + 1 < n && b[i + 1] == b'\\' {
                let mut j = i + 2;
                while j < n && b[j] != b'\'' {
                    if b[j] == b'\\' {
                        j += 1;
                    }
                    j += 1;
                }
                wipe(&mut out, i + 1, j.min(n));
                i = j + 1;
            } else {
                // One (possibly multi-byte) char then a quote: a char literal.
                let ch_len = src[i + 1..].chars().next().map_or(1, char::len_utf8);
                if i + 1 + ch_len < n && b[i + 1 + ch_len] == b'\'' {
                    wipe(&mut out, i + 1, i + 1 + ch_len);
                    i += 2 + ch_len;
                } else {
                    i += 1;
                }
            }
        } else {
            i += 1;
        }
    }
    // Every byte replaced was ASCII-safe or became a space, so this is valid UTF-8.
    String::from_utf8(out).unwrap_or_default()
}

/// One top-level item of a file: its byte range in the source (leading doc comments and
/// attributes included) and what it is.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Item {
    /// Start byte (the first attribute or doc comment).
    pub start: usize,
    /// End byte (exclusive), after the closing `}` or `;`.
    pub end: usize,
    /// What it is.
    pub kind: ItemKind,
    /// `#[cfg(test)]` on it.
    pub test_only: bool,
    /// `pub` in any form.
    pub public: bool,
}

/// The kinds of item the registry cares about.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ItemKind {
    /// `mod name;` (a file module).
    ModDecl(String),
    /// `mod name { … }` (inline; its contents are not tracked).
    ModInline(String),
    /// A `use` statement and its bindings.
    Use(Vec<UseBinding>),
    /// Something that defines names (`fn`, `struct`, `const`, …).
    Def(Vec<String>),
    /// `impl` blocks, inner attributes and anything else that defines no name.
    Other,
}

/// One binding of a `use` tree: `use a::b::{c as d}` gives path `[a, b, c]`, name `d`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UseBinding {
    /// The full path.
    pub path: Vec<String>,
    /// The bound name; `None` for a glob (`*`).
    pub name: Option<String>,
}

/// Splits a file into its top-level items, reading the blanked text `code` (same offsets as the
/// source `src`). An item starts at its first doc comment or attribute.
pub fn items(src: &str, code: &str) -> Vec<Item> {
    let sb = src.as_bytes();
    let b = code.as_bytes();
    let n = b.len();
    let mut out = Vec::new();
    let mut i = 0;
    while i < n {
        // Skip whitespace between items (comments count as part of the next item).
        let prev = i;
        while i < n && b[i].is_ascii_whitespace() {
            i += 1;
        }
        if i >= n {
            break;
        }
        let mut start = prev;
        while start < i && sb[start].is_ascii_whitespace() {
            start += 1;
        }
        // Inner attribute: its own item.
        if b[i] == b'#' && i + 1 < n && b[i + 1] == b'!' {
            let end = match_bracket(b, i + 2).map_or(n, |e| e + 1);
            out.push(Item {
                start,
                end,
                kind: ItemKind::Other,
                test_only: false,
                public: false,
            });
            i = end;
            continue;
        }
        // Outer attributes.
        let mut test_only = false;
        let mut is_component = false;
        let mut j = i;
        loop {
            while j < n && b[j].is_ascii_whitespace() {
                j += 1;
            }
            if j < n && b[j] == b'#' && j + 1 < n && b[j + 1] == b'[' {
                let e = match_bracket(b, j + 1).unwrap_or(n - 1);
                let attr: String = code[j + 2..e].split_whitespace().collect();
                if attr == "cfg(test)" {
                    test_only = true;
                }
                if attr == "component" {
                    is_component = true;
                }
                j = e + 1;
            } else {
                break;
            }
        }
        let head_start = j;
        // Find the end of the item: `;` at depth 0, or a `}` back to depth 0 (unless the item is
        // one that always ends with `;`, like `const X: T = T { … };`).
        let mut cut = (head_start + 200).min(n);
        while !code.is_char_boundary(cut) {
            cut -= 1;
        }
        let first_words = words(&code[head_start..cut]);
        let semi_item = first_words
            .iter()
            .take(4)
            .any(|w| matches!(w.as_str(), "const" | "static" | "use" | "let" | "type"))
            && !first_words.iter().take(4).any(|w| w == "fn");
        let mut depth: i32 = 0;
        let mut k = head_start;
        let mut end = n;
        while k < n {
            match b[k] {
                b'{' | b'(' | b'[' => depth += 1,
                b'}' | b')' | b']' => {
                    depth -= 1;
                    if depth == 0 && b[k] == b'}' && !semi_item {
                        end = k + 1;
                        break;
                    }
                }
                b';' if depth == 0 => {
                    end = k + 1;
                    break;
                }
                _ => {}
            }
            k += 1;
        }
        let head = &code[head_start..end];
        let (public, kind) = classify(head, is_component);
        out.push(Item {
            start,
            end,
            kind,
            test_only,
            public,
        });
        i = end;
    }
    out
}

/// The index of the bracket closing the one at `open`.
fn match_bracket(b: &[u8], open: usize) -> Option<usize> {
    let mut depth = 0;
    for (k, &c) in b.iter().enumerate().skip(open) {
        match c {
            b'[' | b'(' | b'{' => depth += 1,
            b']' | b')' | b'}' => {
                depth -= 1;
                if depth == 0 {
                    return Some(k);
                }
            }
            _ => {}
        }
    }
    None
}

/// Identifier-ish words of a piece of code.
fn words(s: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut cur = String::new();
    for c in s.chars() {
        if c.is_alphanumeric() || c == '_' || c == '!' {
            cur.push(c);
        } else {
            if !cur.is_empty() {
                out.push(std::mem::take(&mut cur));
            }
            if c == '{' || c == '(' || c == '=' || c == ':' || c == ';' || c == '<' {
                out.push(c.to_string());
            }
        }
    }
    if !cur.is_empty() {
        out.push(cur);
    }
    out
}

fn classify(head: &str, is_component: bool) -> (bool, ItemKind) {
    let w = words(head);
    let mut i = 0;
    let mut public = false;
    if w.first().is_some_and(|x| x == "pub") {
        public = true;
        i = 1;
        // pub(crate) / pub(super) / pub(in path)
        if w.get(1).is_some_and(|x| x == "(") {
            i = 2;
            while i < w.len() && !matches!(w[i].as_str(), "crate" | "super" | "self" | "in") {
                i += 1;
            }
            i += 1;
            if w.get(i - 1).is_some_and(|x| x == "in") {
                while i < w.len() && w[i] != "mod" && w[i] != "fn" && w[i] != "use" {
                    i += 1;
                }
            }
        }
    }
    while i < w.len() && matches!(w[i].as_str(), "async" | "unsafe" | "extern" | "default") {
        i += 1;
    }
    let Some(kw) = w.get(i) else {
        return (public, ItemKind::Other);
    };
    let name = w.get(i + 1).cloned().unwrap_or_default();
    let kind = match kw.as_str() {
        "use" => ItemKind::Use(parse_use(head)),
        "mod" => {
            if head.trim_end().ends_with(';') {
                ItemKind::ModDecl(name)
            } else {
                ItemKind::ModInline(name)
            }
        }
        "const" if name == "fn" => ItemKind::Def(vec![w.get(i + 2).cloned().unwrap_or_default()]),
        "const" if name == "_" => ItemKind::Other,
        "fn" => {
            let mut v = vec![name.clone()];
            if is_component {
                v.push(format!("{name}Props"));
            }
            ItemKind::Def(v)
        }
        "struct" | "enum" | "trait" | "type" | "const" | "static" | "union" => {
            ItemKind::Def(vec![name])
        }
        "macro_rules!" => ItemKind::Def(vec![name]),
        _ => ItemKind::Other,
    };
    (public, kind)
}

/// Parses the bindings of a `use` statement (`[pub] use tree;`), from blanked code.
pub fn parse_use(head: &str) -> Vec<UseBinding> {
    let Some(at) = find_word(head, "use") else {
        return Vec::new();
    };
    let body = head[at + 3..].trim().trim_end_matches(';').trim();
    let toks = use_tokens(body);
    let mut out = Vec::new();
    let mut pos = 0;
    tree(&toks, &mut pos, Vec::new(), &mut out);
    out
}

fn find_word(s: &str, word: &str) -> Option<usize> {
    let b = s.as_bytes();
    let mut from = 0;
    while let Some(p) = s[from..].find(word) {
        let at = from + p;
        let before = at == 0 || !(b[at - 1].is_ascii_alphanumeric() || b[at - 1] == b'_');
        let after = at + word.len() >= b.len()
            || !(b[at + word.len()].is_ascii_alphanumeric() || b[at + word.len()] == b'_');
        if before && after {
            return Some(at);
        }
        from = at + word.len();
    }
    None
}

fn use_tokens(s: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut cur = String::new();
    let cs: Vec<char> = s.chars().collect();
    let mut i = 0;
    while i < cs.len() {
        let c = cs[i];
        if c.is_alphanumeric() || c == '_' || c == '#' {
            cur.push(c);
        } else {
            if !cur.is_empty() {
                out.push(std::mem::take(&mut cur));
            }
            match c {
                ':' if cs.get(i + 1) == Some(&':') => {
                    out.push("::".into());
                    i += 1;
                }
                '{' | '}' | ',' | '*' => out.push(c.to_string()),
                _ => {}
            }
        }
        i += 1;
    }
    if !cur.is_empty() {
        out.push(cur);
    }
    out
}

/// One use tree at `pos`: `a::b::{…}`, `a::b`, `a::b as c`, `*`.
fn tree(t: &[String], pos: &mut usize, prefix: Vec<String>, out: &mut Vec<UseBinding>) {
    let mut path = prefix;
    // Leading `::`.
    if t.get(*pos).is_some_and(|x| x == "::") {
        *pos += 1;
    }
    loop {
        match t.get(*pos).map(String::as_str) {
            Some("{") => {
                *pos += 1;
                loop {
                    if t.get(*pos).is_some_and(|x| x == "}") {
                        *pos += 1;
                        break;
                    }
                    tree(t, pos, path.clone(), out);
                    if t.get(*pos).is_some_and(|x| x == ",") {
                        *pos += 1;
                    }
                    if *pos >= t.len() {
                        break;
                    }
                }
                return;
            }
            Some("*") => {
                *pos += 1;
                out.push(UseBinding { path, name: None });
                return;
            }
            Some(seg) if seg != "," && seg != "}" && seg != "::" => {
                let seg = seg.to_string();
                *pos += 1;
                if t.get(*pos).is_some_and(|x| x == "::") {
                    path.push(seg);
                    *pos += 1;
                    continue;
                }
                // Last segment, maybe `as name`.
                let mut bind = if seg == "self" {
                    path.last().cloned().unwrap_or_default()
                } else {
                    seg.clone()
                };
                if seg != "self" {
                    path.push(seg);
                }
                if t.get(*pos).is_some_and(|x| x == "as") {
                    bind = t.get(*pos + 1).cloned().unwrap_or_default();
                    *pos += 2;
                }
                if bind != "_" {
                    out.push(UseBinding {
                        path,
                        name: Some(bind),
                    });
                }
                return;
            }
            _ => return,
        }
    }
}

/// Every path (`a::b::c`) and lone identifier in a piece of blanked code, in order. Generic
/// arguments break a path (`Vec::<T>::new` gives `Vec` and `T` and `new`).
pub fn paths(code: &str) -> Vec<Vec<String>> {
    let b = code.as_bytes();
    let n = b.len();
    let mut out = Vec::new();
    let mut cur: Vec<String> = Vec::new();
    let mut i = 0;
    let mut expect_seg = false;
    while i < n {
        let c = b[i];
        if c.is_ascii_alphabetic() || c == b'_' {
            let s = i;
            while i < n && (b[i].is_ascii_alphanumeric() || b[i] == b'_') {
                i += 1;
            }
            let mut w = &code[s..i];
            // Raw identifier r#name.
            if w == "r" && i < n && b[i] == b'#' {
                let s2 = i + 1;
                i = s2;
                while i < n && (b[i].is_ascii_alphanumeric() || b[i] == b'_') {
                    i += 1;
                }
                w = &code[s2..i];
            }
            // A lifetime or label: skip.
            if s > 0 && b[s - 1] == b'\'' {
                continue;
            }
            if !expect_seg && !cur.is_empty() {
                out.push(std::mem::take(&mut cur));
            }
            cur.push(w.to_string());
            expect_seg = false;
            continue;
        }
        if c.is_ascii_digit() {
            // Numbers (and suffixes like 1u8) are not identifiers.
            while i < n && (b[i].is_ascii_alphanumeric() || b[i] == b'_' || b[i] == b'.') {
                i += 1;
            }
            if !cur.is_empty() {
                out.push(std::mem::take(&mut cur));
            }
            expect_seg = false;
            continue;
        }
        if c == b':' && i + 1 < n && b[i + 1] == b':' && !cur.is_empty() {
            i += 2;
            expect_seg = true;
            continue;
        }
        if c.is_ascii_whitespace() && expect_seg {
            i += 1;
            continue;
        }
        if !cur.is_empty() {
            out.push(std::mem::take(&mut cur));
        }
        expect_seg = false;
        i += 1;
    }
    if !cur.is_empty() {
        out.push(cur);
    }
    out
}

/// The string arguments of `include_str!` / `include_bytes!` in `src`, found through `code`.
pub fn includes(src: &str, code: &str) -> Vec<String> {
    let mut out = Vec::new();
    for mac in ["include_str!", "include_bytes!"] {
        let mut from = 0;
        while let Some(p) = code[from..].find(mac) {
            let at = from + p + mac.len();
            from = at;
            let rest = &src[at..];
            if let Some(q1) = rest.find('"')
                && rest[..q1]
                    .trim_start()
                    .trim_start_matches('(')
                    .trim()
                    .is_empty()
                && let Some(q2) = rest[q1 + 1..].find('"')
            {
                out.push(rest[q1 + 1..q1 + 1 + q2].to_string());
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn blank_keeps_offsets_and_hides_literals() {
        let src = "let a = \"x::y\"; // crate::z\nlet b = 'c'; let r = r#\"q\"#; fn f<'a>() {}";
        let b = blank(src);
        assert_eq!(b.len(), src.len());
        assert!(!b.contains("x::y"));
        assert!(!b.contains("crate::z"));
        assert!(b.contains("fn f<'a>"));
        assert!(!b.contains("\"q\""));
        assert!(b.contains('\n'));
    }

    #[test]
    fn items_split_at_depth_zero() {
        let src = "//! docs\n#![allow(x)]\nuse a::{b, c::d as e};\n/// doc\n#[component]\npub fn Foo(x: u8) -> Element { rsx! {} }\nconst A: T = T { a: 1 };\nmod kid;\n#[cfg(test)]\nmod tests { fn t() {} }\nimpl X { fn y() {} }\npub(crate) struct S(u8);\n";
        let code = blank(src);
        let it = items(src, &code);
        let kinds: Vec<_> = it.iter().map(|i| i.kind.clone()).collect();
        assert!(matches!(kinds[0], ItemKind::Other));
        assert!(
            matches!(&kinds[1], ItemKind::Use(b) if b.len() == 2 && b[1].name.as_deref() == Some("e") && b[1].path == ["a", "c", "d"])
        );
        assert_eq!(
            kinds[2],
            ItemKind::Def(vec!["Foo".into(), "FooProps".into()])
        );
        assert!(it[2].public);
        assert!(src[it[2].start..it[2].end].starts_with("/// doc"));
        assert_eq!(kinds[3], ItemKind::Def(vec!["A".into()]));
        assert!(src[it[3].start..it[3].end].ends_with("};"));
        assert_eq!(kinds[4], ItemKind::ModDecl("kid".into()));
        assert!(it[5].test_only);
        assert_eq!(kinds[6], ItemKind::Other);
        assert_eq!(kinds[7], ItemKind::Def(vec!["S".into()]));
        assert!(it[7].public);
    }

    #[test]
    fn use_trees_expand() {
        let b =
            parse_use("pub use crate::{brand::BrandIcon, composer::icons::{self, Icon}, x::*};");
        let got: Vec<(String, Option<String>)> = b
            .iter()
            .map(|u| (u.path.join("::"), u.name.clone()))
            .collect();
        assert_eq!(
            got,
            vec![
                ("crate::brand::BrandIcon".into(), Some("BrandIcon".into())),
                ("crate::composer::icons".into(), Some("icons".into())),
                ("crate::composer::icons::Icon".into(), Some("Icon".into())),
                ("crate::x".into(), None),
            ]
        );
    }

    #[test]
    fn paths_and_includes() {
        let src = "let x = super::foo::bar(Icon::Send, 3u8); y.z(); const C: &str = include_str!(\"../a/b.css\");";
        let code = blank(src);
        let p = paths(&code);
        assert!(p.contains(&vec!["super".into(), "foo".into(), "bar".into()]));
        assert!(p.contains(&vec!["Icon".into(), "Send".into()]));
        assert!(p.contains(&vec!["z".into()]));
        assert_eq!(includes(src, &code), vec!["../a/b.css".to_string()]);
    }
}
