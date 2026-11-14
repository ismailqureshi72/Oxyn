//! Command palette, command menu, AI command, AI search bar and settings command
//! (`command-palette.md`, `ai-command.md`, `ai-search-bar.md`, `settings-command.md`).
//!
//! One matcher and one result model serve all of them:
//! - **Matching** (Arc UI Command palette): every word of the query must match, anywhere in the
//!   label, at a word start in the description or keywords, or as letters in order across the
//!   label ("shpr" finds *Share project*). Label prefixes rank first; groups follow their best
//!   match; the matched letters are returned so they render in medium weight.
//! - **Palette**: grouped results with one active row (↑ ↓ wrap, Home / End, Tab jumps a section),
//!   Enter runs, Escape clears the query and then closes. affectUI's Command Menu adds ⌘1–⌘5 for
//!   the first five suggestions, a tray that confirms what ran, and an "Ask the assistant"
//!   fallback when nothing matches.
//! - **AI command** (Arc UI): a fixed-size palette with an Ask mode (Tab): the answer streams in
//!   place with numbered sources and runnable next steps.
//! - **AI search bar** (Arc UI): grouped results with an "Ask AI about …" row first; Tab streams
//!   a short answer whose `[n]` citations light the matching result.
//! - **Settings command** (Arc UI): settings on one page; a search that grows into a palette and
//!   flies to the setting; a draft with an unsaved-changes bar, validation and Save / Discard.
//!
//! Oxyn plus: scoped modes by prefix (`>` commands, `@` files, `#` threads), recents and pinned
//! sections, inline toggles for switch settings. Rebuilt from the docs; no source used.

use std::{collections::BTreeMap, time::Duration};

use crate::ClockMs;

/// One command / result.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Command {
    /// Id.
    pub id: String,
    /// Label.
    pub label: String,
    /// Quiet line.
    pub description: Option<String>,
    /// Group heading ("Actions" when absent).
    pub group: Option<String>,
    /// Extra words to match.
    pub keywords: Vec<String>,
    /// Icon name.
    pub icon: Option<String>,
    /// Brand logo text (affectUI tool logos; drawn as a monogram tile).
    pub logo: Option<String>,
    /// Shortcut ("Mod+Shift+N").
    pub shortcut: Option<String>,
    /// Trailing hint ("Jump to", "Linear").
    pub hint: Option<String>,
    /// Shown while the query is empty.
    pub suggested: bool,
    /// Oxyn plus: scope (`>` command, `@` file, `#` thread).
    pub scope: Option<char>,
}

impl Command {
    /// A command.
    pub fn new(id: &str, label: &str) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            ..Default::default()
        }
    }
    /// In a group.
    pub fn group(mut self, g: &str) -> Self {
        self.group = Some(g.into());
        self
    }
    /// Description.
    pub fn desc(mut self, d: &str) -> Self {
        self.description = Some(d.into());
        self
    }
    /// Keywords.
    pub fn keywords(mut self, k: &[&str]) -> Self {
        self.keywords = k.iter().map(|s| s.to_string()).collect();
        self
    }
    /// Icon.
    pub fn icon(mut self, i: &str) -> Self {
        self.icon = Some(i.into());
        self
    }
    /// Logo.
    pub fn logo(mut self, l: &str) -> Self {
        self.logo = Some(l.into());
        self
    }
    /// Shortcut.
    pub fn shortcut(mut self, s: &str) -> Self {
        self.shortcut = Some(s.into());
        self
    }
    /// Hint.
    pub fn hint(mut self, h: &str) -> Self {
        self.hint = Some(h.into());
        self
    }
    /// Suggested.
    pub fn suggested(mut self) -> Self {
        self.suggested = true;
        self
    }
    /// Scope.
    pub fn scope(mut self, c: char) -> Self {
        self.scope = Some(c);
        self
    }
    /// Group or "Actions".
    pub fn group_name(&self) -> &str {
        self.group.as_deref().unwrap_or("Actions")
    }
}

/// A match: score and matched char ranges in the label.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Hit {
    /// Higher is better.
    pub score: i32,
    /// Matched chars in the label, as `(start, end)` char indices.
    pub ranges: Vec<(usize, usize)>,
}

fn word_starts(s: &str) -> Vec<usize> {
    let cs: Vec<char> = s.chars().collect();
    (0..cs.len())
        .filter(|&i| i == 0 || !cs[i - 1].is_alphanumeric())
        .collect()
}

fn find_chars(hay: &[char], needle: &[char], from: usize) -> Option<usize> {
    if needle.is_empty() || hay.len() < needle.len() {
        return None;
    }
    (from..=hay.len() - needle.len()).find(|&i| hay[i..i + needle.len()] == *needle)
}

/// Matches `query` against a command (empty query matches everything with score 0).
pub fn fuzzy(query: &str, c: &Command) -> Option<Hit> {
    let label: Vec<char> = c.label.to_lowercase().chars().collect();
    let starts = word_starts(&c.label);
    let mut hit = Hit::default();
    for w in query.split_whitespace() {
        let w: Vec<char> = w.to_lowercase().chars().collect();
        if let Some(i) = find_chars(&label, &w, 0) {
            hit.score += 100
                + if i == 0 {
                    200
                } else if starts.contains(&i) {
                    60
                } else {
                    0
                };
            hit.ranges.push((i, i + w.len()));
            continue;
        }
        let ws: String = w.iter().collect();
        let in_other = c.description.iter().chain(c.keywords.iter()).any(|d| {
            let d = d.to_lowercase();
            d.match_indices(&ws)
                .any(|(i, _)| i == 0 || !d[..i].chars().last().is_some_and(char::is_alphanumeric))
        });
        if in_other {
            hit.score += 40;
            continue;
        }
        // Letters in order across the label.
        let mut at = 0;
        let mut rs = vec![];
        for ch in &w {
            let p = label[at..].iter().position(|x| x == ch)?;
            rs.push((at + p, at + p + 1));
            at += p + 1;
        }
        let span = rs.last().map(|r| r.1).unwrap_or(0) - rs.first().map(|r| r.0).unwrap_or(0);
        hit.score += (30 - span as i32).max(5);
        hit.ranges.extend(rs);
    }
    hit.ranges.sort_unstable();
    Some(hit)
}

/// Splits `text` into (segment, matched) runs for rendering highlights.
pub fn highlight(text: &str, ranges: &[(usize, usize)]) -> Vec<(String, bool)> {
    let mut out: Vec<(String, bool)> = vec![];
    for (i, ch) in text.chars().enumerate() {
        let m = ranges.iter().any(|&(a, b)| i >= a && i < b);
        match out.last_mut() {
            Some((s, mm)) if *mm == m => s.push(ch),
            _ => out.push((ch.to_string(), m)),
        }
    }
    out
}

/// A result section.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Section {
    /// Heading.
    pub label: String,
    /// Rows: index into the items and the hit.
    pub rows: Vec<(usize, Hit)>,
}

/// Groups and ranks `items` for `query`; with an empty query shows suggested items (or all).
pub fn search(items: &[Command], query: &str, scope: Option<char>, limit: usize) -> Vec<Section> {
    let empty = query.trim().is_empty();
    let any_suggested = items.iter().any(|c| c.suggested);
    let mut hits: Vec<(usize, Hit)> = items
        .iter()
        .enumerate()
        .filter(|(_, c)| scope.is_none() || c.scope == scope)
        .filter(|(_, c)| !empty || !any_suggested || c.suggested || scope.is_some())
        .filter_map(|(i, c)| fuzzy(query, c).map(|h| (i, h)))
        .collect();
    hits.sort_by(|a, b| b.1.score.cmp(&a.1.score).then(a.0.cmp(&b.0)));
    hits.truncate(limit);
    let mut sections: Vec<Section> = vec![];
    for (i, h) in hits {
        let g = items[i].group_name().to_string();
        match sections.iter_mut().find(|s| s.label == g) {
            Some(s) => s.rows.push((i, h)),
            None => sections.push(Section {
                label: g,
                rows: vec![(i, h)],
            }),
        }
    }
    if empty {
        // Keep declaration order when nothing is typed.
        for s in &mut sections {
            s.rows.sort_by_key(|r| r.0);
        }
        sections.sort_by_key(|s| s.rows.first().map(|r| r.0).unwrap_or(usize::MAX));
    }
    sections
}

/// Keys the palette handles.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PaletteKey {
    /// ↓.
    Down,
    /// ↑.
    Up,
    /// Home.
    Home,
    /// End.
    End,
    /// Tab: next section.
    Tab,
    /// Shift+Tab: previous section.
    ShiftTab,
    /// Enter.
    Enter,
    /// Escape.
    Escape,
    /// ⌘1–⌘5.
    Digit(u8),
    /// Backspace in an empty field (leaves a scope).
    Backspace,
}

/// What a key or click led to.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PaletteOutcome {
    /// Nothing for the host.
    None,
    /// Run this command (id) with the query.
    Run(String, String),
    /// Ask the assistant with the query (affectUI fallback / AI mode).
    Ask(String),
    /// Close the surrounding dialog.
    Close,
}

/// Command palette / command menu.
#[derive(Clone, Debug, PartialEq)]
pub struct Palette {
    /// Items.
    pub items: Vec<Command>,
    /// Query.
    pub query: String,
    /// Active row in the flat list.
    pub active: usize,
    /// Oxyn plus: scope from a typed prefix.
    pub scope: Option<char>,
    /// Oxyn plus: most recent ids first.
    pub recents: Vec<String>,
    /// Max rows (Arc caps by height; 50).
    pub limit: usize,
    /// affectUI: the "Ask the assistant" row when nothing matches.
    pub ask_fallback: bool,
    /// Last thing that ran (the tray line and the live region).
    pub ran: Option<(String, ClockMs)>,
    /// Oxyn plus: scoped modes enabled.
    pub scopes: bool,
    /// The last move came from the keyboard (highlight jumps rather than glides).
    pub jump: bool,
}

impl Palette {
    /// A palette over `items`.
    pub fn new(items: Vec<Command>) -> Self {
        Self {
            items,
            query: String::new(),
            active: 0,
            scope: None,
            recents: vec![],
            limit: 50,
            ask_fallback: true,
            ran: None,
            scopes: true,
            jump: false,
        }
    }
    /// Sections for the current query (recents first when empty, Oxyn plus).
    pub fn sections(&self) -> Vec<Section> {
        let mut s = search(&self.items, &self.query, self.scope, self.limit);
        if self.query.trim().is_empty() && self.scope.is_none() && !self.recents.is_empty() {
            let rows: Vec<(usize, Hit)> = self
                .recents
                .iter()
                .filter_map(|id| self.items.iter().position(|c| &c.id == id))
                .map(|i| (i, Hit::default()))
                .take(3)
                .collect();
            if !rows.is_empty() {
                for sec in &mut s {
                    sec.rows.retain(|r| !rows.iter().any(|x| x.0 == r.0));
                }
                s.retain(|x| !x.rows.is_empty());
                s.insert(
                    0,
                    Section {
                        label: "Recent".into(),
                        rows,
                    },
                );
            }
        }
        s
    }
    /// Flat row order (item indices).
    pub fn flat(&self) -> Vec<usize> {
        self.sections()
            .iter()
            .flat_map(|s| s.rows.iter().map(|r| r.0))
            .collect()
    }
    /// Shows the Ask row (nothing matched and there is a query).
    pub fn asking(&self) -> bool {
        self.ask_fallback && !self.query.trim().is_empty() && self.flat().is_empty()
    }
    /// Active item index.
    pub fn active_item(&self) -> Option<usize> {
        self.flat().get(self.active).copied()
    }
    /// Typing: sets the query (a leading `>`, `@` or `#` picks a scope) and makes the first row
    /// active.
    pub fn set_query(&mut self, q: &str) {
        let mut q = q.to_string();
        if self.scopes
            && self.scope.is_none()
            && let Some(c) = q.chars().next().filter(|c| matches!(c, '>' | '@' | '#'))
        {
            self.scope = Some(c);
            q = q[c.len_utf8()..].trim_start().to_string();
        }
        self.query = q;
        self.active = 0;
        self.jump = true;
    }
    /// Suggestion `n` (1–5) for ⌘1–⌘5: the first five rows of the "Suggestions" group.
    pub fn suggestion(&self, n: u8) -> Option<usize> {
        let sugg: Vec<usize> = self
            .items
            .iter()
            .enumerate()
            .filter(|(_, c)| c.group_name() == "Suggestions")
            .map(|(i, _)| i)
            .take(5)
            .collect();
        sugg.get(n.checked_sub(1)? as usize).copied()
    }
    /// ⌘n label for an item, if it is one of the first five suggestions.
    pub fn digit_for(&self, item: usize) -> Option<u8> {
        (1..=5).find(|n| self.suggestion(*n) == Some(item))
    }
    /// Pointer over row `i` of the flat list.
    pub fn hover(&mut self, i: usize) {
        self.active = i;
        self.jump = false;
    }
    /// Runs item `item`.
    pub fn run(&mut self, item: usize, now: ClockMs) -> PaletteOutcome {
        let Some(c) = self.items.get(item) else {
            return PaletteOutcome::None;
        };
        let id = c.id.clone();
        self.ran = Some((c.label.clone(), now));
        self.recents.retain(|r| r != &id);
        self.recents.insert(0, id.clone());
        self.recents.truncate(5);
        let q = std::mem::take(&mut self.query);
        self.active = 0;
        PaletteOutcome::Run(id, q)
    }
    /// A key.
    pub fn key(&mut self, k: PaletteKey, now: ClockMs) -> PaletteOutcome {
        let secs = self.sections();
        let n: usize = secs.iter().map(|s| s.rows.len()).sum();
        self.jump = true;
        match k {
            PaletteKey::Down if n > 0 => self.active = (self.active + 1) % n,
            PaletteKey::Up if n > 0 => self.active = (self.active + n - 1) % n,
            PaletteKey::Home => self.active = 0,
            PaletteKey::End => self.active = n.saturating_sub(1),
            PaletteKey::Tab | PaletteKey::ShiftTab if n > 0 => {
                let mut starts = vec![];
                let mut acc = 0;
                for s in &secs {
                    starts.push(acc);
                    acc += s.rows.len();
                }
                let cur = starts.iter().rposition(|&s| s <= self.active).unwrap_or(0);
                let next = if k == PaletteKey::Tab {
                    (cur + 1) % starts.len()
                } else {
                    (cur + starts.len() - 1) % starts.len()
                };
                self.active = starts[next];
            }
            PaletteKey::Enter => {
                if self.asking() {
                    let q = std::mem::take(&mut self.query);
                    self.ran = Some((format!("Asked: {q}"), now));
                    return PaletteOutcome::Ask(q);
                }
                if let Some(i) = self.active_item() {
                    return self.run(i, now);
                }
            }
            PaletteKey::Digit(d) => {
                if let Some(i) = self.suggestion(d) {
                    return self.run(i, now);
                }
            }
            PaletteKey::Backspace if self.query.is_empty() => self.scope = None,
            PaletteKey::Escape => {
                if self.scope.is_some() && self.query.is_empty() {
                    self.scope = None;
                } else if self.query.is_empty() {
                    return PaletteOutcome::Close;
                } else {
                    self.query.clear();
                    self.active = 0;
                }
            }
            _ => {}
        }
        PaletteOutcome::None
    }
    /// Polite announcement of the results.
    pub fn announce(&self) -> String {
        let n = self.flat().len();
        match n {
            0 if self.asking() => "No results. Press Enter to ask the assistant".into(),
            0 => "No results".into(),
            1 => "1 result".into(),
            n => format!("{n} results"),
        }
    }
    /// Scope label.
    pub fn scope_label(&self) -> Option<&'static str> {
        Some(match self.scope? {
            '>' => "Commands",
            '@' => "Files",
            '#' => "Threads",
            _ => return None,
        })
    }
}

/// Where an AI answer is.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum AnswerPhase {
    /// Nothing asked.
    #[default]
    Idle,
    /// Asked, nothing arrived ("Searching your workspace").
    Thinking,
    /// Text arriving.
    Streaming,
    /// Finished.
    Done,
    /// Failed (Retry).
    Error,
}

/// A source the answer cites as `[n]`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Source {
    /// Id (matches a result id in the AI search bar).
    pub id: String,
    /// Chip title.
    pub title: String,
    /// Longer text for the tooltip and screen readers.
    pub description: Option<String>,
    /// Brand mark text.
    pub mark: Option<String>,
}

impl Source {
    /// A source.
    pub fn new(id: &str, title: &str) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            description: None,
            mark: None,
        }
    }
    /// With a mark.
    pub fn mark(mut self, m: &str) -> Self {
        self.mark = Some(m.into());
        self
    }
}

/// Part of an answer.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Seg {
    /// Text.
    Text(String),
    /// Inline code (`` `x` ``).
    Code(String),
    /// Citation `[n]` (1-based).
    Cite(usize),
}

/// Parses `[n]` citations and `` `code` `` spans.
pub fn segments(text: &str) -> Vec<Seg> {
    let mut out = vec![];
    let mut buf = String::new();
    let cs: Vec<char> = text.chars().collect();
    let mut i = 0;
    let flush = |buf: &mut String, out: &mut Vec<Seg>| {
        if !buf.is_empty() {
            out.push(Seg::Text(std::mem::take(buf)));
        }
    };
    while i < cs.len() {
        if cs[i] == '[' {
            let j = (i + 1..cs.len()).find(|&j| !cs[j].is_ascii_digit());
            if let Some(j) = j
                && j > i + 1
                && cs[j] == ']'
            {
                flush(&mut buf, &mut out);
                let n: String = cs[i + 1..j].iter().collect();
                out.push(Seg::Cite(n.parse().unwrap_or(0)));
                i = j + 1;
                continue;
            }
        }
        if cs[i] == '`'
            && let Some(j) = (i + 1..cs.len()).find(|&j| cs[j] == '`')
        {
            flush(&mut buf, &mut out);
            out.push(Seg::Code(cs[i + 1..j].iter().collect()));
            i = j + 1;
            continue;
        }
        buf.push(cs[i]);
        i += 1;
    }
    flush(&mut buf, &mut out);
    out
}

/// A scripted / streamed answer, revealed over time.
#[derive(Clone, Debug, PartialEq)]
pub struct AnswerStream {
    /// Full text so far (the host appends as tokens arrive).
    pub text: String,
    /// Sources, in citation order.
    pub sources: Vec<Source>,
    /// Asked at.
    pub asked: Option<ClockMs>,
    /// Time before the first text.
    pub think: Duration,
    /// Reveal speed (chars per second) when the whole text is known (demos, replay).
    pub cps: f32,
    /// Failed.
    pub failed: bool,
    /// The host marked it complete.
    pub complete: bool,
}

impl Default for AnswerStream {
    fn default() -> Self {
        Self {
            text: String::new(),
            sources: vec![],
            asked: None,
            think: Duration::from_millis(700),
            cps: 90.0,
            failed: false,
            complete: false,
        }
    }
}

impl AnswerStream {
    /// Starts an answer with known text (scripted), complete when fully revealed.
    pub fn scripted(text: &str, sources: Vec<Source>, now: ClockMs) -> Self {
        Self {
            text: text.into(),
            sources,
            asked: Some(now),
            complete: true,
            ..Default::default()
        }
    }
    /// Chars visible at `now`.
    pub fn shown(&self, now: ClockMs) -> usize {
        let Some(a) = self.asked else {
            return 0;
        };
        let t = now.since(a).saturating_sub(self.think).as_secs_f32();
        ((t * self.cps) as usize).min(self.text.chars().count())
    }
    /// Phase at `now`.
    pub fn phase(&self, now: ClockMs) -> AnswerPhase {
        if self.failed {
            return AnswerPhase::Error;
        }
        let Some(a) = self.asked else {
            return AnswerPhase::Idle;
        };
        if now.since(a) < self.think {
            return AnswerPhase::Thinking;
        }
        if self.complete && self.shown(now) >= self.text.chars().count() {
            AnswerPhase::Done
        } else {
            AnswerPhase::Streaming
        }
    }
    /// Visible text at `now`, cut at a char boundary.
    pub fn visible(&self, now: ClockMs) -> String {
        self.text.chars().take(self.shown(now)).collect()
    }
    /// Citation numbers present in the visible text.
    pub fn cited(&self, now: ClockMs) -> Vec<usize> {
        let mut v: Vec<usize> = segments(&self.visible(now))
            .into_iter()
            .filter_map(|s| match s {
                Seg::Cite(n) => Some(n),
                _ => None,
            })
            .collect();
        v.sort_unstable();
        v.dedup();
        v
    }
    /// Number (1-based) a source id is cited as.
    pub fn number_of(&self, id: &str) -> Option<usize> {
        self.sources.iter().position(|s| s.id == id).map(|i| i + 1)
    }
}

/// An action the answer suggests (run in place).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AiAction {
    /// Id.
    pub id: String,
    /// Label ("Create issue in Linear").
    pub label: String,
    /// Quiet detail.
    pub meta: Option<String>,
    /// Mark text.
    pub mark: Option<String>,
    /// Label when done ("Created ENG-2481").
    pub done_label: String,
    /// Started at.
    pub started: Option<ClockMs>,
    /// Failed (Enter retries).
    pub failed: bool,
}

/// Action state at a time.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ActionState {
    /// Ready.
    Idle,
    /// Running.
    Running,
    /// Done.
    Done,
    /// Error.
    Error,
}

impl AiAction {
    /// An action.
    pub fn new(id: &str, label: &str, done: &str) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            meta: None,
            mark: None,
            done_label: done.into(),
            started: None,
            failed: false,
        }
    }
    /// With a mark.
    pub fn mark(mut self, m: &str) -> Self {
        self.mark = Some(m.into());
        self
    }
    /// State at `now` (simulated 900 ms run).
    pub fn state(&self, now: ClockMs) -> ActionState {
        match self.started {
            None => ActionState::Idle,
            Some(_) if self.failed => ActionState::Error,
            Some(t) if now.since(t) < Duration::from_millis(900) => ActionState::Running,
            Some(_) => ActionState::Done,
        }
    }
}

/// AI command mode.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum AiMode {
    /// Search pages and actions.
    #[default]
    Search,
    /// Ask AI.
    Ask,
}

/// AI command (Arc UI): a palette with an Ask mode.
#[derive(Clone, Debug, PartialEq)]
pub struct AiCommand {
    /// The palette.
    pub palette: Palette,
    /// Mode.
    pub mode: AiMode,
    /// The question asked.
    pub question: String,
    /// The answer.
    pub answer: AnswerStream,
    /// Suggested next steps.
    pub actions: Vec<AiAction>,
    /// Questions offered while Ask is open and empty.
    pub suggested_questions: Vec<String>,
    /// Active row in Ask mode (suggestions / actions).
    pub ask_active: usize,
}

impl AiCommand {
    /// A palette over `items`.
    pub fn new(items: Vec<Command>) -> Self {
        let mut palette = Palette::new(items);
        palette.ask_fallback = false;
        Self {
            palette,
            mode: AiMode::Search,
            question: String::new(),
            answer: AnswerStream::default(),
            actions: vec![],
            suggested_questions: vec![],
            ask_active: 0,
        }
    }
    /// Tab in Search: switches to Ask (carrying the query); Tab in Ask with nothing typed goes
    /// back.
    pub fn tab(&mut self) -> AiMode {
        self.mode = match self.mode {
            AiMode::Search => AiMode::Ask,
            AiMode::Ask => AiMode::Search,
        };
        self.ask_active = 0;
        self.mode
    }
    /// Asks `q` with a scripted answer (the host streams real ones into `answer`).
    pub fn ask(&mut self, q: &str, answer: AnswerStream, actions: Vec<AiAction>) {
        self.mode = AiMode::Ask;
        self.question = q.into();
        self.answer = answer;
        self.actions = actions;
        self.ask_active = 0;
    }
    /// Escape: closes the answer / goes back to Search, then defers to the palette.
    pub fn escape(&mut self, now: ClockMs) -> PaletteOutcome {
        if self.mode == AiMode::Ask {
            if self.answer.asked.is_some() {
                self.answer = AnswerStream::default();
                self.actions.clear();
                self.question.clear();
            } else {
                self.mode = AiMode::Search;
            }
            return PaletteOutcome::None;
        }
        self.palette.key(PaletteKey::Escape, now)
    }
    /// Runs suggested action `i`.
    pub fn run_action(&mut self, i: usize, now: ClockMs) {
        if let Some(a) = self.actions.get_mut(i) {
            a.failed = false;
            a.started = Some(now);
        }
    }
    /// Footer status line.
    pub fn status(&self, now: ClockMs, thinking_label: &str) -> String {
        match self.answer.phase(now) {
            AnswerPhase::Idle => match self.mode {
                AiMode::Search => self.palette.announce(),
                AiMode::Ask => "Ask a question".into(),
            },
            AnswerPhase::Thinking => thinking_label.into(),
            AnswerPhase::Streaming => "Answering…".into(),
            AnswerPhase::Done => format!("Answer with {} sources", self.answer.sources.len()),
            AnswerPhase::Error => "Something went wrong. Retry".into(),
        }
    }
}

/// AI search bar (Arc UI).
#[derive(Clone, Debug, PartialEq)]
pub struct AiSearch {
    /// Results (group = section).
    pub results: Vec<Command>,
    /// Query.
    pub query: String,
    /// Active row: 0 = Ask AI row (when there is a query), then results.
    pub active: usize,
    /// Recent searches (session, three at most).
    pub recent: Vec<String>,
    /// Suggested questions.
    pub suggestions: Vec<String>,
    /// The answer.
    pub answer: AnswerStream,
    /// A citation is hovered / focused: lights its result.
    pub lit: Option<String>,
}

impl AiSearch {
    /// Over `results`.
    pub fn new(results: Vec<Command>) -> Self {
        Self {
            results,
            query: String::new(),
            active: 0,
            recent: vec![],
            suggestions: vec![],
            answer: AnswerStream::default(),
            lit: None,
        }
    }
    /// Sections for the query.
    pub fn sections(&self) -> Vec<Section> {
        if self.query.trim().is_empty() {
            return vec![];
        }
        search(&self.results, &self.query, None, 30)
    }
    /// Rows in order (Ask row is index 0 when there is a query).
    pub fn row_count(&self) -> usize {
        if self.query.trim().is_empty() {
            self.recent.len() + self.suggestions.len()
        } else {
            1 + self.sections().iter().map(|s| s.rows.len()).sum::<usize>()
        }
    }
    /// Typing.
    pub fn set_query(&mut self, q: &str) {
        self.query = q.into();
        self.active = 0;
        self.answer = AnswerStream::default();
        self.lit = None;
    }
    /// ↓ / ↑ (wrap).
    pub fn step(&mut self, down: bool) {
        let n = self.row_count();
        if n == 0 {
            return;
        }
        self.active = if down {
            (self.active + 1) % n
        } else {
            (self.active + n - 1) % n
        };
    }
    /// Remembers a search (three at most, newest first).
    pub fn remember(&mut self, q: &str) {
        let q = q.trim();
        if q.is_empty() {
            return;
        }
        self.recent.retain(|r| r != q);
        self.recent.insert(0, q.into());
        self.recent.truncate(3);
    }
    /// Escape: closes the answer first, then clears the field.
    pub fn escape(&mut self) -> bool {
        if self.answer.asked.is_some() {
            self.answer = AnswerStream::default();
            self.lit = None;
            true
        } else if !self.query.is_empty() {
            self.set_query("");
            true
        } else {
            false
        }
    }
    /// The citation badge for a result row.
    pub fn badge(&self, id: &str, now: ClockMs) -> Option<usize> {
        let n = self.answer.number_of(id)?;
        self.answer.cited(now).contains(&n).then_some(n)
    }
}

/// A setting's control.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SettingKind {
    /// On / off.
    Switch,
    /// Segmented options.
    Segmented(Vec<String>),
    /// Select options.
    Select(Vec<String>),
    /// Text with an optional prefix ("acme.dev/") and a required flag.
    Text {
        /// Prefix shown before the field.
        prefix: Option<String>,
        /// Must not be empty.
        required: bool,
        /// Max length.
        max: Option<usize>,
    },
}

/// A setting value.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SettingValue {
    /// Switch.
    Bool(bool),
    /// Anything else.
    Text(String),
}

impl SettingValue {
    /// Display.
    pub fn display(&self) -> String {
        match self {
            Self::Bool(b) => if *b { "On" } else { "Off" }.into(),
            Self::Text(t) => t.clone(),
        }
    }
}

/// A setting.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Setting {
    /// Id.
    pub id: String,
    /// Section id.
    pub section: String,
    /// Label.
    pub label: String,
    /// Hint.
    pub description: String,
    /// Control.
    pub kind: SettingKind,
    /// Search words.
    pub keywords: Vec<String>,
}

impl Setting {
    /// A setting.
    pub fn new(id: &str, section: &str, label: &str, description: &str, kind: SettingKind) -> Self {
        Self {
            id: id.into(),
            section: section.into(),
            label: label.into(),
            description: description.into(),
            kind,
            keywords: vec![],
        }
    }
    /// Keywords.
    pub fn keywords(mut self, k: &[&str]) -> Self {
        self.keywords = k.iter().map(|s| s.to_string()).collect();
        self
    }
    /// Validation error for `v`.
    pub fn validate(&self, v: &SettingValue) -> Option<String> {
        match (&self.kind, v) {
            (SettingKind::Text { required, max, .. }, SettingValue::Text(t)) => {
                if *required && t.trim().is_empty() {
                    Some(format!("{} is required", self.label))
                } else if max.is_some_and(|m| t.chars().count() > m) {
                    Some(format!("Use {} characters or fewer", max.unwrap_or(0)))
                } else {
                    None
                }
            }
            _ => None,
        }
    }
}

/// Save state of the unsaved-changes bar.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum SavePhase {
    /// Nothing pending or the bar is open with changes.
    #[default]
    Idle,
    /// Saving.
    Saving,
    /// "Settings saved" (the bar shrinks away).
    Saved,
    /// Save failed (Retry).
    Failed,
}

/// Settings command (Arc UI).
#[derive(Clone, Debug, PartialEq)]
pub struct SettingsCommand {
    /// Sections: (id, label, one line).
    pub sections: Vec<(String, String, String)>,
    /// Settings.
    pub settings: Vec<Setting>,
    /// Saved values.
    pub saved: BTreeMap<String, SettingValue>,
    /// Draft values.
    pub draft: BTreeMap<String, SettingValue>,
    /// Palette open.
    pub open: bool,
    /// Palette query.
    pub query: String,
    /// Active result.
    pub active: usize,
    /// Setting ids offered before anything is typed.
    pub suggested: Vec<String>,
    /// Last jump target and when (the row breathes twice).
    pub flash: Option<(String, ClockMs)>,
    /// Save phase.
    pub save: SavePhase,
}

impl SettingsCommand {
    /// From definitions and saved values.
    pub fn new(
        sections: Vec<(String, String, String)>,
        settings: Vec<Setting>,
        saved: BTreeMap<String, SettingValue>,
    ) -> Self {
        Self {
            sections,
            settings,
            draft: saved.clone(),
            saved,
            open: false,
            query: String::new(),
            active: 0,
            suggested: vec![],
            flash: None,
            save: SavePhase::Idle,
        }
    }
    /// Current draft value.
    pub fn value(&self, id: &str) -> Option<&SettingValue> {
        self.draft.get(id)
    }
    /// Edits a value.
    pub fn set(&mut self, id: &str, v: SettingValue) {
        self.draft.insert(id.into(), v);
        if self.save != SavePhase::Saving {
            self.save = SavePhase::Idle;
        }
    }
    /// Changed ids, in definition order.
    pub fn changed(&self) -> Vec<String> {
        self.settings
            .iter()
            .filter(|s| self.draft.get(&s.id) != self.saved.get(&s.id))
            .map(|s| s.id.clone())
            .collect()
    }
    /// Edited settings in a section.
    pub fn edited_in(&self, section: &str) -> usize {
        let ch = self.changed();
        self.settings
            .iter()
            .filter(|s| s.section == section && ch.contains(&s.id))
            .count()
    }
    /// Validation errors, in definition order.
    pub fn errors(&self) -> Vec<(String, String)> {
        self.settings
            .iter()
            .filter_map(|s| {
                let v = self.draft.get(&s.id)?;
                s.validate(v).map(|e| (s.id.clone(), e))
            })
            .collect()
    }
    /// Save: blocked by the first invalid row (jumps there); otherwise starts saving.
    pub fn save(&mut self, now: ClockMs) -> Result<Vec<String>, String> {
        if let Some((id, _)) = self.errors().into_iter().next() {
            self.jump(&id, now);
            return Err(id);
        }
        self.save = SavePhase::Saving;
        Ok(self.changed())
    }
    /// The host finished saving.
    pub fn saved(&mut self, ok: bool) {
        if ok {
            self.saved = self.draft.clone();
            self.save = SavePhase::Saved;
        } else {
            self.save = SavePhase::Failed;
        }
    }
    /// Discards the draft.
    pub fn discard(&mut self) {
        self.draft = self.saved.clone();
        self.save = SavePhase::Idle;
    }
    /// Palette results (indices into `settings`).
    pub fn results(&self) -> Vec<usize> {
        if self.query.trim().is_empty() {
            return self
                .suggested
                .iter()
                .filter_map(|id| self.settings.iter().position(|s| &s.id == id))
                .collect();
        }
        let cmds: Vec<Command> = self
            .settings
            .iter()
            .map(|s| {
                let sec = self
                    .sections
                    .iter()
                    .find(|x| x.0 == s.section)
                    .map(|x| x.1.clone())
                    .unwrap_or_default();
                let mut kw = s.keywords.clone();
                kw.push(sec);
                Command {
                    id: s.id.clone(),
                    label: s.label.clone(),
                    description: Some(s.description.clone()),
                    keywords: kw,
                    ..Default::default()
                }
            })
            .collect();
        let mut hits: Vec<(usize, Hit)> = cmds
            .iter()
            .enumerate()
            .filter_map(|(i, c)| fuzzy(&self.query, c).map(|h| (i, h)))
            .collect();
        hits.sort_by(|a, b| b.1.score.cmp(&a.1.score).then(a.0.cmp(&b.0)));
        hits.into_iter().map(|h| h.0).collect()
    }
    /// Opens the palette (⌘K, Ctrl+K or `/`).
    pub fn open_palette(&mut self) {
        self.open = true;
        self.query.clear();
        self.active = 0;
    }
    /// Escape / scrim.
    pub fn close_palette(&mut self) {
        self.open = false;
    }
    /// ↑ / ↓.
    pub fn step(&mut self, down: bool) {
        let n = self.results().len();
        if n > 0 {
            self.active = if down {
                (self.active + 1) % n
            } else {
                (self.active + n - 1) % n
            };
        }
    }
    /// Enter: jumps to the active result.
    pub fn enter(&mut self, now: ClockMs) -> Option<String> {
        let i = *self.results().get(self.active)?;
        let id = self.settings[i].id.clone();
        self.jump(&id, now);
        Some(id)
    }
    /// Flies to a setting: closes the palette, flashes the row.
    pub fn jump(&mut self, id: &str, now: ClockMs) {
        self.open = false;
        self.flash = Some((id.into(), now));
    }
    /// Unsaved-changes bar line.
    pub fn bar_label(&self) -> String {
        match self.save {
            SavePhase::Saving => "Saving…".into(),
            SavePhase::Saved => "Settings saved".into(),
            SavePhase::Failed => "Couldn't save. Retry".into(),
            SavePhase::Idle => match self.changed().len() {
                1 => "1 unsaved change".into(),
                n => format!("{n} unsaved changes"),
            },
        }
    }
    /// The bar is shown.
    pub fn bar(&self) -> bool {
        !self.changed().is_empty() || self.save != SavePhase::Idle
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn items() -> Vec<Command> {
        vec![
            Command::new("share", "Share project")
                .group("Actions")
                .keywords(&["invite", "link"]),
            Command::new("new", "New thread")
                .group("Actions")
                .shortcut("Mod+N"),
            Command::new("settings", "Open settings")
                .group("Go to")
                .desc("Preferences and keys"),
            Command::new("sum", "Summarize thread")
                .group("Suggestions")
                .logo("AI")
                .suggested(),
            Command::new("issue", "Create Linear issue")
                .group("Suggestions")
                .logo("L")
                .suggested(),
            Command::new("readme", "README.md")
                .group("Files")
                .scope('@'),
        ]
    }

    #[test]
    fn matching_rules() {
        let c = &items()[0];
        let h = fuzzy("share", c).unwrap();
        assert_eq!(h.ranges, vec![(0, 5)]);
        assert!(fuzzy("pro", c).unwrap().score > fuzzy("roj", c).unwrap().score);
        assert!(fuzzy("invite", c).is_some(), "keyword word start");
        assert!(
            fuzzy("nvite", c).is_none(),
            "not at a word start, not in label order"
        );
        let s = fuzzy("shpr", c).unwrap();
        assert_eq!(s.ranges.len(), 4, "letters in order across the label");
        assert!(fuzzy("share zz", c).is_none(), "every word must match");
        assert!(
            fuzzy("pref", &items()[2]).is_some(),
            "description word start"
        );
        let hl = highlight("Share project", &h.ranges);
        assert_eq!(hl[0], ("Share".to_string(), true));
    }

    #[test]
    fn sections_and_keys() {
        let mut p = Palette::new(items());
        // Empty: only suggested rows.
        assert_eq!(p.flat(), vec![3, 4]);
        p.set_query("t");
        let secs = p.sections();
        assert!(secs.len() >= 2);
        assert_eq!(p.key(PaletteKey::Up, ClockMs(0)), PaletteOutcome::None);
        assert_eq!(p.active, p.flat().len() - 1, "wraps");
        p.key(PaletteKey::Home, ClockMs(0));
        p.key(PaletteKey::Tab, ClockMs(0));
        assert_eq!(p.active, secs[0].rows.len());
        let first = p.flat()[p.active];
        assert_eq!(
            p.key(PaletteKey::Enter, ClockMs(5)),
            PaletteOutcome::Run(items()[first].id.clone(), "t".into())
        );
        assert!(p.query.is_empty() && p.ran.is_some());
        assert_eq!(p.sections()[0].label, "Recent");
        assert_eq!(
            p.key(PaletteKey::Digit(2), ClockMs(9)),
            PaletteOutcome::Run("issue".into(), String::new())
        );
        assert_eq!(p.digit_for(3), Some(1));
        p.set_query("zzzz");
        assert!(p.asking());
        assert_eq!(
            p.key(PaletteKey::Enter, ClockMs(10)),
            PaletteOutcome::Ask("zzzz".into())
        );
        p.set_query("abc");
        assert_eq!(p.key(PaletteKey::Escape, ClockMs(10)), PaletteOutcome::None);
        assert_eq!(
            p.key(PaletteKey::Escape, ClockMs(10)),
            PaletteOutcome::Close
        );
    }

    #[test]
    fn scopes() {
        let mut p = Palette::new(items());
        p.set_query("@rea");
        assert_eq!(p.scope, Some('@'));
        assert_eq!(p.scope_label(), Some("Files"));
        assert_eq!(p.flat(), vec![5]);
        p.set_query("");
        p.key(PaletteKey::Backspace, ClockMs(0));
        assert_eq!(p.scope, None);
    }

    #[test]
    fn answers_stream_with_citations() {
        let segs = segments("Use `retry` with backoff [1] and idempotency keys [2].");
        assert!(matches!(segs[1], Seg::Code(ref c) if c == "retry"));
        assert!(segs.contains(&Seg::Cite(2)));
        assert_eq!(
            segments("[x] literal"),
            vec![Seg::Text("[x] literal".into())]
        );
        let a = AnswerStream::scripted(
            "Retries back off [1].",
            vec![Source::new("doc-1", "Webhooks")],
            ClockMs(0),
        );
        assert_eq!(a.phase(ClockMs(100)), AnswerPhase::Thinking);
        assert_eq!(a.phase(ClockMs(800)), AnswerPhase::Streaming);
        assert_eq!(a.phase(ClockMs(5000)), AnswerPhase::Done);
        assert_eq!(a.cited(ClockMs(5000)), vec![1]);
        assert_eq!(a.number_of("doc-1"), Some(1));
    }

    #[test]
    fn ai_command_modes() {
        let mut c = AiCommand::new(items());
        assert_eq!(c.tab(), AiMode::Ask);
        c.ask(
            "How do retries work?",
            AnswerStream::scripted("They back off.", vec![], ClockMs(0)),
            vec![AiAction::new(
                "lin",
                "Create issue in Linear",
                "Created ENG-2481",
            )],
        );
        c.run_action(0, ClockMs(100));
        assert_eq!(c.actions[0].state(ClockMs(500)), ActionState::Running);
        assert_eq!(c.actions[0].state(ClockMs(1100)), ActionState::Done);
        assert_eq!(c.escape(ClockMs(0)), PaletteOutcome::None);
        assert!(c.answer.asked.is_none());
        c.escape(ClockMs(0));
        assert_eq!(c.mode, AiMode::Search);
        assert_eq!(c.escape(ClockMs(0)), PaletteOutcome::Close);
    }

    #[test]
    fn ai_search_rows() {
        let mut s = AiSearch::new(items());
        s.suggestions = vec!["How do I share?".into()];
        s.remember("figma");
        s.remember("auth");
        s.remember("figma");
        assert_eq!(s.recent, vec!["figma", "auth"]);
        assert_eq!(s.row_count(), 3);
        s.set_query("share");
        assert_eq!(s.row_count(), 2);
        s.step(false);
        assert_eq!(s.active, 1);
        s.answer = AnswerStream::scripted(
            "Use the link [1].",
            vec![Source::new("share", "Share")],
            ClockMs(0),
        );
        assert_eq!(s.badge("share", ClockMs(9000)), Some(1));
        assert!(s.escape() && s.answer.asked.is_none());
        assert!(s.escape() && s.query.is_empty());
        assert!(!s.escape());
    }

    #[test]
    fn settings_draft_save_and_jump() {
        let settings = vec![
            Setting::new(
                "name",
                "general",
                "Workspace name",
                "Shown in the sidebar",
                SettingKind::Text {
                    prefix: None,
                    required: true,
                    max: Some(32),
                },
            ),
            Setting::new(
                "digest",
                "notify",
                "Daily digest",
                "A summary each morning",
                SettingKind::Switch,
            )
            .keywords(&["email"]),
            Setting::new(
                "theme",
                "general",
                "Theme",
                "Light or dark",
                SettingKind::Segmented(vec!["Light".into(), "Dark".into()]),
            ),
        ];
        let mut saved = BTreeMap::new();
        saved.insert("name".to_string(), SettingValue::Text("Acme".into()));
        saved.insert("digest".to_string(), SettingValue::Bool(false));
        saved.insert("theme".to_string(), SettingValue::Text("Dark".into()));
        let sections = vec![
            ("general".to_string(), "General".to_string(), String::new()),
            (
                "notify".to_string(),
                "Notifications".to_string(),
                String::new(),
            ),
        ];
        let mut s = SettingsCommand::new(sections, settings, saved);
        s.set("digest", SettingValue::Bool(true));
        s.set("name", SettingValue::Text(" ".into()));
        assert_eq!(s.changed(), vec!["name", "digest"]);
        assert_eq!(s.edited_in("general"), 1);
        assert_eq!(s.bar_label(), "2 unsaved changes");
        assert_eq!(s.save(ClockMs(0)), Err("name".into()));
        assert_eq!(s.flash.as_ref().unwrap().0, "name");
        s.set("name", SettingValue::Text("Acme Labs".into()));
        assert!(s.save(ClockMs(10)).is_ok());
        s.saved(true);
        assert_eq!(s.bar_label(), "Settings saved");
        assert!(s.changed().is_empty());
        s.open_palette();
        s.query = "email".into();
        assert_eq!(s.results(), vec![1]);
        assert_eq!(s.enter(ClockMs(20)).as_deref(), Some("digest"));
        assert!(!s.open);
        s.set("theme", SettingValue::Text("Light".into()));
        s.discard();
        assert!(!s.bar());
    }
}
