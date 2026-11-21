//! Publish popover: a Publish button in a document toolbar opens a popover that slides out of the
//! button. It holds the page address (editable slug), who can open it, a host to deploy to (with
//! its official mark) and two switches. Publishing walks through upload, build and live steps;
//! once live the popover shows the link with Copy, a view counter and Unpublish.

use crate::aikit::rove;

/// The document.
pub const DOC: (&str, &str) = ("Launch plan", "Quillworks · edited 4 min ago");
/// Base address.
pub const BASE: &str = "quillworks.notes.page/";
/// Who can open it.
pub const ACCESS: [&str; 3] = ["Anyone with the link", "Team only", "Password"];
/// Hosts: (name, mark).
pub const HOSTS: [(&str, &str); 3] = [
    ("Cloudflare", "cloudflare"),
    ("Vercel", "vercel"),
    ("Netlify", "netlify"),
];
/// Steps: (doing, ms).
pub const STEPS: [(&str, u64); 3] = [
    ("Uploading 12 files", 900),
    ("Building the page", 1_100),
    ("Going live", 600),
];

/// Where the popover is.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase {
    /// Closed, not published.
    Closed,
    /// Open with the form.
    Form,
    /// Publishing (step index).
    Publishing(usize),
    /// Live.
    Live,
    /// The slug is taken.
    Taken,
}

/// The popover.
#[derive(Clone, Debug, PartialEq)]
pub struct Publish {
    /// Open.
    pub open: bool,
    /// Slug.
    pub slug: String,
    /// Access choice.
    pub access: usize,
    /// Host choice.
    pub host: usize,
    /// Search engines may index it.
    pub index: bool,
    /// Show the agent's edit history on the page.
    pub history: bool,
    /// Publish pressed at.
    pub started: Option<u64>,
    /// Link copied at.
    pub copied: Option<u64>,
    /// The first publish hits a taken slug.
    pub taken: bool,
}

impl Publish {
    /// A popover; `open` starts it open.
    pub fn new(open: bool, taken: bool) -> Self {
        Self {
            open,
            slug: "launch-plan".into(),
            access: 0,
            host: 0,
            index: false,
            history: true,
            started: None,
            copied: None,
            taken,
        }
    }

    fn total() -> u64 {
        STEPS.iter().map(|s| s.1).sum()
    }

    /// The phase.
    pub fn phase(&self, now: u64) -> Phase {
        if let Some(s) = self.started {
            let e = now.saturating_sub(s);
            if self.taken && e >= STEPS[0].1 {
                return Phase::Taken;
            }
            if e >= Self::total() {
                return if self.open {
                    Phase::Live
                } else {
                    Phase::Closed
                };
            }
            let mut a = 0;
            for (i, st) in STEPS.iter().enumerate() {
                a += st.1;
                if e < a {
                    return Phase::Publishing(i);
                }
            }
        }
        if self.open {
            Phase::Form
        } else {
            Phase::Closed
        }
    }

    /// Published and live (popover open or not).
    pub fn live(&self, now: u64) -> bool {
        !self.taken && self.started.is_some_and(|s| now >= s + Self::total())
    }

    /// Progress 0..1 while publishing.
    pub fn progress(&self, now: u64) -> f64 {
        self.started
            .map(|s| (now.saturating_sub(s) as f64 / Self::total() as f64).min(1.0))
            .unwrap_or(0.0)
    }

    /// The full address.
    pub fn url(&self) -> String {
        format!("{BASE}{}", self.slug)
    }

    /// Views since going live (sample counter).
    pub fn views(&self, now: u64) -> u64 {
        self.started
            .map(|s| now.saturating_sub(s + Self::total()) / 2_300)
            .unwrap_or(0)
    }

    /// Cleans a typed slug (lowercase, dashes).
    pub fn set_slug(&mut self, s: &str) {
        let mut out = String::new();
        for ch in s.chars() {
            if ch.is_ascii_alphanumeric() {
                out.push(ch.to_ascii_lowercase());
            } else if !out.ends_with('-') && !out.is_empty() {
                out.push('-');
            }
        }
        self.slug = out.chars().take(40).collect();
        if self.taken {
            self.started = None;
            self.taken = false;
        }
    }

    /// Handles a press id.
    pub fn press(&mut self, id: &str, now: u64) -> Option<String> {
        match id {
            "pp-toggle" => {
                self.open = !self.open;
                Some(
                    if self.open {
                        "Publish options"
                    } else {
                        "Closed"
                    }
                    .into(),
                )
            }
            "pp-close" => {
                self.open = false;
                Some("Closed".into())
            }
            "pp-go"
                if matches!(self.phase(now), Phase::Form | Phase::Taken)
                    && !self.slug.is_empty() =>
            {
                while self.slug.ends_with('-') {
                    self.slug.pop();
                }
                if self.phase(now) == Phase::Taken {
                    self.taken = false;
                    self.slug.push_str("-2");
                }
                self.started = Some(now);
                Some("Publishing".into())
            }
            "pp-copy" if self.live(now) => {
                self.copied = Some(now);
                Some(format!("Copied {}", self.url()))
            }
            "pp-unpublish" if self.live(now) => {
                self.started = None;
                Some("Unpublished".into())
            }
            "pp-index" => {
                self.index = !self.index;
                Some(format!(
                    "Indexing {}",
                    if self.index { "on" } else { "off" }
                ))
            }
            "pp-history" => {
                self.history = !self.history;
                Some(format!(
                    "Edit history {}",
                    if self.history { "shown" } else { "hidden" }
                ))
            }
            _ => {
                if let Some(i) = crate::aikit::nth(id, "pp-access-") {
                    self.access = i.min(ACCESS.len() - 1);
                    return Some(ACCESS[self.access].into());
                }
                let i = crate::aikit::nth(id, "pp-host-")?;
                self.host = i.min(HOSTS.len() - 1);
                Some(format!("Deploys to {}", HOSTS[self.host].0))
            }
        }
    }

    /// Keys: Escape closes, Enter publishes (or copies when live), arrows move `part`
    /// (`access` or `host`).
    pub fn key(&mut self, part: &str, k: &str, now: u64) -> Option<String> {
        match k {
            "Escape" if self.open => self.press("pp-close", now),
            "Enter" if self.live(now) => self.press("pp-copy", now),
            "Enter" => self.press("pp-go", now),
            _ if part == "access" => {
                let i = rove(self.access, ACCESS.len(), k);
                self.press(&format!("pp-access-{i}"), now)
            }
            _ if part == "host" => {
                let i = rove(self.host, HOSTS.len(), k);
                self.press(&format!("pp-host-{i}"), now)
            }
            _ => None,
        }
    }
}
