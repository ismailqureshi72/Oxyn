//! Screens S11 (Agent apps III): **Workspace Coder**, a desktop agent app in the chat-plus-computer
//! pattern. Chats in the sidebar; a conversation where the agent keeps a todo list and a tool log
//! while it works in its own workspace; a Workspace pane beside it (Files, Terminal, Browser,
//! Preview) showing what it does; and a finished deliverable you preview at desktop or phone width
//! and publish. Mode (Chat, Researcher, Workspace), model and Thinking in the composer. Our own name
//! and design, inspired by how such apps behave. Every name, file, number and company is fictional.

pub use super::cmdstudio::{
    PERSON, PERSON_EMAIL, PERSON_INI, PERSON_PLAN, ScreenState, StateList, prompt_error,
};

/// The app's name.
pub const WC_APP: &str = "Workspace Coder";

/// The state bar of Workspace Coder.
pub const WC_STATES: StateList = &[
    ("sample", "Data"),
    ("loading", "Loading"),
    ("empty", "Empty"),
    ("preview", "Preview"),
    ("files", "Files"),
];

/// Split a state key into the data state and the own state (`preview`, `files` or "").
pub fn wc_split(k: &str) -> (ScreenState, &'static str) {
    match k {
        "preview" => (ScreenState::Sample, "preview"),
        "files" => (ScreenState::Sample, "files"),
        _ => (ScreenState::from_key(k), ""),
    }
}

/// Models: name and brand mark id.
pub const WC_MODELS: [(&str, &str); 3] = [
    ("Kimi K3", "kimi"),
    ("DeepSeek V4", "deepseek"),
    ("Qwen 4 Max", "qwen"),
];

/// Modes: label, line and glyph.
pub const WC_MODES: [(&str, &str, &str); 3] = [
    ("Chat", "Quick answers", "chat"),
    ("Researcher", "Reads many sources, writes a report", "scope"),
    (
        "Workspace",
        "Works in its own computer, builds files",
        "laptop",
    ),
];

/// The workspace pane's tabs: key and label.
pub const WS_TABS: [(&str, &str); 4] = [
    ("files", "Files"),
    ("terminal", "Terminal"),
    ("browser", "Browser"),
    ("preview", "Preview"),
];

/// Tab index after an arrow key (wraps).
pub fn tab_step(i: usize, forward: bool) -> usize {
    let n = WS_TABS.len();
    if forward {
        (i + 1) % n
    } else {
        (i + n - 1) % n
    }
}

/// A chat in the sidebar.
#[derive(Clone, Debug, PartialEq)]
pub struct Chat {
    /// Stable id.
    pub id: u32,
    /// Title.
    pub title: String,
    /// Index into [`WC_MODES`].
    pub mode: usize,
    /// Working right now.
    pub running: bool,
    /// Day group ("Today", "Earlier").
    pub day: &'static str,
}

fn ch(id: u32, title: &str, mode: usize, running: bool, day: &'static str) -> Chat {
    Chat {
        id,
        title: title.to_string(),
        mode,
        running,
        day,
    }
}

/// The sample chats (empty for loading and empty).
pub fn wc_chats(st: ScreenState) -> Vec<Chat> {
    if st != ScreenState::Sample {
        return Vec::new();
    }
    vec![
        ch(1, "Moving budget for three cities", 2, true, "Today"),
        ch(2, "Summarise the solar grant rules", 1, false, "Today"),
        ch(3, "Slides for the Q3 field review", 2, false, "Earlier"),
        ch(4, "Regex for invoice numbers", 0, false, "Earlier"),
        ch(5, "Compare three e-bike plans", 1, false, "Earlier"),
    ]
}

/// Start a chat from a prompt at the top of the list; it runs in Workspace mode when picked.
pub fn add_chat(list: &mut Vec<Chat>, prompt: &str, mode: usize) -> Result<u32, &'static str> {
    if let Some(e) = prompt_error(prompt) {
        return Err(e);
    }
    let id = list.iter().map(|c| c.id).max().unwrap_or(0) + 1;
    let t = super::taskdeck::title_from(prompt);
    list.insert(0, ch(id, &t, mode.min(2), mode == 2, "Today"));
    Ok(id)
}

/// The open chat's prompt.
pub const WC_PROMPT: &str = "I'm moving for work and can pick Lisbon, Porto or Valencia. Compare a month's costs for one person and build me a small page with the numbers and a chart that I can share.";

/// The agent's opening line.
pub const WC_INTRO: &str = "I'll gather typical monthly costs for each city, put them in a data file, then build a one-page site with a chart you can share.";

/// The agent's closing line.
pub const WC_DONE: &str = "Done. Porto comes out cheapest at about €1,520 a month, mostly thanks to rent. The page has the table, a chart and the sources; preview it beside the chat or publish a link.";

/// The todo list: text. Steps before `at` are done; `at` runs.
pub const WC_TODOS: [&str; 6] = [
    "Collect rent, food, transport and bills for each city",
    "Write data/costs.csv",
    "Build the page: table and summary",
    "Add the bar chart",
    "Check it at phone width",
    "Make a preview link",
];

/// The step the sample is on while running.
pub const WC_AT: usize = 3;

/// Todo progress: done and total at a step (`done` means all finished).
pub fn todo_count(at: usize, done: bool) -> (usize, usize) {
    let n = WC_TODOS.len();
    if done { (n, n) } else { (at.min(n), n) }
}

/// The tool log: glyph, verb, object.
pub const WC_LOG: [(&str, &str, &str); 6] = [
    (
        "globe",
        "Searched",
        "monthly cost of living Lisbon Porto Valencia",
    ),
    ("web", "Read", "citycosts.example/portugal"),
    ("web", "Read", "rentindex.example/valencia"),
    ("pen", "Wrote", "data/costs.csv"),
    ("pen", "Wrote", "index.html"),
    ("term", "Ran", "python3 check.py"),
];

/// How many log lines show at a step.
pub fn log_upto(at: usize, done: bool) -> usize {
    if done {
        WC_LOG.len()
    } else {
        (at + 2).min(WC_LOG.len())
    }
}

/// A file in the workspace.
#[derive(Clone, Debug, PartialEq)]
pub struct WsFile {
    /// Path.
    pub path: &'static str,
    /// Lines.
    pub lines: Vec<&'static str>,
}

/// The sample workspace files.
pub fn ws_files(st: ScreenState) -> Vec<WsFile> {
    if st != ScreenState::Sample {
        return Vec::new();
    }
    vec![
        WsFile {
            path: "index.html",
            lines: vec![
                "<!doctype html>",
                "<title>Move Budget</title>",
                "<link rel=\"stylesheet\" href=\"styles.css\">",
                "<h1>A month in three cities</h1>",
                "<table id=\"costs\"></table>",
                "<canvas id=\"chart\"></canvas>",
                "<script src=\"app.js\"></script>",
            ],
        },
        WsFile {
            path: "app.js",
            lines: vec![
                "const rows = await load('data/costs.csv');",
                "renderTable(rows, '#costs');",
                "renderBars(rows.map(r => r.total), '#chart');",
            ],
        },
        WsFile {
            path: "styles.css",
            lines: vec![
                "body { font: 16px/1.5 system-ui; margin: 2rem; }",
                "table { border-collapse: collapse; }",
                "td, th { padding: .4rem .8rem; }",
            ],
        },
        WsFile {
            path: "data/costs.csv",
            lines: vec![
                "city,rent,food,transport,bills,total",
                "Lisbon,1250,380,40,150,1820",
                "Porto,980,360,40,140,1520",
                "Valencia,1050,340,45,155,1590",
            ],
        },
        WsFile {
            path: "check.py",
            lines: vec![
                "import csv",
                "rows = list(csv.DictReader(open('data/costs.csv')))",
                "assert all(int(r['total']) == sum(int(r[k]) for k in ('rent','food','transport','bills')) for r in rows)",
                "print('ok', len(rows), 'cities')",
            ],
        },
    ]
}

/// The terminal transcript: (is a command, text).
pub const WS_TERM: [(bool, &str); 5] = [
    (true, "python3 check.py"),
    (false, "ok 3 cities"),
    (true, "npx serve . -l 4173"),
    (false, "Serving on http://localhost:4173"),
    (
        false,
        "GET / 200 · GET /app.js 200 · GET /data/costs.csv 200",
    ),
];

/// The browser tab: address and results (title, site, line).
pub const WS_BROWSER: (&str, [(&str, &str, &str); 3]) = (
    "search.example/?q=monthly+cost+of+living+porto",
    [
        (
            "Porto: cost of living for one person",
            "citycosts.example",
            "Rent for a one-bed in the centre averages €980; groceries about €360.",
        ),
        (
            "Rent index, Iberia, autumn",
            "rentindex.example",
            "Valencia one-bed rents rose 6% on the year to about €1,050.",
        ),
        (
            "Monthly transit passes compared",
            "transitguide.example",
            "Unlimited passes run €40 in Lisbon and Porto and €45 in Valencia.",
        ),
    ],
);

/// The deliverable's cities: name and monthly total (€).
pub fn preview_rows(files: &[WsFile]) -> Vec<(String, u32)> {
    files
        .iter()
        .find(|f| f.path.ends_with(".csv"))
        .map(|f| {
            f.lines
                .iter()
                .skip(1)
                .filter_map(|l| {
                    let c: Vec<&str> = l.split(',').collect();
                    Some((c.first()?.to_string(), c.last()?.parse().ok()?))
                })
                .collect()
        })
        .unwrap_or_default()
}

/// The cheapest row's index.
pub fn cheapest(rows: &[(String, u32)]) -> Option<usize> {
    (0..rows.len()).min_by_key(|i| rows[*i].1)
}

/// The preview link for a title ("Move Budget" → move-budget.pages.example).
pub fn publish_url(title: &str) -> String {
    let slug: String = title
        .to_lowercase()
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '-' })
        .collect();
    let slug = slug
        .split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("-");
    format!("{slug}.pages.example")
}

// ─────────────────────────────────────────────────────────── Data ──

/// Everything Workspace Coder shows. Pass it to the screen's `data` prop; `Default` is the sample.
#[derive(Clone, Debug, PartialEq)]
pub struct WorkspaceCoderData {
    /// Chats in the sidebar; the first one opens.
    pub chats: Vec<Chat>,
    /// The workspace's files (Files and Preview read them).
    pub files: Vec<WsFile>,
    /// The deliverable's title.
    pub deliverable: String,
}

impl WorkspaceCoderData {
    /// The data for a demo state (sample, or nothing for empty and loading).
    pub fn for_state(st: ScreenState) -> Self {
        Self {
            chats: wc_chats(st),
            files: ws_files(st),
            deliverable: "Move Budget".to_string(),
        }
    }
}

impl Default for WorkspaceCoderData {
    fn default() -> Self {
        Self::for_state(ScreenState::Sample)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn s11_workspace_coder() {
        assert_eq!(wc_split("files").1, "files");
        assert_eq!(tab_step(3, true), 0);
        assert_eq!(tab_step(0, false), 3);
        let mut l = wc_chats(ScreenState::Sample);
        assert!(add_chat(&mut l, "", 2).is_err());
        let id = add_chat(&mut l, "Plan a week of dinners. Then shop.", 2).unwrap();
        assert_eq!(l[0].id, id);
        assert!(l[0].running);
        assert_eq!(todo_count(WC_AT, false), (3, 6));
        assert_eq!(todo_count(WC_AT, true), (6, 6));
        assert_eq!(log_upto(WC_AT, false), 5);
        let d = WorkspaceCoderData::default();
        let rows = preview_rows(&d.files);
        assert_eq!(rows.len(), 3);
        assert_eq!(cheapest(&rows), Some(1));
        assert_eq!(publish_url("Move Budget!"), "move-budget.pages.example");
        assert!(
            WorkspaceCoderData::for_state(ScreenState::Loading)
                .files
                .is_empty()
        );
    }
}
