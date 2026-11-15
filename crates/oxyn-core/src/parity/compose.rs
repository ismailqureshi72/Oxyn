//! Composers (plan #28, chunk 4b): the behaviour behind the affectUI Chat Composer and Run
//! Controls, the Arc UI AI composer, Rich text editor and both model pickers.
//!
//! - [`trigger`] / [`ring`] / [`stream_upto`]: `/` commands and `@` mentions at the caret, the ring
//!   counter that appears at 80 % of the limit, and replies that stream in whole-word chunks.
//! - [`Doc`]: a small block editor: markdown shortcuts, bold / italic / strike / code marks, links,
//!   a slash menu, list indent, undo and redo, and clean HTML and Markdown out.
//! - [`splice`] / [`pixel_mark`] / [`send_key`]: run controls: notes sent while an agent ran drawn
//!   into the answer where they landed, an agent's pixel mark, and Enter / ⌥ Enter routing.
//! - [`Effort`] / [`snap`]: effort levels a model accepts, snapping to the nearest one.
//!
//! Behaviour-only rebuilds; nothing is copied.

/// What a trigger character opened.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Trigger {
    /// `/` at the start of a line: commands.
    Slash,
    /// `@` after a space or at the start: files and docs.
    Mention,
}

/// The open trigger for `text` with the caret at its end: kind, byte offset of the trigger
/// character and the query typed after it.
pub fn trigger(text: &str) -> Option<(Trigger, usize, String)> {
    let line_start = text.rfind('\n').map_or(0, |i| i + 1);
    let line = &text[line_start..];
    if let Some(q) = line.strip_prefix('/')
        && !q.contains(char::is_whitespace)
    {
        return Some((Trigger::Slash, line_start, q.to_string()));
    }
    let at = text.rfind('@')?;
    let before_ok = at == 0
        || text[..at]
            .chars()
            .next_back()
            .is_some_and(char::is_whitespace);
    let q = &text[at + 1..];
    (before_ok && !q.contains(char::is_whitespace)).then(|| (Trigger::Mention, at, q.to_string()))
}

/// Built-in commands of the AI composer.
pub const COMMANDS: [(&str, &str); 4] = [
    ("summarize", "Condense the thread or a file"),
    ("translate", "Translate into another language"),
    ("explain", "Explain code or a concept"),
    ("fix", "Find and fix a bug"),
];

/// Options matching a query (case-insensitive prefix first, then substring).
pub fn filter<'a>(items: &[&'a str], q: &str) -> Vec<&'a str> {
    let q = q.to_lowercase();
    let mut a: Vec<&str> = items
        .iter()
        .copied()
        .filter(|i| i.to_lowercase().starts_with(&q))
        .collect();
    a.extend(
        items
            .iter()
            .copied()
            .filter(|i| !i.to_lowercase().starts_with(&q) && i.to_lowercase().contains(&q)),
    );
    a
}

/// Tone of the ring counter.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RingTone {
    /// 80–95 %.
    Quiet,
    /// From 95 %.
    Warn,
    /// Over the limit: sending stops.
    Over,
}

/// The ring counter: hidden under 80 % of `max`; the fill and tone otherwise.
pub fn ring(len: usize, max: usize) -> Option<(f32, RingTone)> {
    let f = len as f32 / max.max(1) as f32;
    if f < 0.8 {
        None
    } else if len > max {
        Some((1.0, RingTone::Over))
    } else if f >= 0.95 {
        Some((f, RingTone::Warn))
    } else {
        Some((f, RingTone::Quiet))
    }
}

/// Byte length of `text` revealed `elapsed_ms` into a stream: chunks of whole words every 34 ms,
/// about 80 words a second.
pub fn stream_upto(text: &str, elapsed_ms: u64) -> usize {
    let words = ((elapsed_ms / 34) * 27 / 10) as usize;
    if words == 0 {
        return 0;
    }
    let mut n = 0usize;
    let mut in_word = false;
    for (i, c) in text.char_indices() {
        if c.is_whitespace() {
            if in_word {
                n += 1;
                if n >= words {
                    return i;
                }
            }
            in_word = false;
        } else {
            in_word = true;
        }
    }
    text.len()
}

/// Words of a dictation transcript shown `elapsed_ms` into recording (one every `per_word` ms).
pub fn dictated(transcript: &str, elapsed_ms: u64, per_word: u64) -> String {
    let n = (elapsed_ms / per_word.max(1)) as usize;
    transcript
        .split_whitespace()
        .take(n)
        .collect::<Vec<_>>()
        .join(" ")
}

/// Status of a run.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RunStatus {
    /// Nothing asked yet, or finished and cleared.
    Idle,
    /// Streaming: caret and Stop.
    Running,
    /// Stopped by the user: Resume, Retry, Dismiss.
    Stopped,
    /// Failed: Resume, Retry, Dismiss.
    Error,
    /// Finished: Retry.
    Done,
}

impl RunStatus {
    /// From an id.
    pub fn from_id(s: &str) -> Self {
        match s {
            "running" => Self::Running,
            "stopped" => Self::Stopped,
            "error" => Self::Error,
            "done" => Self::Done,
            _ => Self::Idle,
        }
    }
    /// The run's buttons.
    pub fn actions(self) -> &'static [&'static str] {
        match self {
            Self::Running => &["stop"],
            Self::Stopped | Self::Error => &["resume", "retry", "dismiss"],
            Self::Done => &["retry"],
            Self::Idle => &[],
        }
    }
}

/// What Enter does in the run composer.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SendAct {
    /// Starts a run.
    Send,
    /// Sends a note into the running answer.
    Steer,
    /// Waits for the run to finish.
    Queue,
}

/// Enter sends when idle; while running Enter steers and ⌥ Enter queues.
pub fn send_key(status: RunStatus, alt: bool) -> SendAct {
    match (status, alt) {
        (RunStatus::Running, true) => SendAct::Queue,
        (RunStatus::Running, false) => SendAct::Steer,
        _ => SendAct::Send,
    }
}

/// A piece of a steered answer.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RunSeg {
    /// Answer text.
    Text(String),
    /// A note the user sent at this point.
    Steer(String),
}

/// Draws steers into the answer at the character they landed on (clamped to what has streamed).
pub fn splice(answer: &str, steers: &[(usize, String)]) -> Vec<RunSeg> {
    let mut s: Vec<&(usize, String)> = steers.iter().collect();
    s.sort_by_key(|x| x.0);
    let chars: Vec<char> = answer.chars().collect();
    let mut out = Vec::new();
    let mut at = 0usize;
    for (pos, text) in s {
        let p = (*pos).min(chars.len()).max(at);
        if p > at {
            out.push(RunSeg::Text(chars[at..p].iter().collect()));
        }
        out.push(RunSeg::Steer(text.clone()));
        at = p;
    }
    if at < chars.len() {
        out.push(RunSeg::Text(chars[at..].iter().collect()));
    }
    out
}

/// A 5×5 mirrored pixel mark for an agent id (row-major, `true` = filled).
pub fn pixel_mark(id: &str) -> [bool; 25] {
    let mut h: u32 = 0x811c_9dc5;
    for b in id.bytes() {
        h ^= u32::from(b);
        h = h.wrapping_mul(0x0100_0193);
    }
    let mut m = [false; 25];
    for r in 0..5 {
        for c in 0..3 {
            let bit = (h >> (r * 3 + c)) & 1 == 1;
            m[r * 5 + c] = bit;
            m[r * 5 + (4 - c)] = bit;
        }
    }
    if m.iter().all(|x| !x) {
        m[12] = true;
    }
    m
}

/// Reasoning effort.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Effort {
    /// Low.
    Low,
    /// Medium.
    Medium,
    /// High.
    High,
    /// Extra high.
    XHigh,
    /// Max.
    Max,
}

impl Effort {
    /// All levels.
    pub const ALL: [Effort; 5] = [
        Effort::Low,
        Effort::Medium,
        Effort::High,
        Effort::XHigh,
        Effort::Max,
    ];
    /// Label.
    pub fn label(self) -> &'static str {
        match self {
            Effort::Low => "Low",
            Effort::Medium => "Medium",
            Effort::High => "High",
            Effort::XHigh => "Extra high",
            Effort::Max => "Max",
        }
    }
    /// Id.
    pub fn id(self) -> &'static str {
        match self {
            Effort::Low => "low",
            Effort::Medium => "medium",
            Effort::High => "high",
            Effort::XHigh => "xhigh",
            Effort::Max => "max",
        }
    }
    /// From an id.
    pub fn from_id(s: &str) -> Self {
        Self::ALL
            .into_iter()
            .find(|e| e.id() == s)
            .unwrap_or(Effort::Medium)
    }
}

/// A level the model does not accept snaps to the nearest one it does (ties go lower).
pub fn snap(want: Effort, accepted: &[Effort]) -> Effort {
    accepted
        .iter()
        .copied()
        .min_by_key(|e| ((*e as i32 - want as i32).abs(), *e as i32))
        .unwrap_or(want)
}

/// "$3 / $15" per 1M tokens.
pub fn price_pair(input: f64, output: f64) -> String {
    let f = |v: f64| {
        if v.fract() == 0.0 {
            format!("${v:.0}")
        } else {
            format!("${v:.2}")
        }
    };
    format!("{} / {}", f(input), f(output))
}

// ---------------------------------------------------------------------------------------------
// Rich text
// ---------------------------------------------------------------------------------------------

/// Bold mark bit.
pub const BOLD: u8 = 1;
/// Italic mark bit.
pub const ITALIC: u8 = 2;
/// Strikethrough mark bit.
pub const STRIKE: u8 = 4;
/// Inline code mark bit.
pub const CODE: u8 = 8;

/// Kind of a block.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    /// Paragraph.
    P,
    /// Heading 1.
    H1,
    /// Heading 2.
    H2,
    /// Heading 3.
    H3,
    /// Bulleted list item.
    Bullet,
    /// Numbered list item.
    Numbered,
    /// To-do item.
    Todo,
    /// Quote.
    Quote,
    /// Code block.
    Code,
    /// Divider.
    Divider,
}

impl Kind {
    /// List-like (Enter continues it; Tab indents).
    pub fn listy(self) -> bool {
        matches!(self, Kind::Bullet | Kind::Numbered | Kind::Todo)
    }
}

/// The slash menu's blocks: id, label, kind.
pub const BLOCKS: [(&str, &str, Kind); 9] = [
    ("text", "Text", Kind::P),
    ("h1", "Heading 1", Kind::H1),
    ("h2", "Heading 2", Kind::H2),
    ("h3", "Heading 3", Kind::H3),
    ("bullet", "Bulleted list", Kind::Bullet),
    ("numbered", "Numbered list", Kind::Numbered),
    ("todo", "To-do", Kind::Todo),
    ("quote", "Quote", Kind::Quote),
    ("code", "Code block", Kind::Code),
];

/// One block.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Line {
    /// Kind.
    pub kind: Kind,
    /// Characters.
    pub chars: Vec<char>,
    /// Marks per character.
    pub marks: Vec<u8>,
    /// Link per character: index + 1 into [`Doc::hrefs`], 0 for none.
    pub link: Vec<u16>,
    /// List indent, 0..=3.
    pub indent: u8,
    /// To-do checked.
    pub checked: bool,
}

impl Line {
    /// A block of `kind` with plain text.
    pub fn new(kind: Kind, text: &str) -> Self {
        let chars: Vec<char> = text.chars().collect();
        let n = chars.len();
        Self {
            kind,
            chars,
            marks: vec![0; n],
            link: vec![0; n],
            indent: 0,
            checked: false,
        }
    }
    /// The text.
    pub fn text(&self) -> String {
        self.chars.iter().collect()
    }
    /// Runs of equal marks and link: `(text, marks, link)`.
    pub fn runs(&self) -> Vec<(String, u8, u16)> {
        let mut out: Vec<(String, u8, u16)> = Vec::new();
        for i in 0..self.chars.len() {
            let (m, l) = (self.marks[i], self.link[i]);
            match out.last_mut() {
                Some(r) if r.1 == m && r.2 == l => r.0.push(self.chars[i]),
                _ => out.push((self.chars[i].to_string(), m, l)),
            }
        }
        out
    }
}

/// The open slash menu.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Slash {
    /// Character offset of the `/`.
    pub at: usize,
    /// Query typed after it.
    pub query: String,
    /// Highlighted row among the matches.
    pub active: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct Snap {
    lines: Vec<Line>,
    caret: (usize, usize),
}

/// A small block editor (Arc Rich text editor behaviour). Selection lives inside one block.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Doc {
    /// Blocks.
    pub lines: Vec<Line>,
    /// Link targets.
    pub hrefs: Vec<String>,
    /// Caret: block and character offset.
    pub caret: (usize, usize),
    /// Selection anchor offset in the caret's block.
    pub anchor: Option<usize>,
    /// Marks for the next typed character when nothing is selected.
    pub pending: Option<u8>,
    /// Slash menu.
    pub slash: Option<Slash>,
    undo: Vec<Snap>,
    redo: Vec<Snap>,
    typing: bool,
}

fn esc(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

impl Doc {
    /// An empty document.
    pub fn empty() -> Self {
        Self {
            lines: vec![Line::new(Kind::P, "")],
            hrefs: vec![],
            caret: (0, 0),
            anchor: None,
            pending: None,
            slash: None,
            undo: vec![],
            redo: vec![],
            typing: false,
        }
    }

    /// Parses Markdown: headings, lists, to-dos, quotes, fences, dividers and inline `**`, `*`,
    /// `~~`, backticks and `[text](url)`.
    pub fn from_markdown(md: &str) -> Self {
        let mut d = Self::empty();
        d.lines.clear();
        let mut code: Option<Vec<String>> = None;
        for raw in md.lines() {
            if let Some(buf) = code.as_mut() {
                if raw.trim_start().starts_with("```") {
                    let l = Line::new(Kind::Code, &buf.join("\n"));
                    d.lines.push(l);
                    code = None;
                } else {
                    buf.push(raw.to_string());
                }
                continue;
            }
            if raw.trim_start().starts_with("```") {
                code = Some(vec![]);
                continue;
            }
            if raw.trim().is_empty() {
                continue;
            }
            let indent = (raw.len() - raw.trim_start().len()) / 2;
            let t = raw.trim_start();
            let (kind, rest, checked) = if t == "---" {
                (Kind::Divider, "", false)
            } else if let Some(r) = t.strip_prefix("### ") {
                (Kind::H3, r, false)
            } else if let Some(r) = t.strip_prefix("## ") {
                (Kind::H2, r, false)
            } else if let Some(r) = t.strip_prefix("# ") {
                (Kind::H1, r, false)
            } else if let Some(r) = t.strip_prefix("- [ ] ") {
                (Kind::Todo, r, false)
            } else if let Some(r) = t.strip_prefix("- [x] ") {
                (Kind::Todo, r, true)
            } else if let Some(r) = t.strip_prefix("- ").or_else(|| t.strip_prefix("* ")) {
                (Kind::Bullet, r, false)
            } else if let Some(r) = t.strip_prefix("> ") {
                (Kind::Quote, r, false)
            } else if t.len() > 2
                && t.split_once(". ")
                    .is_some_and(|(n, _)| !n.is_empty() && n.chars().all(|c| c.is_ascii_digit()))
            {
                (
                    Kind::Numbered,
                    t.split_once(". ").map_or("", |x| x.1),
                    false,
                )
            } else {
                (Kind::P, t, false)
            };
            let mut l = d.inline(rest);
            l.kind = kind;
            l.checked = checked;
            l.indent = if kind.listy() { indent.min(3) as u8 } else { 0 };
            d.lines.push(l);
        }
        if let Some(buf) = code {
            d.lines.push(Line::new(Kind::Code, &buf.join("\n")));
        }
        if d.lines.is_empty() {
            d.lines.push(Line::new(Kind::P, ""));
        }
        let last = d.lines.len() - 1;
        d.caret = (last, d.lines[last].chars.len());
        d
    }

    fn inline(&mut self, s: &str) -> Line {
        let mut l = Line::new(Kind::P, "");
        let c: Vec<char> = s.chars().collect();
        let mut i = 0;
        let mut m = 0u8;
        let link = 0u16;
        while i < c.len() {
            let rest: String = c[i..].iter().collect();
            if m & CODE == 0 && rest.starts_with("**") {
                m ^= BOLD;
                i += 2;
            } else if m & CODE == 0 && rest.starts_with("~~") {
                m ^= STRIKE;
                i += 2;
            } else if c[i] == '`' {
                m ^= CODE;
                i += 1;
            } else if m & CODE == 0 && (c[i] == '*' || c[i] == '_') {
                m ^= ITALIC;
                i += 1;
            } else if m & CODE == 0 && c[i] == '[' && link == 0 {
                if let Some(close) = rest.find("](")
                    && let Some(end) = rest[close..].find(')')
                {
                    let text = &rest[1..close];
                    let href = &rest[close + 2..close + end];
                    self.hrefs.push(href.to_string());
                    let li = self.hrefs.len() as u16;
                    for ch in text.chars() {
                        l.chars.push(ch);
                        l.marks.push(m);
                        l.link.push(li);
                    }
                    i += rest[..close + end + 1].chars().count();
                    continue;
                }
                l.chars.push(c[i]);
                l.marks.push(m);
                l.link.push(link);
                i += 1;
            } else {
                l.chars.push(c[i]);
                l.marks.push(m);
                l.link.push(link);
                i += 1;
            }
        }
        l
    }

    fn inline_md(&self, l: &Line) -> String {
        let mut out = String::new();
        for (text, m, link) in l.runs() {
            let mut t = if m & CODE != 0 {
                format!("`{text}`")
            } else {
                text
            };
            if m & ITALIC != 0 {
                t = format!("*{t}*");
            }
            if m & BOLD != 0 {
                t = format!("**{t}**");
            }
            if m & STRIKE != 0 {
                t = format!("~~{t}~~");
            }
            if link > 0 {
                t = format!("[{t}]({})", self.hrefs[link as usize - 1]);
            }
            out.push_str(&t);
        }
        out
    }

    fn inline_html(&self, l: &Line) -> String {
        let mut out = String::new();
        for (text, m, link) in l.runs() {
            let mut t = esc(&text);
            if m & CODE != 0 {
                t = format!("<code>{t}</code>");
            }
            if m & ITALIC != 0 {
                t = format!("<em>{t}</em>");
            }
            if m & BOLD != 0 {
                t = format!("<strong>{t}</strong>");
            }
            if m & STRIKE != 0 {
                t = format!("<s>{t}</s>");
            }
            if link > 0 {
                t = format!(
                    "<a href=\"{}\">{t}</a>",
                    esc(&self.hrefs[link as usize - 1])
                );
            }
            out.push_str(&t);
        }
        out
    }

    /// Markdown out.
    pub fn markdown(&self) -> String {
        let mut out: Vec<String> = Vec::new();
        let mut n = 0;
        for l in &self.lines {
            let pad = "  ".repeat(l.indent as usize);
            n = if l.kind == Kind::Numbered { n + 1 } else { 0 };
            let body = self.inline_md(l);
            out.push(match l.kind {
                Kind::P => body,
                Kind::H1 => format!("# {body}"),
                Kind::H2 => format!("## {body}"),
                Kind::H3 => format!("### {body}"),
                Kind::Bullet => format!("{pad}- {body}"),
                Kind::Numbered => format!("{pad}{n}. {body}"),
                Kind::Todo => format!("{pad}- [{}] {body}", if l.checked { "x" } else { " " }),
                Kind::Quote => format!("> {body}"),
                Kind::Code => format!("```\n{}\n```", l.text()),
                Kind::Divider => "---".into(),
            });
        }
        let mut s = String::new();
        for (i, part) in out.iter().enumerate() {
            if i > 0 {
                let tight = self.lines[i].kind.listy() && self.lines[i - 1].kind.listy();
                s.push_str(if tight { "\n" } else { "\n\n" });
            }
            s.push_str(part);
        }
        s.trim().to_string()
    }

    /// Clean HTML out: canonical tags and only `href` attributes.
    pub fn html(&self) -> String {
        let mut out = String::new();
        let mut open: Option<&str> = None;
        for l in &self.lines {
            let want = match l.kind {
                Kind::Bullet | Kind::Todo => Some("ul"),
                Kind::Numbered => Some("ol"),
                _ => None,
            };
            if open != want {
                if let Some(o) = open {
                    out.push_str(&format!("</{o}>"));
                }
                if let Some(w) = want {
                    out.push_str(&format!("<{w}>"));
                }
                open = want;
            }
            let b = self.inline_html(l);
            out.push_str(&match l.kind {
                Kind::P => format!("<p>{b}</p>"),
                Kind::H1 => format!("<h1>{b}</h1>"),
                Kind::H2 => format!("<h2>{b}</h2>"),
                Kind::H3 => format!("<h3>{b}</h3>"),
                Kind::Bullet | Kind::Numbered => format!("<li>{b}</li>"),
                Kind::Todo => format!("<li>[{}] {b}</li>", if l.checked { "x" } else { " " }),
                Kind::Quote => format!("<blockquote>{b}</blockquote>"),
                Kind::Code => format!("<pre><code>{}</code></pre>", esc(&l.text())),
                Kind::Divider => "<hr>".into(),
            });
        }
        if let Some(o) = open {
            out.push_str(&format!("</{o}>"));
        }
        out
    }

    /// Plain text, one block per line.
    pub fn text(&self) -> String {
        self.lines
            .iter()
            .map(Line::text)
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// Nothing typed.
    pub fn is_empty(&self) -> bool {
        self.lines.len() == 1 && self.lines[0].chars.is_empty() && self.lines[0].kind == Kind::P
    }

    /// Selection `(start, end)` in the caret's block, when not collapsed.
    pub fn selection(&self) -> Option<(usize, usize)> {
        let a = self.anchor?;
        let c = self.caret.1;
        (a != c).then(|| (a.min(c), a.max(c)))
    }

    /// Undo is available.
    pub fn can_undo(&self) -> bool {
        !self.undo.is_empty()
    }
    /// Redo is available.
    pub fn can_redo(&self) -> bool {
        !self.redo.is_empty()
    }

    fn snap(&mut self, typing: bool) {
        if typing && self.typing {
            return;
        }
        self.undo.push(Snap {
            lines: self.lines.clone(),
            caret: self.caret,
        });
        if self.undo.len() > 100 {
            self.undo.remove(0);
        }
        self.redo.clear();
        self.typing = typing;
    }

    /// Steps back.
    pub fn undo(&mut self) {
        if let Some(s) = self.undo.pop() {
            self.redo.push(Snap {
                lines: std::mem::replace(&mut self.lines, s.lines),
                caret: self.caret,
            });
            self.caret = s.caret;
            self.anchor = None;
            self.typing = false;
        }
    }

    /// Steps forward.
    pub fn redo(&mut self) {
        if let Some(s) = self.redo.pop() {
            self.undo.push(Snap {
                lines: std::mem::replace(&mut self.lines, s.lines),
                caret: self.caret,
            });
            self.caret = s.caret;
            self.anchor = None;
            self.typing = false;
        }
    }

    fn cut_selection(&mut self) {
        if let Some((s, e)) = self.selection() {
            let l = &mut self.lines[self.caret.0];
            l.chars.drain(s..e);
            l.marks.drain(s..e);
            l.link.drain(s..e);
            self.caret.1 = s;
        }
        self.anchor = None;
    }

    /// Types a character: replaces the selection, takes the pending marks (or the marks of the
    /// character before), and runs markdown shortcuts on Space and the slash menu on `/`.
    pub fn type_char(&mut self, ch: char) {
        self.snap(ch != ' ');
        self.cut_selection();
        let (li, at) = self.caret;
        let l = &mut self.lines[li];
        let inherit = if at > 0 && l.marks[at - 1] & CODE == 0 {
            l.marks[at - 1]
        } else {
            0
        };
        let m = self.pending.unwrap_or(inherit);
        l.chars.insert(at, ch);
        l.marks.insert(at, m);
        l.link.insert(at, 0);
        self.caret.1 += 1;
        if let Some(s) = self.slash.as_mut() {
            if ch == ' ' {
                self.slash = None;
            } else {
                s.query.push(ch);
                s.active = 0;
            }
            return;
        }
        if ch == ' ' && l.kind == Kind::P {
            let head: String = l.chars[..at].iter().collect();
            let k = match head.as_str() {
                "#" => Some(Kind::H1),
                "##" => Some(Kind::H2),
                "###" => Some(Kind::H3),
                "-" | "*" => Some(Kind::Bullet),
                "1." => Some(Kind::Numbered),
                "[]" => Some(Kind::Todo),
                ">" => Some(Kind::Quote),
                "```" => Some(Kind::Code),
                "---" => Some(Kind::Divider),
                _ => None,
            };
            if let Some(k) = k {
                let n = at + 1;
                l.chars.drain(..n);
                l.marks.drain(..n);
                l.link.drain(..n);
                l.kind = k;
                self.caret.1 = 0;
                if k == Kind::Divider {
                    self.lines.insert(li + 1, Line::new(Kind::P, ""));
                    self.caret = (li + 1, 0);
                }
            }
        }
        let l = &self.lines[self.caret.0];
        if ch == '/' && l.kind != Kind::Code {
            let i = self.caret.1 - 1;
            if i == 0 || l.chars[i - 1] == ' ' {
                self.slash = Some(Slash {
                    at: i,
                    query: String::new(),
                    active: 0,
                });
            }
        }
    }

    /// Slash menu matches.
    pub fn slash_items(&self) -> Vec<(&'static str, &'static str, Kind)> {
        let q = self
            .slash
            .as_ref()
            .map(|s| s.query.to_lowercase())
            .unwrap_or_default();
        BLOCKS
            .iter()
            .copied()
            .filter(|(id, label, _)| {
                q.is_empty() || label.to_lowercase().contains(&q) || id.starts_with(&q)
            })
            .collect()
    }

    /// Moves the slash highlight.
    pub fn slash_move(&mut self, d: i32) {
        let n = self.slash_items().len() as i32;
        if let Some(s) = self.slash.as_mut()
            && n > 0
        {
            s.active = (s.active as i32 + d).rem_euclid(n) as usize;
        }
    }

    /// Chooses a slash row (the highlighted one when `None`): the block turns into it and the
    /// typed `/query` goes away.
    pub fn slash_choose(&mut self, i: Option<usize>) {
        let items = self.slash_items();
        let Some(s) = self.slash.take() else { return };
        let Some((_, _, k)) = items.get(i.unwrap_or(s.active)).copied() else {
            return;
        };
        self.snap(false);
        let l = &mut self.lines[self.caret.0];
        let end = (s.at + 1 + s.query.chars().count()).min(l.chars.len());
        l.chars.drain(s.at..end);
        l.marks.drain(s.at..end);
        l.link.drain(s.at..end);
        l.kind = k;
        self.caret.1 = s.at;
    }

    /// Escape on the slash menu.
    pub fn slash_close(&mut self) {
        self.slash = None;
    }

    /// Backspace: selection, then the character before; at a block start a styled block turns
    /// back into text, a list item outdents, and a text block joins the one above.
    pub fn backspace(&mut self) {
        self.snap(false);
        if self.selection().is_some() {
            self.cut_selection();
            return;
        }
        self.anchor = None;
        let (li, at) = self.caret;
        if let Some(s) = self.slash.as_mut()
            && s.query.pop().is_none()
        {
            self.slash = None;
        }
        if at > 0 {
            let l = &mut self.lines[li];
            l.chars.remove(at - 1);
            l.marks.remove(at - 1);
            l.link.remove(at - 1);
            self.caret.1 -= 1;
            return;
        }
        let l = &mut self.lines[li];
        if l.kind.listy() && l.indent > 0 {
            l.indent -= 1;
        } else if l.kind != Kind::P {
            l.kind = Kind::P;
        } else if li > 0 {
            let cur = self.lines.remove(li);
            let prev = &mut self.lines[li - 1];
            if prev.kind == Kind::Divider {
                self.lines[li - 1] = cur;
                self.caret = (li - 1, 0);
                return;
            }
            let n = prev.chars.len();
            prev.chars.extend(cur.chars);
            prev.marks.extend(cur.marks);
            prev.link.extend(cur.link);
            self.caret = (li - 1, n);
        }
    }

    /// Enter: splits the block. An empty list item turns into text; lists and to-dos continue.
    /// In a code block, or with `soft` (Shift+Enter), inserts a line break instead.
    pub fn enter(&mut self, soft: bool) {
        if self.slash.is_some() {
            self.slash_choose(None);
            return;
        }
        self.snap(false);
        self.cut_selection();
        let (li, at) = self.caret;
        let kind = self.lines[li].kind;
        if soft || kind == Kind::Code {
            let l = &mut self.lines[li];
            l.chars.insert(at, '\n');
            l.marks.insert(at, 0);
            l.link.insert(at, 0);
            self.caret.1 += 1;
            return;
        }
        if kind.listy() && self.lines[li].chars.is_empty() {
            self.lines[li].kind = Kind::P;
            self.lines[li].indent = 0;
            return;
        }
        let l = &mut self.lines[li];
        let mut n = Line::new(
            if kind.listy() || kind == Kind::Quote {
                kind
            } else {
                Kind::P
            },
            "",
        );
        n.indent = l.indent;
        n.chars = l.chars.split_off(at);
        n.marks = l.marks.split_off(at);
        n.link = l.link.split_off(at);
        self.lines.insert(li + 1, n);
        self.caret = (li + 1, 0);
    }

    /// Tab / Shift+Tab: indents a list item, or two spaces in a code block.
    pub fn tab(&mut self, out: bool) -> bool {
        let li = self.caret.0;
        match self.lines[li].kind {
            k if k.listy() => {
                self.snap(false);
                let l = &mut self.lines[li];
                l.indent = if out {
                    l.indent.saturating_sub(1)
                } else {
                    (l.indent + 1).min(3)
                };
                true
            }
            Kind::Code if !out => {
                self.type_char(' ');
                self.type_char(' ');
                true
            }
            _ => false,
        }
    }

    /// Arrow left/right (`dir`), extending the selection with Shift.
    pub fn move_h(&mut self, dir: i32, shift: bool) {
        let (li, at) = self.caret;
        if shift && self.anchor.is_none() {
            self.anchor = Some(at);
        }
        if !shift {
            if let Some((s, e)) = self.selection() {
                self.caret.1 = if dir < 0 { s } else { e };
                self.anchor = None;
                return;
            }
            self.anchor = None;
        }
        let len = self.lines[li].chars.len();
        if dir < 0 {
            if at > 0 {
                self.caret.1 -= 1;
            } else if li > 0 && !shift {
                self.caret = (li - 1, self.lines[li - 1].chars.len());
            }
        } else if at < len {
            self.caret.1 += 1;
        } else if li + 1 < self.lines.len() && !shift {
            self.caret = (li + 1, 0);
        }
        self.typing = false;
        self.slash = None;
        self.pending = None;
    }

    /// Arrow up/down: the block above or below, keeping the column.
    pub fn move_v(&mut self, dir: i32) {
        let (li, at) = self.caret;
        let n = (li as i32 + dir).clamp(0, self.lines.len() as i32 - 1) as usize;
        self.caret = (n, at.min(self.lines[n].chars.len()));
        self.anchor = None;
        self.typing = false;
        self.pending = None;
    }

    /// Home / End (with Shift to select).
    pub fn edge(&mut self, end: bool, shift: bool) {
        if shift && self.anchor.is_none() {
            self.anchor = Some(self.caret.1);
        } else if !shift {
            self.anchor = None;
        }
        self.caret.1 = if end {
            self.lines[self.caret.0].chars.len()
        } else {
            0
        };
    }

    /// Pointer down on a character position.
    pub fn place(&mut self, line: usize, at: usize) {
        let li = line.min(self.lines.len() - 1);
        self.caret = (li, at.min(self.lines[li].chars.len()));
        self.anchor = Some(self.caret.1);
        self.slash = None;
        self.typing = false;
        self.pending = None;
    }

    /// Pointer dragged to a position: extends the selection inside the caret's block.
    pub fn drag_to(&mut self, line: usize, at: usize) {
        if line == self.caret.0 {
            self.caret.1 = at.min(self.lines[line].chars.len());
        }
    }

    /// Selects the whole block (double click falls back to this; ⌘A).
    pub fn select_block(&mut self) {
        self.anchor = Some(0);
        self.caret.1 = self.lines[self.caret.0].chars.len();
    }

    /// Marks active at the caret or across the whole selection.
    pub fn active_marks(&self) -> u8 {
        let l = &self.lines[self.caret.0];
        match self.selection() {
            Some((s, e)) => l.marks[s..e].iter().fold(0xff, |a, m| a & m),
            None => self.pending.unwrap_or(if self.caret.1 > 0 {
                l.marks[self.caret.1 - 1]
            } else {
                0
            }),
        }
    }

    /// ⌘B / ⌘I / ⌘⇧X / ⌘E: toggles a mark on the selection, or for the next typed text.
    pub fn toggle(&mut self, mark: u8) {
        let on = self.active_marks() & mark != 0;
        match self.selection() {
            Some((s, e)) => {
                self.snap(false);
                let l = &mut self.lines[self.caret.0];
                for m in &mut l.marks[s..e] {
                    if on {
                        *m &= !mark;
                    } else {
                        *m |= mark;
                    }
                }
            }
            None => {
                let cur = self.active_marks();
                self.pending = Some(if on { cur & !mark } else { cur | mark });
            }
        }
    }

    /// The link under the selection or caret.
    pub fn link_here(&self) -> Option<String> {
        let l = &self.lines[self.caret.0];
        let i = self
            .selection()
            .map(|s| s.0)
            .unwrap_or(self.caret.1.saturating_sub(1));
        l.link
            .get(i)
            .filter(|x| **x > 0)
            .map(|x| self.hrefs[*x as usize - 1].clone())
    }

    /// ⌘K: links the selection (empty `href` removes the link).
    pub fn set_link(&mut self, href: &str) {
        let Some((s, e)) = self.selection() else {
            return;
        };
        self.snap(false);
        let li = if href.trim().is_empty() {
            0
        } else {
            self.hrefs.push(href.trim().to_string());
            self.hrefs.len() as u16
        };
        for x in &mut self.lines[self.caret.0].link[s..e] {
            *x = li;
        }
    }

    /// Checks or unchecks a to-do.
    pub fn check(&mut self, line: usize) {
        self.snap(false);
        if let Some(l) = self.lines.get_mut(line) {
            l.checked = !l.checked;
        }
    }

    /// Replaces everything with Markdown as one undo step.
    pub fn set_markdown(&mut self, md: &str) {
        self.snap(false);
        let n = Self::from_markdown(md);
        self.lines = n.lines;
        self.hrefs = n.hrefs;
        self.caret = n.caret;
        self.anchor = None;
    }

    /// Empties the editor as one undo step.
    pub fn clear(&mut self) {
        self.snap(false);
        self.lines = vec![Line::new(Kind::P, "")];
        self.caret = (0, 0);
        self.anchor = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn triggers_ring_stream() {
        assert_eq!(trigger("/sum"), Some((Trigger::Slash, 0, "sum".into())));
        assert_eq!(trigger("hi /sum"), None);
        assert_eq!(
            trigger("look at @rea"),
            Some((Trigger::Mention, 8, "rea".into()))
        );
        assert_eq!(trigger("mail a@b"), None);
        assert_eq!(ring(10, 100), None);
        assert_eq!(ring(85, 100), Some((0.85, RingTone::Quiet)));
        assert_eq!(ring(101, 100).map(|r| r.1), Some(RingTone::Over));
        let t = "one two three four five six seven";
        assert_eq!(stream_upto(t, 0), 0);
        assert_eq!(&t[..stream_upto(t, 34)], "one two");
        assert_eq!(stream_upto(t, 10_000), t.len());
        assert_eq!(dictated("a b c d", 350, 100), "a b c");
        assert_eq!(
            filter(&["summarize", "translate", "fix"], "t"),
            vec!["translate"]
        );
    }

    #[test]
    fn run_controls() {
        assert_eq!(send_key(RunStatus::Running, true), SendAct::Queue);
        assert_eq!(send_key(RunStatus::Running, false), SendAct::Steer);
        assert_eq!(send_key(RunStatus::Done, false), SendAct::Send);
        let s = splice("Hello world", &[(5, "shorter".into())]);
        assert_eq!(
            s,
            vec![
                RunSeg::Text("Hello".into()),
                RunSeg::Steer("shorter".into()),
                RunSeg::Text(" world".into())
            ]
        );
        assert_eq!(pixel_mark("agent"), pixel_mark("agent"));
        let m = pixel_mark("x");
        assert_eq!(m[0], m[4]);
        assert_eq!(
            RunStatus::Stopped.actions(),
            &["resume", "retry", "dismiss"]
        );
        assert_eq!(
            snap(Effort::Max, &[Effort::Low, Effort::High]),
            Effort::High
        );
        assert_eq!(
            snap(Effort::Medium, &[Effort::Low, Effort::High]),
            Effort::Low
        );
        assert_eq!(price_pair(3.0, 15.0), "$3 / $15");
    }

    #[test]
    fn rich_text_shortcuts_marks_and_output() {
        let mut d = Doc::empty();
        for c in "## Plan".chars() {
            d.type_char(c);
        }
        assert_eq!(d.lines[0].kind, Kind::H2);
        d.enter(false);
        for c in "- ship it".chars() {
            d.type_char(c);
        }
        assert_eq!(d.lines[1].kind, Kind::Bullet);
        d.edge(false, false);
        d.edge(true, true);
        d.toggle(BOLD);
        assert_eq!(d.markdown(), "## Plan\n\n- **ship it**");
        assert_eq!(
            d.html(),
            "<h2>Plan</h2><ul><li><strong>ship it</strong></li></ul>"
        );
        d.set_link("https://x.dev");
        assert!(d.html().contains("<a href=\"https://x.dev\">"));
        d.undo();
        d.undo();
        assert_eq!(d.markdown(), "## Plan\n\n- ship it");
        d.redo();
        assert!(d.markdown().contains("**ship it**"));
        d.edge(true, false);
        d.enter(false);
        d.enter(false);
        assert_eq!(d.lines[2].kind, Kind::P, "empty list item turns into text");
        d.type_char('/');
        d.type_char('q');
        assert_eq!(d.slash_items().len(), 1);
        d.enter(false);
        assert_eq!(d.lines[2].kind, Kind::Quote);
        assert!(d.lines[2].chars.is_empty());
        d.backspace();
        assert_eq!(d.lines[2].kind, Kind::P);
        d.backspace();
        assert_eq!(d.lines.len(), 2);
        assert!(d.can_undo());
    }

    #[test]
    fn markdown_round_trip() {
        let md = "# Title\n\nSome *soft* and `code` with [a link](https://a.b).\n\n1. one\n\n- [x] done\n\n```\nlet x = 1;\n```";
        let d = Doc::from_markdown(md);
        assert_eq!(d.lines.len(), 5);
        assert_eq!(d.lines[4].kind, Kind::Code);
        let out = d.markdown();
        assert!(out.contains("*soft*"));
        assert!(out.contains("[a link](https://a.b)"));
        assert!(out.contains("- [x] done"));
        let mut e = Doc::empty();
        e.toggle(ITALIC);
        e.type_char('a');
        assert_eq!(e.markdown(), "*a*");
    }
}
