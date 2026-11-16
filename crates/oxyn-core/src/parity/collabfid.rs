//! affectUI fidelity pass (plan #32, overnight batch, family B part 6): the model behind the exact
//! **Collaboration Stream** – what agents say to each other: a quiet chat where every message
//! shows who it is from and who it is for, handoffs and questions for you tagged, files as chips
//! that open a preview, details under a message, and a composer that names agents with @.
//! Behaviour only; times are minutes after midnight.

/// One agent of the run.
#[derive(Clone, Debug, PartialEq)]
pub struct StreamAgent {
    /// Id.
    pub id: String,
    /// Name.
    pub name: String,
    /// Model (shown in the @ list).
    pub model: String,
}

/// What a message is.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StreamKind {
    /// A request.
    Ask,
    /// An answer.
    Reply,
    /// Work handed on – tagged Handoff.
    Handoff,
    /// Something shared.
    Share,
    /// A question for you – tagged Needs you until you reply.
    Question,
    /// A quiet line "X started Y".
    Spawn,
}

/// What an attachment is (drawn icon).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AttKind {
    /// Tool calls / code.
    Code,
    /// A table (.csv).
    Table,
    /// A document (.md, .txt, .pdf).
    Doc,
    /// A link (a PR, a URL).
    Link,
    /// A folder ("…/").
    Folder,
    /// A frame / image.
    Frame,
}

impl AttKind {
    /// `code` · `table` · `doc` · `link` · `folder` · `frame`.
    pub fn id(self) -> &'static str {
        match self {
            Self::Code => "code",
            Self::Table => "table",
            Self::Doc => "doc",
            Self::Link => "link",
            Self::Folder => "folder",
            Self::Frame => "frame",
        }
    }
}

/// Tells the kind from a label: "…/" a folder, .csv a table, .md / .txt / .pdf a document,
/// "PR #" or a URL a link, .png / .fig a frame, anything else code.
pub fn att_kind(label: &str) -> AttKind {
    let l = label.to_ascii_lowercase();
    if l.ends_with('/') {
        AttKind::Folder
    } else if l.ends_with(".csv") {
        AttKind::Table
    } else if l.ends_with(".md") || l.ends_with(".txt") || l.ends_with(".pdf") {
        AttKind::Doc
    } else if l.starts_with("pr #") || l.starts_with("http") {
        AttKind::Link
    } else if l.ends_with(".png") || l.ends_with(".fig") || l.ends_with(".jpg") {
        AttKind::Frame
    } else {
        AttKind::Code
    }
}

/// A file or result on a message.
#[derive(Clone, Debug, PartialEq)]
pub struct StreamAttachment {
    /// Label on the chip.
    pub label: String,
    /// One line under the preview's title.
    pub meta: String,
    /// The preview's lines.
    pub preview: Vec<String>,
}

/// One message.
#[derive(Clone, Debug, PartialEq)]
pub struct StreamMessage {
    /// Id.
    pub id: String,
    /// Sender: an agent id or `you`.
    pub from: String,
    /// Receiver: an agent id, `you` or `all`.
    pub to: String,
    /// Kind.
    pub kind: StreamKind,
    /// Text (brand names listed in [`STREAM_LOGOS`] get their mark).
    pub text: String,
    /// Minutes after midnight.
    pub at: u32,
    /// The id it answers.
    pub reply_to: Option<String>,
    /// Still being written.
    pub typing: bool,
    /// Chips.
    pub attachments: Vec<StreamAttachment>,
    /// (label, value) facts shown under it when picked.
    pub details: Vec<(String, String)>,
}

/// "14:04".
pub fn stream_time(at: u32) -> String {
    let m = at % (24 * 60);
    format!("{:02}:{:02}", m / 60, m % 60)
}

/// The name for an id: `you` → You, `all` → Everyone.
pub fn stream_name(agents: &[StreamAgent], id: &str) -> String {
    match id {
        "you" => "You".into(),
        "all" => "Everyone".into(),
        _ => agents
            .iter()
            .find(|a| a.id == id)
            .map_or_else(|| id.to_string(), |a| a.name.clone()),
    }
}

/// The colour slot (0–5) and dot pattern for an id: `you` is slot 0 (ink), agents follow in order.
pub fn stream_mark(agents: &[StreamAgent], id: &str) -> (usize, [bool; 9]) {
    const P: [[bool; 9]; 6] = [
        [true, true, true, true, true, true, true, true, true],
        [true, false, true, false, true, false, true, false, true],
        [true, true, false, false, false, false, false, true, true],
        [false, false, false, true, true, true, true, true, true],
        [true, false, true, false, false, false, false, true, false],
        [false, true, false, true, false, true, false, true, false],
    ];
    let slot = if id == "you" {
        0
    } else {
        agents
            .iter()
            .position(|a| a.id == id)
            .map_or(5, |i| 1 + i % 5)
    };
    (slot, P[slot])
}

/// A question that still waits on you: a question to `you` with no message from `you` that
/// has `reply_to` set to it.
pub fn stream_needs_you(msgs: &[StreamMessage], m: &StreamMessage) -> bool {
    m.kind == StreamKind::Question
        && m.to == "you"
        && !msgs
            .iter()
            .any(|r| r.from == "you" && r.reply_to.as_deref() == Some(m.id.as_str()))
}

/// The quote shows unless the answered message sits right above.
pub fn stream_quote(msgs: &[StreamMessage], i: usize) -> Option<&StreamMessage> {
    let r = msgs.get(i)?.reply_to.as_deref()?;
    if i > 0 && msgs[i - 1].id == r {
        return None;
    }
    msgs.iter().find(|m| m.id == r)
}

/// Whether a message stays when an agent tab is picked (from or to it; `all` stays for everyone).
pub fn stream_involves(m: &StreamMessage, agent: &str) -> bool {
    m.from == agent || m.to == agent
}

/// Agents for the @ list, filtered by what follows the @ (name starts with it, any case).
pub fn stream_mentions<'a>(agents: &'a [StreamAgent], q: &str) -> Vec<&'a StreamAgent> {
    let q = q.trim().to_ascii_lowercase();
    agents
        .iter()
        .filter(|a| q.is_empty() || a.name.to_ascii_lowercase().starts_with(&q))
        .collect()
}

/// The text after the last "@" that starts a word, if the caret sits in a mention.
pub fn stream_at_query(text: &str) -> Option<String> {
    let i = text.rfind('@')?;
    if i > 0 && !text[..i].ends_with(' ') {
        return None;
    }
    let q = &text[i + 1..];
    (!q.contains(' ')).then(|| q.to_string())
}

/// Names that get their brand mark in the text.
pub const STREAM_LOGOS: [(&str, &str); 5] = [
    ("Notion", "notion"),
    ("Linear", "linear"),
    ("Figma", "figma"),
    ("Stripe", "stripe"),
    ("GitHub", "github"),
];

/// Splits text into plain runs and brand names: (text, brand id or "").
pub fn stream_segments(text: &str) -> Vec<(String, &'static str)> {
    let mut out: Vec<(String, &'static str)> = vec![];
    let mut rest = text;
    loop {
        let hit = STREAM_LOGOS
            .iter()
            .filter_map(|(n, b)| rest.find(n).map(|i| (i, *n, *b)))
            .min_by_key(|h| h.0);
        match hit {
            Some((i, n, b)) => {
                if i > 0 {
                    out.push((rest[..i].to_string(), ""));
                }
                out.push((n.to_string(), b));
                rest = &rest[i + n.len()..];
            }
            None => {
                if !rest.is_empty() {
                    out.push((rest.to_string(), ""));
                }
                return out;
            }
        }
    }
}

/// The demo run's agents (our own).
pub fn stream_demo_agents() -> Vec<StreamAgent> {
    let a = |id: &str, name: &str, model: &str| StreamAgent {
        id: id.into(),
        name: name.into(),
        model: model.into(),
    };
    vec![
        a("analyst", "Analyst", "Gemini 3.8 Flash"),
        a("editor", "Editor", "Sonnet 5.5"),
        a("planner", "Planner", "Gemini 3.8"),
        a("builder", "Builder", "Opus 5.5"),
    ]
}

/// The demo run's messages (our own copy): a docs-page refresh.
pub fn stream_demo() -> Vec<StreamMessage> {
    let m = |id: &str, from: &str, to: &str, kind, text: &str, at: u32| StreamMessage {
        id: id.into(),
        from: from.into(),
        to: to.into(),
        kind,
        text: text.into(),
        at,
        reply_to: None,
        typing: false,
        attachments: vec![],
        details: vec![],
    };
    let att = |label: &str, meta: &str, lines: &[&str]| StreamAttachment {
        label: label.into(),
        meta: meta.into(),
        preview: lines.iter().map(|s| s.to_string()).collect(),
    };
    use StreamKind::*;
    let mut v = vec![
        m(
            "brief",
            "you",
            "planner",
            Ask,
            "Refresh the onboarding docs by Thursday – shorter steps, one screenshot per step.",
            13 * 60 + 58,
        ),
        m(
            "s1",
            "planner",
            "analyst",
            Spawn,
            "Planner started Analyst",
            13 * 60 + 59,
        ),
        m(
            "ask1",
            "planner",
            "analyst",
            Ask,
            "Find where new users drop off in the current guide.",
            14 * 60,
        ),
        m(
            "find",
            "analyst",
            "planner",
            Reply,
            "Step 4 loses a third of readers – the API key part. Notes are in Notion.",
            14 * 60 + 4,
        ),
        m(
            "hand",
            "planner",
            "editor",
            Handoff,
            "Rewrite steps 3–5 around the key; keep each under 60 words.",
            14 * 60 + 6,
        ),
        m(
            "draft",
            "editor",
            "builder",
            Share,
            "Draft is ready – can you wire the new screenshots in?",
            14 * 60 + 9,
        ),
        m(
            "q",
            "builder",
            "you",
            Question,
            "The guide uses the old dashboard. Should the screenshots show the new one, or wait for its launch?",
            14 * 60 + 11,
        ),
        m(
            "up",
            "planner",
            "all",
            Share,
            "Steps 3–5 are rewritten and up for review on the preview build.",
            14 * 60 + 12,
        ),
    ];
    v[3].reply_to = Some("ask1".into());
    v[3].attachments = vec![
        att(
            "search ×4",
            "4 calls",
            &[
                "search(\"onboarding guide drop off\")",
                "search(\"api key step completion\")",
                "read(analytics/funnel.csv)",
                "read(notion: Docs research)",
            ],
        ),
        att(
            "funnel.csv",
            "5 rows",
            &["step,readers", "1,1000", "2,912", "3,840", "4,561", "5,548"],
        ),
    ];
    v[3].details = vec![
        ("Tool calls".into(), "4".into()),
        ("Tokens".into(), "18.2k".into()),
        ("Took".into(), "3m 41s".into()),
    ];
    v[5].attachments = vec![att(
        "steps-3-5.md",
        "212 words",
        &["## Step 3 – Create a key", "Open Settings, then API keys…"],
    )];
    v[6].reply_to = Some("brief".into());
    v[7].attachments = vec![att(
        "PR #217",
        "Open · 3 files",
        &["docs/onboarding.md +48 −71"],
    )];
    v
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kinds_quotes_needs_and_text() {
        assert_eq!(att_kind("src/"), AttKind::Folder);
        assert_eq!(att_kind("funnel.csv"), AttKind::Table);
        assert_eq!(att_kind("notes.md"), AttKind::Doc);
        assert_eq!(att_kind("PR #217"), AttKind::Link);
        assert_eq!(att_kind("search ×4"), AttKind::Code);
        assert_eq!(stream_time(14 * 60 + 4), "14:04");
        let a = stream_demo_agents();
        let mut m = stream_demo();
        assert_eq!(stream_name(&a, "all"), "Everyone");
        assert_eq!(stream_mark(&a, "you").0, 0);
        assert_eq!(stream_mark(&a, "editor").0, 2);
        // The answer to ask1 sits right under it: no quote; the question quotes the brief.
        assert!(stream_quote(&m, 3).is_none());
        assert_eq!(stream_quote(&m, 6).map(|q| q.id.as_str()), Some("brief"));
        assert!(stream_needs_you(&m, &m[6].clone()));
        let mut r = m[0].clone();
        r.id = "ans".into();
        r.reply_to = Some("q".into());
        m.push(r);
        assert!(!stream_needs_you(&m, &m[6].clone()));
        assert!(stream_involves(&m[2], "analyst"));
        assert!(!stream_involves(&m[5], "analyst"));
        assert_eq!(stream_mentions(&a, "ed").len(), 1);
        assert_eq!(stream_mentions(&a, "").len(), 4);
        assert_eq!(stream_at_query("hi @Bu").as_deref(), Some("Bu"));
        assert_eq!(stream_at_query("a@b"), None);
        let s = stream_segments("Notes are in Notion and Stripe.");
        assert_eq!(s.len(), 5);
        assert_eq!(s[1], ("Notion".to_string(), "notion"));
    }
}
