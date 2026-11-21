//! Screens S16 (Agent apps IV): **Halo Chat**, a desktop assistant app. Chats in the sidebar
//! (pinned first) with helpers (saved instructions) above them; a conversation where the
//! assistant's tool calls show as cards (a web search with its sources, code it ran with the
//! output, a table it built); a composer with Tools (Deep research, Canvas, Create image, Run
//! code), a mode and a model; a pane with the sources and a canvas draft. A mini chat that opens
//! over any app from a shortcut, and a research plan to edit before the research starts. Our own
//! name and design, inspired by how such apps behave. Every name, site, number and company is
//! fictional.

pub use super::cmdstudio::{
    PERSON, PERSON_EMAIL, PERSON_INI, PERSON_PLAN, ScreenState, StateList, prompt_error,
};

/// The app's name.
pub const HC_APP: &str = "Halo Chat";

/// The state bar of Halo Chat.
pub const HC_STATES: StateList = &[
    ("sample", "Data"),
    ("loading", "Loading"),
    ("empty", "Empty"),
    ("mini", "Mini chat"),
    ("research", "Research plan"),
];

/// Split a state key into the data state and the own state (`mini`, `research` or "").
pub fn hc_split(k: &str) -> (ScreenState, &'static str) {
    match k {
        "mini" => (ScreenState::Sample, "mini"),
        "research" => (ScreenState::Sample, "research"),
        _ => (ScreenState::from_key(k), ""),
    }
}

/// Modes: label and line.
pub const HC_MODES: [(&str, &str); 3] = [
    ("Fast", "Answers quickly"),
    ("Thinking", "Reasons before it answers"),
    ("Deep think", "Longest reasoning, for hard problems"),
];

/// Models: name and brand mark id.
pub const HC_MODELS: [(&str, &str); 3] = [
    ("Gemini 3 Flash", "gemini"),
    ("Gemini 3 Pro", "gemini"),
    ("Sonnet 5.5", "claude"),
];

/// Tools in the composer: key, label and glyph.
pub const HC_TOOLS: [(&str, &str, &str); 4] = [
    ("research", "Deep research", "scope"),
    ("canvas", "Canvas", "draw"),
    ("image", "Create image", "image"),
    ("code", "Run code", "term"),
];

/// Helpers (saved instructions): name, line and glyph.
pub const HC_HELPERS: [(&str, &str, &str); 3] = [
    ("Trip planner", "Plans days around opening hours", "globe"),
    ("Code reviewer", "Reads a diff, flags risks", "term"),
    ("Study partner", "Quizzes you on your notes", "book"),
];

/// The pane's tabs: key and label.
pub const HC_TABS: [(&str, &str); 2] = [("sources", "Sources"), ("canvas", "Canvas")];

/// A chat in the sidebar.
#[derive(Clone, Debug, PartialEq)]
pub struct HcChat {
    /// Stable id.
    pub id: u32,
    /// Title.
    pub title: String,
    /// Pinned to the top.
    pub pinned: bool,
    /// Last message, an absolute ISO date.
    pub at: &'static str,
}

fn ch(id: u32, title: &str, pinned: bool, at: &'static str) -> HcChat {
    HcChat {
        id,
        title: title.to_string(),
        pinned,
        at,
    }
}

/// The sample chats (empty for loading and empty).
pub fn hc_chats(st: ScreenState) -> Vec<HcChat> {
    if st != ScreenState::Sample {
        return Vec::new();
    }
    vec![
        ch(1, "E-bikes for a hilly commute", true, "2026-11-19"),
        ch(2, "Sourdough starter schedule", true, "2026-11-16"),
        ch(3, "Lisbon in three days", false, "2026-11-18"),
        ch(4, "Explain index fragmentation", false, "2026-11-17"),
        ch(5, "Cover letter, product designer", false, "2026-11-14"),
        ch(6, "Balcony tomato varieties", false, "2026-11-11"),
    ]
}

/// Pin or unpin a chat; pinned chats sort first, then by date, newest first.
pub fn toggle_pin(list: &mut [HcChat], id: u32) {
    if let Some(c) = list.iter_mut().find(|c| c.id == id) {
        c.pinned = !c.pinned;
    }
    list.sort_by(|a, b| b.pinned.cmp(&a.pinned).then(b.at.cmp(a.at)));
}

/// Start a chat from a prompt at the top of the unpinned chats.
pub fn add_chat(list: &mut Vec<HcChat>, prompt: &str) -> Result<u32, &'static str> {
    if let Some(e) = prompt_error(prompt) {
        return Err(e);
    }
    let id = list.iter().map(|c| c.id).max().unwrap_or(0) + 1;
    let at = list.iter().position(|c| !c.pinned).unwrap_or(list.len());
    let t = super::taskdeck::title_from(prompt);
    list.insert(at, ch(id, &t, false, "2026-11-19"));
    Ok(id)
}

/// The open chat's prompt.
pub const HC_PROMPT: &str = "Compare three e-bikes under $2,000 for a 12 km hilly commute. Table the price, range and motor torque.";

/// The search card: query and the sources it read (title, site, ISO date).
pub const HC_QUERY: &str = "commuter e-bike under 2000 range hills torque review";

/// Sources the search read.
pub const HC_SOURCES: [(&str, &str, &str); 5] = [
    (
        "Long-term test: the Fenwick C3 on steep routes",
        "pedalnotes.example",
        "2026-10-02",
    ),
    (
        "Mid-drive vs hub motors on hills",
        "velolab.example",
        "2026-08-21",
    ),
    (
        "Arden Glide 2 review",
        "cyclequarterly.example",
        "2026-09-14",
    ),
    ("Range claims, measured", "ridewatt.example", "2026-07-30"),
    (
        "Brisa Urban: a year of commuting",
        "spokesandsense.example",
        "2026-06-11",
    ),
];

/// The bikes: name, price in dollars, range in km, torque in N·m.
pub const HC_BIKES: [(&str, u32, u32, u32); 3] = [
    ("Fenwick C3", 1_890, 85, 85),
    ("Arden Glide 2", 1_650, 70, 60),
    ("Brisa Urban", 1_480, 60, 50),
];

/// Whole dollars with thousands separators, `$1,890`.
pub fn dollars(n: u32) -> String {
    let d = n.to_string();
    let mut out = String::new();
    for (i, c) in d.chars().enumerate() {
        if i > 0 && (d.len() - i).is_multiple_of(3) {
            out.push(',');
        }
        out.push(c);
    }
    format!("${out}")
}

/// The code the assistant ran (Python) and its output.
pub const HC_CODE: [&str; 3] = [
    "bikes = [(\"Fenwick C3\", 1890, 85), (\"Arden Glide 2\", 1650, 70), (\"Brisa Urban\", 1480, 60)]",
    "for name, usd, km in bikes:",
    "    print(f\"{name}: ${usd / km:.2f} per km of range\")",
];

/// Dollars per km of range, to two decimals (the code card's output).
pub fn usd_per_km(usd: u32, km: u32) -> String {
    if km == 0 {
        return "n/a".to_string();
    }
    let c = (usd as u64 * 100 + km as u64 / 2) / km as u64;
    format!("${}.{:02}", c / 100, c % 100)
}

/// The assistant's answer under the cards.
pub const HC_ANSWER: &str = "For 12 km with climbs, the Fenwick C3 is the pick: its 85 N·m mid-drive holds speed on hills and its range covers a week of commutes. The Arden Glide 2 costs less per km of range if your route is gentler.";

/// The research plan (own state `research`): step and whether it is on.
pub const HC_PLAN: [(&str, bool); 5] = [
    ("Find commuter e-bikes under $2,000 sold this year", true),
    ("Collect measured range on hilly routes, not claims", true),
    ("Compare motor torque and gearing for 8 % grades", true),
    ("Check service networks within 10 km", false),
    ("Write a report with a shortlist and a table", true),
];

/// Why the research cannot start, or `None` when it can.
pub fn plan_error(steps: &[bool]) -> Option<&'static str> {
    if steps.iter().any(|s| *s) {
        None
    } else {
        Some("Keep at least one step")
    }
}

/// The research's estimated time in minutes: 2 per step on, at least 3.
pub fn plan_minutes(steps: &[bool]) -> u32 {
    (steps.iter().filter(|s| **s).count() as u32 * 2).max(3)
}

/// The canvas draft: heading and paragraphs.
pub const HC_CANVAS: [&str; 4] = [
    "Commute e-bike shortlist",
    "Route: 12 km each way, two climbs of about 8 %. Budget: $2,000.",
    "Pick: Fenwick C3. Mid-drive, 85 N·m, about 85 km of range measured.",
    "Runner-up: Arden Glide 2, if the hills turn out gentler than they look.",
];

/// The mini chat's shortcut.
pub const HC_SHORTCUT: &str = "Alt + Space";

// ─────────────────────────────────────────────────────────── Data ──

/// Everything Halo Chat shows. Pass it to the screen's `data` prop; `Default` is the sample.
#[derive(Clone, Debug, PartialEq)]
pub struct HaloChatData {
    /// Chats in the sidebar; the first one opens.
    pub chats: Vec<HcChat>,
}

impl HaloChatData {
    /// The data for a demo state (sample, or nothing for empty and loading).
    pub fn for_state(st: ScreenState) -> Self {
        Self {
            chats: hc_chats(st),
        }
    }
}

impl Default for HaloChatData {
    fn default() -> Self {
        Self::for_state(ScreenState::Sample)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn s16_halochat() {
        assert_eq!(hc_split("mini").1, "mini");
        assert_eq!(hc_split("loading").0, ScreenState::Loading);
        let mut l = hc_chats(ScreenState::Sample);
        let id = add_chat(&mut l, "Plan a picnic for eight").unwrap();
        assert_eq!((l[2].id, l[2].pinned), (id, false));
        toggle_pin(&mut l, 6);
        assert_eq!((l[2].id, l[2].pinned), (6, true));
        toggle_pin(&mut l, 1);
        assert_eq!(l[0].id, 2);
        assert_eq!(usd_per_km(1890, 85), "$22.24");
        assert_eq!(usd_per_km(1480, 60), "$24.67");
        assert_eq!(usd_per_km(10, 0), "n/a");
        assert_eq!(dollars(1_890), "$1,890");
        assert_eq!(dollars(950), "$950");
        assert_eq!(dollars(1_234_567), "$1,234,567");
        let on: Vec<bool> = HC_PLAN.iter().map(|s| s.1).collect();
        assert_eq!(plan_error(&on), None);
        assert_eq!(plan_minutes(&on), 8);
        assert_eq!(plan_error(&[false, false]), Some("Keep at least one step"));
        assert_eq!(plan_minutes(&[true]), 3);
        assert!(HaloChatData::for_state(ScreenState::Empty).chats.is_empty());
    }
}
