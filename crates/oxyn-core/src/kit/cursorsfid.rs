//! affectUI fidelity pass (plan #32, overnight batch, family B part 3): the model behind the exact
//! **Presence cursors** – agents working in a document like teammates in a multiplayer editor: live
//! pointers with a flag (name, cost, what it does), selections, suggestions typed letter by
//! letter, an image that loads and gets resized, code fixes, comment pins and a table view.
//! Behaviour only; offsets are character indexes into a block's current text and time is passed
//! in as elapsed milliseconds.

/// What a block of the document is.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BlockKind {
    /// The page title (h2).
    Title,
    /// A section heading (h3).
    Heading,
    /// A paragraph.
    Paragraph,
    /// A list item; consecutive items form one list.
    Item,
    /// A code block (monospace, keeps its line breaks).
    Code,
    /// An image; its text is the caption.
    Image,
    /// A table; `rows` hold cell blocks, the first row is the header.
    Table,
    /// A table cell; with a `tone` it shows as a status pill.
    Cell,
}

/// One block of the document.
#[derive(Clone, Debug, PartialEq)]
pub struct DocBlock {
    /// Id the cursors, comments and suggestions point at.
    pub id: String,
    /// Kind.
    pub kind: BlockKind,
    /// The text (an image's caption).
    pub text: String,
    /// Image: share of the page width (0–1).
    pub width: f32,
    /// Image: width / height.
    pub ratio: f32,
    /// Image: `left`, `right` or empty – the next text runs beside it.
    pub float: String,
    /// Image: still loading (grey with a shimmer).
    pub loading: bool,
    /// Table: rows of cells.
    pub rows: Vec<Vec<DocBlock>>,
    /// Cell: a status tone (`blue`, `grey`, `amber`, `green`, `red`) – shows as a pill.
    pub tone: String,
}

impl DocBlock {
    /// A block of `kind` with `text`.
    pub fn new(id: &str, kind: BlockKind, text: &str) -> Self {
        Self {
            id: id.into(),
            kind,
            text: text.into(),
            width: 0.4,
            ratio: 2.5,
            float: String::new(),
            loading: false,
            rows: vec![],
            tone: String::new(),
        }
    }
    /// A table cell, with a status tone or none.
    pub fn cell(id: &str, text: &str, tone: &str) -> Self {
        let mut b = Self::new(id, BlockKind::Cell, text);
        b.tone = tone.into();
        b
    }
    /// This block or a cell inside it.
    pub fn find(&self, id: &str) -> Option<&DocBlock> {
        if self.id == id {
            return Some(self);
        }
        self.rows.iter().flatten().find_map(|c| c.find(id))
    }
    fn find_mut(&mut self, id: &str) -> Option<&mut DocBlock> {
        if self.id == id {
            return Some(self);
        }
        self.rows.iter_mut().flatten().find_map(|c| c.find_mut(id))
    }
}

/// Finds a block (or a table cell) by id.
pub fn find_block<'a>(blocks: &'a [DocBlock], id: &str) -> Option<&'a DocBlock> {
    blocks.iter().find_map(|b| b.find(id))
}

/// Someone who can be in the document.
#[derive(Clone, Debug, PartialEq)]
pub struct CursorAgent {
    /// Id.
    pub id: String,
    /// Name on its flag and face.
    pub name: String,
    /// Brand mark on its face (a model maker: `claude`, `gemini`, `mistral`, `openai`, …).
    pub brand: String,
    /// Its colour (CSS colour); empty = the shared ink, the marks tell agents apart.
    pub color: String,
    /// The fill of its face while you follow it; empty = a light wash of its colour.
    pub tint: String,
    /// What it has cost so far (dollars).
    pub cost: f64,
}

impl CursorAgent {
    /// An agent with a brand mark and the shared ink.
    pub fn new(id: &str, name: &str, brand: &str) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            brand: brand.into(),
            color: String::new(),
            tint: String::new(),
            cost: 0.0,
        }
    }
}

/// Where on an image a pointer sits.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ImagePoint {
    /// Its bottom-right handle (holding it).
    Corner,
    /// Its middle.
    Middle,
}

/// Where an agent is.
#[derive(Clone, Debug, PartialEq)]
pub struct AgentCursor {
    /// The block (or cell) it is in.
    pub block: String,
    /// Character offset of its caret; with `in_sugg`, an offset into that suggestion's insert.
    pub at: Option<usize>,
    /// A suggestion it is typing into.
    pub in_sugg: String,
    /// A selected passage (start, end) – or, on an image, that it holds the image.
    pub select: Option<(usize, usize)>,
    /// On an image: the corner or the middle.
    pub point: Option<ImagePoint>,
    /// What it is doing (on its flag).
    pub doing: String,
    /// Typing: the caret holds still and the pointer follows every letter.
    pub typing: bool,
}

impl AgentCursor {
    /// At offset `at` of `block`, doing `doing`.
    pub fn at(block: &str, at: usize, doing: &str) -> Self {
        Self {
            block: block.into(),
            at: Some(at),
            in_sugg: String::new(),
            select: None,
            point: None,
            doing: doing.into(),
            typing: false,
        }
    }
}

/// A comment pin on the page.
#[derive(Clone, Debug, PartialEq)]
pub struct DocComment {
    /// Id.
    pub id: String,
    /// The agent that wrote it.
    pub agent: String,
    /// The block it sits beside.
    pub block: String,
    /// Along the page (0–1) at the block's top edge.
    pub x: f32,
    /// Down from there (px).
    pub y: f32,
    /// The comment.
    pub text: String,
    /// Window kept open (while its agent writes); otherwise a round pin that grows on hover.
    pub open: bool,
}

/// A change waiting for you.
#[derive(Clone, Debug, PartialEq)]
pub struct DocSuggestion {
    /// Id.
    pub id: String,
    /// The agent suggesting it.
    pub agent: String,
    /// The block (or cell).
    pub block: String,
    /// Character offset where it starts.
    pub at: usize,
    /// Text it removes from `at` (struck through).
    pub remove: String,
    /// Text it inserts after that (underlined).
    pub insert: String,
    /// Why (in its window and the announcement).
    pub note: String,
}

fn char_len(s: &str) -> usize {
    s.chars().count()
}

fn byte_at(s: &str, ci: usize) -> usize {
    s.char_indices().nth(ci).map(|x| x.0).unwrap_or(s.len())
}

/// Writes a suggestion into the blocks and moves the other suggestions in that block along.
pub fn accept_suggestion(
    blocks: &[DocBlock],
    suggestions: &[DocSuggestion],
    id: &str,
) -> (Vec<DocBlock>, Vec<DocSuggestion>) {
    let mut out = blocks.to_vec();
    let Some(s) = suggestions.iter().find(|s| s.id == id) else {
        return (out, suggestions.to_vec());
    };
    let rlen = char_len(&s.remove);
    let delta = char_len(&s.insert) as isize - rlen as isize;
    for b in out.iter_mut() {
        if let Some(t) = b.find_mut(&s.block) {
            let a = byte_at(&t.text, s.at);
            let e = byte_at(&t.text, s.at + rlen);
            t.text = format!("{}{}{}", &t.text[..a], s.insert, &t.text[e..]);
            break;
        }
    }
    let rest = suggestions
        .iter()
        .filter(|x| x.id != id)
        .cloned()
        .map(|mut x| {
            if x.block == s.block && x.at > s.at {
                x.at = (x.at as isize + delta).max(0) as usize;
            }
            x
        })
        .collect();
    (out, rest)
}

/// One run of a block's text as it is drawn.
#[derive(Clone, Debug, PartialEq, Default)]
pub struct Seg {
    /// The text (empty for a caret).
    pub text: String,
    /// Selected by this agent (index).
    pub sel: Option<usize>,
    /// Struck through by this suggestion (index).
    pub del: Option<usize>,
    /// Inserted by this suggestion (index).
    pub ins: Option<usize>,
    /// A caret of this agent (index) – where its pointer is anchored.
    pub caret: Option<usize>,
}

/// Splits a block's text into runs: selections, struck and inserted suggestion text, and the
/// carets of the agents in it. `cursors[i]` belongs to agent `i`.
pub fn segments(
    block: &str,
    text: &str,
    cursors: &[Option<AgentCursor>],
    suggestions: &[DocSuggestion],
) -> Vec<Seg> {
    let n = char_len(text);
    let chars: Vec<char> = text.chars().collect();
    let sugs: Vec<(usize, &DocSuggestion)> = suggestions
        .iter()
        .enumerate()
        .filter(|(_, s)| s.block == block)
        .collect();
    let mut cuts = vec![0, n];
    for c in cursors.iter().flatten().filter(|c| c.block == block) {
        if let Some((a, b)) = c.select {
            cuts.push(a.min(n));
            cuts.push(b.min(n));
        }
        if c.in_sugg.is_empty()
            && let Some(a) = c.at
        {
            cuts.push(a.min(n));
        }
    }
    for (_, s) in &sugs {
        cuts.push(s.at.min(n));
        cuts.push((s.at + char_len(&s.remove)).min(n));
    }
    cuts.sort_unstable();
    cuts.dedup();
    let mut out: Vec<Seg> = vec![];
    for (k, &p) in cuts.iter().enumerate() {
        // Things anchored at p: carets first, then inserts that end a removal here.
        for (i, c) in cursors.iter().enumerate() {
            if let Some(c) = c
                && c.block == block
                && c.in_sugg.is_empty()
                && c.at.map(|a| a.min(n)) == Some(p)
            {
                out.push(Seg {
                    caret: Some(i),
                    ..Seg::default()
                });
            }
        }
        for (si, s) in &sugs {
            if (s.at + char_len(&s.remove)).min(n) != p {
                continue;
            }
            // A cursor typing into this insert splits it at its caret.
            let typer = cursors.iter().enumerate().find_map(|(i, c)| {
                c.as_ref()
                    .filter(|c| c.in_sugg == s.id)
                    .map(|c| (i, c.at.unwrap_or(char_len(&s.insert))))
            });
            let ins: Vec<char> = s.insert.chars().collect();
            match typer {
                Some((ai, at)) => {
                    let at = at.min(ins.len());
                    if at > 0 {
                        out.push(Seg {
                            text: ins[..at].iter().collect(),
                            ins: Some(*si),
                            ..Seg::default()
                        });
                    }
                    out.push(Seg {
                        caret: Some(ai),
                        ins: Some(*si),
                        ..Seg::default()
                    });
                    if at < ins.len() {
                        out.push(Seg {
                            text: ins[at..].iter().collect(),
                            ins: Some(*si),
                            ..Seg::default()
                        });
                    }
                }
                None => {
                    if !ins.is_empty() {
                        out.push(Seg {
                            text: s.insert.clone(),
                            ins: Some(*si),
                            ..Seg::default()
                        });
                    }
                }
            }
        }
        let Some(&q) = cuts.get(k + 1) else { break };
        if q <= p {
            continue;
        }
        let sel = cursors.iter().enumerate().find_map(|(i, c)| {
            c.as_ref()
                .filter(|c| c.block == block)
                .and_then(|c| c.select)
                .filter(|(a, b)| *a <= p && q <= *b)
                .map(|_| i)
        });
        let del = sugs
            .iter()
            .find(|(_, s)| s.at <= p && q <= s.at + char_len(&s.remove))
            .map(|(i, _)| *i);
        out.push(Seg {
            text: chars[p..q].iter().collect(),
            sel,
            del,
            ..Seg::default()
        });
    }
    out
}

/// Formats a cost the affectUI way: `$0.42`.
pub fn format_cost(v: f64) -> String {
    format!("${:.2}", v.max(0.0))
}

/// "3 suggestions" while there are any, otherwise "4 here".
pub fn presence_label(suggestions: usize, here: usize) -> String {
    if suggestions > 0 {
        format!(
            "{suggestions} suggestion{}",
            if suggestions == 1 { "" } else { "s" }
        )
    } else {
        format!("{here} here")
    }
}

/// "Researcher suggests removing: about 40%" / "…adding: 38%" – a suggestion's label.
pub fn suggestion_label(agent: &str, s: &DocSuggestion, removing: bool) -> String {
    if removing {
        format!("{agent} suggests removing: {}", s.remove)
    } else {
        format!("{agent} suggests adding: {}", s.insert)
    }
}

/// The face's toggle label: "Follow Coder – Reading the config", "Stop following …", or
/// "Follow Coder (not in the document)".
pub fn face_label(name: &str, doing: Option<&str>, following: bool) -> String {
    let verb = if following {
        "Stop following"
    } else {
        "Follow"
    };
    match doing {
        Some(d) if !d.is_empty() => format!("{verb} {name} – {d}"),
        Some(_) => format!("{verb} {name}"),
        None => format!("{verb} {name} (not in the document)"),
    }
}

/// The whole document at one moment.
#[derive(Clone, Debug, PartialEq)]
pub struct CursorScene {
    /// The document's name in the header.
    pub title: String,
    /// The blocks.
    pub blocks: Vec<DocBlock>,
    /// Everyone who can be in it.
    pub agents: Vec<CursorAgent>,
    /// Per agent (same order): where it is, or `None` when it left.
    pub cursors: Vec<Option<AgentCursor>>,
    /// Comment pins.
    pub comments: Vec<DocComment>,
    /// Suggestions waiting.
    pub suggestions: Vec<DocSuggestion>,
    /// The latest thing to announce ("Researcher is writing a comment").
    pub status: String,
}

/// The first `per`-ms letters of `s` typed since `t0`.
fn typed(s: &str, t0: u64, t: u64, per: u64) -> String {
    let k = (t.saturating_sub(t0) / per.max(1)) as usize;
    s.chars().take(k).collect()
}

/// A reading caret moving along a block's text.
fn reading(text: &str, t0: u64, t: u64, per: u64, until: usize) -> usize {
    ((t.saturating_sub(t0) / per.max(1)) as usize).min(until.min(char_len(text)))
}

fn find_in(text: &str, needle: &str) -> usize {
    text.find(needle)
        .map(|b| text[..b].chars().count())
        .unwrap_or(0)
}

/// Cost: a base plus a rate while the agent has been in the document.
fn cost(t: u64, from: u64, to: u64, base: f64, per_s: f64) -> f64 {
    if t < from {
        return 0.0;
    }
    base + (t.min(to) - from) as f64 / 1000.0 * per_s
}

const DOC_TITLE: &str = "Atlas 4 – launch outline";
const DOC_P1: &str = "Teams spend close to 30% of each week chasing updates. Atlas gathers that work into one morning summary, so nobody has to compile it.";
const DOC_P2: &str = "The hero shot is the summary itself: a single quiet page a manager opens first thing. Brief, up to date and drawn from real work, so the week begins without a check-in call.";
const DOC_CODE: &str = "const timeline = {\n  beta: \"Mar 2\",\n  general: \"mid-March\",\n};";
const DOC_P3: &str = "Launch week: one blog post, a walkthrough clip and a note to the beta list.";

fn doc_blocks(img_w: f32, loading: bool) -> Vec<DocBlock> {
    let mut img = DocBlock::new("img", BlockKind::Image, "Hero shot");
    img.width = img_w;
    img.ratio = 2.55;
    img.float = "left".into();
    img.loading = loading;
    vec![
        DocBlock::new("title", BlockKind::Title, DOC_TITLE),
        DocBlock::new("p1", BlockKind::Paragraph, DOC_P1),
        img,
        DocBlock::new("p2", BlockKind::Paragraph, DOC_P2),
        DocBlock::new("h1", BlockKind::Heading, "Timeline"),
        DocBlock::new("code", BlockKind::Code, DOC_CODE),
        DocBlock::new("p3", BlockKind::Paragraph, DOC_P3),
        DocBlock::new("h2", BlockKind::Heading, "In the release"),
        DocBlock::new(
            "li1",
            BlockKind::Item,
            "A morning summary for every team, drawn from the work itself",
        ),
        DocBlock::new(
            "li2",
            BlockKind::Item,
            "Reminders when a thread has gone silent for days",
        ),
    ]
}

fn doc_agents() -> Vec<CursorAgent> {
    vec![
        CursorAgent::new("researcher", "Researcher", "claude"),
        CursorAgent::new("designer", "Designer", "gemini"),
        CursorAgent::new("coder", "Coder", "mistral"),
        CursorAgent::new("writer", "Writer", "openai"),
    ]
}

fn table_agents() -> Vec<CursorAgent> {
    vec![
        CursorAgent::new("planner", "Planner", "claude"),
        CursorAgent::new("analyst", "Analyst", "gemini"),
        CursorAgent::new("ops", "Ops", "mistral"),
        CursorAgent::new("reviewer", "Reviewer", "openai"),
    ]
}

/// How long one pass of a demo tab lasts before it starts over (ms).
pub fn cursors_loop_ms(tab: &str) -> u64 {
    if tab == "table" { 21_000 } else { 24_000 }
}

/// The demo tabs: (id, label).
pub fn cursor_tabs() -> Vec<(String, String)> {
    vec![
        ("doc".into(), "Doc".into()),
        ("table".into(), "Table".into()),
    ]
}

/// The demo document (`doc` or `table` tab) `t` ms into a pass.
pub fn cursors_at(tab: &str, t: u64) -> CursorScene {
    if tab == "table" {
        table_at(t)
    } else {
        doc_at(t)
    }
}

fn doc_at(t: u64) -> CursorScene {
    let mut status = String::new();
    let mut suggestions = vec![];
    let mut comments = vec![];
    let mut cursors: Vec<Option<AgentCursor>> = vec![None; 4];

    // Writer: reads the title, picks a sharper word, types it.
    let w_sel = find_in(DOC_TITLE, "outline");
    let w_end = char_len(DOC_TITLE);
    if t >= 1_200 {
        cursors[3] = Some(if t < 3_000 {
            AgentCursor::at(
                "title",
                reading(DOC_TITLE, 1_200, t, 60, w_end),
                "Reading the outline",
            )
        } else if t < 4_200 {
            let mut c = AgentCursor::at("title", w_end, "Finding a sharper word");
            c.select = Some((w_sel, w_end));
            c
        } else {
            let ins = typed("schedule", 4_200, t, 130);
            let mut c = AgentCursor::at(
                "title",
                char_len(&ins),
                if t < 5_400 { "Renaming" } else { "Done" },
            );
            c.in_sugg = "s-writer".into();
            c.typing = t < 5_400;
            c
        });
    }
    if t >= 4_200 {
        suggestions.push(DocSuggestion {
            id: "s-writer".into(),
            agent: "writer".into(),
            block: "title".into(),
            at: w_sel,
            remove: "outline".into(),
            insert: typed("schedule", 4_200, t, 130),
            note: "It has dates in it now – it reads as a schedule".into(),
        });
        if t >= 5_400 {
            status =
                "Writer suggests a change: It has dates in it now – it reads as a schedule".into();
        }
    }

    // Researcher: reads the intro, checks the figure, leaves a note, fixes it.
    let r_a = find_in(DOC_P1, "close to 30%");
    let r_b = r_a + char_len("close to 30%");
    if (600..9_600).contains(&t) {
        cursors[0] = Some(if t < 2_000 {
            AgentCursor::at("p1", reading(DOC_P1, 600, t, 45, r_a), "Skimming the intro")
        } else if t < 6_500 {
            let mut c = AgentCursor::at(
                "p1",
                r_b,
                if t < 4_000 {
                    "Checking the figure"
                } else {
                    "Writing a note"
                },
            );
            c.select = Some((r_a, r_b));
            c
        } else {
            let ins = typed("27%", 6_500, t, 160);
            let mut c = AgentCursor::at(
                "p1",
                char_len(&ins),
                if t < 7_600 {
                    "Fixing the figure"
                } else {
                    "Done"
                },
            );
            c.in_sugg = "s-researcher".into();
            c.typing = t < 7_600;
            c
        });
    }
    if t >= 4_000 {
        let text = "Spring poll of 860 teams: 27%, not 30%.";
        comments.push(DocComment {
            id: "c-researcher".into(),
            agent: "researcher".into(),
            block: "p1".into(),
            x: 0.0,
            y: 0.0,
            text: if t < 6_500 {
                typed(text, 4_000, t, 45)
            } else {
                text.into()
            },
            open: t < 6_500,
        });
        if t < 6_500 && status.is_empty() {
            status = "Researcher is writing a comment".into();
        }
    }
    if t >= 6_500 {
        suggestions.push(DocSuggestion {
            id: "s-researcher".into(),
            agent: "researcher".into(),
            block: "p1".into(),
            at: r_a,
            remove: "close to 30%".into(),
            insert: typed("27%", 6_500, t, 160),
            note: "The spring poll says 27%".into(),
        });
    }

    // Designer: waits for the hero to render, then resizes it.
    let loading = t < 5_200;
    let img_w = if t < 6_000 {
        0.40
    } else if t < 8_600 {
        0.40 + 0.10 * ((t - 6_000) as f32 / 2_600.0)
    } else {
        0.50
    };
    if (2_500..14_000).contains(&t) {
        let (doing, point, hold) = if t < 5_200 {
            ("Waiting for the render", ImagePoint::Middle, false)
        } else if t < 6_000 {
            ("Looks right", ImagePoint::Middle, false)
        } else if t < 9_000 {
            ("Resizing the hero", ImagePoint::Corner, true)
        } else {
            ("Looks right", ImagePoint::Middle, false)
        };
        let mut c = AgentCursor::at("img", 0, doing);
        c.at = None;
        c.point = Some(point);
        c.select = hold.then_some((0, 0));
        cursors[1] = Some(c);
    }

    // Coder: reads the config, fixes the date, asks about the format, checks the rest.
    let c_a = find_in(DOC_CODE, "mid-March");
    let c_b = c_a + char_len("mid-March");
    if (8_000..20_000).contains(&t) {
        cursors[2] = Some(if t < 9_500 {
            AgentCursor::at(
                "code",
                c_a.min(reading(DOC_CODE, 8_000, t, 30, c_a)),
                "Reading the config",
            )
        } else if t < 10_500 {
            let mut c = AgentCursor::at("code", c_b, "Fixing the date");
            c.select = Some((c_a, c_b));
            c
        } else if t < 14_000 {
            let ins = typed("Mar 16", 10_500, t, 110);
            let mut c = AgentCursor::at(
                "code",
                char_len(&ins),
                if t < 11_500 {
                    "Fixing the date"
                } else {
                    "Leaving a note"
                },
            );
            c.in_sugg = "s-coder".into();
            c.typing = t < 11_200;
            c
        } else {
            AgentCursor::at(
                "p3",
                reading(DOC_P3, 14_000, t, 55, char_len(DOC_P3)),
                if t < 18_500 {
                    "Checking the rest"
                } else {
                    "Done"
                },
            )
        });
    }
    if t >= 10_500 {
        suggestions.push(DocSuggestion {
            id: "s-coder".into(),
            agent: "coder".into(),
            block: "code".into(),
            at: c_a,
            remove: "mid-March".into(),
            insert: typed("Mar 16", 10_500, t, 110),
            note: "Same style as the beta date".into(),
        });
    }
    if t >= 11_500 {
        let text = "Can beta and general use one date style? I'd write \"Mar 2\" for both.";
        comments.push(DocComment {
            id: "c-coder".into(),
            agent: "coder".into(),
            block: "code".into(),
            x: 0.0,
            y: 0.0,
            text: if t < 14_000 {
                typed(text, 11_500, t, 32)
            } else {
                text.into()
            },
            open: t < 14_000,
        });
        if t < 14_000 {
            status = "Coder is writing a comment".into();
        }
    }
    // Suggestions in document order.
    let order = ["title", "p1", "code"];
    suggestions.sort_by_key(|s| order.iter().position(|b| *b == s.block).unwrap_or(9));

    let mut agents = doc_agents();
    agents[0].cost = cost(t, 600, 9_600, 0.04, 0.026);
    agents[1].cost = cost(t, 2_500, 14_000, 0.03, 0.019);
    agents[2].cost = cost(t, 8_000, 20_000, 0.02, 0.021);
    agents[3].cost = cost(t, 1_200, 24_000, 0.02, 0.009);
    CursorScene {
        title: "Marketing / Releases".into(),
        blocks: doc_blocks(img_w, loading),
        agents,
        cursors,
        comments,
        suggestions,
        status,
    }
}

const ROWS: [[(&str, &str); 4]; 4] = [
    [
        ("Draft the blog post", ""),
        ("Writer", ""),
        ("Mar 2", ""),
        ("Doing", "blue"),
    ],
    [
        ("Film the walkthrough", ""),
        ("Designer", ""),
        ("Mar 4", ""),
        ("To do", "grey"),
    ],
    [
        ("Mail the beta list", ""),
        ("Ops", ""),
        ("Mar 11", ""),
        ("To do", "grey"),
    ],
    [
        ("Open office hours", ""),
        ("—", ""),
        ("Mar 13", ""),
        ("Idea", "amber"),
    ],
];

fn table_at(t: u64) -> CursorScene {
    let mut rows = vec![vec![
        DocBlock::cell("r0c0", "Task", ""),
        DocBlock::cell("r0c1", "Owner", ""),
        DocBlock::cell("r0c2", "Due", ""),
        DocBlock::cell("r0c3", "Status", ""),
    ]];
    for (r, row) in ROWS.iter().enumerate() {
        rows.push(
            row.iter()
                .enumerate()
                .map(|(c, (txt, tone))| DocBlock::cell(&format!("r{}c{c}", r + 1), txt, tone))
                .collect(),
        );
    }
    if t >= 6_500 {
        let task = if t < 8_500 {
            typed("Press page", 6_500, t, 150)
        } else {
            "Press page".into()
        };
        let on = t >= 8_500;
        rows.push(vec![
            DocBlock::cell("r5c0", &task, ""),
            DocBlock::cell("r5c1", if on { "Designer" } else { "" }, ""),
            DocBlock::cell("r5c2", if on { "Mar 6" } else { "" }, ""),
            DocBlock::cell(
                "r5c3",
                if on { "To do" } else { "" },
                if on { "grey" } else { "" },
            ),
        ]);
    }
    let mut table = DocBlock::new("tasks", BlockKind::Table, "");
    table.rows = rows;
    let blocks = vec![
        DocBlock::new("title", BlockKind::Title, "Atlas 4 – launch checklist"),
        table,
    ];

    let mut status = String::new();
    let mut suggestions = vec![];
    let mut comments = vec![];
    let mut cursors: Vec<Option<AgentCursor>> = vec![None; 4];

    // Planner: reads the tasks, then moves a date.
    if (500..9_000).contains(&t) {
        cursors[0] = Some(if t < 3_000 {
            AgentCursor::at(
                "r1c0",
                reading("Draft the blog post", 500, t, 120, 19),
                "Reading the tasks",
            )
        } else if t < 4_500 {
            let mut c = AgentCursor::at("r3c2", 6, "Checking the dates");
            c.select = Some((0, 6));
            c
        } else {
            let ins = typed("Mar 9", 4_500, t, 120);
            let mut c = AgentCursor::at(
                "r3c2",
                char_len(&ins),
                if t < 5_200 {
                    "Checking the dates"
                } else {
                    "Done"
                },
            );
            c.in_sugg = "s-planner".into();
            c.typing = t < 5_200;
            c
        });
    }
    if t >= 4_500 {
        suggestions.push(DocSuggestion {
            id: "s-planner".into(),
            agent: "planner".into(),
            block: "r3c2".into(),
            at: 0,
            remove: "Mar 11".into(),
            insert: typed("Mar 9", 4_500, t, 120),
            note: "Two days before office hours".into(),
        });
    }
    // Ops: moves a status along.
    if (1_500..12_000).contains(&t) {
        cursors[2] = Some(if t < 3_500 {
            let mut c = AgentCursor::at("r2c3", 5, "Updating the status");
            c.select = Some((0, 5));
            c
        } else {
            let ins = typed("Doing", 3_500, t, 130);
            let mut c = AgentCursor::at(
                "r2c3",
                char_len(&ins),
                if t < 5_000 {
                    "Updating the status"
                } else {
                    "Done"
                },
            );
            c.in_sugg = "s-ops".into();
            c.typing = t < 4_200;
            c
        });
    }
    if t >= 3_500 {
        suggestions.push(DocSuggestion {
            id: "s-ops".into(),
            agent: "ops".into(),
            block: "r2c3".into(),
            at: 0,
            remove: "To do".into(),
            insert: typed("Doing", 3_500, t, 130),
            note: "Filming started this morning".into(),
        });
    }
    // Analyst: finds an owner.
    if (2_500..16_000).contains(&t) {
        cursors[1] = Some(if t < 4_500 {
            let mut c = AgentCursor::at("r4c1", 1, "Finding an owner");
            c.select = Some((0, 1));
            c
        } else {
            let ins = typed("Sam", 4_500, t, 150);
            let mut c = AgentCursor::at(
                "r4c1",
                char_len(&ins),
                if t < 6_000 {
                    "Finding an owner"
                } else {
                    "Done"
                },
            );
            c.in_sugg = "s-analyst".into();
            c.typing = t < 5_000;
            c
        });
    }
    if t >= 4_500 {
        suggestions.push(DocSuggestion {
            id: "s-analyst".into(),
            agent: "analyst".into(),
            block: "r4c1".into(),
            at: 0,
            remove: "—".into(),
            insert: typed("Sam", 4_500, t, 150),
            note: "Sam ran the last two sessions".into(),
        });
    }
    // Reviewer: adds a task, then says the draft is ready.
    if (6_500..13_000).contains(&t) {
        cursors[3] = Some(if t < 8_500 {
            let mut c = AgentCursor::at(
                "r5c0",
                char_len(&typed("Press page", 6_500, t, 150)),
                "Adding a task",
            );
            c.typing = true;
            c
        } else if t < 9_000 {
            AgentCursor::at("r1c3", 5, "Reading on")
        } else {
            AgentCursor::at(
                "r1c3",
                5,
                if t < 11_500 { "Writing a note" } else { "Done" },
            )
        });
    }
    if t >= 9_000 {
        let text = "First draft is up – ready for a look today.";
        comments.push(DocComment {
            id: "c-reviewer".into(),
            agent: "reviewer".into(),
            block: "r1c0".into(),
            x: 0.0,
            y: 0.0,
            text: if t < 11_500 {
                typed(text, 9_000, t, 50)
            } else {
                text.into()
            },
            open: t < 11_500,
        });
        if t < 11_500 {
            status = "Reviewer is writing a comment".into();
        }
    }
    if status.is_empty() && t >= 3_500 {
        status = "Ops suggests a change: Filming started this morning".into();
    }
    let order = ["r2c3", "r3c2", "r4c1"];
    suggestions.sort_by_key(|s| order.iter().position(|b| *b == s.block).unwrap_or(9));

    let mut agents = table_agents();
    agents[0].cost = cost(t, 500, 9_000, 0.03, 0.017);
    agents[1].cost = cost(t, 2_500, 16_000, 0.02, 0.012);
    agents[2].cost = cost(t, 1_500, 12_000, 0.02, 0.014);
    agents[3].cost = cost(t, 6_500, 13_000, 0.03, 0.024);
    CursorScene {
        title: "Marketing / Releases".into(),
        blocks,
        agents,
        cursors,
        comments,
        suggestions,
        status,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accept_moves_the_rest_along() {
        let blocks = vec![DocBlock::new("p", BlockKind::Paragraph, "one two three")];
        let s = |id: &str, at, r: &str, i: &str| DocSuggestion {
            id: id.into(),
            agent: "a".into(),
            block: "p".into(),
            at,
            remove: r.into(),
            insert: i.into(),
            note: String::new(),
        };
        let sugs = vec![s("a", 0, "one", "uno"), s("b", 8, "three", "3")];
        let (b, rest) = accept_suggestion(&blocks, &sugs, "a");
        assert_eq!(b[0].text, "uno two three");
        assert_eq!(rest.len(), 1);
        assert_eq!(rest[0].at, 8);
        let (b, rest) = accept_suggestion(&b, &rest, "b");
        assert_eq!(b[0].text, "uno two 3");
        assert!(rest.is_empty());
    }

    #[test]
    fn segments_split_on_suggestions_and_carets() {
        let text = "Atlas – launch outline";
        let mut cur = AgentCursor::at("t", 3, "Renaming");
        cur.in_sugg = "s".into();
        let sugs = vec![DocSuggestion {
            id: "s".into(),
            agent: "w".into(),
            block: "t".into(),
            at: find_in(text, "outline"),
            remove: "outline".into(),
            insert: "schedule".into(),
            note: String::new(),
        }];
        let segs = segments("t", text, &[Some(cur)], &sugs);
        let shown: Vec<String> = segs.iter().map(|s| s.text.clone()).collect();
        assert_eq!(
            shown,
            vec!["Atlas – launch ", "outline", "sch", "", "edule"]
        );
        assert_eq!(segs[1].del, Some(0));
        assert_eq!(segs[3].caret, Some(0));
        assert_eq!(segs[4].ins, Some(0));
    }

    #[test]
    fn the_demo_settles_with_three_suggestions() {
        let s = cursors_at("doc", 16_000);
        assert_eq!(s.suggestions.len(), 3);
        assert_eq!(s.suggestions[0].insert, "schedule");
        assert_eq!(presence_label(s.suggestions.len(), 3), "3 suggestions");
        assert_eq!(format_cost(0.4199), "$0.42");
        let t = cursors_at("table", 12_000);
        assert_eq!(t.blocks[1].rows.len(), 6);
        assert_eq!(t.suggestions.len(), 3);
        assert_eq!(
            face_label("Coder", None, false),
            "Follow Coder (not in the document)"
        );
    }
}
