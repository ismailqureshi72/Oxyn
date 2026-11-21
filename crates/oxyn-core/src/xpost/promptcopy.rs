//! Agent prompt copy: a "Give to your agent" split button. The main part copies an agent-ready
//! prompt for the chosen coding agent; the chevron opens a menu of agents (each with its official
//! mark) and formats (a prompt, a skill file or an install command). A preview shows exactly what
//! will be copied, and the button morphs to a check with the agent's name for a moment.

use crate::aikit::rove;

/// Agents: (name, mark, how it takes a skill).
pub const AGENTS: [(&str, &str, &str); 5] = [
    ("Claude Code", "claudecode", ".claude/skills/"),
    ("Codex", "codex", "AGENTS.md"),
    ("Cursor", "cursor", ".cursor/rules/"),
    (
        "GitHub Copilot",
        "githubcopilot",
        ".github/copilot-instructions.md",
    ),
    ("Windsurf", "windsurf", ".windsurf/rules/"),
];
/// Formats.
pub const FORMATS: [&str; 3] = ["Prompt", "Skill file", "Install command"];
/// What is being handed over: (component, crate path).
pub const ITEM: (&str, &str) = ("Test ticks", "oxyn_dioxus::TestTicks");
/// Ms the copied state lasts.
pub const COPIED_MS: u64 = 1_800;

/// The button.
#[derive(Clone, Debug, PartialEq)]
pub struct PromptCopy {
    /// Agent.
    pub agent: usize,
    /// Format.
    pub format: usize,
    /// Menu open with this row focused (agents then formats).
    pub menu: Option<usize>,
    /// Copied at.
    pub copied: Option<u64>,
    /// Clipboard refused (error scenario).
    pub fail: bool,
}

impl PromptCopy {
    /// A button.
    pub fn new(agent: usize, format: usize, fail: bool) -> Self {
        Self {
            agent: agent.min(AGENTS.len() - 1),
            format: format.min(FORMATS.len() - 1),
            menu: None,
            copied: None,
            fail,
        }
    }

    /// The text that will be copied.
    pub fn text(&self) -> String {
        let (name, _, place) = AGENTS[self.agent];
        match self.format {
            1 => format!(
                "# {place}oxyn-{slug}.md\n---\nname: oxyn-{slug}\nwhen: adding a test run view\n---\nUse `{path}`. Pass the host clock as `now`.\nKeep the tick grid keyboard reachable.",
                slug = "test-ticks",
                path = ITEM.1
            ),
            2 => format!(
                "oxyn add test-ticks --agent {}",
                name.to_lowercase().replace(' ', "-")
            ),
            _ => format!(
                "Add the Oxyn {} component to this project.\n1. Run `oxyn add test-ticks`.\n2. Render `{}` where test results show.\n3. Wire `now` to the app clock.\n(For {name}.)",
                ITEM.0, ITEM.1
            ),
        }
    }

    /// Copied state showing.
    pub fn copied(&self, now: u64) -> bool {
        self.copied.is_some_and(|c| now < c + COPIED_MS)
    }

    /// Rows in the menu.
    pub fn rows() -> usize {
        AGENTS.len() + FORMATS.len()
    }

    /// Handles a press id.
    pub fn press(&mut self, id: &str, now: u64) -> Option<String> {
        match id {
            "ga-copy" => {
                self.menu = None;
                self.copied = Some(now);
                Some(if self.fail {
                    "Couldn't copy. Select the preview and copy it instead".into()
                } else {
                    format!(
                        "{} copied for {}",
                        FORMATS[self.format], AGENTS[self.agent].0
                    )
                })
            }
            "ga-menu" => {
                self.menu = if self.menu.is_some() {
                    None
                } else {
                    Some(self.agent)
                };
                self.menu.map(|_| "Choose an agent and a format".into())
            }
            "ga-close" => self.menu.take().map(|_| "Closed".into()),
            _ => {
                if let Some(i) = crate::aikit::nth(id, "ga-agent-") {
                    self.agent = i.min(AGENTS.len() - 1);
                    return Some(format!("For {}", AGENTS[self.agent].0));
                }
                let i = crate::aikit::nth(id, "ga-format-")?;
                self.format = i.min(FORMATS.len() - 1);
                Some(format!("Format: {}", FORMATS[self.format]))
            }
        }
    }

    /// Menu keys: arrows move, Enter picks, Escape closes. Closed: Enter copies, ArrowDown opens.
    pub fn key(&mut self, k: &str, now: u64) -> Option<String> {
        let Some(m) = self.menu else {
            return match k {
                "ArrowDown" => self.press("ga-menu", now),
                "Enter" | "Space" => self.press("ga-copy", now),
                _ => None,
            };
        };
        match k {
            "Escape" => self.press("ga-close", now),
            "Enter" | "Space" => {
                if m < AGENTS.len() {
                    self.press(&format!("ga-agent-{m}"), now)
                } else {
                    self.press(&format!("ga-format-{}", m - AGENTS.len()), now)
                }
            }
            _ => {
                let i = rove(m, Self::rows(), k);
                self.menu = Some(i);
                Some(if i < AGENTS.len() {
                    AGENTS[i].0.to_string()
                } else {
                    FORMATS[i - AGENTS.len()].to_string()
                })
            }
        }
    }
}
