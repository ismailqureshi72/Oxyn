//! MCP servers: the Model Context Protocol servers that give an agent its tools. The header says
//! how many context tokens the switched-on tools take, over a stacked budget bar split per server and a
//! budget past which the overflow turns amber. Each server row has its mark, a status word
//! (Connected, Needs sign in, Error, Connecting), its transport and the tools it has on; open, it
//! lists every tool with a switch, read or write and its token cost, the default for tool calls,
//! the endpoint and Remove (with Undo). Add server opens a sheet: a searchable directory, a
//! simulated authorize step, tools discovered one by one, then the server joins the list.

use super::{norm_key, nth, rove, thousands};

/// A server's status.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SrvSt {
    /// Connected.
    Connected,
    /// Needs sign in.
    NeedsAuth,
    /// Error.
    Error,
    /// Connecting (after Retry or sign in).
    Connecting,
}

impl SrvSt {
    /// Word and hue.
    pub fn label(self) -> (&'static str, &'static str) {
        match self {
            SrvSt::Connected => ("Connected", "green"),
            SrvSt::NeedsAuth => ("Needs sign in", "amber"),
            SrvSt::Error => ("Error", "red"),
            SrvSt::Connecting => ("Connecting", "blue"),
        }
    }
}

/// One tool: name, writes, tokens, on.
#[derive(Clone, Debug, PartialEq)]
pub struct Tool {
    /// Tool id.
    pub name: &'static str,
    /// Changes something (else read only).
    pub write: bool,
    /// Tokens its definition costs.
    pub tokens: u64,
    /// Enabled.
    pub on: bool,
}

/// One server.
#[derive(Clone, Debug, PartialEq)]
pub struct Server {
    /// Name.
    pub name: &'static str,
    /// Brand id.
    pub brand: &'static str,
    /// Local (stdio) rather than remote.
    pub local: bool,
    /// Endpoint or command.
    pub endpoint: &'static str,
    /// Base status.
    pub st: SrvSt,
    /// Error text.
    pub error: &'static str,
    /// Always allow reads (else ask every time).
    pub allow_reads: bool,
    /// Tools.
    pub tools: Vec<Tool>,
    /// Connecting since (Retry), host ms.
    pub retry: Option<u64>,
}

fn tl(list: &[(&'static str, bool, u64, bool)]) -> Vec<Tool> {
    list.iter()
        .map(|&(name, write, tokens, on)| Tool {
            name,
            write,
            tokens,
            on,
        })
        .collect()
}

/// A directory entry: name, brand, one line, needs OAuth, tools (name, writes, tokens).
pub type Entry = (
    &'static str,
    &'static str,
    &'static str,
    bool,
    &'static [(&'static str, bool, u64)],
);

/// The directory behind Add server.
pub const DIRECTORY: [Entry; 5] = [
    (
        "Stripe",
        "stripe",
        "Customers, invoices and refunds",
        true,
        &[
            ("list_customers", false, 380),
            ("get_invoice", false, 310),
            ("create_refund", true, 440),
        ],
    ),
    (
        "Slack",
        "slack",
        "Read channels and post messages",
        true,
        &[("search_messages", false, 420), ("post_message", true, 360)],
    ),
    (
        "Figma",
        "figma",
        "Frames, components and comments",
        true,
        &[("get_file", false, 450), ("list_comments", false, 300)],
    ),
    (
        "Supabase",
        "supabase",
        "Tables, SQL and storage",
        true,
        &[
            ("list_tables", false, 280),
            ("execute_sql", true, 520),
            ("list_buckets", false, 260),
        ],
    ),
    (
        "Filesystem",
        "",
        "Read and write files in one folder (local)",
        false,
        &[("read_file", false, 240), ("write_file", true, 260)],
    ),
];
/// The context budget for tool definitions, tokens.
pub const BUDGET: u64 = 5_500;
/// Authorizing (simulated), ms.
pub const AUTH_MS: u64 = 1_200;
/// One tool discovered, ms.
pub const TOOL_MS: u64 = 450;
/// Connecting after Retry, ms.
pub const RETRY_MS: u64 = 1_500;
/// Undo window, ms.
pub const UNDO_MS: u64 = 6_000;

/// The Add server sheet.
#[derive(Clone, Debug, PartialEq)]
pub enum Sheet {
    /// Browsing the directory with a search.
    Browse(String),
    /// Directory entry chosen: authorize screen.
    Authorize(usize),
    /// Authorized at, discovering tools.
    Discover(usize, u64),
}

/// The live state of the block.
#[derive(Clone, Debug, PartialEq)]
pub struct Mcp {
    /// Servers.
    pub servers: Vec<Server>,
    /// Open row.
    pub open: Option<usize>,
    /// Focused row.
    pub focus: usize,
    /// The sheet, when open.
    pub sheet: Option<Sheet>,
    /// The last removed server with its place and time (for Undo).
    pub removed: Option<(usize, Server, u64)>,
    /// Server signing in through the sheet (its index), if any.
    pub signing: Option<usize>,
    /// Directory entries already added.
    pub added: Vec<usize>,
}

impl Mcp {
    /// The sample agent's servers.
    pub fn new() -> Self {
        let srv = |name, brand, local, endpoint, st, error, tools| Server {
            name,
            brand,
            local,
            endpoint,
            st,
            error,
            allow_reads: true,
            tools,
            retry: None,
        };
        Self {
            servers: vec![
                srv(
                    "GitHub",
                    "github",
                    false,
                    "https://mcp.example-git.dev/sse",
                    SrvSt::Connected,
                    "",
                    tl(&[
                        ("list_issues", false, 420, true),
                        ("get_pull_request", false, 380, true),
                        ("search_code", false, 510, true),
                        ("create_issue", true, 460, true),
                        ("comment_on_pr", true, 400, true),
                        ("merge_pull_request", true, 520, false),
                    ]),
                ),
                srv(
                    "Linear",
                    "linear",
                    false,
                    "https://mcp.example-issues.dev/sse",
                    SrvSt::Connected,
                    "",
                    tl(&[
                        ("list_issues", false, 390, true),
                        ("get_issue", false, 300, true),
                        ("create_issue", true, 450, true),
                        ("update_status", true, 410, true),
                    ]),
                ),
                srv(
                    "Notion",
                    "notion",
                    false,
                    "https://mcp.example-docs.dev/mcp",
                    SrvSt::Connected,
                    "",
                    tl(&[
                        ("search", false, 470, true),
                        ("get_page", false, 350, true),
                        ("append_block", true, 430, false),
                    ]),
                ),
                srv(
                    "Sentry",
                    "sentry",
                    false,
                    "https://mcp.example-errors.dev/sse",
                    SrvSt::NeedsAuth,
                    "",
                    tl(&[
                        ("list_issues", false, 360, true),
                        ("get_event", false, 420, true),
                        ("resolve_issue", true, 380, true),
                    ]),
                ),
                srv(
                    "Postgres",
                    "postgresql",
                    true,
                    "npx pg-mcp postgres://localhost:5432/app",
                    SrvSt::Error,
                    "Connection refused on localhost:5432",
                    tl(&[
                        ("query", false, 610, true),
                        ("list_tables", false, 280, true),
                    ]),
                ),
            ],
            open: None,
            focus: 0,
            sheet: None,
            removed: None,
            signing: None,
            added: Vec::new(),
        }
    }

    /// Status of server `i` at `now`.
    pub fn st(&self, i: usize, now: u64) -> SrvSt {
        let s = &self.servers[i];
        match s.retry {
            Some(t) if now < t + RETRY_MS => SrvSt::Connecting,
            Some(_) => SrvSt::Connected,
            None => s.st,
        }
    }

    /// Tokens server `i` costs at `now` (counts switched-on tools of healthy servers only).
    pub fn cost(&self, i: usize, now: u64) -> u64 {
        if self.st(i, now) != SrvSt::Connected {
            return 0;
        }
        self.servers[i]
            .tools
            .iter()
            .filter(|t| t.on)
            .map(|t| t.tokens)
            .sum()
    }

    /// Tools on for server `i`.
    pub fn on(&self, i: usize) -> usize {
        self.servers[i].tools.iter().filter(|t| t.on).count()
    }

    /// Total tools counted and tokens at `now`.
    pub fn totals(&self, now: u64) -> (usize, u64) {
        let n = (0..self.servers.len())
            .filter(|&i| self.st(i, now) == SrvSt::Connected)
            .map(|i| self.on(i))
            .sum();
        (n, (0..self.servers.len()).map(|i| self.cost(i, now)).sum())
    }

    /// The header line (`18 tools · 6.4k tokens of context`).
    pub fn header(&self, now: u64) -> String {
        let (n, t) = self.totals(now);
        format!("{n} tools · {:.1}k tokens of context", t as f64 / 1000.0)
    }

    /// The meter's value text, with how far over budget it is.
    pub fn meter_text(&self, now: u64) -> String {
        let t = self.totals(now).1;
        let mut s = format!("{} of {} budgeted tokens", thousands(t), thousands(BUDGET));
        if t > BUDGET {
            s.push_str(&format!(", {} over", thousands(t - BUDGET)));
        }
        s
    }

    /// The sheet's directory filtered by its search (indices not yet added).
    pub fn directory(&self) -> Vec<usize> {
        let q = match &self.sheet {
            Some(Sheet::Browse(q)) => q.to_lowercase(),
            _ => String::new(),
        };
        (0..DIRECTORY.len())
            .filter(|i| !self.added.contains(i))
            .filter(|&i| {
                q.is_empty()
                    || DIRECTORY[i].0.to_lowercase().contains(&q)
                    || DIRECTORY[i].2.to_lowercase().contains(&q)
            })
            .collect()
    }

    /// Tools discovered so far in the sheet at `now`, and whether discovery finished.
    pub fn discovered(&self, now: u64) -> Option<(usize, usize, bool)> {
        match self.sheet {
            Some(Sheet::Discover(d, t)) => {
                let total = DIRECTORY[d].4.len();
                let n = ((now.saturating_sub(t) / TOOL_MS) as usize).min(total);
                Some((d, n, n == total))
            }
            _ => None,
        }
    }

    /// Sets the directory search.
    pub fn search(&mut self, q: &str) {
        self.sheet = Some(Sheet::Browse(q.to_string()));
    }

    /// Presses: `mcp-open-<i>`, `mcp-tool-<i>-<t>`, `mcp-perm-<i>-<0|1>`, `mcp-retry-<i>`,
    /// `mcp-signin-<i>`, `mcp-remove-<i>`, `mcp-undo`, `mcp-add`, `mcp-pick-<d>`, `mcp-auth`,
    /// `mcp-finish`, `mcp-close`.
    pub fn press(&mut self, id: &str, now: u64) -> Option<String> {
        if let Some(i) = nth(id, "mcp-open-") {
            self.open = if self.open == Some(i) { None } else { Some(i) };
            self.focus = i;
            return None;
        }
        if let Some(rest) = id.strip_prefix("mcp-tool-") {
            let (i, t) = rest.split_once('-')?;
            let (i, t): (usize, usize) = (i.parse().ok()?, t.parse().ok()?);
            let tool = self.servers.get_mut(i)?.tools.get_mut(t)?;
            tool.on = !tool.on;
            let (name, on) = (tool.name, tool.on);
            return Some(format!(
                "{name} {}. {}",
                if on { "on" } else { "off" },
                self.header(now)
            ));
        }
        if let Some(rest) = id.strip_prefix("mcp-perm-") {
            let (i, v) = rest.split_once('-')?;
            let s = self.servers.get_mut(i.parse::<usize>().ok()?)?;
            s.allow_reads = v == "1";
            return Some(format!(
                "{}: {}",
                s.name,
                if s.allow_reads {
                    "always allow reads"
                } else {
                    "ask every time"
                }
            ));
        }
        if let Some(i) = nth(id, "mcp-retry-") {
            if self.st(i, now) != SrvSt::Error {
                return None;
            }
            self.servers[i].retry = Some(now);
            return Some(format!("Reconnecting {}", self.servers[i].name));
        }
        if let Some(i) = nth(id, "mcp-signin-") {
            if self.st(i, now) != SrvSt::NeedsAuth {
                return None;
            }
            self.signing = Some(i);
            self.sheet = Some(Sheet::Authorize(usize::MAX));
            return Some(format!("Sign in to {}", self.servers[i].name));
        }
        if let Some(i) = nth(id, "mcp-remove-") {
            if i >= self.servers.len() {
                return None;
            }
            let s = self.servers.remove(i);
            let name = s.name;
            self.removed = Some((i, s, now));
            self.open = None;
            self.focus = self.focus.min(self.servers.len().saturating_sub(1));
            return Some(format!("{name} removed. Undo for a few seconds"));
        }
        match id {
            "mcp-undo" => {
                let (i, s, t) = self.removed.take()?;
                if now >= t + UNDO_MS {
                    return None;
                }
                let name = s.name;
                self.servers.insert(i.min(self.servers.len()), s);
                Some(format!("{name} restored"))
            }
            "mcp-add" => {
                self.sheet = Some(Sheet::Browse(String::new()));
                self.signing = None;
                Some("Add a server".into())
            }
            "mcp-auth" => match self.sheet {
                Some(Sheet::Authorize(d)) => {
                    if let Some(i) = self.signing.take() {
                        self.servers[i].retry = Some(now.saturating_sub(RETRY_MS));
                        self.servers[i].st = SrvSt::Connected;
                        self.sheet = None;
                        return Some(format!(
                            "{} connected. Simulated, no account was contacted",
                            self.servers[i].name
                        ));
                    }
                    self.sheet = Some(Sheet::Discover(d, now + AUTH_MS));
                    Some(format!(
                        "Authorized {}. Discovering tools. Simulated, no account was contacted",
                        DIRECTORY[d].0
                    ))
                }
                _ => None,
            },
            "mcp-finish" => {
                let (d, _, done) = self.discovered(now)?;
                if !done {
                    return None;
                }
                let e = &DIRECTORY[d];
                let tools =
                    e.4.iter()
                        .map(|&(name, write, tokens)| Tool {
                            name,
                            write,
                            tokens,
                            on: !write,
                        })
                        .collect();
                self.servers.insert(
                    0,
                    Server {
                        name: e.0,
                        brand: e.1,
                        local: !e.3,
                        endpoint: if e.3 {
                            "https://mcp.example.dev/sse"
                        } else {
                            "npx fs-mcp ./workspace"
                        },
                        st: SrvSt::Connected,
                        error: "",
                        allow_reads: true,
                        tools,
                        retry: None,
                    },
                );
                self.added.push(d);
                self.sheet = None;
                self.open = Some(0);
                Some(format!("{} added with {} tools", e.0, e.4.len()))
            }
            "mcp-close" => self.sheet.take().map(|_| {
                self.signing = None;
                "Closed".to_string()
            }),
            _ => {
                let d = nth(id, "mcp-pick-")?;
                if d >= DIRECTORY.len() {
                    return None;
                }
                self.sheet = Some(if DIRECTORY[d].3 {
                    Sheet::Authorize(d)
                } else {
                    Sheet::Discover(d, now)
                });
                Some(format!(
                    "{}: {}",
                    DIRECTORY[d].0,
                    if DIRECTORY[d].3 {
                        "authorize"
                    } else {
                        "discovering tools"
                    }
                ))
            }
        }
    }

    /// Keys: Up, Down, Home, End between rows; Enter or Space opens; Escape closes the sheet;
    /// `z` with Cmd undoes.
    pub fn key(&mut self, key: &str, cmd: bool, now: u64) -> Option<String> {
        let k = norm_key(key);
        match k {
            "Escape" => self.press("mcp-close", now),
            "z" | "Z" if cmd => self.press("mcp-undo", now),
            "ArrowUp" | "ArrowDown" | "Home" | "End" if self.sheet.is_none() => {
                self.focus = rove(self.focus, self.servers.len(), k);
                None
            }
            "Enter" | "Space" if self.sheet.is_none() => {
                self.press(&format!("mcp-open-{}", self.focus), now)
            }
            _ => None,
        }
    }
}

impl Default for Mcp {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn counts_toggles_and_flows() {
        let mut m = Mcp::new();
        assert_eq!(m.totals(0), (11, 4_540));
        m.press("mcp-tool-0-5", 0);
        assert_eq!(m.totals(0).1, 5_060);
        m.press("mcp-retry-4", 0);
        assert_eq!(m.st(4, 100), SrvSt::Connecting);
        assert_eq!(m.st(4, 2_000), SrvSt::Connected);
        assert!(m.totals(2_000).1 > BUDGET);
        assert!(m.meter_text(2_000).contains("over"));
        m.press("mcp-signin-3", 2_000);
        m.press("mcp-auth", 2_000);
        assert_eq!(m.st(3, 2_000), SrvSt::Connected);
        m.press("mcp-add", 0);
        m.search("pay");
        assert!(m.directory().is_empty());
        m.search("refund");
        assert_eq!(m.directory(), vec![0]);
        m.press("mcp-pick-0", 0);
        m.press("mcp-auth", 0);
        assert!(m.press("mcp-finish", 1_300).is_none());
        assert_eq!(m.discovered(3_000).map(|d| d.2), Some(true));
        m.press("mcp-finish", 3_000);
        assert_eq!(m.servers[0].name, "Stripe");
        m.press("mcp-remove-0", 4_000);
        assert_eq!(m.servers[0].name, "GitHub");
        m.key("z", true, 5_000);
        assert_eq!(m.servers[0].name, "Stripe");
        m.key("Escape", false, 5_000);
        m.key("ArrowDown", false, 5_000);
        assert_eq!(m.focus, 1);
    }
}
