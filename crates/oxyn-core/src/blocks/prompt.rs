//! Prompt bench: edit, test and ship a prompt template. System and User tabs show the template
//! with `{{variable}}` placeholders as chips, a field per variable fills them from saved test
//! cases, Run streams a reply with latency, tokens and cost, versions can be compared (a word
//! diff against production) and promoted, with Undo.

use super::{first_words, nth, rove, usd, words_at};

/// Variables: name and series hue.
pub const VARS: [(&str, &str); 4] = [
    ("customer", "blue"),
    ("product", "violet"),
    ("tone", "teal"),
    ("question", "amber"),
];

/// Versions: label, note, system template, user template.
pub const VERSIONS: [(&str, &str, &str, &str); 4] = [
    (
        "v1",
        "First draft",
        "You are a support assistant for {{product}}. Answer questions from customers.",
        "{{customer}} asks: {{question}}",
    ),
    (
        "v2",
        "Added tone",
        "You are a support assistant for {{product}}. Answer in a {{tone}} voice and keep it short.",
        "{{customer}} asks: {{question}}",
    ),
    (
        "v3",
        "Cites the help centre",
        "You are a support assistant for {{product}}. Answer in a {{tone}} voice, keep it under 80 words and link one help article when it helps.",
        "Customer {{customer}} writes: {{question}} Reply to them by name.",
    ),
    (
        "v4",
        "Asks before refunds",
        "You are a careful support assistant for {{product}}. Answer in a {{tone}} voice, keep it under 80 words, link one help article when it helps and never promise a refund without asking a teammate.",
        "Customer {{customer}} writes: {{question}} Reply to them by name and say what happens next.",
    ),
];

/// Saved test cases: name and values per variable.
pub const CASES: [(&str, [&str; 4]); 3] = [
    (
        "Billing",
        [
            "Dana",
            "Quillstack",
            "warm",
            "Why was I charged twice this month?",
        ],
    ),
    (
        "Export",
        [
            "Ravi",
            "Quillstack",
            "plain",
            "How do I export my notes as PDF?",
        ],
    ),
    (
        "Angry",
        [
            "Mo",
            "Quillstack",
            "calm",
            "Your app deleted my project. Fix it now.",
        ],
    ),
];

/// Replies per case.
pub const REPLIES: [&str; 3] = [
    "Hi Dana, sorry about the double charge. One payment was a pre-authorisation that drops off within three days, so you are only billed once. If it is still there on Friday, reply here and a teammate will look at a refund with you. More in Billing and invoices.",
    "Hi Ravi, open the note, choose Share, then Export as PDF. To export a whole notebook, use Settings, then Data, then Export all. Large notebooks arrive by email within a few minutes.",
    "Hi Mo, I am sorry, that is stressful. Deleted projects stay in Trash for 30 days, so it should be safe: open Trash in the sidebar and choose Restore. If it is not there, reply and a teammate will check the backups with you right away.",
];

/// Models: name, brand, first token ms, cost per 1k tokens.
pub const MODELS: [(&str, &str, u64, f64); 3] = [
    ("Claude Sonnet 4.5", "claude", 640, 0.009),
    ("GPT-5", "openai", 720, 0.011),
    ("Gemini 2.5 Flash", "gemini", 410, 0.002),
];

/// Effort levels.
pub const EFFORT: [&str; 3] = ["Low", "Medium", "High"];

/// One piece of a template: text or a variable index.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Part {
    /// Plain text.
    Text(String),
    /// A variable chip.
    Var(usize),
}

/// Splits a template into text and variable chips.
pub fn parts(t: &str) -> Vec<Part> {
    let mut out = Vec::new();
    let mut rest = t;
    while let Some(a) = rest.find("{{") {
        let Some(b) = rest[a..].find("}}") else { break };
        if a > 0 {
            out.push(Part::Text(rest[..a].to_string()));
        }
        let name = &rest[a + 2..a + b];
        match VARS.iter().position(|v| v.0 == name) {
            Some(i) => out.push(Part::Var(i)),
            None => out.push(Part::Text(rest[a..a + b + 2].to_string())),
        }
        rest = &rest[a + b + 2..];
    }
    if !rest.is_empty() {
        out.push(Part::Text(rest.to_string()));
    }
    out
}

/// A word diff: (`-1` removed, `0` same, `1` added, word).
pub fn diff(old: &str, new: &str) -> Vec<(i8, String)> {
    let a: Vec<&str> = old.split_whitespace().collect();
    let b: Vec<&str> = new.split_whitespace().collect();
    let (n, m) = (a.len(), b.len());
    let mut l = vec![vec![0u16; m + 1]; n + 1];
    for i in (0..n).rev() {
        for j in (0..m).rev() {
            l[i][j] = if a[i] == b[j] {
                l[i + 1][j + 1] + 1
            } else {
                l[i + 1][j].max(l[i][j + 1])
            };
        }
    }
    let (mut i, mut j, mut out) = (0, 0, Vec::new());
    while i < n || j < m {
        if i < n && j < m && a[i] == b[j] {
            out.push((0, a[i].to_string()));
            i += 1;
            j += 1;
        } else if j < m && (i == n || l[i][j + 1] >= l[i + 1][j]) {
            out.push((1, b[j].to_string()));
            j += 1;
        } else {
            out.push((-1, a[i].to_string()));
            i += 1;
        }
    }
    out
}

/// Which menu is open.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Menu {
    /// Versions.
    Version,
    /// Model and effort.
    Model,
}

/// The studio.
#[derive(Clone, Debug, PartialEq)]
pub struct Studio {
    /// Version being edited.
    pub version: usize,
    /// Version production serves.
    pub prod: usize,
    /// Production before the last promote (Undo).
    pub undo: Option<usize>,
    /// Tab: 0 System, 1 User.
    pub tab: usize,
    /// Test case.
    pub case: usize,
    /// Variable values.
    pub values: [String; 4],
    /// Model.
    pub model: usize,
    /// Effort.
    pub effort: usize,
    /// Run: start and case.
    pub run: Option<(u64, usize)>,
    /// Stopped at word n.
    pub stopped: Option<usize>,
    /// The next run fails.
    pub fail: bool,
    /// The last run failed.
    pub failed: bool,
    /// Compare mode.
    pub compare: bool,
    /// Open menu.
    pub menu: Option<Menu>,
}

impl Studio {
    /// A studio, optionally running case 0 at `now`.
    pub fn new(auto: bool, fail: bool, now: u64) -> Self {
        let mut s = Self {
            version: 3,
            prod: 2,
            undo: None,
            tab: 0,
            case: 0,
            values: CASES[0].1.map(String::from),
            model: 0,
            effort: 1,
            run: None,
            stopped: None,
            fail,
            failed: false,
            compare: false,
            menu: None,
        };
        if auto {
            s.press("ps-run", now);
        }
        s
    }

    /// The template on the current tab.
    pub fn template(&self, v: usize) -> &'static str {
        if self.tab == 0 {
            VERSIONS[v].2
        } else {
            VERSIONS[v].3
        }
    }

    /// Variables used by the current version (either tab).
    pub fn used(&self) -> Vec<usize> {
        let v = self.version;
        (0..VARS.len())
            .filter(|i| {
                let k = format!("{{{{{}}}}}", VARS[*i].0);
                VERSIONS[v].2.contains(&k) || VERSIONS[v].3.contains(&k)
            })
            .collect()
    }

    /// Set a variable value (typing in its field).
    pub fn set_value(&mut self, i: usize, s: &str) {
        if let Some(v) = self.values.get_mut(i) {
            *v = s.to_string();
        }
    }

    /// Words of the reply visible at `now`.
    pub fn words(&self, now: u64) -> usize {
        let Some((t, c)) = self.run else { return 0 };
        let at = t + MODELS[self.model].2;
        if now < at || self.failed {
            return 0;
        }
        let n = words_at(REPLIES[c], now - at, 3);
        self.stopped.map_or(n, |s| s.min(n))
    }

    /// The reply as shown.
    pub fn reply(&self, now: u64) -> String {
        self.run
            .map(|(_, c)| first_words(REPLIES[c], self.words(now)))
            .unwrap_or_default()
    }

    /// Running (waiting or streaming).
    pub fn running(&self, now: u64) -> bool {
        self.run.is_some_and(|(_, c)| {
            !self.failed
                && self.stopped.is_none()
                && self.words(now) < REPLIES[c].split_whitespace().count()
        })
    }

    /// Run stats once finished: latency, tokens, cost.
    pub fn stats(&self, now: u64) -> Option<(String, u32, String)> {
        let (_, c) = self.run?;
        if self.running(now) || self.failed {
            return None;
        }
        let toks = (VERSIONS[self.version].2.len() + REPLIES[c].len()) as u32 / 4
            + 40 * self.effort as u32;
        let ms = MODELS[self.model].2 + 900 + 250 * self.effort as u64;
        Some((
            format!("{:.1}s", ms as f64 / 1000.0),
            toks,
            usd(f64::from(toks) / 1000.0 * MODELS[self.model].3),
        ))
    }

    /// Empty variables used by this version.
    pub fn empty(&self) -> Vec<usize> {
        self.used()
            .into_iter()
            .filter(|&i| self.values[i].trim().is_empty())
            .collect()
    }

    /// Presses: `ps-tab-<i>`, `ps-case-<i>`, `ps-run` (Run or Stop), `ps-retry`, `ps-ver` (menu),
    /// `ps-ver-<i>`, `ps-promote`, `ps-undo`, `ps-compare`, `ps-model` (menu), `ps-model-<i>`,
    /// `ps-effort-<i>`, `ps-copy`.
    pub fn press(&mut self, id: &str, now: u64) -> Option<String> {
        match id {
            "ps-run" | "ps-retry" => {
                if self.running(now) {
                    self.stopped = Some(self.words(now));
                    return Some("Stopped".into());
                }
                if let Some(&e) = self.empty().first() {
                    return Some(format!("Fill in {} first", VARS[e].0));
                }
                self.run = Some((now, self.case));
                self.stopped = None;
                self.failed = false;
                self.menu = None;
                if self.fail {
                    self.fail = false;
                    self.failed = true;
                    return Some("Run failed: the model returned an error. Retry".into());
                }
                return Some(format!(
                    "Running {} on {} with {}. The reply is simulated",
                    VERSIONS[self.version].0, CASES[self.case].0, MODELS[self.model].0
                ));
            }
            "ps-ver" => {
                self.menu = if self.menu == Some(Menu::Version) {
                    None
                } else {
                    Some(Menu::Version)
                };
                return None;
            }
            "ps-model" => {
                self.menu = if self.menu == Some(Menu::Model) {
                    None
                } else {
                    Some(Menu::Model)
                };
                return None;
            }
            "ps-promote" => {
                if self.version == self.prod {
                    return None;
                }
                self.undo = Some(self.prod);
                self.prod = self.version;
                self.menu = None;
                return Some(format!(
                    "{} promoted to production. Undo is available",
                    VERSIONS[self.version].0
                ));
            }
            "ps-undo" => {
                let p = self.undo.take()?;
                self.prod = p;
                return Some(format!("Production is back on {}", VERSIONS[p].0));
            }
            "ps-compare" => {
                self.compare = !self.compare;
                self.menu = None;
                return Some(if self.compare {
                    format!(
                        "Comparing {} with production {}",
                        VERSIONS[self.version].0, VERSIONS[self.prod].0
                    )
                } else {
                    "Back to editing".into()
                });
            }
            "ps-copy" => return self.run.map(|_| "Reply copied".into()),
            _ => {}
        }
        if let Some(i) = nth(id, "ps-tab-") {
            self.tab = i.min(1);
            return None;
        }
        if let Some(i) = nth(id, "ps-case-") {
            let c = CASES.get(i)?;
            self.case = i;
            self.values = c.1.map(String::from);
            return Some(format!("Test case {}", c.0));
        }
        if let Some(i) = nth(id, "ps-ver-") {
            VERSIONS.get(i)?;
            self.version = i;
            self.menu = None;
            return Some(format!("Editing {}", VERSIONS[i].0));
        }
        if let Some(i) = nth(id, "ps-model-") {
            MODELS.get(i)?;
            self.model = i;
            self.menu = None;
            return Some(format!("Model {}", MODELS[i].0));
        }
        if let Some(i) = nth(id, "ps-effort-") {
            EFFORT.get(i)?;
            self.effort = i;
            return Some(format!("Effort {}", EFFORT[i]));
        }
        None
    }

    /// Keys: Left/Right move tabs (or effort with the model menu open); Up/Down move versions or
    /// models in an open menu; Escape closes a menu or leaves compare; Mod+Enter is `run`.
    pub fn key(&mut self, key: &str, now: u64) -> Option<String> {
        let k = super::norm_key(key);
        match (k, self.menu) {
            ("Escape", Some(_)) => {
                self.menu = None;
                None
            }
            ("Escape", None) if self.compare => self.press("ps-compare", now),
            ("ArrowUp" | "ArrowDown", Some(Menu::Version)) => {
                let i = rove(self.version, VERSIONS.len(), k);
                self.version = i;
                Some(format!("Editing {}", VERSIONS[i].0))
            }
            ("ArrowUp" | "ArrowDown", Some(Menu::Model)) => {
                let i = rove(self.model, MODELS.len(), k);
                self.model = i;
                Some(format!("Model {}", MODELS[i].0))
            }
            ("ArrowLeft" | "ArrowRight" | "Home" | "End", Some(Menu::Model)) => {
                let i = rove(self.effort, EFFORT.len(), k);
                self.press(&format!("ps-effort-{i}"), now)
            }
            ("ArrowLeft" | "ArrowRight" | "Home" | "End", None) => {
                self.tab = rove(self.tab, 2, k);
                Some(if self.tab == 0 { "System" } else { "User" }.into())
            }
            ("run", _) => self.press("ps-run", now),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn template_parts_and_diff() {
        let p = parts("Hi {{customer}}, about {{product}}.");
        assert_eq!(p[1], Part::Var(0));
        assert_eq!(p[3], Part::Var(1));
        let d = diff("a b c", "a x c");
        assert_eq!(
            d,
            vec![
                (0, "a".into()),
                (1, "x".into()),
                (-1, "b".into()),
                (0, "c".into())
            ]
        );
    }

    #[test]
    fn run_promote_undo() {
        let mut s = Studio::new(false, true, 0);
        s.press("ps-run", 0);
        assert!(s.failed);
        s.press("ps-retry", 10);
        assert!(s.running(1_000));
        assert!(s.stats(20_000).is_some());
        s.press("ps-promote", 20_000);
        assert_eq!(s.prod, 3);
        s.press("ps-undo", 20_000);
        assert_eq!(s.prod, 2);
        s.set_value(3, " ");
        assert!(s.press("ps-run", 21_000).unwrap().starts_with("Fill in"));
        s.press("ps-case-2", 0);
        assert_eq!(s.values[0], "Mo");
        s.key("right", 0);
        assert_eq!(s.tab, 1);
        s.press("ps-compare", 0);
        s.key("escape", 0);
        assert!(!s.compare);
    }
}
