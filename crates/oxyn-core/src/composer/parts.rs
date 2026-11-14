//! Composer parts (plan #27, chunk 3): voice recorder, dictation, mention / slash input, mode
//! switch, permission selector, tools menu, queue stack and agent steering.
//!
//! Specs: `composer-voice-input.md`, `composer-slash-mentions.md`, `composer-mode-switch.md`,
//! `composer-permission-selector.md`, `composer-tools-menu.md`, `composer-queue-stack.md`.
//! Behaviour follows Arc UI (Voice recorder, Dictation button, Mention input) and affectUI (Agent
//! Steering), rebuilt from their docs; no source used.

use std::time::Duration;

use crate::ClockMs;

/// `m:ss` for a duration in ms.
pub fn mmss(ms: u64) -> String {
    let s = ms / 1000;
    format!("{}:{:02}", s / 60, s % 60)
}

// ─────────────────────────────────────── voice recorder ──

/// Recorder phase.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RecPhase {
    /// Mic button only.
    #[default]
    Idle,
    /// Waiting for the permission prompt.
    Requesting,
    /// Recording (level meter, timer).
    Recording,
    /// Paused (timer holds).
    Paused,
    /// Stopped: play back, send or discard.
    Review,
    /// Microphone permission denied.
    Denied,
}

impl RecPhase {
    /// Id.
    pub fn id(self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::Requesting => "requesting",
            Self::Recording => "recording",
            Self::Paused => "paused",
            Self::Review => "review",
            Self::Denied => "denied",
        }
    }
    /// From an id.
    pub fn from_id(s: &str) -> Self {
        match s {
            "requesting" => Self::Requesting,
            "recording" => Self::Recording,
            "paused" => Self::Paused,
            "review" => Self::Review,
            "denied" => Self::Denied,
            _ => Self::Idle,
        }
    }
}

/// A voice recorder: accumulates recorded time across pauses and stops at `max`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Recorder {
    /// Phase.
    pub phase: RecPhase,
    /// Recorded before the current segment (ms).
    pub banked: u64,
    /// Current segment start.
    pub since: Option<ClockMs>,
    /// Limit (ms).
    pub max: u64,
}

impl Recorder {
    /// A recorder with a limit.
    pub fn new(max: Duration) -> Self {
        Self {
            phase: RecPhase::Idle,
            banked: 0,
            since: None,
            max: max.as_millis() as u64,
        }
    }
    /// Mic pressed: ask for permission.
    pub fn request(&mut self) {
        if matches!(self.phase, RecPhase::Idle | RecPhase::Denied) {
            self.phase = RecPhase::Requesting;
        }
    }
    /// Permission answered.
    pub fn permission(&mut self, granted: bool, now: ClockMs) {
        if self.phase != RecPhase::Requesting {
            return;
        }
        if granted {
            self.phase = RecPhase::Recording;
            self.banked = 0;
            self.since = Some(now);
        } else {
            self.phase = RecPhase::Denied;
        }
    }
    /// Recorded ms at `now`.
    pub fn elapsed(&self, now: ClockMs) -> u64 {
        let live = match (self.phase, self.since) {
            (RecPhase::Recording, Some(s)) => now.since(s).as_millis() as u64,
            _ => 0,
        };
        (self.banked + live).min(self.max)
    }
    /// Pause / resume.
    pub fn toggle_pause(&mut self, now: ClockMs) {
        match self.phase {
            RecPhase::Recording => {
                self.banked = self.elapsed(now);
                self.since = None;
                self.phase = RecPhase::Paused;
            }
            RecPhase::Paused => {
                self.since = Some(now);
                self.phase = RecPhase::Recording;
            }
            _ => {}
        }
    }
    /// Stop: go to review.
    pub fn stop(&mut self, now: ClockMs) {
        if matches!(self.phase, RecPhase::Recording | RecPhase::Paused) {
            self.banked = self.elapsed(now);
            self.since = None;
            self.phase = RecPhase::Review;
        }
    }
    /// Advances the clock; stops at the limit. Returns true when it just stopped.
    pub fn tick(&mut self, now: ClockMs) -> bool {
        if self.phase == RecPhase::Recording && self.elapsed(now) >= self.max {
            self.stop(now);
            return true;
        }
        false
    }
    /// Discard: back to idle.
    pub fn discard(&mut self) {
        *self = Self {
            max: self.max,
            ..Self::new(Duration::ZERO)
        };
    }
    /// Remaining ms before the limit.
    pub fn remaining(&self, now: ClockMs) -> u64 {
        self.max - self.elapsed(now)
    }
    /// Whether the limit warning shows (last 10 s).
    pub fn near_limit(&self, now: ClockMs) -> bool {
        matches!(self.phase, RecPhase::Recording | RecPhase::Paused)
            && self.remaining(now) <= 10_000
    }
}

/// A deterministic level meter: `n` bar heights (0–1) at `t` ms (the demo's stand-in for an
/// analyser node). Silence (`level` 0) draws flat bars.
pub fn levels(t: u64, n: usize, level: f32) -> Vec<f32> {
    (0..n)
        .map(|i| {
            let x = (t as f32 / 90.0) + i as f32 * 0.7;
            let v = ((x.sin() * 0.5 + 0.5) * 0.6 + ((x * 2.3).cos() * 0.5 + 0.5) * 0.4) * level;
            v.clamp(0.06, 1.0)
        })
        .collect()
}

// ─────────────────────────────────────────── dictation ──

/// Dictation phase.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum DictPhase {
    /// Ready.
    #[default]
    Idle,
    /// Listening: interim words stream into the field.
    Listening,
    /// Finalising the transcript.
    Transcribing,
    /// Recognition failed.
    Error,
}

/// Interim words heard `elapsed` after listening began at `wpm` words per minute.
pub fn interim(script: &str, elapsed: Duration, wpm: f32) -> String {
    let n = (elapsed.as_secs_f32() * wpm / 60.0) as usize;
    script
        .split_whitespace()
        .take(n)
        .collect::<Vec<_>>()
        .join(" ")
}

/// Inserts dictated text at the caret with sensible spacing; returns the new text and caret.
pub fn insert_dictation(text: &str, caret: usize, words: &str) -> (String, usize) {
    let caret = caret.min(text.len());
    let (a, b) = text.split_at(caret);
    let lead = if a.is_empty() || a.ends_with(char::is_whitespace) {
        ""
    } else {
        " "
    };
    let trail = if b.is_empty() || b.starts_with(char::is_whitespace) {
        ""
    } else {
        " "
    };
    let ins = format!("{lead}{words}{trail}");
    let out = format!("{a}{ins}{b}");
    (out, caret + lead.len() + words.len())
}

// ─────────────────────────────────────── mentions / slash ──

/// Trigger kind.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Trigger {
    /// `@` people, files, agents.
    Mention,
    /// `/` commands (only at the start of a line).
    Slash,
}

/// The active trigger token around the caret: kind, byte start of the trigger and the query.
pub fn active_token(text: &str, caret: usize) -> Option<(Trigger, usize, String)> {
    let caret = caret.min(text.len());
    let head = &text[..caret];
    let start = head
        .rfind(|c: char| c.is_whitespace())
        .map(|i| i + 1)
        .unwrap_or(0);
    let tok = &head[start..];
    let mut ch = tok.chars();
    let kind = match ch.next()? {
        '@' => Trigger::Mention,
        '/' => {
            let line_start = head[..start].rfind('\n').map(|i| i + 1).unwrap_or(0);
            if !head[line_start..start].trim().is_empty() {
                return None;
            }
            Trigger::Slash
        }
        _ => return None,
    };
    Some((kind, start, ch.as_str().to_string()))
}

/// A suggestion.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Suggestion {
    /// Id inserted (`maya`, `src/auth.rs`, `review`).
    pub id: String,
    /// Label.
    pub label: String,
    /// Group (`People`, `Files`, `Agents`, `Commands`).
    pub group: String,
    /// Hint (role, path, description).
    pub hint: String,
}

impl Suggestion {
    /// A suggestion.
    pub fn new(id: &str, label: &str, group: &str, hint: &str) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            group: group.into(),
            hint: hint.into(),
        }
    }
}

/// Ranks suggestions for `query`: prefix matches first, then substring matches, then the rest
/// dropped; capped at `limit`.
pub fn rank(all: &[Suggestion], query: &str, limit: usize) -> Vec<Suggestion> {
    let q = query.to_lowercase();
    let mut pre = vec![];
    let mut sub = vec![];
    for s in all {
        let l = s.label.to_lowercase();
        let id = s.id.to_lowercase();
        if q.is_empty() || l.starts_with(&q) || id.starts_with(&q) {
            pre.push(s.clone());
        } else if l.contains(&q) || id.contains(&q) {
            sub.push(s.clone());
        }
    }
    pre.extend(sub);
    pre.truncate(limit);
    pre
}

/// Accepts a suggestion: replaces the token with `@id ` (or `/id `). Returns text and caret.
pub fn accept(text: &str, caret: usize, kind: Trigger, start: usize, id: &str) -> (String, usize) {
    let caret = caret.min(text.len());
    let sigil = if kind == Trigger::Mention { '@' } else { '/' };
    let ins = format!("{sigil}{id} ");
    let out = format!("{}{}{}", &text[..start], ins, &text[caret..]);
    (out, start + ins.len())
}

/// Splits text into plain runs and `@mention` / leading `/command` pills for display.
pub fn pills(text: &str, known: &[String]) -> Vec<(bool, String)> {
    let mut out: Vec<(bool, String)> = vec![];
    for (i, w) in text.split_inclusive(' ').enumerate() {
        let core = w.trim_end();
        let pill = (core.starts_with('@') && known.iter().any(|k| *k == core[1..]))
            || (i == 0 && core.starts_with('/') && known.iter().any(|k| *k == core[1..]));
        match out.last_mut() {
            Some((false, s)) if !pill => s.push_str(w),
            _ => {
                if pill {
                    out.push((true, core.to_string()));
                    if w.len() > core.len() {
                        out.push((false, w[core.len()..].to_string()));
                    }
                } else {
                    out.push((false, w.to_string()));
                }
            }
        }
    }
    out
}

// ─────────────────────────────────────────── mode switch ──

/// A mode.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Mode {
    /// Id.
    pub id: String,
    /// Label.
    pub label: String,
    /// Description.
    pub desc: String,
    /// Placeholder the composer shows.
    pub placeholder: String,
    /// Restrictiveness: lower = more restrictive.
    pub rank: u8,
}

impl Mode {
    /// A mode.
    pub fn new(id: &str, label: &str, desc: &str, placeholder: &str, rank: u8) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            desc: desc.into(),
            placeholder: placeholder.into(),
            rank,
        }
    }
}

/// Ask · Plan · Agent.
pub fn default_modes() -> Vec<Mode> {
    vec![
        Mode::new(
            "ask",
            "Ask",
            "Answers only; never edits.",
            "Ask anything…",
            0,
        ),
        Mode::new(
            "plan",
            "Plan",
            "Drafts a plan for you to approve.",
            "Describe what to plan…",
            1,
        ),
        Mode::new(
            "agent",
            "Agent",
            "Edits, runs and opens PRs.",
            "Tell the agent what to do…",
            2,
        ),
    ]
}

/// When a mode change applies.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Applied {
    /// Right away.
    Now,
    /// After the current run's turn.
    NextTurn,
}

/// Mode switch state.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ModeSwitch {
    /// Modes in cycle order.
    pub modes: Vec<Mode>,
    /// Current.
    pub current: usize,
    /// Pending (mid-run loosening).
    pub pending: Option<usize>,
}

impl ModeSwitch {
    /// From modes and the current id.
    pub fn new(modes: Vec<Mode>, current: &str) -> Self {
        let current = modes.iter().position(|m| m.id == current).unwrap_or(0);
        Self {
            modes,
            current,
            pending: None,
        }
    }
    /// Selects `i`. While `busy`, tightening applies now and loosening waits for the next turn.
    pub fn select(&mut self, i: usize, busy: bool) -> Option<Applied> {
        if i >= self.modes.len() || (i == self.current && self.pending.is_none()) {
            return None;
        }
        if i == self.current {
            self.pending = None;
            return Some(Applied::Now);
        }
        if busy && self.modes[i].rank > self.modes[self.current].rank {
            self.pending = Some(i);
            Some(Applied::NextTurn)
        } else {
            self.current = i;
            self.pending = None;
            Some(Applied::Now)
        }
    }
    /// Shift+Tab: next mode in order.
    pub fn cycle(&mut self, busy: bool) -> Option<Applied> {
        let from = self.pending.unwrap_or(self.current);
        self.select((from + 1) % self.modes.len(), busy)
    }
    /// The run finished its turn: a pending mode takes over.
    pub fn turn_end(&mut self) {
        if let Some(p) = self.pending.take() {
            self.current = p;
        }
    }
    /// Current mode.
    pub fn mode(&self) -> &Mode {
        &self.modes[self.current]
    }
}

// ───────────────────────────────────── permission selector ──

/// Risk of a permission preset.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Risk {
    /// Read only.
    Low,
    /// Edits with approval.
    Medium,
    /// Runs without asking.
    High,
}

impl Risk {
    /// Id.
    pub fn id(self) -> &'static str {
        match self {
            Self::Low => "low",
            Self::Medium => "medium",
            Self::High => "high",
        }
    }
}

/// A permission preset.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Preset {
    /// Id.
    pub id: String,
    /// Label.
    pub label: String,
    /// What it allows.
    pub desc: String,
    /// Risk.
    pub risk: Risk,
}

impl Preset {
    /// A preset.
    pub fn new(id: &str, label: &str, desc: &str, risk: Risk) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            desc: desc.into(),
            risk,
        }
    }
}

/// Read only · Ask before edits · Auto-edit · Full access.
pub fn default_presets() -> Vec<Preset> {
    vec![
        Preset::new(
            "read",
            "Read only",
            "Reads and searches; asks for everything else.",
            Risk::Low,
        ),
        Preset::new(
            "ask",
            "Ask before edits",
            "Edits after you approve each change.",
            Risk::Medium,
        ),
        Preset::new(
            "auto",
            "Auto-edit",
            "Edits files freely; asks before commands.",
            Risk::Medium,
        ),
        Preset::new(
            "full",
            "Full access",
            "Runs commands and pushes without asking.",
            Risk::High,
        ),
    ]
}

/// Result of choosing a preset.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Choice {
    /// Applied.
    Applied(String),
    /// High-risk preset waits for a confirm.
    Confirm(String),
    /// Blocked by policy.
    Blocked(String),
}

/// Permission selector state.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Permissions {
    /// Presets.
    pub presets: Vec<Preset>,
    /// Current id.
    pub current: String,
    /// Waiting for a confirm.
    pub confirming: Option<String>,
    /// Highest risk the workspace policy allows.
    pub ceiling: Risk,
}

impl Permissions {
    /// A selector.
    pub fn new(presets: Vec<Preset>, current: &str) -> Self {
        Self {
            presets,
            current: current.into(),
            confirming: None,
            ceiling: Risk::High,
        }
    }
    /// The current preset.
    pub fn preset(&self) -> Option<&Preset> {
        self.presets.iter().find(|p| p.id == self.current)
    }
    /// Chooses a preset.
    pub fn choose(&mut self, id: &str) -> Option<Choice> {
        let p = self.presets.iter().find(|p| p.id == id)?.clone();
        if p.risk > self.ceiling {
            return Some(Choice::Blocked(id.into()));
        }
        if p.risk == Risk::High && self.current != id {
            self.confirming = Some(id.into());
            return Some(Choice::Confirm(id.into()));
        }
        self.current = id.into();
        self.confirming = None;
        Some(Choice::Applied(id.into()))
    }
    /// Confirm or cancel the pending high-risk preset.
    pub fn confirm(&mut self, yes: bool) -> Option<String> {
        let id = self.confirming.take()?;
        if yes {
            self.current = id.clone();
            Some(id)
        } else {
            None
        }
    }
}

// ─────────────────────────────────────────── tools menu ──

/// A tool.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Tool {
    /// Id.
    pub id: String,
    /// Label.
    pub label: String,
    /// Group (`Built in`, `MCP · GitHub`).
    pub group: String,
    /// Enabled.
    pub on: bool,
    /// Needs sign-in before it can be enabled.
    pub needs_auth: bool,
}

impl Tool {
    /// A tool.
    pub fn new(id: &str, label: &str, group: &str, on: bool) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            group: group.into(),
            on,
            needs_auth: false,
        }
    }
    /// Needs sign-in.
    pub fn auth(mut self) -> Self {
        self.needs_auth = true;
        self
    }
}

/// Tools visible for a query, grouped in first-seen group order.
pub fn tool_groups(tools: &[Tool], query: &str) -> Vec<(String, Vec<usize>)> {
    let q = query.to_lowercase();
    let mut out: Vec<(String, Vec<usize>)> = vec![];
    for (i, t) in tools.iter().enumerate() {
        if !q.is_empty()
            && !t.label.to_lowercase().contains(&q)
            && !t.group.to_lowercase().contains(&q)
        {
            continue;
        }
        match out.iter_mut().find(|(g, _)| *g == t.group) {
            Some((_, v)) => v.push(i),
            None => out.push((t.group.clone(), vec![i])),
        }
    }
    out
}

/// Toggles a tool unless it needs sign-in; returns whether it changed.
pub fn toggle_tool(tools: &mut [Tool], i: usize) -> bool {
    match tools.get_mut(i) {
        Some(t) if !t.needs_auth => {
            t.on = !t.on;
            true
        }
        _ => false,
    }
}

/// Toggles a whole group: all on unless all are already on.
pub fn toggle_group(tools: &mut [Tool], group: &str) {
    let all_on = tools
        .iter()
        .filter(|t| t.group == group && !t.needs_auth)
        .all(|t| t.on);
    for t in tools
        .iter_mut()
        .filter(|t| t.group == group && !t.needs_auth)
    {
        t.on = !all_on;
    }
}

// ─────────────────────────────────────── queue / steering ──

/// A queued follow-up.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Queued {
    /// Id.
    pub id: String,
    /// Text.
    pub text: String,
}

/// Queue stack: follow-ups typed while the agent runs, sent one per finished turn.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Queue {
    /// Items, next first.
    pub items: Vec<Queued>,
    /// Paused: nothing auto-sends.
    pub paused: bool,
    /// Editing id.
    pub editing: Option<String>,
    next: u32,
}

impl Queue {
    /// From texts.
    pub fn new(texts: &[&str]) -> Self {
        let mut q = Self::default();
        for t in texts {
            q.push(t);
        }
        q
    }
    /// Adds to the end.
    pub fn push(&mut self, text: &str) -> String {
        self.next += 1;
        let id = format!("q{}", self.next);
        self.items.push(Queued {
            id: id.clone(),
            text: text.trim().into(),
        });
        id
    }
    /// Removes an item.
    pub fn remove(&mut self, id: &str) -> Option<Queued> {
        let i = self.items.iter().position(|q| q.id == id)?;
        Some(self.items.remove(i))
    }
    /// Moves an item by `delta`.
    pub fn shift(&mut self, id: &str, delta: i32) {
        if let Some(i) = self.items.iter().position(|q| q.id == id) {
            let j = (i as i32 + delta).clamp(0, self.items.len() as i32 - 1) as usize;
            let it = self.items.remove(i);
            self.items.insert(j, it);
        }
    }
    /// Saves an edit (empty text removes it).
    pub fn edit(&mut self, id: &str, text: &str) {
        if text.trim().is_empty() {
            self.remove(id);
        } else if let Some(q) = self.items.iter_mut().find(|q| q.id == id) {
            q.text = text.trim().into();
        }
        self.editing = None;
    }
    /// The run finished a turn: returns the next item to send unless paused.
    pub fn turn_end(&mut self) -> Option<Queued> {
        if self.paused || self.items.is_empty() {
            return None;
        }
        Some(self.items.remove(0))
    }
    /// Send now: pulls an item out to interrupt with.
    pub fn send_now(&mut self, id: &str) -> Option<Queued> {
        self.remove(id)
    }
}

/// Steering message phase (affectUI Agent Steering).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SteerPhase {
    /// Waiting for the next step boundary.
    Pending,
    /// Read by the agent at a step boundary.
    Delivered,
    /// The agent acknowledged and changed course.
    Applied,
}

/// A steering note on a running agent.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Steer {
    /// Text.
    pub text: String,
    /// Sent at.
    pub at: ClockMs,
    /// Interrupt (stops the current step) instead of waiting.
    pub interrupt: bool,
}

impl Steer {
    /// Phase at `now`, given the agent's step length: an interrupt delivers at once; otherwise it
    /// waits for the next step boundary, then applies one step later.
    pub fn phase(&self, now: ClockMs, step: Duration) -> SteerPhase {
        let e = now.since(self.at).as_millis() as u64;
        let s = step.as_millis().max(1) as u64;
        let deliver = if self.interrupt {
            0
        } else {
            s - (self.at.0 % s)
        };
        if e < deliver {
            SteerPhase::Pending
        } else if e < deliver + s {
            SteerPhase::Delivered
        } else {
            SteerPhase::Applied
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recorder_pauses_and_limits() {
        let mut r = Recorder::new(Duration::from_secs(60));
        r.request();
        r.permission(true, ClockMs(0));
        assert_eq!(r.elapsed(ClockMs(5_000)), 5_000);
        r.toggle_pause(ClockMs(5_000));
        assert_eq!(r.elapsed(ClockMs(9_000)), 5_000);
        r.toggle_pause(ClockMs(9_000));
        assert!(r.near_limit(ClockMs(60_000)));
        assert!(r.tick(ClockMs(70_000)));
        assert_eq!(r.phase, RecPhase::Review);
        assert_eq!(r.banked, 60_000);
        assert_eq!(mmss(r.banked), "1:00");
        r.discard();
        assert_eq!(r.phase, RecPhase::Idle);
        assert_eq!(r.max, 60_000);
        let mut d = Recorder::new(Duration::from_secs(5));
        d.request();
        d.permission(false, ClockMs(0));
        assert_eq!(d.phase, RecPhase::Denied);
    }

    #[test]
    fn dictation_inserts_with_spacing() {
        assert_eq!(interim("a b c d", Duration::from_secs(1), 120.0), "a b");
        let (t, c) = insert_dictation("Fix thetest", 7, "flaky");
        assert_eq!(t, "Fix the flaky test");
        assert_eq!(c, 13);
    }

    #[test]
    fn tokens_rank_and_accept() {
        assert_eq!(
            active_token("hi @ma", 6),
            Some((Trigger::Mention, 3, "ma".into()))
        );
        assert_eq!(
            active_token("/rev", 4),
            Some((Trigger::Slash, 0, "rev".into()))
        );
        assert_eq!(active_token("a /rev", 6), None);
        let all = vec![
            Suggestion::new("maya", "Maya Patel", "People", "Platform"),
            Suggestion::new("sam", "Sam Okafor", "People", "Design"),
            Suggestion::new("auth", "src/auth.rs", "Files", ""),
        ];
        let r = rank(&all, "a", 5);
        assert_eq!(r[0].id, "auth");
        assert_eq!(r.len(), 3);
        let (t, c) = accept("hi @ma", 6, Trigger::Mention, 3, "maya");
        assert_eq!(t, "hi @maya ");
        assert_eq!(c, 9);
        let p = pills("ask @maya now", &["maya".into()]);
        assert_eq!(
            p,
            vec![
                (false, "ask ".into()),
                (true, "@maya".into()),
                (false, " now".into())
            ]
        );
    }

    #[test]
    fn mode_rank_rules() {
        let mut m = ModeSwitch::new(default_modes(), "plan");
        assert_eq!(m.select(2, true), Some(Applied::NextTurn));
        assert_eq!(m.mode().id, "plan");
        m.turn_end();
        assert_eq!(m.mode().id, "agent");
        assert_eq!(m.select(0, true), Some(Applied::Now));
        assert_eq!(m.cycle(false), Some(Applied::Now));
        assert_eq!(m.mode().id, "plan");
    }

    #[test]
    fn permissions_confirm_and_ceiling() {
        let mut p = Permissions::new(default_presets(), "ask");
        assert_eq!(p.choose("full"), Some(Choice::Confirm("full".into())));
        assert_eq!(p.current, "ask");
        assert_eq!(p.confirm(true), Some("full".into()));
        assert_eq!(p.current, "full");
        p.ceiling = Risk::Medium;
        assert_eq!(p.choose("full"), Some(Choice::Blocked("full".into())));
    }

    #[test]
    fn tools_and_queue() {
        let mut t = vec![
            Tool::new("web", "Web search", "Built in", true),
            Tool::new("code", "Code runner", "Built in", false),
            Tool::new("gh", "GitHub", "MCP", false).auth(),
        ];
        assert_eq!(tool_groups(&t, "").len(), 2);
        assert_eq!(tool_groups(&t, "git")[0].1, vec![2]);
        assert!(!toggle_tool(&mut t, 2));
        toggle_group(&mut t, "Built in");
        assert!(t[0].on && t[1].on);
        let mut q = Queue::new(&["one", "two", "three"]);
        q.shift("q3", -2);
        assert_eq!(q.items[0].text, "three");
        q.paused = true;
        assert_eq!(q.turn_end(), None);
        q.paused = false;
        assert_eq!(q.turn_end().map(|x| x.text), Some("three".into()));
        q.edit("q1", "  ");
        assert_eq!(q.items.len(), 1);
        let s = Steer {
            text: "use leeway".into(),
            at: ClockMs(1_500),
            interrupt: false,
        };
        let st = Duration::from_millis(2_000);
        assert_eq!(s.phase(ClockMs(1_600), st), SteerPhase::Pending);
        assert_eq!(s.phase(ClockMs(2_100), st), SteerPhase::Delivered);
        assert_eq!(s.phase(ClockMs(4_100), st), SteerPhase::Applied);
    }
}
