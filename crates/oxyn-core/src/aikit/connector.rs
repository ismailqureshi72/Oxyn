//! Connector setup: the "Connect your tools" step of an assistant. Every app is linked without
//! leaving the card: a redirect, the grant, then an initial import shown by a thin bar; afterwards
//! the card lists what access was given and can unlink the app after a confirming press.

use super::{nth, thousands};

/// Redirect step, ms.
pub const REDIRECT_MS: u64 = 600;
/// Authorise step, ms.
pub const AUTH_MS: u64 = 900;
/// First sync, ms.
pub const SYNC_MS: u64 = 2_200;

/// An app: mark, name, what it syncs, item noun, items, scopes.
pub type App = (
    &'static str,
    &'static str,
    &'static str,
    &'static str,
    u64,
    [&'static str; 2],
);

/// Apps.
pub const APPS: [App; 7] = [
    (
        "gmail",
        "Gmail",
        "Read and draft email",
        "emails",
        4_210,
        ["Read messages", "Create drafts"],
    ),
    (
        "googledrive",
        "Google Drive",
        "Search docs and sheets",
        "files",
        1_388,
        ["Read files", "Search files"],
    ),
    (
        "notion",
        "Notion",
        "Pages and databases",
        "pages",
        642,
        ["Read pages", "Update databases"],
    ),
    (
        "slack",
        "Slack",
        "Channels you are in",
        "messages",
        12_900,
        ["Read channels", "Post as the assistant"],
    ),
    (
        "github",
        "GitHub",
        "Issues and pull requests",
        "issues",
        318,
        ["Read repositories", "Comment on issues"],
    ),
    (
        "linear",
        "Linear",
        "Issues and projects",
        "issues",
        957,
        ["Read issues", "Create issues"],
    ),
    (
        "hubspot",
        "HubSpot",
        "Contacts and deals",
        "records",
        2_045,
        ["Read contacts", "Update deals"],
    ),
];

/// A connection's state.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ConnSt {
    /// Not connected.
    Off,
    /// Redirecting to the app.
    Redirecting,
    /// Waiting for the grant.
    Authorizing,
    /// First sync, with the fraction done.
    Syncing(f64),
    /// Connected.
    On,
    /// The grant was denied (retry).
    Denied,
    /// Asking to confirm Disconnect.
    Confirm,
}

/// The live state of the card.
#[derive(Clone, Debug, PartialEq)]
pub struct Connectors {
    /// Connected at (per app): `Some(t)` once Connect was pressed.
    pub started: [Option<u64>; 7],
    /// Apps already connected before the card opened.
    pub done: [bool; 7],
    /// Disconnect confirm open.
    pub confirm: Option<usize>,
    /// Apps whose next grant is denied (the error scenario: HubSpot).
    pub deny: [bool; 7],
    /// Search.
    pub query: String,
    /// Expanded row (scopes).
    pub open: Option<usize>,
}

impl Connectors {
    /// Gmail connected already; `deny` makes HubSpot's first grant fail.
    pub fn new(deny: bool) -> Self {
        let mut d = [false; 7];
        d[6] = deny;
        let mut done = [false; 7];
        done[0] = true;
        Self {
            started: [None; 7],
            done,
            confirm: None,
            deny: d,
            query: String::new(),
            open: Some(0),
        }
    }

    /// State of app `i`.
    pub fn st(&self, i: usize, now: u64) -> ConnSt {
        if self.confirm == Some(i) {
            return ConnSt::Confirm;
        }
        if self.done[i] {
            return ConnSt::On;
        }
        let Some(t) = self.started[i] else {
            return ConnSt::Off;
        };
        let el = now.saturating_sub(t);
        if el < REDIRECT_MS {
            ConnSt::Redirecting
        } else if el < REDIRECT_MS + AUTH_MS {
            ConnSt::Authorizing
        } else if self.deny[i] {
            ConnSt::Denied
        } else if el < REDIRECT_MS + AUTH_MS + SYNC_MS {
            ConnSt::Syncing((el - REDIRECT_MS - AUTH_MS) as f64 / SYNC_MS as f64)
        } else {
            ConnSt::On
        }
    }

    /// The status line of app `i`.
    pub fn line(&self, i: usize, now: u64) -> String {
        let a = APPS[i];
        match self.st(i, now) {
            ConnSt::Off | ConnSt::Confirm => a.2.to_string(),
            ConnSt::Redirecting => format!("Opening {}…", a.1),
            ConnSt::Authorizing => "Waiting for you to allow access…".into(),
            ConnSt::Syncing(f) => format!(
                "Syncing {} of {} {}",
                thousands((a.4 as f64 * f) as u64),
                thousands(a.4),
                a.3
            ),
            ConnSt::On => format!("{} {} synced", thousands(a.4), a.3),
            ConnSt::Denied => "Access was not granted".into(),
        }
    }

    /// Apps matching the search.
    pub fn rows(&self) -> Vec<usize> {
        let q = self.query.to_lowercase();
        (0..APPS.len())
            .filter(|&i| {
                q.is_empty()
                    || APPS[i].1.to_lowercase().contains(&q)
                    || APPS[i].2.to_lowercase().contains(&q)
            })
            .collect()
    }

    /// Connected count.
    pub fn connected(&self, now: u64) -> usize {
        (0..APPS.len())
            .filter(|&i| matches!(self.st(i, now), ConnSt::On | ConnSt::Confirm))
            .count()
    }

    /// Presses: `cs-connect-<i>`, `cs-retry-<i>`, `cs-disc-<i>` (asks), `cs-yes-<i>`, `cs-no`,
    /// `cs-open-<i>`, `cs-q:<text>`.
    pub fn press(&mut self, id: &str, now: u64) -> Option<String> {
        if let Some(q) = id.strip_prefix("cs-q:") {
            self.query = q.to_string();
            return None;
        }
        if let Some(i) = nth(id, "cs-connect-") {
            if i >= APPS.len() || self.st(i, now) != ConnSt::Off {
                return None;
            }
            self.started[i] = Some(now);
            self.open = Some(i);
            return Some(format!("Connecting {}, simulated", APPS[i].1));
        }
        if let Some(i) = nth(id, "cs-retry-") {
            if self.st(i, now) != ConnSt::Denied {
                return None;
            }
            self.deny[i] = false;
            self.started[i] = Some(now);
            return Some(format!("Connecting {} again", APPS[i].1));
        }
        if let Some(i) = nth(id, "cs-disc-") {
            if self.st(i, now) != ConnSt::On {
                return None;
            }
            self.confirm = Some(i);
            return Some(format!(
                "Disconnect {}? The assistant loses access.",
                APPS[i].1
            ));
        }
        if let Some(i) = nth(id, "cs-yes-") {
            if self.confirm != Some(i) {
                return None;
            }
            self.confirm = None;
            self.done[i] = false;
            self.started[i] = None;
            return Some(format!("Disconnected {}", APPS[i].1));
        }
        if let Some(i) = nth(id, "cs-open-") {
            self.open = if self.open == Some(i) { None } else { Some(i) };
            return None;
        }
        if id == "cs-no" {
            self.confirm = None;
            return Some("Kept connected".into());
        }
        None
    }

    /// Escape cancels a Disconnect confirm.
    pub fn key(&mut self, key: &str, now: u64) -> Option<String> {
        if super::norm_key(key) == "Escape" && self.confirm.is_some() {
            return self.press("cs-no", now);
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn connectors_connect_deny_and_disconnect() {
        let mut c = Connectors::new(true);
        assert_eq!(c.st(0, 0), ConnSt::On);
        c.press("cs-connect-2", 0);
        assert_eq!(c.st(2, 100), ConnSt::Redirecting);
        assert_eq!(c.st(2, 1_000), ConnSt::Authorizing);
        assert!(matches!(c.st(2, 2_000), ConnSt::Syncing(_)));
        assert!(c.line(2, 2_000).starts_with("Syncing"));
        assert_eq!(c.st(2, 10_000), ConnSt::On);
        c.press("cs-connect-6", 0);
        assert_eq!(c.st(6, 5_000), ConnSt::Denied);
        c.press("cs-retry-6", 5_000);
        assert_eq!(c.st(6, 20_000), ConnSt::On);
        assert_eq!(c.connected(20_000), 3);
        c.press("cs-disc-2", 20_000);
        assert_eq!(c.st(2, 20_000), ConnSt::Confirm);
        c.key("Escape", 20_000);
        assert_eq!(c.st(2, 20_000), ConnSt::On);
        c.press("cs-disc-2", 20_000);
        c.press("cs-yes-2", 20_000);
        assert_eq!(c.st(2, 20_000), ConnSt::Off);
        c.press("cs-q:spot", 0);
        assert_eq!(c.rows(), vec![6]);
    }
}
