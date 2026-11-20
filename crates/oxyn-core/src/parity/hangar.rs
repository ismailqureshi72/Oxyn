//! Screens S16 (Agent apps IV): **Hangar**, an agent manager. Conversations grouped by
//! workspace in the sidebar (with a count of those that wait on you); the open conversation with
//! the agent's task list and the artifacts it made (an implementation plan, a task list, a
//! walkthrough recording, a screenshot); comments on the plan's lines before the agent goes on;
//! a Planning / Fast mode and a model in the composer; and two surfaces beside the chat, an
//! Editor with the agent's edits and a Browser the agent drives. Our own name and design,
//! inspired by how such apps behave. Every name, file, number and company is fictional.

pub use super::cmdstudio::{
    PERSON, PERSON_EMAIL, PERSON_INI, PERSON_PLAN, ScreenState, StateList, prompt_error,
};

/// The app's name.
pub const HG_APP: &str = "Hangar";

/// The state bar of Hangar.
pub const HG_STATES: StateList = &[
    ("sample", "Data"),
    ("loading", "Loading"),
    ("empty", "Empty"),
    ("artifact", "Artifact review"),
    ("browser", "Browser surface"),
];

/// Split a state key into the data state and the own state (`artifact`, `browser` or "").
pub fn hg_split(k: &str) -> (ScreenState, &'static str) {
    match k {
        "artifact" => (ScreenState::Sample, "artifact"),
        "browser" => (ScreenState::Sample, "browser"),
        _ => (ScreenState::from_key(k), ""),
    }
}

/// Modes: label and line. Planning writes a plan to review first; Fast acts at once.
pub const HG_MODES: [(&str, &str); 2] = [
    ("Planning", "Writes a plan you review before it acts"),
    ("Fast", "Acts at once, for small tasks"),
];

/// Models: name and brand mark id.
pub const HG_MODELS: [(&str, &str); 3] = [
    ("Gemini 3 Pro", "gemini"),
    ("Sonnet 5.5", "claude"),
    ("GPT-OSS 120B", "openai"),
];

/// The surfaces beside the chat: key and label.
pub const HG_SURFACES: [(&str, &str); 3] = [
    ("editor", "Editor"),
    ("browser", "Browser"),
    ("artifact", "Plan"),
];

/// The workspaces the sample conversations belong to.
pub const HG_WORKSPACES: [&str; 2] = ["atlas-store", "tidepool-docs"];

/// A conversation in the sidebar.
#[derive(Clone, Debug, PartialEq)]
pub struct HgConv {
    /// Stable id.
    pub id: u32,
    /// Title.
    pub title: String,
    /// Workspace it belongs to.
    pub workspace: &'static str,
    /// `run` working, `wait` waits on your review, `ok` done.
    pub status: &'static str,
    /// Last activity, an absolute ISO time.
    pub at: &'static str,
}

fn cv(
    id: u32,
    title: &str,
    workspace: &'static str,
    status: &'static str,
    at: &'static str,
) -> HgConv {
    HgConv {
        id,
        title: title.to_string(),
        workspace,
        status,
        at,
    }
}

/// The sample conversations (empty for loading and empty).
pub fn hg_convs(st: ScreenState) -> Vec<HgConv> {
    if st != ScreenState::Sample {
        return Vec::new();
    }
    vec![
        cv(
            1,
            "Mobile checkout summary",
            "atlas-store",
            "wait",
            "2026-11-19 10:12",
        ),
        cv(
            2,
            "Gift card balance check",
            "atlas-store",
            "run",
            "2026-11-19 10:05",
        ),
        cv(
            3,
            "Fix flaky cart test",
            "atlas-store",
            "ok",
            "2026-11-18 16:40",
        ),
        cv(
            4,
            "Search for the API guide",
            "tidepool-docs",
            "ok",
            "2026-11-18 11:22",
        ),
        cv(
            5,
            "Broken links report",
            "tidepool-docs",
            "",
            "2026-11-17 09:03",
        ),
    ]
}

/// Conversations that wait on your review.
pub fn waiting(list: &[HgConv]) -> usize {
    list.iter().filter(|c| c.status == "wait").count()
}

/// Start a conversation from a prompt at the top of the first workspace; it runs at once.
pub fn add_conv(list: &mut Vec<HgConv>, prompt: &str) -> Result<u32, &'static str> {
    if let Some(e) = prompt_error(prompt) {
        return Err(e);
    }
    let id = list.iter().map(|c| c.id).max().unwrap_or(0) + 1;
    let t = super::taskdeck::title_from(prompt);
    list.insert(0, cv(id, &t, HG_WORKSPACES[0], "run", "2026-11-19 10:15"));
    Ok(id)
}

/// The open conversation's prompt.
pub const HG_PROMPT: &str = "On phones the checkout summary hides the total under the fold. Make it a sticky footer with the total and the Pay button, and check it in the browser at 390 px.";

/// The agent's task list: label and state (`ok` done, `run` now, "" next).
pub const HG_TASKS: [(&str, &str); 5] = [
    ("Read the checkout layout and styles", "ok"),
    ("Write the implementation plan", "ok"),
    ("Move total and Pay into a sticky footer", "run"),
    ("Check 390 px in the browser", ""),
    ("Record a walkthrough", ""),
];

/// Tasks done out of all.
pub fn tasks_done(t: &[(&str, &str)]) -> (usize, usize) {
    (t.iter().filter(|x| x.1 == "ok").count(), t.len())
}

/// Artifacts: key, title, glyph and line.
pub const HG_ARTIFACTS: [(&str, &str, &str, &str); 3] = [
    ("plan", "Implementation plan", "doc", "5 steps · 3 files"),
    ("walk", "Walkthrough", "video", "Recording · 0:42"),
    (
        "shot",
        "Checkout at 390 px",
        "image",
        "Screenshot · 390 × 844",
    ),
];

/// The plan artifact's lines; a line takes comments.
pub const HG_PLAN: [&str; 6] = [
    "Goal: total and Pay stay visible on phones.",
    "Wrap the summary in a footer that sticks below 640 px.",
    "Keep the item list scrollable above it.",
    "Move Pay out of the list into the footer.",
    "Add safe-area padding for phones with a home bar.",
    "Check 390 px and 768 px in the browser and record it.",
];

/// A comment on a plan line.
#[derive(Clone, Debug, PartialEq)]
pub struct HgNote {
    /// The plan line it sits under (index into [`HG_PLAN`]).
    pub line: usize,
    /// What it says.
    pub text: String,
}

/// The longest comment.
pub const NOTE_CAP: usize = 280;

/// Why a comment cannot be added, or `None` when it can.
pub fn note_error(text: &str) -> Option<&'static str> {
    let n = text.trim().chars().count();
    if n == 0 {
        Some("Write a comment first")
    } else if n > NOTE_CAP {
        Some("Keep a comment under 280 characters")
    } else {
        None
    }
}

/// Add a comment under a plan line.
pub fn add_note(list: &mut Vec<HgNote>, line: usize, text: &str) -> Result<(), &'static str> {
    if let Some(e) = note_error(text) {
        return Err(e);
    }
    if line >= HG_PLAN.len() {
        return Err("Pick a line of the plan");
    }
    list.push(HgNote {
        line,
        text: text.trim().to_string(),
    });
    Ok(())
}

/// The plan's review button: Proceed with no comments, or send them back.
pub fn review_label(notes: usize) -> String {
    match notes {
        0 => "Proceed".to_string(),
        1 => "Send 1 comment".to_string(),
        n => format!("Send {n} comments"),
    }
}

/// The editor surface: the file and its lines, (sign, text). `+` the agent's edit.
pub const HG_FILE: &str = "src/checkout/Summary.tsx";

/// The edited lines of [`HG_FILE`].
pub const HG_CODE: [(char, &str); 8] = [
    (' ', "export function Summary({ cart }: Props) {"),
    (' ', "  return ("),
    ('+', "    <div className=\"summary\" data-sticky>"),
    (' ', "      <ItemList items={cart.items} />"),
    ('+', "      <footer className=\"summary-foot\">"),
    ('+', "        <Total value={cart.total} />"),
    ('+', "        <PayButton cart={cart} />"),
    ('+', "      </footer>"),
];

/// The browser surface's address.
pub const HG_URL: &str = "localhost:5173/checkout";

/// What the agent did in the browser: glyph, verb, target, time.
pub const HG_BROWSE: [(&str, &str, &str, &str); 4] = [
    ("globe", "Opened", "localhost:5173/checkout", "10:13:02"),
    ("phone", "Resized", "390 × 844", "10:13:05"),
    ("finger", "Scrolled", "the item list to the end", "10:13:09"),
    ("image", "Captured", "Checkout at 390 px", "10:13:12"),
];

/// The cart in the browser preview: item and price in cents.
pub const HG_CART: [(&str, u32); 3] = [
    ("Trail runner, size 42", 11_900),
    ("Merino socks ×2", 2_400),
    ("Shipping", 600),
];

/// Cents as a price, `$129.00`.
pub fn price(cents: u32) -> String {
    format!("${}.{:02}", cents / 100, cents % 100)
}

/// The cart's total in cents.
pub fn cart_total(items: &[(&str, u32)]) -> u32 {
    items.iter().map(|x| x.1).sum()
}

// ─────────────────────────────────────────────────────────── Data ──

/// Everything Hangar shows. Pass it to the screen's `data` prop; `Default` is the sample.
#[derive(Clone, Debug, PartialEq)]
pub struct HangarData {
    /// Conversations in the sidebar; the first one opens.
    pub convs: Vec<HgConv>,
    /// Comments already on the plan.
    pub notes: Vec<HgNote>,
}

impl HangarData {
    /// The data for a demo state (sample, or nothing for empty and loading).
    pub fn for_state(st: ScreenState) -> Self {
        let notes = if st == ScreenState::Sample {
            vec![HgNote {
                line: 4,
                text: "Only on iOS; Android keeps its own nav bar.".to_string(),
            }]
        } else {
            Vec::new()
        };
        Self {
            convs: hg_convs(st),
            notes,
        }
    }
}

impl Default for HangarData {
    fn default() -> Self {
        Self::for_state(ScreenState::Sample)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn s16_hangar() {
        assert_eq!(hg_split("browser").1, "browser");
        assert_eq!(hg_split("empty").0, ScreenState::Empty);
        let mut l = hg_convs(ScreenState::Sample);
        assert_eq!(waiting(&l), 1);
        assert!(add_conv(&mut l, "hi").is_err());
        let id = add_conv(&mut l, "Add a coupon field to checkout").unwrap();
        assert_eq!((l[0].id, l[0].status), (id, "run"));
        assert_eq!(tasks_done(&HG_TASKS), (2, 5));
        let mut n = Vec::new();
        assert_eq!(review_label(n.len()), "Proceed");
        assert!(add_note(&mut n, 1, "  ").is_err());
        assert!(add_note(&mut n, 9, "fine").is_err());
        add_note(&mut n, 1, "Use 600 px, our tablet breakpoint").unwrap();
        assert_eq!(review_label(n.len()), "Send 1 comment");
        add_note(&mut n, 3, "Keep Apple Pay above Pay").unwrap();
        assert_eq!(review_label(n.len()), "Send 2 comments");
        assert_eq!(cart_total(&HG_CART), 14_900);
        assert_eq!(price(14_900), "$149.00");
        assert!(HangarData::for_state(ScreenState::Loading).convs.is_empty());
    }
}
