//! Meeting summary: AI notes for one call. A short summarising pass, then Summary, Decisions
//! and Action items stream in; each line links to its transcript quote, and action items can be
//! checked off or sent to Linear or Notion (simulated).

use super::{STREAM_TICK_MS, norm_key, nth};

/// Summarising lasts this long, ms.
pub const THINK_MS: u64 = 1_000;

/// Sending an action item takes this long, ms.
pub const SEND_MS: u64 = 800;

/// The meeting: title, platform mark, when, minutes, transcript words.
pub const MEETING: (&str, &str, &str, u32, u32) = (
    "Pricing page launch sync",
    "zoom",
    "2026-11-18 15:00",
    34,
    5_820,
);

/// Attendees (fictional).
pub const ATTENDEES: [&str; 4] = ["Rina Okafor", "Tomas Lind", "Priya Sethi", "Jonah Weiss"];

/// One note line: section (0 summary, 1 decisions, 2 action items), text, timestamp, speaker,
/// quote, owner (action items), due (action items).
pub type Line = (
    u8,
    &'static str,
    &'static str,
    &'static str,
    &'static str,
    &'static str,
    &'static str,
);

/// The notes.
pub const LINES: [Line; 8] = [
    (
        0,
        "The team agreed the new pricing page ships on Tuesday with annual billing as the default.",
        "02:14",
        "Rina Okafor",
        "If annual is the default we should say the monthly price right next to it.",
        "",
        "",
    ),
    (
        0,
        "Checkout errors dropped after the retry fix, but the EU tax line still needs a final check.",
        "11:40",
        "Tomas Lind",
        "Errors are down about seventy percent since the retry fix went out.",
        "",
        "",
    ),
    (
        1,
        "Annual billing is the default, with the monthly price shown beside it.",
        "03:02",
        "Priya Sethi",
        "Let's default to annual and show monthly right beside it.",
        "",
        "",
    ),
    (
        1,
        "The Team plan keeps three included seats for launch.",
        "17:25",
        "Rina Okafor",
        "Three seats stays, we can revisit after the first month.",
        "",
        "",
    ),
    (
        1,
        "Launch moves only if EU tax is not verified by Monday noon.",
        "26:10",
        "Jonah Weiss",
        "Monday noon is the cutoff, otherwise we hold the launch.",
        "",
        "",
    ),
    (
        2,
        "Verify the EU tax line on the checkout summary.",
        "12:05",
        "Tomas Lind",
        "I'll check the EU tax line against the new rates.",
        "Tomas Lind",
        "Nov 23",
    ),
    (
        2,
        "Update the FAQ for annual billing and refunds.",
        "19:48",
        "Priya Sethi",
        "The FAQ still talks about monthly only, I'll fix it.",
        "Priya Sethi",
        "Nov 24",
    ),
    (
        2,
        "Prepare the launch email and the changelog post.",
        "29:30",
        "Jonah Weiss",
        "I can have the email and the changelog ready by Monday.",
        "Jonah Weiss",
        "Nov 24",
    ),
];

/// Card status.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Status {
    /// The summarising pass.
    Summarizing,
    /// Notes ready (streaming in after the pass).
    Ready,
    /// Failed.
    Error,
    /// No transcript.
    Empty,
}

/// One send of an action item.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Send {
    /// 0 Linear, 1 Notion.
    pub target: u8,
    /// Started at.
    pub at: u64,
    /// Fails (the first Notion send in the fail scenario).
    pub fails: bool,
}

/// A send's state.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SendSt {
    /// Not sent.
    None,
    /// Sending.
    Sending(u8),
    /// Sent with a reference number.
    Sent(u8, u32),
    /// Failed.
    Failed(u8),
}

/// The live state of a meeting summary card.
#[derive(Clone, Debug, PartialEq)]
pub struct Meeting {
    /// Status when last changed.
    pub status: Status,
    /// When the pass started.
    pub since: u64,
    /// Stream the notes in after the pass.
    pub stream: bool,
    /// Checked action items (by line index).
    pub checked: [bool; 8],
    /// Sends per line.
    pub sends: [Option<Send>; 8],
    /// Open quote (line index).
    pub quote: Option<usize>,
    /// Brief (summary + action items only) instead of full notes (Oxyn extra).
    pub brief: bool,
    /// The first send fails.
    pub fail_send: bool,
}

impl Meeting {
    /// `scenario`: `play` (summarise at `now`), `ready`, `error`, `empty`.
    pub fn new(scenario: &str, now: u64) -> Self {
        Self {
            status: match scenario {
                "ready" => Status::Ready,
                "error" => Status::Error,
                "empty" => Status::Empty,
                _ => Status::Summarizing,
            },
            since: if scenario == "ready" { 0 } else { now },
            stream: scenario != "ready",
            checked: [false; 8],
            sends: [None; 8],
            quote: None,
            brief: false,
            fail_send: false,
        }
    }

    /// Status at `now`.
    pub fn status_at(&self, now: u64) -> Status {
        if self.status == Status::Summarizing && now >= self.since + THINK_MS {
            Status::Ready
        } else {
            self.status
        }
    }

    /// Lines shown (brief drops decisions).
    pub fn lines(&self) -> Vec<usize> {
        (0..LINES.len())
            .filter(|&i| !(self.brief && LINES[i].0 == 1))
            .collect()
    }

    /// Words of line `i` visible at `now` (all when not streaming).
    pub fn visible(&self, i: usize, now: u64) -> usize {
        let n = LINES[i].1.split_whitespace().count();
        if self.status_at(now) != Status::Ready {
            return 0;
        }
        if !self.stream {
            return n;
        }
        let start = if self.status == Status::Summarizing {
            self.since + THINK_MS
        } else {
            self.since
        };
        let ticks = now.saturating_sub(start) / STREAM_TICK_MS + 1;
        let before: usize = self
            .lines()
            .iter()
            .take_while(|&&j| j != i)
            .map(|&j| LINES[j].1.split_whitespace().count())
            .sum();
        ((ticks as usize) * 5).saturating_sub(before).min(n)
    }

    /// True while notes are still streaming.
    pub fn streaming(&self, now: u64) -> bool {
        self.status_at(now) == Status::Ready
            && self
                .lines()
                .iter()
                .any(|&i| self.visible(i, now) < LINES[i].1.split_whitespace().count())
    }

    /// Send state of line `i`.
    pub fn send_st(&self, i: usize, now: u64) -> SendSt {
        match self.sends[i] {
            None => SendSt::None,
            Some(s) if now < s.at + SEND_MS => SendSt::Sending(s.target),
            Some(s) if s.fails => SendSt::Failed(s.target),
            Some(s) => SendSt::Sent(s.target, 140 + i as u32 * 3),
        }
    }

    /// The reference chip of a sent item (`PRC-155`, `Notion page`).
    pub fn reference(target: u8, n: u32) -> String {
        if target == 0 {
            format!("PRC-{n}")
        } else {
            "Notion page".into()
        }
    }

    /// Done action items over all action items.
    pub fn progress(&self) -> (usize, usize) {
        let items: Vec<usize> = (0..LINES.len()).filter(|&i| LINES[i].0 == 2).collect();
        (
            items.iter().filter(|&&i| self.checked[i]).count(),
            items.len(),
        )
    }

    /// Presses: `ms-check-<i>`, `ms-linear-<i>`, `ms-notion-<i>`, `ms-quote-<i>` (toggle),
    /// `ms-close`, `ms-regen`, `ms-copy`, `ms-brief`.
    pub fn press(&mut self, id: &str, now: u64) -> Option<String> {
        for (p, target) in [("ms-linear-", 0u8), ("ms-notion-", 1u8)] {
            if let Some(i) = nth(id, p) {
                if i >= LINES.len() || LINES[i].0 != 2 {
                    return None;
                }
                if matches!(self.send_st(i, now), SendSt::Sending(_) | SendSt::Sent(..)) {
                    return None;
                }
                let fails = self.fail_send && target == 1 && self.sends[i].is_none();
                self.sends[i] = Some(Send {
                    target,
                    at: now,
                    fails,
                });
                return Some(format!(
                    "Sending to {}, simulated, nothing was sent",
                    if target == 0 { "Linear" } else { "Notion" }
                ));
            }
        }
        if let Some(i) = nth(id, "ms-check-") {
            if i >= LINES.len() || LINES[i].0 != 2 {
                return None;
            }
            self.checked[i] = !self.checked[i];
            return Some(format!(
                "{} {}",
                LINES[i].1,
                if self.checked[i] {
                    "checked"
                } else {
                    "unchecked"
                }
            ));
        }
        if let Some(i) = nth(id, "ms-quote-") {
            self.quote = if self.quote == Some(i) { None } else { Some(i) };
            return self
                .quote
                .map(|q| format!("{}: {}", LINES[q].3, LINES[q].4));
        }
        match id {
            "ms-close" => {
                self.quote = None;
                None
            }
            "ms-regen" => {
                self.status = Status::Summarizing;
                self.since = now;
                self.stream = true;
                self.quote = None;
                Some("Summarizing the meeting".into())
            }
            "ms-copy" => Some("Copied the notes".into()),
            "ms-brief" => {
                self.brief = !self.brief;
                Some(
                    if self.brief {
                        "Brief notes"
                    } else {
                        "Full notes"
                    }
                    .into(),
                )
            }
            _ => None,
        }
    }

    /// Escape closes an open quote.
    pub fn key(&mut self, key: &str) -> Option<String> {
        (norm_key(key) == "Escape")
            .then(|| self.quote.take())
            .flatten()
            .map(|_| "Quote closed".into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn meeting_streams_checks_and_sends() {
        let mut m = Meeting::new("play", 0);
        assert_eq!(m.status_at(500), Status::Summarizing);
        assert_eq!(m.visible(0, 500), 0);
        assert!(m.visible(0, 1_000) > 0);
        assert!(m.streaming(1_100));
        assert!(!m.streaming(10_000));
        m.press("ms-check-5", 10_000);
        assert_eq!(m.progress(), (1, 3));
        assert!(m.press("ms-check-0", 0).is_none());
        m.fail_send = true;
        m.press("ms-notion-6", 10_000);
        assert_eq!(m.send_st(6, 10_100), SendSt::Sending(1));
        assert_eq!(m.send_st(6, 11_000), SendSt::Failed(1));
        m.press("ms-notion-6", 11_000);
        assert_eq!(m.send_st(6, 12_000), SendSt::Sent(1, 158));
        m.press("ms-linear-7", 12_000);
        assert!(matches!(m.send_st(7, 13_000), SendSt::Sent(0, _)));
        m.press("ms-quote-2", 0);
        assert_eq!(m.quote, Some(2));
        m.key("Escape");
        assert_eq!(m.quote, None);
        m.press("ms-brief", 0);
        assert_eq!(m.lines().len(), 5);
        let r = Meeting::new("ready", 0);
        assert_eq!(r.visible(7, 0), LINES[7].1.split_whitespace().count());
    }
}
