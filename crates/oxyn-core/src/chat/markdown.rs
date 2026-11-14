//! Streaming markdown (`streaming-markdown.md`): a tolerant block parser for text that is still
//! arriving, plus a reveal clock that lets text catch up smoothly instead of landing in bursts.
//!
//! Partial input never renders broken: an unclosed code fence becomes an open code block, a
//! dangling `**` or backtick is held back until it closes, and a half-typed table row waits.

use std::time::Duration;

use crate::ClockMs;

/// An inline run.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Inline {
    /// Plain text.
    Text(String),
    /// `**bold**`.
    Bold(String),
    /// `*italic*` / `_italic_`.
    Italic(String),
    /// `` `code` ``.
    Code(String),
    /// `[text](url)`.
    Link(String, String),
    /// `[n]` citation (1-based).
    Cite(u32),
}

/// A block.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Block {
    /// `#`–`###` heading.
    Heading(u8, Vec<Inline>),
    /// Paragraph.
    Para(Vec<Inline>),
    /// List items (ordered or not).
    List(bool, Vec<Vec<Inline>>),
    /// Fenced code (`open` while the fence has not closed).
    Code {
        /// Language after the fence.
        lang: String,
        /// Body.
        text: String,
        /// Still streaming.
        open: bool,
    },
    /// `>` quote.
    Quote(Vec<Inline>),
    /// `---`.
    Rule,
    /// Table: header cells and rows.
    Table(Vec<String>, Vec<Vec<String>>),
}

/// Parses inline markdown. Unclosed markers at the end are held back (not shown as `**`).
pub fn inline(s: &str) -> Vec<Inline> {
    let cs: Vec<char> = s.chars().collect();
    let mut out = Vec::new();
    let mut buf = String::new();
    let flush = |buf: &mut String, out: &mut Vec<Inline>| {
        if !buf.is_empty() {
            out.push(Inline::Text(std::mem::take(buf)));
        }
    };
    let find = |from: usize, pat: &[char]| -> Option<usize> {
        (from..cs.len()).find(|&j| cs[j..].starts_with(pat))
    };
    let mut i = 0;
    while i < cs.len() {
        let c = cs[i];
        if c == '*' && cs.get(i + 1) == Some(&'*') {
            match find(i + 2, &['*', '*']) {
                Some(j) => {
                    flush(&mut buf, &mut out);
                    out.push(Inline::Bold(cs[i + 2..j].iter().collect()));
                    i = j + 2;
                }
                None => break,
            }
            continue;
        }
        if (c == '*' || c == '_') && cs.get(i + 1).is_some_and(|n| !n.is_whitespace()) {
            match find(i + 1, &[c]) {
                Some(j) if j > i + 1 => {
                    flush(&mut buf, &mut out);
                    out.push(Inline::Italic(cs[i + 1..j].iter().collect()));
                    i = j + 1;
                }
                Some(_) => {
                    buf.push(c);
                    i += 1;
                }
                None => {
                    if c == '_' {
                        buf.push(c);
                        i += 1;
                        continue;
                    }
                    break;
                }
            }
            continue;
        }
        if c == '`' {
            match find(i + 1, &['`']) {
                Some(j) => {
                    flush(&mut buf, &mut out);
                    out.push(Inline::Code(cs[i + 1..j].iter().collect()));
                    i = j + 1;
                }
                None => break,
            }
            continue;
        }
        if c == '[' {
            if let Some(j) = find(i + 1, &[']']) {
                let inner: String = cs[i + 1..j].iter().collect();
                if let Ok(n) = inner.parse::<u32>() {
                    flush(&mut buf, &mut out);
                    out.push(Inline::Cite(n));
                    i = j + 1;
                    continue;
                }
                if cs.get(j + 1) == Some(&'(') {
                    if let Some(k) = find(j + 2, &[')']) {
                        flush(&mut buf, &mut out);
                        out.push(Inline::Link(inner, cs[j + 2..k].iter().collect()));
                        i = k + 1;
                        continue;
                    }
                    break;
                }
            } else {
                // A citation still arriving ("[1").
                if cs[i + 1..].iter().all(|c| c.is_ascii_digit()) {
                    break;
                }
            }
        }
        buf.push(c);
        i += 1;
    }
    flush(&mut buf, &mut out);
    out
}

/// Plain text of inlines (for copy and labels).
pub fn plain(v: &[Inline]) -> String {
    v.iter()
        .map(|x| match x {
            Inline::Text(s) | Inline::Bold(s) | Inline::Italic(s) | Inline::Code(s) => s.clone(),
            Inline::Link(t, _) => t.clone(),
            Inline::Cite(n) => format!("[{n}]"),
        })
        .collect()
}

fn list_item(l: &str) -> Option<(bool, &str)> {
    let t = l.trim_start();
    if let Some(r) = t.strip_prefix("- ").or_else(|| t.strip_prefix("* ")) {
        return Some((false, r));
    }
    let d = t.find(". ")?;
    if d > 0 && t[..d].chars().all(|c| c.is_ascii_digit()) {
        return Some((true, &t[d + 2..]));
    }
    None
}

fn cells(l: &str) -> Vec<String> {
    l.trim()
        .trim_matches('|')
        .split('|')
        .map(|c| c.trim().to_string())
        .collect()
}

/// Parses `src` (possibly mid-stream) into blocks. With `streaming`, the last line is held back
/// when it could still become a different block (a half-typed table or fence).
pub fn parse(src: &str) -> Vec<Block> {
    let mut out: Vec<Block> = Vec::new();
    let lines: Vec<&str> = src.lines().collect();
    let mut i = 0;
    let mut para: Vec<String> = Vec::new();
    let end_para = |para: &mut Vec<String>, out: &mut Vec<Block>| {
        if !para.is_empty() {
            out.push(Block::Para(inline(&para.join(" "))));
            para.clear();
        }
    };
    while i < lines.len() {
        let l = lines[i];
        let t = l.trim();
        if let Some(lang) = t.strip_prefix("```") {
            end_para(&mut para, &mut out);
            let mut body = Vec::new();
            let mut j = i + 1;
            let mut closed = false;
            while j < lines.len() {
                if lines[j].trim().starts_with("```") {
                    closed = true;
                    break;
                }
                body.push(lines[j]);
                j += 1;
            }
            out.push(Block::Code {
                lang: lang.trim().to_string(),
                text: body.join("\n"),
                open: !closed,
            });
            i = j + 1;
            continue;
        }
        if t.is_empty() {
            end_para(&mut para, &mut out);
            i += 1;
            continue;
        }
        if t.starts_with('#') {
            let level = t.chars().take_while(|c| *c == '#').count();
            if (1..=3).contains(&level) && t[level..].starts_with(' ') {
                end_para(&mut para, &mut out);
                out.push(Block::Heading(level as u8, inline(t[level..].trim())));
                i += 1;
                continue;
            }
        }
        if t == "---" || t == "***" {
            end_para(&mut para, &mut out);
            out.push(Block::Rule);
            i += 1;
            continue;
        }
        if let Some(q) = t.strip_prefix('>') {
            end_para(&mut para, &mut out);
            out.push(Block::Quote(inline(q.trim())));
            i += 1;
            continue;
        }
        if t.starts_with('|') {
            end_para(&mut para, &mut out);
            let head = cells(t);
            let mut rows = Vec::new();
            let mut j = i + 1;
            if j < lines.len() && lines[j].trim().starts_with('|') && lines[j].contains("---") {
                j += 1;
            }
            while j < lines.len() && lines[j].trim().starts_with('|') {
                let r = cells(lines[j]);
                // A row still arriving (fewer cells than the header) waits.
                if j + 1 == lines.len() && r.len() < head.len() && !src.ends_with('\n') {
                    break;
                }
                rows.push(r);
                j += 1;
            }
            out.push(Block::Table(head, rows));
            i = j;
            continue;
        }
        if let Some((ordered, first)) = list_item(t) {
            end_para(&mut para, &mut out);
            let mut items = vec![inline(first)];
            let mut j = i + 1;
            while j < lines.len() {
                match list_item(lines[j]) {
                    Some((o, r)) if o == ordered => items.push(inline(r)),
                    _ => break,
                }
                j += 1;
            }
            out.push(Block::List(ordered, items));
            i = j;
            continue;
        }
        para.push(t.to_string());
        i += 1;
    }
    end_para(&mut para, &mut out);
    out
}

/// Headings (level, text) for a table of contents.
pub fn headings(blocks: &[Block]) -> Vec<(u8, String)> {
    blocks
        .iter()
        .filter_map(|b| match b {
            Block::Heading(l, v) => Some((*l, plain(v))),
            _ => None,
        })
        .collect()
}

/// Citation numbers used, in order of first appearance.
pub fn citations(blocks: &[Block]) -> Vec<u32> {
    let mut v = Vec::new();
    let mut add = |xs: &[Inline]| {
        for x in xs {
            if let Inline::Cite(n) = x
                && !v.contains(n)
            {
                v.push(*n);
            }
        }
    };
    for b in blocks {
        match b {
            Block::Heading(_, x) | Block::Para(x) | Block::Quote(x) => add(x),
            Block::List(_, items) => items.iter().for_each(|x| add(x)),
            _ => {}
        }
    }
    v
}

/// Reveal speed preset.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Pace {
    /// 60 chars / s.
    Calm,
    /// 140 chars / s.
    #[default]
    Normal,
    /// 400 chars / s.
    Fast,
    /// Everything at once.
    Instant,
}

impl Pace {
    /// From an id.
    pub fn from_id(s: &str) -> Self {
        match s {
            "calm" => Self::Calm,
            "fast" => Self::Fast,
            "instant" => Self::Instant,
            _ => Self::Normal,
        }
    }
    /// Characters per second (0 = instant).
    pub fn cps(self) -> f32 {
        match self {
            Self::Calm => 60.0,
            Self::Normal => 140.0,
            Self::Fast => 400.0,
            Self::Instant => 0.0,
        }
    }
}

/// Stream phase.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum StreamPhase {
    /// Waiting for the first token.
    #[default]
    Waiting,
    /// Tokens arriving.
    Streaming,
    /// Finished.
    Done,
    /// Stopped by the user.
    Stopped,
    /// Failed.
    Error,
}

impl StreamPhase {
    /// Id.
    pub fn id(self) -> &'static str {
        match self {
            Self::Waiting => "waiting",
            Self::Streaming => "streaming",
            Self::Done => "done",
            Self::Stopped => "stopped",
            Self::Error => "error",
        }
    }
}

/// A markdown stream with a reveal clock: received text is shown at `pace`, speeding up when far
/// behind so the reader never waits more than ~1.2 s after the last token.
#[derive(Clone, Debug, PartialEq)]
pub struct MarkdownStream {
    /// Received text.
    pub text: String,
    /// Characters shown.
    pub shown: usize,
    /// Pace.
    pub pace: Pace,
    /// Phase.
    pub phase: StreamPhase,
    last: Option<ClockMs>,
}

impl MarkdownStream {
    /// A stream.
    pub fn new(pace: Pace) -> Self {
        Self {
            text: String::new(),
            shown: 0,
            pace,
            phase: StreamPhase::Waiting,
            last: None,
        }
    }
    /// A finished stream showing everything.
    pub fn done(text: &str) -> Self {
        Self {
            shown: text.chars().count(),
            text: text.into(),
            pace: Pace::Instant,
            phase: StreamPhase::Done,
            last: None,
        }
    }
    /// Appends a chunk.
    pub fn push(&mut self, chunk: &str, now: ClockMs) {
        if matches!(self.phase, StreamPhase::Stopped | StreamPhase::Error) {
            return;
        }
        self.text.push_str(chunk);
        if self.phase == StreamPhase::Waiting {
            self.phase = StreamPhase::Streaming;
            self.last = Some(now);
        }
    }
    /// The model finished.
    pub fn finish(&mut self) {
        if self.phase != StreamPhase::Stopped {
            self.phase = StreamPhase::Done;
        }
    }
    /// The user stopped it: what is shown stays.
    pub fn stop(&mut self) {
        self.phase = StreamPhase::Stopped;
        let s: String = self.text.chars().take(self.shown).collect();
        self.text = s;
    }
    /// Failed.
    pub fn fail(&mut self) {
        self.phase = StreamPhase::Error;
    }
    /// Advances the reveal.
    pub fn tick(&mut self, now: ClockMs) {
        let total = self.text.chars().count();
        let Some(last) = self.last else {
            self.last = Some(now);
            return;
        };
        let dt = now.since(last).as_secs_f32();
        self.last = Some(now);
        if self.pace == Pace::Instant {
            self.shown = total;
            return;
        }
        let behind = total.saturating_sub(self.shown) as f32;
        let cps = self.pace.cps().max(behind / 1.2);
        self.shown = (self.shown + (cps * dt).ceil() as usize).min(total);
    }
    /// Visible text.
    pub fn visible(&self) -> String {
        self.text.chars().take(self.shown).collect()
    }
    /// Still revealing.
    pub fn revealing(&self) -> bool {
        self.shown < self.text.chars().count() || self.phase == StreamPhase::Streaming
    }
    /// Blocks of the visible text.
    pub fn blocks(&self) -> Vec<Block> {
        parse(&self.visible())
    }
}

/// Splits `text` into chunks of about `n` characters on word boundaries (demo token streams).
pub fn chunks(text: &str, n: usize) -> Vec<String> {
    let mut out = Vec::new();
    let mut cur = String::new();
    for w in text.split_inclusive(' ') {
        cur.push_str(w);
        if cur.chars().count() >= n {
            out.push(std::mem::take(&mut cur));
        }
    }
    if !cur.is_empty() {
        out.push(cur);
    }
    out
}

/// Characters revealed `elapsed` after start at `cps` (deterministic demos without push).
pub fn revealed_at(total: usize, elapsed: Duration, cps: f32) -> usize {
    if cps <= 0.0 {
        return total;
    }
    ((elapsed.as_secs_f32() * cps) as usize).min(total)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inline_runs_and_held_markers() {
        let v = inline("Use **bold**, *it*, `code` and [docs](https://x.dev) [2].");
        assert!(v.contains(&Inline::Bold("bold".into())));
        assert!(v.contains(&Inline::Italic("it".into())));
        assert!(v.contains(&Inline::Code("code".into())));
        assert!(v.contains(&Inline::Link("docs".into(), "https://x.dev".into())));
        assert!(v.contains(&Inline::Cite(2)));
        assert_eq!(inline("half **bo"), vec![Inline::Text("half ".into())]);
        assert_eq!(inline("see [1"), vec![Inline::Text("see ".into())]);
        assert_eq!(plain(&inline("a `b` [3]")), "a b [3]");
        assert_eq!(
            inline("snake_case word"),
            vec![Inline::Text("snake_case word".into())]
        );
    }

    #[test]
    fn blocks_including_open_fence() {
        let b = parse(
            "# Title\n\nPara one\ncontinues.\n\n- a\n- b\n\n1. x\n2. y\n\n> q\n\n---\n```rust\nfn x() {}\n",
        );
        assert!(matches!(&b[0], Block::Heading(1, _)));
        assert!(matches!(&b[1], Block::Para(_)));
        assert!(matches!(&b[2], Block::List(false, v) if v.len() == 2));
        assert!(matches!(&b[3], Block::List(true, _)));
        assert!(matches!(&b[4], Block::Quote(_)));
        assert_eq!(b[5], Block::Rule);
        assert!(matches!(&b[6], Block::Code { open: true, lang, .. } if lang == "rust"));
        let t = parse("| a | b |\n|---|---|\n| 1 | 2 |\n| 3");
        assert!(matches!(&t[0], Block::Table(h, r) if h.len() == 2 && r.len() == 1));
        assert_eq!(headings(&b), vec![(1, "Title".into())]);
        assert_eq!(citations(&parse("x [2] y [1] z [2]")), vec![2, 1]);
    }

    #[test]
    fn stream_reveals_and_catches_up() {
        let mut s = MarkdownStream::new(Pace::Normal);
        s.push("Hello world, this is a stream.", ClockMs(0));
        assert_eq!(s.phase, StreamPhase::Streaming);
        s.tick(ClockMs(100));
        assert!(s.shown > 0 && s.shown < 31);
        s.tick(ClockMs(2000));
        assert_eq!(s.visible(), "Hello world, this is a stream.");
        s.push(&"x".repeat(1000), ClockMs(2000));
        s.tick(ClockMs(2600));
        assert!(s.shown > 400, "catches up when far behind");
        s.stop();
        assert_eq!(s.phase, StreamPhase::Stopped);
        assert_eq!(s.text.chars().count(), s.shown);
        assert_eq!(chunks("a b c d", 3).len(), 2);
        assert_eq!(revealed_at(100, Duration::from_millis(500), 100.0), 50);
    }
}
