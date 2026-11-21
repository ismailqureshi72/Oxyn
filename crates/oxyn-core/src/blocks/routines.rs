//! Agent routines: scheduled agents in one card. Each row reads as a sentence: the routine's name
//! and an on/off switch, its schedule in plain words with a live countdown to the next run, the
//! sources it reads, where it posts and the model (as marks), and its last 14 runs as ticks. Run
//! now plays the run's steps on a thin progress line and adds a tick; a schedule that comes due
//! runs by itself. Opening a row shows the latest report, the instruction and a frequency switch.

use super::{dur, norm_key, nth, rove};

/// How often a routine runs.
pub const FREQS: [&str; 4] = ["Hourly", "Daily", "Weekdays", "Weekly"];

/// A routine's run result.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tick {
    /// Succeeded.
    Ok,
    /// Failed.
    Failed,
    /// Skipped (nothing new).
    Skipped,
}

impl Tick {
    /// Word.
    pub fn word(self) -> &'static str {
        match self {
            Tick::Ok => "Succeeded",
            Tick::Failed => "Failed",
            Tick::Skipped => "Skipped",
        }
    }
    /// Hue.
    pub fn hue(self) -> &'static str {
        match self {
            Tick::Ok => "green",
            Tick::Failed => "red",
            Tick::Skipped => "neutral",
        }
    }
}

/// One routine.
#[derive(Clone, Debug, PartialEq)]
pub struct Routine {
    /// Name.
    pub name: String,
    /// What it does, as written.
    pub instruction: String,
    /// Frequency index into [`FREQS`].
    pub freq: usize,
    /// Time of day in minutes (minute past the hour for hourly).
    pub at_min: u32,
    /// Day for weekly (0 Monday … 6 Sunday).
    pub day: u32,
    /// Sources (brand ids).
    pub sources: Vec<&'static str>,
    /// Destination (`#channel` or `Direct message`).
    pub dest: String,
    /// Model: brand id and label.
    pub model: (&'static str, &'static str),
    /// On.
    pub enabled: bool,
    /// Last runs, oldest first.
    pub history: Vec<Tick>,
    /// Series colour of its progress line.
    pub hue: &'static str,
    /// A run in progress: started at, manual, fails.
    pub run: Option<(u64, bool, bool)>,
    /// The scheduled run due during the preview already happened.
    pub ran_due: bool,
}

/// Days of the week.
pub const DAYS: [&str; 7] = ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"];
/// The preview's wall clock at its start: Monday 08:58:40, in seconds since Monday 00:00.
pub const WALL0: u64 = 8 * 3600 + 58 * 60 + 40;
/// One step of a run, ms.
pub const STEP_MS: u64 = 1_100;

/// The steps of a run of `r`.
pub fn steps(r: &Routine) -> Vec<String> {
    let mut v: Vec<String> = r
        .sources
        .iter()
        .map(|s| format!("Reading {}", brand_name(s)))
        .collect();
    v.push("Writing the summary".into());
    v.push(format!("Posting to {}", r.dest));
    v
}

/// Display name of a source brand id.
pub fn brand_name(id: &str) -> &'static str {
    match id {
        "linear" => "Linear",
        "github" => "GitHub",
        "sentry" => "Sentry",
        "googlecalendar" => "Google Calendar",
        "notion" => "Notion",
        "slack" => "Slack",
        _ => "the source",
    }
}

#[allow(clippy::too_many_arguments)]
fn r(
    name: &str,
    instruction: &str,
    freq: usize,
    at_min: u32,
    day: u32,
    sources: Vec<&'static str>,
    dest: &str,
    model: (&'static str, &'static str),
    enabled: bool,
    pattern: &str,
    hue: &'static str,
) -> Routine {
    let history = pattern
        .chars()
        .map(|c| match c {
            'x' => Tick::Failed,
            '-' => Tick::Skipped,
            _ => Tick::Ok,
        })
        .collect();
    Routine {
        name: name.into(),
        instruction: instruction.into(),
        freq,
        at_min,
        day,
        sources,
        dest: dest.into(),
        model,
        enabled,
        history,
        hue,
        run: None,
        ran_due: false,
    }
}

/// The schedule in words (`Weekdays at 09:00`).
pub fn schedule(r: &Routine) -> String {
    let hm = format!("{:02}:{:02}", r.at_min / 60, r.at_min % 60);
    match r.freq {
        0 => format!("Every hour at :{:02}", r.at_min % 60),
        1 => format!("Every day at {hm}"),
        2 => format!("Weekdays at {hm}"),
        _ => format!("Every {} at {hm}", DAYS[r.day as usize % 7]),
    }
}

/// Seconds from `wall` (seconds since Monday 00:00) to the next run of `r`.
pub fn next_in(r: &Routine, wall: u64) -> u64 {
    let week = 7 * 86_400;
    let w = wall % week;
    let cands: Vec<u64> = match r.freq {
        0 => (0..24 * 7)
            .map(|h| h * 3600 + u64::from(r.at_min % 60) * 60)
            .collect(),
        1 => (0..7)
            .map(|d| d * 86_400 + u64::from(r.at_min) * 60)
            .collect(),
        2 => (0..5)
            .map(|d| d * 86_400 + u64::from(r.at_min) * 60)
            .collect(),
        _ => vec![u64::from(r.day % 7) * 86_400 + u64::from(r.at_min) * 60],
    };
    cands
        .iter()
        .map(|&c| if c > w { c - w } else { c + week - w })
        .min()
        .unwrap_or(week)
}

/// The live state of the block.
#[derive(Clone, Debug, PartialEq)]
pub struct Routines {
    /// Clock start, ms.
    pub t0: u64,
    /// Routines.
    pub list: Vec<Routine>,
    /// Open row.
    pub open: Option<usize>,
    /// Focused row (roving).
    pub focus: usize,
    /// The next manual run of Error triage fails (the error scenario).
    pub fail: bool,
}

impl Routines {
    /// The sample routines at `now`.
    pub fn new(fail: bool, now: u64) -> Self {
        Self {
            t0: now,
            list: vec![
                r(
                    "Morning stand-up digest",
                    "Summarise what changed in Linear and GitHub since yesterday, grouped by project, with blockers first.",
                    2,
                    9 * 60,
                    0,
                    vec!["linear", "github"],
                    "#eng-standup",
                    ("claude", "Claude Sonnet 4.5"),
                    true,
                    "oooooo-oooxoo",
                    "blue",
                ),
                r(
                    "Error triage",
                    "Group new Sentry issues by cause, link the commits that likely caused them and flag anything that pages.",
                    0,
                    15,
                    0,
                    vec!["sentry", "github"],
                    "#oncall",
                    ("openai", "GPT-5"),
                    true,
                    "ooo-oooxoooooo",
                    "violet",
                ),
                r(
                    "Weekly roadmap recap",
                    "Write a short recap of the week's roadmap progress from Linear and the Notion planning doc.",
                    3,
                    16 * 60,
                    4,
                    vec!["linear", "notion"],
                    "Direct message",
                    ("claude", "Claude Opus 4.1"),
                    true,
                    "oooooooooooooo",
                    "teal",
                ),
                r(
                    "Meeting prep",
                    "Before each meeting, gather the related docs and the last notes into a one-page brief.",
                    1,
                    7 * 60 + 30,
                    0,
                    vec!["googlecalendar", "notion"],
                    "Direct message",
                    ("gemini", "Gemini 2.5 Pro"),
                    false,
                    "oooo--oooo----",
                    "amber",
                ),
            ],
            open: None,
            focus: 0,
            fail,
        }
    }

    /// The wall clock at `now`, seconds since Monday 00:00.
    pub fn wall(&self, now: u64) -> u64 {
        WALL0 + now.saturating_sub(self.t0) / 1000
    }

    /// When routine `i`'s first scheduled run during the preview is due (host ms).
    pub fn due(&self, i: usize) -> u64 {
        self.t0 + next_in(&self.list[i], WALL0) * 1000
    }

    /// The run in progress for `i` at `now`: started, manual, fails (a due schedule counts).
    pub fn active(&self, i: usize, now: u64) -> Option<(u64, bool, bool)> {
        let r = &self.list[i];
        let len = STEP_MS * steps(r).len() as u64;
        if let Some(x) = r.run.filter(|x| now < x.0 + len) {
            return Some(x);
        }
        let d = self.due(i);
        (r.enabled && !r.ran_due && now >= d && now < d + len).then_some((d, false, false))
    }

    /// Run history at `now`, finished runs (manual and due) folded in, last 14.
    pub fn history(&self, i: usize, now: u64) -> Vec<Tick> {
        let r = &self.list[i];
        let len = STEP_MS * steps(r).len() as u64;
        let mut h = r.history.clone();
        if r.enabled && !r.ran_due && now >= self.due(i) + len {
            h.push(Tick::Ok);
        }
        if let Some((s, _, f)) = r.run
            && now >= s + len
        {
            h.push(if f { Tick::Failed } else { Tick::Ok });
        }
        let n = h.len();
        h.split_off(n.saturating_sub(14))
    }

    /// Progress of a run: the step index, its label, fraction done, and whether it failed there.
    pub fn progress(&self, i: usize, now: u64) -> Option<(usize, String, f64, bool)> {
        let (s, _, f) = self.active(i, now)?;
        let st = steps(&self.list[i]);
        let el = now.saturating_sub(s);
        let k = ((el / STEP_MS) as usize).min(st.len() - 1);
        let fail_k = if f { Some(0) } else { None };
        if fail_k.is_some_and(|fk| k > fk) {
            return Some((0, st[0].clone(), 1.0 / st.len() as f64, true));
        }
        Some((
            k,
            st[k].clone(),
            el as f64 / (STEP_MS * st.len() as u64) as f64,
            false,
        ))
    }

    /// The countdown line (`Next in 1m 20s`, `Paused`, `Running now`).
    pub fn next_line(&self, i: usize, now: u64) -> String {
        let r = &self.list[i];
        if !r.enabled {
            return "Paused".into();
        }
        if self.active(i, now).is_some() {
            return "Running now".into();
        }
        let s = next_in(r, self.wall(now));
        if s < 86_400 {
            format!("Next in {}", dur(s * 1000))
        } else {
            let at = (self.wall(now) + s) % (7 * 86_400);
            format!(
                "Next {} {:02}:{:02}",
                DAYS[(at / 86_400) as usize],
                (at % 86_400) / 3600,
                (at % 3600) / 60
            )
        }
    }

    /// The last run line (`Last run succeeded`).
    pub fn last_line(&self, i: usize, now: u64) -> (String, Tick) {
        let t = *self.history(i, now).last().unwrap_or(&Tick::Skipped);
        (format!("Last run {}", t.word().to_lowercase()), t)
    }

    /// The outcome tally read for the tick strip.
    pub fn strip_label(&self, i: usize, now: u64) -> String {
        let h = self.history(i, now);
        let n = |t: Tick| h.iter().filter(|&&x| x == t).count();
        format!(
            "{} succeeded, {} failed, {} skipped",
            n(Tick::Ok),
            n(Tick::Failed),
            n(Tick::Skipped)
        )
    }

    /// One sentence for assistive technology about the marks.
    pub fn marks_line(&self, i: usize) -> String {
        let r = &self.list[i];
        let src: Vec<&str> = r.sources.iter().map(|s| brand_name(s)).collect();
        format!(
            "Reads {}, posts to {}, runs on {}",
            src.join(" and "),
            r.dest,
            r.model.1
        )
    }

    /// The latest report (short lines).
    pub fn report(&self, i: usize) -> Vec<String> {
        match i {
            0 => vec![
                "Checkout: 3 PRs merged, 1 waiting on review".into(),
                "Blocked: the billing migration waits on a schema review".into(),
                "Shipped: dark mode for the dashboard".into(),
            ],
            1 => vec![
                "2 new issues, both from the 14:02 deploy".into(),
                "Likely cause: the cache key change in #1287".into(),
                "Nothing paged".into(),
            ],
            2 => vec![
                "7 of 9 roadmap items on track".into(),
                "Search relevance slipped a week".into(),
            ],
            _ => vec!["Nothing yet: this routine is new".into()],
        }
    }

    /// Presses: `ro-open-<i>`, `ro-run-<i>`, `ro-toggle-<i>`, `ro-freq-<i>-<f>`, `ro-new`.
    pub fn press(&mut self, id: &str, now: u64) -> Option<String> {
        if let Some(i) = nth(id, "ro-open-") {
            self.open = if self.open == Some(i) { None } else { Some(i) };
            self.focus = i;
            return None;
        }
        if let Some(i) = nth(id, "ro-run-") {
            let running = self.active(i, now);
            let r = self.list.get_mut(i)?;
            if let Some((_, manual, _)) = running {
                if manual {
                    r.run = None;
                } else {
                    r.ran_due = true;
                }
                return Some(format!("Stopped {}", r.name));
            }
            let f = self.fail && i == 1;
            r.run = Some((now, true, f));
            return Some(format!(
                "Running {}. The preview simulates the run and the post",
                r.name
            ));
        }
        if let Some(i) = nth(id, "ro-toggle-") {
            let r = self.list.get_mut(i)?;
            r.enabled = !r.enabled;
            return Some(format!(
                "{} {}",
                r.name,
                if r.enabled { "on" } else { "paused" }
            ));
        }
        if let Some(rest) = id.strip_prefix("ro-freq-") {
            let (i, f) = rest.split_once('-')?;
            let (i, f): (usize, usize) = (i.parse().ok()?, f.parse().ok()?);
            let r = self.list.get_mut(i)?;
            r.freq = f.min(FREQS.len() - 1);
            r.ran_due = true;
            return Some(format!("{}: {}", r.name, schedule(r)));
        }
        if id == "ro-new" {
            let mut n = r(
                "New routine",
                "Every weekday at 9, summarise new Linear issues into a direct message.",
                2,
                9 * 60,
                0,
                vec!["linear"],
                "Direct message",
                ("claude", "Claude Sonnet 4.5"),
                true,
                "",
                "blue",
            );
            n.ran_due = true;
            self.list.insert(0, n);
            self.open = Some(0);
            self.focus = 0;
            return Some("New routine added: Weekdays at 09:00, reads Linear".into());
        }
        None
    }

    /// Keys on the rows: Up, Down, Home, End move focus; Enter opens; on an open row Left and
    /// Right change its frequency.
    pub fn key(&mut self, key: &str, now: u64) -> Option<String> {
        let k = norm_key(key);
        match k {
            "ArrowUp" | "ArrowDown" | "Home" | "End" => {
                self.focus = rove(self.focus, self.list.len(), k);
                None
            }
            "Enter" | "Space" => self.press(&format!("ro-open-{}", self.focus), now),
            "ArrowLeft" | "ArrowRight" => {
                let i = self.open?;
                let f = rove(self.list[i].freq, FREQS.len(), k);
                self.press(&format!("ro-freq-{i}-{f}"), now)
            }
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn schedule_countdown_and_runs() {
        let mut ro = Routines::new(true, 0);
        assert_eq!(schedule(&ro.list[0]), "Weekdays at 09:00");
        assert_eq!(ro.next_line(0, 0), "Next in 1m 20s");
        assert_eq!(ro.next_line(3, 0), "Paused");
        assert!(ro.next_line(2, 0).starts_with("Next Fri"));
        let len = STEP_MS * 4;
        assert!(ro.active(0, 80_000 + 10).is_some());
        assert_eq!(ro.history(0, 80_000 + len).len(), 14);
        assert_eq!(ro.last_line(0, 80_000 + len).1, Tick::Ok);
        ro.press("ro-run-1", 1_000);
        assert!(ro.progress(1, 2_000).is_some());
        assert_eq!(ro.last_line(1, 1_000 + len).1, Tick::Failed);
        ro.press("ro-toggle-3", 0);
        assert!(ro.list[3].enabled);
        ro.press("ro-open-2", 0);
        ro.key("ArrowLeft", 0);
        assert_eq!(ro.list[2].freq, 2);
        ro.press("ro-new", 0);
        assert_eq!(ro.list.len(), 5);
        assert!(ro.strip_label(2, 0).contains("1 failed"));
        assert!(ro.marks_line(1).contains("Linear and GitHub"));
    }
}
