//! Screens S16 (Agent apps IV): **Courier**, a coding-agent app built around threads. Every active
//! thread in the sidebar with its state and filters (All, Running, Needs you); the open thread
//! with the agent's tool lines and its edits gathered in one card per turn (files, line counts,
//! each diff on demand); suggested replies above the composer; a model and an effort in the
//! composer; and a pod, a cloud machine of the thread's own, beside it with Desktop, Ports and
//! Changes (revert a file). Our own name and design, inspired by how such apps behave. Every
//! name, file, number and company is fictional.

pub use super::cmdstudio::{
    PERSON, PERSON_EMAIL, PERSON_INI, PERSON_PLAN, ScreenState, StateList, prompt_error,
};

/// The app's name.
pub const CR_APP: &str = "Courier";

/// The state bar of Courier.
pub const CR_STATES: StateList = &[
    ("sample", "Data"),
    ("loading", "Loading"),
    ("empty", "Empty"),
    ("changes", "Changes"),
    ("desktop", "Pod desktop"),
];

/// Split a state key into the data state and the own state (`changes`, `desktop` or "").
pub fn cr_split(k: &str) -> (ScreenState, &'static str) {
    match k {
        "changes" => (ScreenState::Sample, "changes"),
        "desktop" => (ScreenState::Sample, "desktop"),
        _ => (ScreenState::from_key(k), ""),
    }
}

/// Models: name and brand mark id.
pub const CR_MODELS: [(&str, &str); 3] = [
    ("Opus 5", "claude"),
    ("GPT-6", "openai"),
    ("Gemini 3 Pro", "gemini"),
];

/// Effort: label and line.
pub const CR_EFFORT: [(&str, &str); 3] = [
    ("Low", "Quick edits, fewer tokens"),
    ("Medium", "The default"),
    ("High", "Thinks longer on hard tasks"),
];

/// The pod's tabs: key and label.
pub const CR_TABS: [(&str, &str); 3] = [
    ("desktop", "Desktop"),
    ("ports", "Ports"),
    ("changes", "Changes"),
];

/// Sidebar filters: key and label.
pub const CR_FILTERS: [(&str, &str); 3] =
    [("all", "All"), ("run", "Running"), ("wait", "Needs you")];

/// A thread in the sidebar.
#[derive(Clone, Debug, PartialEq)]
pub struct CrThread {
    /// Stable id.
    pub id: u32,
    /// Title.
    pub title: String,
    /// `run` working, `wait` needs you, `ok` done.
    pub status: &'static str,
    /// Repository.
    pub repo: &'static str,
    /// Last activity, an absolute ISO time.
    pub at: &'static str,
}

fn th(
    id: u32,
    title: &str,
    status: &'static str,
    repo: &'static str,
    at: &'static str,
) -> CrThread {
    CrThread {
        id,
        title: title.to_string(),
        status,
        repo,
        at,
    }
}

/// The sample threads (empty for loading and empty).
pub fn cr_threads(st: ScreenState) -> Vec<CrThread> {
    if st != ScreenState::Sample {
        return Vec::new();
    }
    vec![
        th(
            1,
            "Cache the price lookup",
            "ok",
            "harbor/shop",
            "2026-11-19 09:31",
        ),
        th(
            2,
            "Upgrade the image pipeline",
            "run",
            "harbor/shop",
            "2026-11-19 09:27",
        ),
        th(
            3,
            "Flaky login e2e on CI",
            "wait",
            "harbor/auth",
            "2026-11-19 08:58",
        ),
        th(
            4,
            "Port the CSV export to streams",
            "run",
            "harbor/reports",
            "2026-11-19 08:40",
        ),
        th(
            5,
            "Remove the old feature flags",
            "ok",
            "harbor/shop",
            "2026-11-18 17:12",
        ),
    ]
}

/// The threads a filter shows.
pub fn filter_threads(list: &[CrThread], f: &str) -> Vec<CrThread> {
    list.iter()
        .filter(|t| f == "all" || t.status == f)
        .cloned()
        .collect()
}

/// Start a thread from a prompt at the top; it runs at once.
pub fn add_thread(list: &mut Vec<CrThread>, prompt: &str) -> Result<u32, &'static str> {
    if let Some(e) = prompt_error(prompt) {
        return Err(e);
    }
    let id = list.iter().map(|t| t.id).max().unwrap_or(0) + 1;
    let t = super::taskdeck::title_from(prompt);
    list.insert(0, th(id, &t, "run", "harbor/shop", "2026-11-19 09:40"));
    Ok(id)
}

/// The open thread's prompt.
pub const CR_PROMPT: &str = "The product page calls the pricing service on every render. Cache prices for 60 s, keyed by SKU and currency.";

/// The agent's tool lines: glyph, verb, object.
pub const CR_TOOLS: [(&str, &str, &str); 4] = [
    ("search", "Searched", "getPrice( in src/"),
    ("file", "Read", "src/pricing/client.ts"),
    ("file", "Read", "src/pages/product.tsx"),
    ("term", "Ran", "pnpm test pricing · 14 passed"),
];

/// A changed file: path, lines added and removed, and its diff (sign, text).
#[derive(Clone, Debug, PartialEq)]
pub struct CrFile {
    /// Path.
    pub path: &'static str,
    /// Lines added.
    pub add: u32,
    /// Lines removed.
    pub del: u32,
    /// The diff.
    pub diff: &'static [(char, &'static str)],
}

/// The sample changes (empty for loading and empty).
pub fn cr_files(st: ScreenState) -> Vec<CrFile> {
    if st != ScreenState::Sample {
        return Vec::new();
    }
    vec![
        CrFile {
            path: "src/pricing/cache.ts",
            add: 28,
            del: 0,
            diff: &[
                ('+', "const TTL_MS = 60_000;"),
                (
                    '+',
                    "const key = (sku: string, cur: string) => `${sku}:${cur}`;",
                ),
                ('+', "export function cached(sku: string, cur: string) {"),
                ('+', "  const hit = store.get(key(sku, cur));"),
                (
                    '+',
                    "  return hit && Date.now() - hit.at < TTL_MS ? hit.price : null;",
                ),
            ],
        },
        CrFile {
            path: "src/pricing/client.ts",
            add: 9,
            del: 3,
            diff: &[
                (
                    ' ',
                    "export async function getPrice(sku: string, cur: string) {",
                ),
                ('-', "  return fetchPrice(sku, cur);"),
                ('+', "  const hit = cached(sku, cur);"),
                ('+', "  if (hit !== null) return hit;"),
                (
                    '+',
                    "  return remember(sku, cur, await fetchPrice(sku, cur));",
                ),
            ],
        },
        CrFile {
            path: "src/pricing/cache.test.ts",
            add: 34,
            del: 0,
            diff: &[
                (
                    '+',
                    "it(\"serves a cached price inside 60 s\", async () => {",
                ),
                ('+', "  await getPrice(\"SKU-1\", \"EUR\");"),
                ('+', "  expect(fetchPrice).toHaveBeenCalledTimes(1);"),
            ],
        },
    ]
}

/// Files, lines added and lines removed over a set of changes.
pub fn cr_totals(files: &[CrFile]) -> (usize, u32, u32) {
    files
        .iter()
        .fold((0, 0, 0), |(n, a, d), f| (n + 1, a + f.add, d + f.del))
}

/// Revert a file: it leaves the changes. Returns whether it was there.
pub fn revert(files: &mut Vec<CrFile>, path: &str) -> bool {
    let n = files.len();
    files.retain(|f| f.path != path);
    files.len() != n
}

/// The agent's closing line.
pub const CR_DONE: &str = "Prices are cached for 60 s per SKU and currency, so a product page now makes one pricing call a minute instead of one per render. The new test covers a hit and an expired entry.";

/// Suggested replies above the composer.
pub const CR_SUGGEST: [&str; 3] = [
    "Add a test for a currency switch",
    "Explain the cache key",
    "Open a pull request",
];

/// Forwarded ports on the pod: port, what serves it, open.
pub const CR_PORTS: [(u16, &str, bool); 3] = [
    (3000, "next dev · product pages", true),
    (5432, "postgres", false),
    (6006, "storybook", true),
];

/// The pod's name and size.
pub const CR_POD: (&str, &str) = ("pod-7f3a", "4 vCPU · 8 GB");

// ─────────────────────────────────────────────────────────── Data ──

/// Everything Courier shows. Pass it to the screen's `data` prop; `Default` is the sample.
#[derive(Clone, Debug, PartialEq)]
pub struct CourierData {
    /// Threads in the sidebar; the first one opens.
    pub threads: Vec<CrThread>,
    /// The open thread's changed files.
    pub files: Vec<CrFile>,
}

impl CourierData {
    /// The data for a demo state (sample, or nothing for empty and loading).
    pub fn for_state(st: ScreenState) -> Self {
        Self {
            threads: cr_threads(st),
            files: cr_files(st),
        }
    }
}

impl Default for CourierData {
    fn default() -> Self {
        Self::for_state(ScreenState::Sample)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn s16_courier() {
        assert_eq!(cr_split("changes").1, "changes");
        assert_eq!(cr_split("loading").0, ScreenState::Loading);
        let mut l = cr_threads(ScreenState::Sample);
        assert_eq!(filter_threads(&l, "all").len(), 5);
        assert_eq!(filter_threads(&l, "run").len(), 2);
        assert_eq!(filter_threads(&l, "wait").len(), 1);
        assert!(add_thread(&mut l, "x").is_err());
        let id = add_thread(&mut l, "Add a price history chart").unwrap();
        assert_eq!((l[0].id, l[0].status), (id, "run"));
        let mut f = cr_files(ScreenState::Sample);
        assert_eq!(cr_totals(&f), (3, 71, 3));
        assert!(revert(&mut f, "src/pricing/cache.test.ts"));
        assert!(!revert(&mut f, "nope.ts"));
        assert_eq!(cr_totals(&f), (2, 37, 3));
        assert!(CourierData::for_state(ScreenState::Empty).files.is_empty());
    }
}
