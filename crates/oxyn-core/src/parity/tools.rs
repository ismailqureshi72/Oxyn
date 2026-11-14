//! Tool calls (plan #28, chunk 3).
//!
//! - [`ToolKindMap`]: the tool-kind map: raw tool names from any agent (Claude Code, Codex,
//!   OpenCode, MCP) to a [`ToolKind`], an icon and present / past verbs, with exact-name and
//!   prefix overrides a host can add, and MCP names split into server and tool.
//! - [`CallStatus`] and [`CallCard`]: one tool call as a line with a live timer that folds open
//!   to arguments and result (open on error, capped code with Show all), and a group header.
//! - [`Json`] and [`JsonTree`]: a parsed JSON value as a collapsible tree with paths, search,
//!   paging and keyboard moves.
//! - [`Unfurl`]: a pasted link that resolves into a preview card, collapses back or is removed.

use crate::{ClockMs, run::ToolKind, verbs::VerbPack};

use super::plan::short_duration;

/// What the map says about a tool name.
#[derive(Clone, Debug, PartialEq)]
pub struct ToolInfo {
    /// Kind.
    pub kind: ToolKind,
    /// Icon name.
    pub icon: &'static str,
    /// "Running" phrase ("Reading").
    pub present: String,
    /// Settled phrase ("Read").
    pub past: String,
    /// MCP server, when the name is `mcp__server__tool`.
    pub server: Option<String>,
    /// Bare tool name (MCP tool part, or the name itself).
    pub tool: String,
}

/// Raw tool name → kind, with overrides.
#[derive(Clone, Debug, PartialEq, Default)]
pub struct ToolKindMap {
    /// Exact names (case-insensitive) → kind.
    pub exact: Vec<(String, ToolKind)>,
    /// Prefixes → kind.
    pub prefixes: Vec<(String, ToolKind)>,
}

impl ToolKindMap {
    /// The built-in aliases for common agents (Claude Code, Codex, OpenCode, Cline) on top of
    /// [`ToolKind::from_name`].
    pub fn standard() -> Self {
        let e = |n: &str, k| (n.to_string(), k);
        Self {
            exact: vec![
                e("multiedit", ToolKind::Edit),
                e("apply_patch", ToolKind::Edit),
                e("str_replace_based_edit_tool", ToolKind::Edit),
                e("notebookedit", ToolKind::Edit),
                e("glob", ToolKind::List),
                e("ls", ToolKind::List),
                e("grep", ToolKind::Search),
                e("codebase_search", ToolKind::Search),
                e("shell", ToolKind::Shell),
                e("local_shell", ToolKind::Shell),
                e("execute_command", ToolKind::Shell),
                e("update_plan", ToolKind::Plan),
                e("todowrite", ToolKind::Plan),
                e("task", ToolKind::Subagent),
                e("webfetch", ToolKind::Fetch),
                e("websearch", ToolKind::WebSearch),
                e("browser_action", ToolKind::Fetch),
                e("image_gen", ToolKind::Media),
            ],
            prefixes: vec![],
        }
    }
    /// Adds an exact override.
    pub fn with(mut self, name: &str, k: ToolKind) -> Self {
        self.exact.insert(0, (name.to_ascii_lowercase(), k));
        self
    }
    /// Adds a prefix override.
    pub fn with_prefix(mut self, prefix: &str, k: ToolKind) -> Self {
        self.prefixes.push((prefix.to_ascii_lowercase(), k));
        self
    }
    /// The kind for a raw name.
    pub fn kind(&self, name: &str) -> ToolKind {
        let l = name.to_ascii_lowercase();
        if let Some((_, k)) = self.exact.iter().find(|(n, _)| *n == l) {
            return *k;
        }
        if let Some((_, k)) = self
            .prefixes
            .iter()
            .filter(|(p, _)| l.starts_with(p.as_str()))
            .max_by_key(|(p, _)| p.len())
        {
            return *k;
        }
        ToolKind::from_name(name)
    }
    /// Everything about a raw name.
    pub fn info(&self, name: &str, pack: &VerbPack) -> ToolInfo {
        let kind = self.kind(name);
        let (server, tool) = match name.strip_prefix("mcp__") {
            Some(rest) => match rest.split_once("__") {
                Some((s, t)) => (Some(s.to_string()), t.to_string()),
                None => (Some(rest.to_string()), rest.to_string()),
            },
            None => (None, name.to_string()),
        };
        let (present, past) = kind.verbs(pack);
        ToolInfo {
            kind,
            icon: kind.icon(),
            present,
            past,
            server,
            tool,
        }
    }
}

/// Status of a tool call.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CallStatus {
    /// Running.
    Running,
    /// Succeeded.
    Success,
    /// Failed.
    Error,
    /// Waiting for approval.
    Approval,
    /// Denied by a person.
    Denied,
}

impl CallStatus {
    /// Kebab id.
    pub fn id(self) -> &'static str {
        match self {
            Self::Running => "running",
            Self::Success => "success",
            Self::Error => "error",
            Self::Approval => "approval",
            Self::Denied => "denied",
        }
    }
    /// From a kebab id.
    pub fn parse(s: &str) -> Self {
        match s {
            "success" => Self::Success,
            "error" => Self::Error,
            "approval" => Self::Approval,
            "denied" => Self::Denied,
            _ => Self::Running,
        }
    }
}

/// `640ms` or `1.2s`.
pub fn ms_label(ms: u64) -> String {
    if ms < 1000 {
        format!("{ms}ms")
    } else if ms < 60_000 {
        format!("{:.1}s", ms as f64 / 1000.0)
    } else {
        short_duration(ms)
    }
}

/// One tool call card's state.
#[derive(Clone, Debug, PartialEq)]
pub struct CallCard {
    /// Tool name (mono).
    pub name: String,
    /// Summary next to the name.
    pub summary: String,
    /// Status.
    pub status: CallStatus,
    /// Arguments (pretty text).
    pub args: Option<String>,
    /// Result text.
    pub result: Option<String>,
    /// Error.
    pub error: Option<String>,
    /// Started.
    pub started: Option<ClockMs>,
    /// Final duration.
    pub duration_ms: Option<u64>,
    /// Open.
    pub open: bool,
    /// Toggled by hand (stops open-on-error).
    pub touched: bool,
}

impl CallCard {
    /// A running call.
    pub fn new(name: &str, summary: &str) -> Self {
        Self {
            name: name.into(),
            summary: summary.into(),
            status: CallStatus::Running,
            args: None,
            result: None,
            error: None,
            started: None,
            duration_ms: None,
            open: false,
            touched: false,
        }
    }
    /// Header time: live while running, final otherwise.
    pub fn time(&self, now: ClockMs) -> Option<String> {
        match (self.status, self.started, self.duration_ms) {
            (CallStatus::Running, Some(s), _) => Some(ms_label(now.0.saturating_sub(s.0))),
            (_, _, Some(d)) => Some(ms_label(d)),
            _ => None,
        }
    }
    /// Header toggle by hand.
    pub fn toggle(&mut self) {
        self.open = !self.open;
        self.touched = true;
    }
    /// The call failed: opens it unless someone toggled it.
    pub fn fail(&mut self, e: &str, ms: u64) {
        self.status = CallStatus::Error;
        self.error = Some(e.into());
        self.duration_ms = Some(ms);
        if !self.touched {
            self.open = true;
        }
    }
    /// Retry: running again, closes an auto-opened card.
    pub fn retry(&mut self, now: ClockMs) {
        self.status = CallStatus::Running;
        self.error = None;
        self.started = Some(now);
        self.duration_ms = None;
        if !self.touched {
            self.open = false;
        }
    }
}

/// Group header for several calls: "Used 4 tools · 2.4s", with a failed count.
pub fn group_label(cards: &[CallCard]) -> String {
    let n = cards.len();
    let ms: u64 = cards.iter().filter_map(|c| c.duration_ms).sum();
    let failed = cards
        .iter()
        .filter(|c| c.status == CallStatus::Error)
        .count();
    let running = cards
        .iter()
        .filter(|c| c.status == CallStatus::Running)
        .count();
    if running > 0 {
        // While anything runs the header reads as live (the view shimmers it).
        return format!("Using tools… · {} of {n} done", n - running);
    }
    let mut s = format!("Used {n} tool{}", if n == 1 { "" } else { "s" });
    if ms > 0 {
        s.push_str(&format!(" · {}", ms_label(ms)));
    }
    if failed > 0 {
        s.push_str(&format!(" · {failed} failed"));
    }
    s
}

/// A JSON value (parsed by a small built-in parser; no dependency).
#[derive(Clone, Debug, PartialEq)]
pub enum Json {
    /// null.
    Null,
    /// true / false.
    Bool(bool),
    /// A number (kept as text).
    Num(String),
    /// A string.
    Str(String),
    /// An array.
    Arr(Vec<Json>),
    /// An object, in order.
    Obj(Vec<(String, Json)>),
}

impl Json {
    /// Parses JSON text.
    pub fn parse(s: &str) -> Option<Json> {
        let mut p = Parser {
            b: s.as_bytes(),
            i: 0,
        };
        let v = p.value()?;
        p.ws();
        (p.i == p.b.len()).then_some(v)
    }
    /// Type name.
    pub fn kind(&self) -> &'static str {
        match self {
            Json::Null => "null",
            Json::Bool(_) => "boolean",
            Json::Num(_) => "number",
            Json::Str(_) => "string",
            Json::Arr(_) => "array",
            Json::Obj(_) => "object",
        }
    }
    /// Short preview: `"text"`, `42`, `[3]`, `{2}`.
    pub fn preview(&self) -> String {
        match self {
            Json::Null => "null".into(),
            Json::Bool(b) => b.to_string(),
            Json::Num(n) => n.clone(),
            Json::Str(s) => format!("\"{s}\""),
            Json::Arr(a) => format!("[{}]", a.len()),
            Json::Obj(o) => format!("{{{}}}", o.len()),
        }
    }
    /// Pretty text (2-space indent).
    pub fn pretty(&self, indent: usize) -> String {
        let pad = "  ".repeat(indent + 1);
        let end = "  ".repeat(indent);
        match self {
            Json::Arr(a) if !a.is_empty() => format!(
                "[\n{}\n{end}]",
                a.iter()
                    .map(|v| format!("{pad}{}", v.pretty(indent + 1)))
                    .collect::<Vec<_>>()
                    .join(",\n")
            ),
            Json::Obj(o) if !o.is_empty() => format!(
                "{{\n{}\n{end}}}",
                o.iter()
                    .map(|(k, v)| format!("{pad}\"{k}\": {}", v.pretty(indent + 1)))
                    .collect::<Vec<_>>()
                    .join(",\n")
            ),
            Json::Arr(_) => "[]".into(),
            Json::Obj(_) => "{}".into(),
            v => v.preview(),
        }
    }
}

struct Parser<'a> {
    b: &'a [u8],
    i: usize,
}

impl Parser<'_> {
    fn ws(&mut self) {
        while self.i < self.b.len() && (self.b[self.i] as char).is_ascii_whitespace() {
            self.i += 1;
        }
    }
    fn lit(&mut self, s: &str) -> bool {
        if self.b[self.i..].starts_with(s.as_bytes()) {
            self.i += s.len();
            true
        } else {
            false
        }
    }
    fn string(&mut self) -> Option<String> {
        self.i += 1;
        let mut out = String::new();
        let start = self.i;
        while self.i < self.b.len() {
            match self.b[self.i] {
                b'"' => {
                    out.push_str(std::str::from_utf8(&self.b[start..self.i]).ok()?);
                    self.i += 1;
                    return Some(out.replace("\\n", "\n").replace("\\\"", "\""));
                }
                b'\\' => self.i += 2,
                _ => self.i += 1,
            }
        }
        None
    }
    fn value(&mut self) -> Option<Json> {
        self.ws();
        let c = *self.b.get(self.i)?;
        match c {
            b'{' => {
                self.i += 1;
                let mut o = Vec::new();
                loop {
                    self.ws();
                    if self.lit("}") {
                        return Some(Json::Obj(o));
                    }
                    if self.b.get(self.i) != Some(&b'"') {
                        return None;
                    }
                    let k = self.string()?;
                    self.ws();
                    if !self.lit(":") {
                        return None;
                    }
                    o.push((k, self.value()?));
                    self.ws();
                    self.lit(",");
                }
            }
            b'[' => {
                self.i += 1;
                let mut a = Vec::new();
                loop {
                    self.ws();
                    if self.lit("]") {
                        return Some(Json::Arr(a));
                    }
                    a.push(self.value()?);
                    self.ws();
                    self.lit(",");
                }
            }
            b'"' => self.string().map(Json::Str),
            b't' if self.lit("true") => Some(Json::Bool(true)),
            b'f' if self.lit("false") => Some(Json::Bool(false)),
            b'n' if self.lit("null") => Some(Json::Null),
            _ => {
                let s = self.i;
                while self.i < self.b.len() && b"+-.eE0123456789".contains(&self.b[self.i]) {
                    self.i += 1;
                }
                (self.i > s)
                    .then(|| Json::Num(String::from_utf8_lossy(&self.b[s..self.i]).into_owned()))
            }
        }
    }
}

/// One visible row of a [`JsonTree`].
#[derive(Clone, Debug, PartialEq)]
pub struct JRow {
    /// Path (`root.users[0].name`).
    pub path: String,
    /// Key or index label.
    pub key: String,
    /// Depth.
    pub depth: usize,
    /// Value preview.
    pub preview: String,
    /// Type.
    pub kind: &'static str,
    /// Branch (object or array).
    pub branch: bool,
    /// Open.
    pub open: bool,
    /// Matches the query.
    pub hit: bool,
    /// "Show n more" row for a paged branch (`path` of the parent).
    pub more: Option<usize>,
}

/// A JSON tree view.
#[derive(Clone, Debug, PartialEq)]
pub struct JsonTree {
    /// Root value.
    pub root: Json,
    /// Root name.
    pub root_name: String,
    /// Open paths.
    pub open: Vec<String>,
    /// Query.
    pub query: String,
    /// Children per page.
    pub page: usize,
    /// Extra pages shown per path.
    pub pages: Vec<(String, usize)>,
    /// Current row index.
    pub current: usize,
}

impl JsonTree {
    /// A tree with the root open.
    pub fn new(root: Json) -> Self {
        Self {
            root,
            root_name: "root".into(),
            open: vec!["root".into()],
            query: String::new(),
            page: 50,
            pages: Vec::new(),
            current: 0,
        }
    }
    /// A tree with every branch open down to `depth` levels (1 opens the root only).
    pub fn with_depth(root: Json, depth: usize) -> Self {
        let mut t = Self::new(root);
        t.open.clear();
        fn walk(v: &Json, path: String, d: usize, max: usize, out: &mut Vec<String>) {
            if d >= max {
                return;
            }
            match v {
                Json::Arr(a) => {
                    out.push(path.clone());
                    for (i, x) in a.iter().enumerate() {
                        walk(x, format!("{path}[{i}]"), d + 1, max, out);
                    }
                }
                Json::Obj(o) => {
                    out.push(path.clone());
                    for (k, x) in o {
                        walk(x, format!("{path}.{k}"), d + 1, max, out);
                    }
                }
                _ => {}
            }
        }
        let name = t.root_name.clone();
        walk(&t.root, name, 0, depth, &mut t.open);
        t
    }
    /// The value at a row's path, pretty printed (for copy value).
    pub fn value_text(&self, path: &str) -> Option<String> {
        let rest = path.strip_prefix(&self.root_name)?;
        let mut v = &self.root;
        let mut it = rest.chars().peekable();
        while let Some(c) = it.next() {
            match c {
                '.' => {
                    let mut k = String::new();
                    while let Some(&n) = it.peek() {
                        if n == '.' || n == '[' {
                            break;
                        }
                        k.push(n);
                        it.next();
                    }
                    v = match v {
                        Json::Obj(o) => &o.iter().find(|(x, _)| *x == k)?.1,
                        _ => return None,
                    };
                }
                '[' => {
                    let mut n = String::new();
                    for d in it.by_ref() {
                        if d == ']' {
                            break;
                        }
                        n.push(d);
                    }
                    v = match v {
                        Json::Arr(a) => a.get(n.parse::<usize>().ok()?)?,
                        _ => return None,
                    };
                }
                _ => return None,
            }
        }
        Some(match v {
            Json::Str(s) => s.clone(),
            other => other.pretty(0),
        })
    }
    fn hit(&self, key: &str, v: &Json) -> bool {
        let q = self.query.to_lowercase();
        !q.is_empty()
            && (key.to_lowercase().contains(&q) || v.preview().to_lowercase().contains(&q))
    }
    fn contains_hit(&self, key: &str, v: &Json) -> bool {
        self.hit(key, v)
            || match v {
                Json::Arr(a) => a.iter().any(|x| self.contains_hit("", x)),
                Json::Obj(o) => o.iter().any(|(k, x)| self.contains_hit(k, x)),
                _ => false,
            }
    }
    fn walk(&self, key: String, path: String, v: &Json, depth: usize, out: &mut Vec<JRow>) {
        let branch = matches!(v, Json::Arr(_) | Json::Obj(_));
        let searching = !self.query.is_empty();
        if searching && !self.contains_hit(&key, v) {
            return;
        }
        let open = branch && (self.open.contains(&path) || searching);
        out.push(JRow {
            path: path.clone(),
            key: key.clone(),
            depth,
            preview: v.preview(),
            kind: v.kind(),
            branch,
            open,
            hit: self.hit(&key, v),
            more: None,
        });
        if !open {
            return;
        }
        let extra = self
            .pages
            .iter()
            .find(|(p, _)| *p == path)
            .map(|x| x.1)
            .unwrap_or(0);
        let limit = self.page * (1 + extra);
        let kids: Vec<(String, String, &Json)> = match v {
            Json::Arr(a) => a
                .iter()
                .enumerate()
                .map(|(i, x)| (format!("{i}"), format!("{path}[{i}]"), x))
                .collect(),
            Json::Obj(o) => o
                .iter()
                .map(|(k, x)| (k.clone(), format!("{path}.{k}"), x))
                .collect(),
            _ => vec![],
        };
        let n = kids.len();
        for (k, p, x) in kids.into_iter().take(limit) {
            self.walk(k, p, x, depth + 1, out);
        }
        if n > limit {
            out.push(JRow {
                path: path.clone(),
                key: String::new(),
                depth: depth + 1,
                preview: format!("Show {} more", (n - limit).min(self.page)),
                kind: "more",
                branch: false,
                open: false,
                hit: false,
                more: Some(n - limit),
            });
        }
    }
    /// Visible rows.
    pub fn rows(&self) -> Vec<JRow> {
        let mut out = Vec::new();
        self.walk(
            self.root_name.clone(),
            self.root_name.clone(),
            &self.root,
            0,
            &mut out,
        );
        out
    }
    /// Toggles a branch.
    pub fn toggle(&mut self, path: &str) {
        if let Some(i) = self.open.iter().position(|p| p == path) {
            self.open.remove(i);
        } else {
            self.open.push(path.into());
        }
    }
    /// Shows the next page of a branch.
    pub fn more(&mut self, path: &str) {
        match self.pages.iter_mut().find(|(p, _)| p == path) {
            Some(x) => x.1 += 1,
            None => self.pages.push((path.into(), 1)),
        }
    }
    /// Keyboard: ↓ ↑ → ← Home End.
    pub fn key(&mut self, k: &str) {
        let rows = self.rows();
        if rows.is_empty() {
            return;
        }
        let i = self.current.min(rows.len() - 1);
        match k {
            "down" => self.current = (i + 1).min(rows.len() - 1),
            "up" => self.current = i.saturating_sub(1),
            "home" => self.current = 0,
            "end" => self.current = rows.len() - 1,
            "right" => {
                let r = &rows[i];
                if r.branch && !r.open {
                    self.toggle(&r.path.clone());
                } else if r.branch {
                    self.current = (i + 1).min(rows.len() - 1);
                }
            }
            "left" => {
                let r = &rows[i];
                if r.branch && r.open {
                    self.toggle(&r.path.clone());
                } else if let Some(p) = rows[..i].iter().rposition(|x| x.depth < r.depth) {
                    self.current = p;
                }
            }
            // Enter / Space: toggle a branch, or load the next page on a Show more row.
            "toggle" => {
                let r = rows[i].clone();
                if r.more.is_some() {
                    self.more(&r.path);
                } else if r.branch {
                    self.toggle(&r.path);
                }
            }
            // Enter / Shift+Enter in search: next / previous match, wrapping.
            "next" | "prev" => {
                let hits: Vec<usize> = (0..rows.len()).filter(|&j| rows[j].hit).collect();
                if hits.is_empty() {
                    return;
                }
                self.current = if k == "next" {
                    *hits.iter().find(|&&j| j > i).unwrap_or(&hits[0])
                } else {
                    *hits
                        .iter()
                        .rev()
                        .find(|&&j| j < i)
                        .unwrap_or(&hits[hits.len() - 1])
                };
            }
            _ => {
                if let Some(n) = k.strip_prefix('#').and_then(|n| n.parse::<usize>().ok()) {
                    self.current = n.min(rows.len() - 1);
                }
            }
        }
    }
}

/// A pasted link's preview.
#[derive(Clone, Debug, PartialEq)]
pub struct Preview {
    /// Site name.
    pub site: String,
    /// Title.
    pub title: String,
    /// Description.
    pub description: String,
    /// Has an image.
    pub image: bool,
}

/// State of a link in a message.
#[derive(Clone, Debug, PartialEq)]
pub enum UnfurlState {
    /// Resolving (shimmer on the link).
    Loading,
    /// No preview: the link stays as text.
    Plain,
    /// Resolved and shown as a card.
    Card(Preview),
    /// Resolved, folded back to an inline link with favicon and title.
    Inline(Preview),
}

/// A link in a composer.
#[derive(Clone, Debug, PartialEq)]
pub struct Unfurl {
    /// URL.
    pub url: String,
    /// State.
    pub state: UnfurlState,
    /// Pasted at.
    pub at: ClockMs,
}

impl Unfurl {
    /// Host of the URL.
    pub fn host(&self) -> &str {
        let s = self.url.split_once("://").map(|x| x.1).unwrap_or(&self.url);
        s.split('/').next().unwrap_or(s)
    }
    /// Resolves (`auto_expand` opens the card).
    pub fn resolve(&mut self, p: Option<Preview>, auto_expand: bool) {
        self.state = match p {
            None => UnfurlState::Plain,
            Some(p) if auto_expand => UnfurlState::Card(p),
            Some(p) => UnfurlState::Inline(p),
        };
    }
    /// Collapse a card / expand an inline link.
    pub fn toggle(&mut self) {
        self.state = match std::mem::replace(&mut self.state, UnfurlState::Loading) {
            UnfurlState::Card(p) => UnfurlState::Inline(p),
            UnfurlState::Inline(p) => UnfurlState::Card(p),
            s => s,
        };
    }
}

/// Finds http(s) URLs in text.
pub fn find_urls(text: &str) -> Vec<String> {
    text.split_whitespace()
        .filter(|w| w.starts_with("https://") || w.starts_with("http://"))
        .map(|w| w.trim_end_matches([',', '.', ')']).to_string())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kind_map() {
        let m = ToolKindMap::standard().with("deploy_vercel", ToolKind::Shell);
        assert_eq!(m.kind("MultiEdit"), ToolKind::Edit);
        assert_eq!(m.kind("apply_patch"), ToolKind::Edit);
        assert_eq!(m.kind("deploy_vercel"), ToolKind::Shell);
        let i = m.info("mcp__github__create_issue", VerbPack::neutral());
        assert_eq!(i.server.as_deref(), Some("github"));
        assert_eq!(i.tool, "create_issue");
        assert_eq!(i.kind, ToolKind::Mcp);
        let p = ToolKindMap::default().with_prefix("db_", ToolKind::Search);
        assert_eq!(p.kind("db_query"), ToolKind::Search);
    }

    #[test]
    fn card_open_on_error() {
        let mut c = CallCard::new("search_web", "pricing pages");
        c.started = Some(ClockMs(0));
        assert_eq!(c.time(ClockMs(1200)).unwrap(), "1.2s");
        c.fail("timeout", 3000);
        assert!(c.open);
        c.retry(ClockMs(4000));
        assert!(!c.open);
        c.toggle();
        c.fail("again", 10);
        assert!(c.open && c.touched);
        let mut d = CallCard::new("x", "");
        d.status = CallStatus::Success;
        d.duration_ms = Some(640);
        assert_eq!(group_label(&[c, d]), "Used 2 tools · 650ms · 1 failed");
    }

    #[test]
    fn json_tree() {
        let j =
            Json::parse(r#"{"users":[{"name":"Ada","age":36},{"name":"Lin"}],"ok":true,"n":null}"#)
                .unwrap();
        let mut t = JsonTree::new(j);
        assert_eq!(t.rows().len(), 4);
        t.toggle("root.users");
        assert_eq!(t.rows()[2].path, "root.users[0]");
        t.query = "lin".into();
        let r = t.rows();
        assert!(r.iter().any(|x| x.hit && x.path == "root.users[1].name"));
        t.query.clear();
        t.page = 1;
        assert!(t.rows().iter().any(|x| x.more == Some(1)));
        t.more("root.users");
        // The users array is fully shown; the root (3 keys at one per page) still pages.
        assert!(
            t.rows()
                .iter()
                .all(|x| x.more.is_none() || x.path != "root.users")
        );
        t.more("root");
        t.more("root");
        assert!(t.rows().iter().all(|x| x.more.is_none()));
        t.current = 0;
        t.key("down");
        assert_eq!(t.current, 1);
        assert!(t.root.pretty(0).contains("\"name\": \"Ada\""));
        assert_eq!(t.value_text("root.users[1].name").as_deref(), Some("Lin"));
        assert_eq!(t.value_text("root.ok").as_deref(), Some("true"));
        t.key("#0");
        assert_eq!(t.current, 0);
        t.query = "a".into();
        t.key("next");
        assert!(t.rows()[t.current].hit);
        let d = JsonTree::with_depth(t.root.clone(), 2);
        assert!(d.open.contains(&"root.users".to_string()));
        assert!(!d.open.contains(&"root.users[0]".to_string()));
    }

    #[test]
    fn unfurl() {
        let u = find_urls("see https://airbnb.com/rooms/12, thanks");
        assert_eq!(u, vec!["https://airbnb.com/rooms/12"]);
        let mut f = Unfurl {
            url: u[0].clone(),
            state: UnfurlState::Loading,
            at: ClockMs(0),
        };
        assert_eq!(f.host(), "airbnb.com");
        f.resolve(
            Some(Preview {
                site: "Airbnb".into(),
                title: "t".into(),
                description: "d".into(),
                image: true,
            }),
            true,
        );
        f.toggle();
        assert!(matches!(f.state, UnfurlState::Inline(_)));
        assert_eq!(ms_label(640), "640ms");
    }
}
