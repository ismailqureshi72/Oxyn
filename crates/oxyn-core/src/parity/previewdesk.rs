//! Screens S16 (Agent apps IV): **Preview Desk**, a desktop app for coding agents that shows
//! their work as previews. Sessions in the sidebar, each on a machine of its own; the session's
//! chat with the agent's steps; preview tabs beside it (the live site, a doc, the diff); Design
//! mode, where you pick an element on the live site and leave a comment for the agent; and an
//! inline diff where a line takes a comment. A machine chip, a working folder and a model in the
//! composer. Our own name and design, inspired by how such apps behave. Every name, file, number
//! and company is fictional.

pub use super::cmdstudio::{
    PERSON, PERSON_EMAIL, PERSON_INI, PERSON_PLAN, ScreenState, StateList, prompt_error,
};

/// The app's name.
pub const PD_APP: &str = "Preview Desk";

/// The state bar of Preview Desk.
pub const PD_STATES: StateList = &[
    ("sample", "Data"),
    ("loading", "Loading"),
    ("empty", "Empty"),
    ("design", "Design mode"),
    ("diff", "Diff review"),
];

/// Split a state key into the data state and the own state (`design`, `diff` or "").
pub fn pd_split(k: &str) -> (ScreenState, &'static str) {
    match k {
        "design" => (ScreenState::Sample, "design"),
        "diff" => (ScreenState::Sample, "diff"),
        _ => (ScreenState::from_key(k), ""),
    }
}

/// Models: name and brand mark id.
pub const PD_MODELS: [(&str, &str); 3] = [
    ("Sonnet 5.5", "claude"),
    ("GPT-6", "openai"),
    ("Gemini 3 Pro", "gemini"),
];

/// Machines a session can run on: name and line.
pub const PD_MACHINES: [(&str, &str); 3] = [
    ("This Mac", "Runs here, in your folder"),
    ("build-box-2", "Persistent machine, 16 cores"),
    ("gpu-box-1", "Persistent machine with a GPU"),
];

/// Preview tabs: key, label, glyph.
pub const PD_TABS: [(&str, &str, &str); 3] = [
    ("site", "localhost:3000", "globe"),
    ("doc", "rollout.md", "doc"),
    ("diff", "Diff", "file"),
];

/// A session in the sidebar.
#[derive(Clone, Debug, PartialEq)]
pub struct PdSession {
    /// Stable id.
    pub id: u32,
    /// Title.
    pub title: String,
    /// Index into [`PD_MACHINES`].
    pub machine: usize,
    /// Working right now.
    pub running: bool,
    /// Comments waiting for the agent.
    pub notes: u32,
}

fn se(id: u32, title: &str, machine: usize, running: bool, notes: u32) -> PdSession {
    PdSession {
        id,
        title: title.to_string(),
        machine,
        running,
        notes,
    }
}

/// The sample sessions (empty for loading and empty).
pub fn pd_sessions(st: ScreenState) -> Vec<PdSession> {
    if st != ScreenState::Sample {
        return Vec::new();
    }
    vec![
        se(1, "Pricing page refresh", 1, true, 0),
        se(2, "Rollout plan for the new billing", 0, false, 2),
        se(3, "Speed up image thumbnails", 2, false, 0),
        se(4, "Onboarding checklist copy", 0, false, 0),
    ]
}

/// Start a session from a prompt at the top on `machine`; it starts working.
pub fn add_session(
    list: &mut Vec<PdSession>,
    prompt: &str,
    machine: usize,
) -> Result<u32, &'static str> {
    if let Some(e) = prompt_error(prompt) {
        return Err(e);
    }
    let id = list.iter().map(|s| s.id).max().unwrap_or(0) + 1;
    let t = super::taskdeck::title_from(prompt);
    list.insert(0, se(id, &t, machine.min(PD_MACHINES.len() - 1), true, 0));
    Ok(id)
}

/// The open session's prompt.
pub const PD_PROMPT: &str = "Refresh the pricing page: three plans side by side, yearly and monthly toggle, and highlight the Team plan.";

/// The agent's steps: glyph, text.
pub const PD_STEPS: [(&str, &str); 5] = [
    ("file", "Read app/pricing/page.tsx and the plan data"),
    ("pen", "Wrote components/PlanCard.tsx"),
    ("pen", "Edited app/pricing/page.tsx"),
    ("term", "Ran the dev server on port 3000"),
    ("globe", "Opened the preview and checked 390 px"),
];

/// The agent's closing line.
pub const PD_DONE: &str = "The page has three plan cards with a Monthly / Yearly switch, and Team is highlighted. Open the preview to check it, or use Design mode to point at anything you want changed.";

/// The plans on the live site: name, monthly price, yearly price per month.
pub const PD_PLANS: [(&str, u32, u32); 3] =
    [("Starter", 12, 10), ("Team", 29, 24), ("Business", 59, 49)];

/// The price a plan card shows for the toggle.
pub fn price(i: usize, yearly: bool) -> u32 {
    PD_PLANS
        .get(i)
        .map(|p| if yearly { p.2 } else { p.1 })
        .unwrap_or(0)
}

/// Elements Design mode can pick on the live site: key and label.
pub const PD_ELEMENTS: [(&str, &str); 4] = [
    ("title", "Heading"),
    ("toggle", "Monthly / Yearly switch"),
    ("team", "Team plan card"),
    ("cta", "Start trial button"),
];

/// A comment for the agent, on an element or a diff line.
#[derive(Clone, Debug, PartialEq)]
pub struct PdNote {
    /// What it is on ("Team plan card", "PlanCard.tsx:14").
    pub on: String,
    /// The comment.
    pub text: String,
}

/// Add a comment; blank text is refused.
pub fn add_note(list: &mut Vec<PdNote>, on: &str, text: &str) -> Result<usize, &'static str> {
    let t = text.trim();
    if t.is_empty() {
        return Err("Write a comment first");
    }
    list.push(PdNote {
        on: on.to_string(),
        text: t.to_string(),
    });
    Ok(list.len())
}

/// The sample comment in Design mode.
pub fn pd_notes(own: &str) -> Vec<PdNote> {
    match own {
        "design" => vec![PdNote {
            on: "Team plan card".to_string(),
            text: "Make the Most popular badge smaller and move it above the name.".to_string(),
        }],
        "diff" => vec![PdNote {
            on: "PlanCard.tsx:9".to_string(),
            text: "Use the shared Price component so the currency formats the same everywhere."
                .to_string(),
        }],
        _ => Vec::new(),
    }
}

/// The diff of components/PlanCard.tsx: (sign, line number, line).
pub const PD_DIFF: [(char, u32, &str); 10] = [
    (
        ' ',
        4,
        "export function PlanCard({ plan, yearly, featured }: Props) {",
    ),
    (' ', 5, "  return ("),
    ('-', 6, "    <div className=\"card\">"),
    (
        '+',
        6,
        "    <div className={featured ? \"card card--featured\" : \"card\"}>",
    ),
    (
        '+',
        7,
        "      {featured && <span className=\"badge\">Most popular</span>}",
    ),
    (' ', 8, "      <h3>{plan.name}</h3>"),
    (
        '+',
        9,
        "      <p className=\"price\">${yearly ? plan.yearly : plan.monthly}</p>",
    ),
    ('-', 9, "      <p className=\"price\">${plan.price}</p>"),
    (
        ' ',
        10,
        "      <a className=\"cta\" href=\"/signup\">Start trial</a>",
    ),
    (' ', 11, "    </div>"),
];

/// The doc tab: title and lines.
pub const PD_DOC: (&str, [&str; 4]) = (
    "Rollout plan",
    [
        "1. Ship the new page behind the pricing_v2 flag on 2026-11-24.",
        "2. Turn it on for 10% of visitors and watch trial starts for a week.",
        "3. Move everyone over on 2026-12-01 if trials hold or rise.",
        "4. Remove the old page and the flag on 2026-12-08.",
    ],
);

// ─────────────────────────────────────────────────────────── Data ──

/// Everything Preview Desk shows. Pass it to the screen's `data` prop; `Default` is the sample.
#[derive(Clone, Debug, PartialEq)]
pub struct PreviewDeskData {
    /// Sessions in the sidebar; the first one opens.
    pub sessions: Vec<PdSession>,
    /// The working folder.
    pub folder: String,
}

impl PreviewDeskData {
    /// The data for a demo state (sample, or nothing for empty and loading).
    pub fn for_state(st: ScreenState) -> Self {
        Self {
            sessions: pd_sessions(st),
            folder: "~/src/storefront".to_string(),
        }
    }
}

impl Default for PreviewDeskData {
    fn default() -> Self {
        Self::for_state(ScreenState::Sample)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn s16_preview_desk() {
        assert_eq!(pd_split("design").1, "design");
        let mut l = pd_sessions(ScreenState::Sample);
        assert!(add_session(&mut l, "", 0).is_err());
        add_session(&mut l, "Make the footer sticky", 9).unwrap();
        assert_eq!((l[0].machine, l[0].running), (2, true));
        assert_eq!(price(1, false), 29);
        assert_eq!(price(1, true), 24);
        let mut n = pd_notes("design");
        assert!(add_note(&mut n, "Heading", "  ").is_err());
        assert_eq!(add_note(&mut n, "Heading", "Shorter, please"), Ok(2));
        assert_eq!(pd_notes("diff").len(), 1);
        assert!(pd_notes("").is_empty());
        assert!(
            PreviewDeskData::for_state(ScreenState::Loading)
                .sessions
                .is_empty()
        );
    }
}
