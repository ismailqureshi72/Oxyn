//! Agent gallery: a marketplace of agent templates. Filter by team, search by agent, job, app or
//! model, preview a sample run, and install inline: choose a workspace, grant access app by app
//! (each connects in turn), and the card confirms with a check. Escape steps back; while apps
//! connect it cancels the install.

use super::{nth, rove, thousands};

/// Teams, in order.
pub const TEAMS: [&str; 5] = ["Sales", "Support", "Engineering", "Marketing", "Ops"];
/// Workspaces offered on install: name, detail.
pub const WORKSPACES: [(&str, &str); 3] = [
    ("Northwind", "12 members"),
    ("Northwind Labs", "4 members"),
    ("Personal", "Just you"),
];
/// One app connecting, ms.
pub const CONNECT_MS: u64 = 900;

/// A template: name, team, job, apps (brand ids), model (brand, label), rating ×10, installs,
/// three preview steps.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Template {
    /// Name.
    pub name: &'static str,
    /// Team index.
    pub team: usize,
    /// One-line job.
    pub job: &'static str,
    /// Apps it connects to.
    pub apps: &'static [&'static str],
    /// Model brand and label.
    pub model: (&'static str, &'static str),
    /// Rating ×10.
    pub rating: u32,
    /// Installs.
    pub installs: u64,
    /// A sample run in three steps.
    pub preview: [&'static str; 3],
}

#[allow(clippy::too_many_arguments)]
const fn t(
    name: &'static str,
    team: usize,
    job: &'static str,
    apps: &'static [&'static str],
    model: (&'static str, &'static str),
    rating: u32,
    installs: u64,
    preview: [&'static str; 3],
) -> Template {
    Template {
        name,
        team,
        job,
        apps,
        model,
        rating,
        installs,
        preview,
    }
}

/// The catalog (fictional agents).
pub const TEMPLATES: [Template; 12] = [
    t(
        "Lead Qualifier",
        0,
        "Scores new leads and books a call with the good ones",
        &["hubspot", "gmail", "googlecalendar"],
        ("claude", "Claude Sonnet 4.5"),
        48,
        12_400,
        [
            "New lead from the pricing page",
            "Fit score 86: books a 20-minute call",
            "Notes the call on the HubSpot deal",
        ],
    ),
    t(
        "Renewal Watch",
        0,
        "Flags accounts that might not renew and drafts a check-in",
        &["hubspot", "slack"],
        ("openai", "GPT-5"),
        46,
        5_900,
        [
            "Usage down 40% at Halcyon Labs",
            "Drafts a check-in for the account owner",
            "Posts it to #renewals for review",
        ],
    ),
    t(
        "Ticket Triage",
        1,
        "Tags, routes and answers the easy tickets first",
        &["intercom", "slack"],
        ("claude", "Claude Haiku 4.5"),
        47,
        18_200,
        [
            "38 new tickets overnight",
            "Answers 12 password resets",
            "Routes 4 billing tickets to the right team",
        ],
    ),
    t(
        "Refund Desk",
        1,
        "Checks invoices and handles refund requests under a limit",
        &["stripe", "intercom"],
        ("claude", "Claude Sonnet 4.5"),
        45,
        7_100,
        [
            "Refund request for $49",
            "Finds the duplicate charge in Stripe",
            "Refunds it and replies to the customer",
        ],
    ),
    t(
        "PR Reviewer",
        2,
        "Reviews pull requests and leaves inline suggestions",
        &["github", "linear"],
        ("openai", "GPT-5 Codex"),
        49,
        23_800,
        [
            "PR #418 opened: pricing toggle",
            "Finds an off-by-one in the yearly total",
            "Leaves 3 inline suggestions",
        ],
    ),
    t(
        "Incident Scribe",
        2,
        "Keeps the incident timeline and writes the review",
        &["sentry", "slack", "github"],
        ("claude", "Claude Opus 4.1"),
        47,
        6_300,
        [
            "Error rate up after the 14:02 deploy",
            "Builds the timeline from Slack and Sentry",
            "Drafts the incident review",
        ],
    ),
    t(
        "Release Notes",
        2,
        "Turns merged pull requests into release notes",
        &["github", "notion"],
        ("gemini", "Gemini 2.5 Flash"),
        44,
        4_200,
        [
            "12 PRs merged this week",
            "Groups them into features and fixes",
            "Publishes the notes to Notion",
        ],
    ),
    t(
        "Campaign Planner",
        3,
        "Plans a launch campaign across email and social",
        &["hubspot", "notion"],
        ("gemini", "Gemini 2.5 Pro"),
        43,
        3_800,
        [
            "Launch: yearly billing",
            "Plans 3 emails and 5 posts",
            "Puts the calendar in Notion",
        ],
    ),
    t(
        "Content Repurposer",
        3,
        "Turns one article into posts, a thread and a newsletter",
        &["notion", "x"],
        ("claude", "Claude Sonnet 4.5"),
        45,
        9_600,
        [
            "Article: How we price seats",
            "Writes a thread and 3 posts",
            "Drafts the newsletter section",
        ],
    ),
    t(
        "Spend Tracker",
        4,
        "Reconciles card spend against budgets every Friday",
        &["stripe", "googledrive", "slack"],
        ("openai", "GPT-5 mini"),
        44,
        2_900,
        [
            "142 card transactions this week",
            "2 over the design budget",
            "Posts the summary to #finance",
        ],
    ),
    t(
        "Onboarding Buddy",
        4,
        "Walks new hires through their first two weeks",
        &["slack", "notion", "googlecalendar"],
        ("claude", "Claude Haiku 4.5"),
        46,
        5_400,
        [
            "New hire starts Monday",
            "Books intro calls with 4 teammates",
            "Sends the day-one checklist",
        ],
    ),
    t(
        "Vendor Desk",
        4,
        "Collects quotes and compares vendors side by side",
        &["gmail", "googledrive"],
        ("gemini", "Gemini 2.5 Pro"),
        42,
        1_700,
        [
            "Three quotes for SSO",
            "Builds a comparison sheet",
            "Recommends one with reasons",
        ],
    ),
];

/// A card's flow.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Flow {
    /// Summary.
    Card,
    /// Sample run preview.
    Preview,
    /// Choosing a workspace.
    Workspace(usize),
    /// Connecting apps since (host ms), into workspace.
    Connect(usize, u64),
}

/// The live state of the gallery.
#[derive(Clone, Debug, PartialEq)]
pub struct Gallery {
    /// Team filter.
    pub team: usize,
    /// Search text.
    pub query: String,
    /// The card in a flow (template index) and its flow.
    pub flow: Option<(usize, Flow)>,
    /// Installed templates and where.
    pub installed: Vec<(usize, usize)>,
    /// The second app fails to connect (the error scenario).
    pub fail: bool,
}

impl Gallery {
    /// The gallery on `team`.
    pub fn new(team: usize, fail: bool) -> Self {
        Self {
            team: team.min(TEAMS.len() - 1),
            query: String::new(),
            flow: None,
            installed: Vec::new(),
            fail,
        }
    }

    /// Does template `i` match the search?
    pub fn matches(&self, i: usize) -> bool {
        let q = self.query.trim().to_lowercase();
        let tp = &TEMPLATES[i];
        q.is_empty()
            || tp.name.to_lowercase().contains(&q)
            || tp.job.to_lowercase().contains(&q)
            || tp.model.1.to_lowercase().contains(&q)
            || tp
                .apps
                .iter()
                .any(|a| a.contains(&q) || app_name(a).to_lowercase().contains(&q))
    }

    /// Templates shown: the team's, matching the search.
    pub fn shown(&self) -> Vec<usize> {
        (0..TEMPLATES.len())
            .filter(|&i| TEMPLATES[i].team == self.team && self.matches(i))
            .collect()
    }

    /// When the team has no match, the team that has one (the empty state's "Show in").
    pub fn elsewhere(&self) -> Option<usize> {
        if !self.shown().is_empty() {
            return None;
        }
        (0..TEMPLATES.len())
            .find(|&i| self.matches(i))
            .map(|i| TEMPLATES[i].team)
    }

    /// Connection progress at `now` for the card in the connect flow: apps connected, failed
    /// app, all done.
    pub fn connect(&self, now: u64) -> Option<(usize, Option<usize>, bool)> {
        let (i, Flow::Connect(_, t)) = self.flow? else {
            return None;
        };
        let n = TEMPLATES[i].apps.len();
        let k = ((now.saturating_sub(t)) / CONNECT_MS) as usize;
        if self.fail && n > 1 && k >= 1 {
            return Some((1, Some(1), false));
        }
        Some((k.min(n), None, k >= n))
    }

    /// Install result of template `i`.
    pub fn installed_in(&self, i: usize) -> Option<&'static str> {
        self.installed
            .iter()
            .find(|x| x.0 == i)
            .map(|x| WORKSPACES[x.1].0)
    }

    /// Settles a finished install at `now` (called on every press).
    pub fn settle(&mut self, now: u64) -> Option<String> {
        let (i, ws) = match self.flow {
            Some((i, Flow::Connect(ws, _))) => (i, ws),
            _ => return None,
        };
        if self.connect(now)?.2 {
            self.flow = None;
            self.installed.push((i, ws));
            return Some(format!(
                "{} installed in {}",
                TEMPLATES[i].name, WORKSPACES[ws].0
            ));
        }
        None
    }

    /// The result line (`3 agents in Engineering`).
    pub fn result_line(&self) -> String {
        let n = self.shown().len();
        format!(
            "{n} agent{} in {}",
            if n == 1 { "" } else { "s" },
            TEAMS[self.team]
        )
    }

    /// Presses: `ag-team-<i>`, `ag-preview-<i>`, `ag-install-<i>`, `ag-ws-<w>`, `ag-grant`,
    /// `ag-back`, `ag-retry`, `ag-clear`.
    pub fn press(&mut self, id: &str, now: u64) -> Option<String> {
        if let Some(s) = self.settle(now)
            && id == "ag-back"
        {
            return Some(s);
        }
        if let Some(i) = nth(id, "ag-team-") {
            self.team = i.min(TEAMS.len() - 1);
            self.flow = None;
            return Some(self.result_line());
        }
        if let Some(i) = nth(id, "ag-preview-") {
            self.flow = Some((i, Flow::Preview));
            return Some(format!("Sample run of {}", TEMPLATES.get(i)?.name));
        }
        if let Some(i) = nth(id, "ag-install-") {
            if self.installed_in(i).is_some() {
                return None;
            }
            self.flow = Some((i, Flow::Workspace(0)));
            return Some(format!(
                "Install {}: choose a workspace",
                TEMPLATES.get(i)?.name
            ));
        }
        if let Some(w) = nth(id, "ag-ws-") {
            if let Some((i, Flow::Workspace(_))) = self.flow {
                self.flow = Some((i, Flow::Workspace(w.min(WORKSPACES.len() - 1))));
                return Some(WORKSPACES[w.min(WORKSPACES.len() - 1)].0.to_string());
            }
            return None;
        }
        match id {
            "ag-grant" => {
                let (i, Flow::Workspace(w)) = self.flow? else {
                    return None;
                };
                self.flow = Some((i, Flow::Connect(w, now)));
                let apps: Vec<&str> = TEMPLATES[i].apps.iter().map(|a| app_name(a)).collect();
                Some(format!(
                    "Connecting {}. Simulated, nothing was connected",
                    apps.join(", ")
                ))
            }
            "ag-retry" => {
                let (i, Flow::Connect(w, _)) = self.flow? else {
                    return None;
                };
                self.fail = false;
                self.flow = Some((i, Flow::Connect(w, now)));
                Some("Connecting again".into())
            }
            "ag-back" => self.flow.take().map(|_| "Back to the summary".to_string()),
            "ag-clear" => {
                self.query.clear();
                Some(self.result_line())
            }
            _ => None,
        }
    }

    /// Keys: Left, Right, Home, End on the team chips (or the workspaces during install);
    /// Escape steps back or clears the search.
    pub fn key(&mut self, key: &str, now: u64) -> Option<String> {
        let k = super::norm_key(key);
        match (k, self.flow) {
            ("Escape", Some(_)) => self.press("ag-back", now),
            ("Escape", None) => self.press("ag-clear", now),
            (
                "ArrowLeft" | "ArrowRight" | "ArrowUp" | "ArrowDown" | "Home" | "End",
                Some((_, Flow::Workspace(w))),
            ) => {
                let n = rove(w, WORKSPACES.len(), k);
                self.press(&format!("ag-ws-{n}"), now)
            }
            ("ArrowLeft" | "ArrowRight" | "Home" | "End", None) => {
                let n = rove(self.team, TEAMS.len(), k);
                (n != self.team)
                    .then(|| self.press(&format!("ag-team-{n}"), now))
                    .flatten()
            }
            _ => None,
        }
    }
}

/// Display name of an app id.
pub fn app_name(id: &str) -> &'static str {
    match id {
        "hubspot" => "HubSpot",
        "gmail" => "Gmail",
        "googlecalendar" => "Google Calendar",
        "googledrive" => "Google Drive",
        "slack" => "Slack",
        "intercom" => "Intercom",
        "stripe" => "Stripe",
        "github" => "GitHub",
        "linear" => "Linear",
        "sentry" => "Sentry",
        "notion" => "Notion",
        "x" => "X",
        _ => "an app",
    }
}

/// Rating and installs in words (`4.8 out of 5, 12,400 installs`).
pub fn rating_words(tp: &Template) -> String {
    format!(
        "{}.{} out of 5, {} installs",
        tp.rating / 10,
        tp.rating % 10,
        thousands(tp.installs)
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn filter_search_install() {
        let mut g = Gallery::new(2, false);
        assert_eq!(g.shown(), vec![4, 5, 6]);
        g.query = "sentry".into();
        assert_eq!(g.shown(), vec![5]);
        g.query = "stripe".into();
        assert!(g.shown().is_empty());
        assert_eq!(g.elsewhere(), Some(1));
        g.key("Escape", 0);
        g.key("ArrowRight", 0);
        assert_eq!(g.team, 3);
        g.press("ag-team-2", 0);
        g.press("ag-install-4", 0);
        g.key("ArrowDown", 0);
        assert_eq!(g.flow, Some((4, Flow::Workspace(1))));
        g.press("ag-grant", 0);
        assert_eq!(g.connect(1_000), Some((1, None, false)));
        g.press("ag-back", 5_000);
        assert_eq!(g.installed_in(4), Some("Northwind Labs"));
        let mut f = Gallery::new(2, true);
        f.press("ag-install-5", 0);
        f.press("ag-grant", 0);
        assert_eq!(f.connect(2_000), Some((1, Some(1), false)));
        f.press("ag-retry", 2_000);
        f.press("ag-preview-0", 9_000);
        assert_eq!(f.installed_in(5), Some("Northwind"));
    }
}
