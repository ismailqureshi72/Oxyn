//! Context Visualizer family (variants 2): what fills a model's context window, shared by the
//! Context bar, Context breakdown, Context stack and Context tide. Behaviour only: the kinds,
//! token counts and every line of copy are our own.
//!
//! A window is a list of [`CtxPart`]s (one per kind, in the order they load) against a `limit` in
//! tokens. Helpers turn that into bar shares, a field of dots, slab heights, a fill level and the
//! number of turns left before an automatic compaction.

/// The kinds a context window is made of, in load order.
pub const CTX_KINDS: [&str; 6] = ["system", "tools", "memory", "files", "chat", "summary"];

/// One kind's share of the window.
#[derive(Clone, Debug, PartialEq)]
pub struct CtxPart {
    /// `system`, `tools`, `memory`, `files`, `chat` or `summary`.
    pub kind: &'static str,
    /// Display label.
    pub label: String,
    /// Tokens.
    pub tokens: u32,
    /// What it holds: (name, tokens), largest first.
    pub items: Vec<(String, u32)>,
}

impl CtxPart {
    fn new(kind: &'static str, label: &str, items: &[(&str, u32)]) -> Self {
        let mut items: Vec<(String, u32)> = items.iter().map(|(n, t)| ((*n).into(), *t)).collect();
        items.sort_by_key(|i| std::cmp::Reverse(i.1));
        Self {
            kind,
            label: label.into(),
            tokens: items.iter().map(|i| i.1).sum(),
            items,
        }
    }
}

/// The demo window: 166.4K of 200K, chat the largest part.
pub fn ctx_demo() -> Vec<CtxPart> {
    vec![
        CtxPart::new(
            "chat",
            "Chat",
            &[
                ("Turns 1–12", 41_200),
                ("Turns 13–24", 33_800),
                ("Turns 25–31", 21_400),
            ],
        ),
        CtxPart::new(
            "files",
            "Files",
            &[
                ("billing/invoice.rs", 14_600),
                ("api/routes.ts", 11_200),
                ("README.md", 6_900),
                ("schema.sql", 5_500),
            ],
        ),
        CtxPart::new(
            "tools",
            "Tools",
            &[
                ("Shell", 6_400),
                ("Edit file", 5_100),
                ("Web search", 4_200),
                ("Browser", 3_200),
            ],
        ),
        CtxPart::new(
            "system",
            "System",
            &[("Instructions", 5_800), ("Safety rules", 1_800)],
        ),
        CtxPart::new(
            "memory",
            "Memory",
            &[("Project notes", 3_100), ("Your preferences", 2_200)],
        ),
    ]
}

/// The kind's colour (`RRGGBB`) — category colour, the one place the family uses hue.
pub fn kind_hex(kind: &str) -> &'static str {
    match kind {
        "chat" => "22B3C4",
        "files" => "3B7DF6",
        "tools" => "9D63F2",
        "system" => "EC4B7E",
        "memory" => "2DB873",
        "summary" => "E3A008",
        _ => "8A8A93",
    }
}

/// The kind's grey in the monochrome look (`RRGGBB`), lightest for the steadiest part; `dark`
/// picks the ramp for a dark card.
pub fn kind_mono(kind: &str, dark: bool) -> &'static str {
    let i = match kind {
        "system" => 0,
        "tools" => 1,
        "memory" => 2,
        "chat" => 3,
        "files" => 4,
        _ => 5,
    };
    let dark_ramp = ["F2F2F3", "C9C9CE", "A3A3AA", "86868E", "6A6A72", "55555C"];
    let light_ramp = ["18181B", "3F3F46", "5E5E66", "7C7C84", "9A9AA1", "B4B4BA"];
    if dark { dark_ramp[i] } else { light_ramp[i] }
}

/// Tokens in all parts.
pub fn ctx_used(parts: &[CtxPart]) -> u32 {
    parts.iter().map(|p| p.tokens).sum()
}

/// Tokens as a short figure: `980`, `7.6K`, `175.6K`, `200K`, `1M`.
pub fn fmt_tokens(t: u32) -> String {
    if t < 1_000 {
        return t.to_string();
    }
    if t >= 1_000_000 {
        let m = t as f64 / 1_000_000.0;
        return if (m - m.round()).abs() < 0.05 {
            format!("{}M", m.round() as u32)
        } else {
            format!("{m:.1}M")
        };
    }
    let k = (t as f64 / 100.0).round() / 10.0;
    if (k - k.round()).abs() < 0.05 {
        format!("{}K", k.round() as u32)
    } else {
        format!("{k:.1}K")
    }
}

/// Whole percent of `limit` used (0 when the limit is 0), capped at 100.
pub fn ctx_pct(used: u32, limit: u32) -> u32 {
    if limit == 0 {
        0
    } else {
        ((used as f64 * 100.0 / limit as f64).round() as u32).min(100)
    }
}

/// How full the window feels: `calm` under 75 %, `warm` from 75 %, `hot` from `auto` %.
pub fn fill_level(used: u32, limit: u32, auto: u32) -> &'static str {
    let p = ctx_pct(used, limit);
    if p >= auto {
        "hot"
    } else if p >= 75 {
        "warm"
    } else {
        "calm"
    }
}

/// Each part's share of the limit in percent (free space is what is left).
pub fn ctx_shares(parts: &[CtxPart], limit: u32) -> Vec<f32> {
    let l = limit.max(1) as f32;
    parts.iter().map(|p| p.tokens as f32 * 100.0 / l).collect()
}

/// A compaction: the chat is squeezed into a summary of about `keep` per mille of its size
/// (at least 1K); other parts stay. Returns the new parts (a `summary` part replaces `chat`).
pub fn ctx_compact(parts: &[CtxPart], keep: u32) -> Vec<CtxPart> {
    parts
        .iter()
        .map(|p| {
            if p.kind != "chat" {
                return p.clone();
            }
            let t = (p.tokens as u64 * keep as u64 / 1000).max(1_000) as u32;
            CtxPart {
                kind: "summary",
                label: "Summary".into(),
                tokens: t,
                items: vec![("Earlier turns, summarised".into(), t)],
            }
        })
        .collect()
}

/// A field of `cells` dots: each dot is the index of the part filling that slice of the window,
/// or `None` for free space. Dots are handed out by cumulative share, so a part keeps its order.
pub fn field_cells(parts: &[CtxPart], limit: u32, cells: usize) -> Vec<Option<usize>> {
    let l = limit.max(1) as f64;
    let mut edges = Vec::with_capacity(parts.len());
    let mut acc = 0.0;
    for p in parts {
        acc += p.tokens as f64 / l;
        edges.push(acc);
    }
    (0..cells)
        .map(|c| {
            let mid = (c as f64 + 0.5) / cells.max(1) as f64;
            edges.iter().position(|e| mid < *e)
        })
        .collect()
}

/// Slab heights in px for a column of `px` high: each part's share, but never thinner than
/// `min` px so a small part keeps a label.
pub fn slab_heights(parts: &[CtxPart], limit: u32, px: f32, min: f32) -> Vec<f32> {
    let l = limit.max(1) as f32;
    parts
        .iter()
        .map(|p| (p.tokens as f32 / l * px).max(if p.tokens > 0 { min } else { 0.0 }))
        .collect()
}

/// Whole turns of `per_turn` tokens left before the window reaches `auto` percent; `None` when
/// it is already there.
pub fn turns_left(used: u32, limit: u32, auto: u32, per_turn: u32) -> Option<u32> {
    let at = (limit as u64 * auto as u64 / 100) as u32;
    if used >= at {
        None
    } else {
        Some((at - used) / per_turn.max(1))
    }
}

/// The parts in load order (system, tools, memory, files, chat, summary) – the stack's bottom-up order.
pub fn ctx_load_order(parts: &[CtxPart]) -> Vec<CtxPart> {
    let mut v = parts.to_vec();
    v.sort_by_key(|p| CTX_KINDS.iter().position(|k| *k == p.kind).unwrap_or(9));
    v
}

/// Adds one turn of `t` tokens to the chat (or the summary after a compaction).
pub fn ctx_add_turn(parts: &mut [CtxPart], t: u32) {
    if let Some(p) = parts
        .iter_mut()
        .find(|p| p.kind == "chat" || p.kind == "summary")
    {
        p.tokens += t;
        let n = p.items.len() + 1;
        p.items.insert(0, (format!("New turn {n}"), t));
    }
}

/// Drops item `i` of part `p` (a file or tool unloaded); the part's tokens follow.
pub fn ctx_drop_item(parts: &mut [CtxPart], p: usize, i: usize) {
    if let Some(part) = parts.get_mut(p)
        && i < part.items.len()
    {
        let (_, t) = part.items.remove(i);
        part.tokens = part.tokens.saturating_sub(t);
    }
}

/// The line a screen reader hears for the window: "Context: 166.4K of 200K tokens, 83 percent".
pub fn ctx_said(used: u32, limit: u32) -> String {
    format!(
        "Context: {} of {} tokens, {} percent",
        fmt_tokens(used),
        fmt_tokens(limit),
        ctx_pct(used, limit)
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn figures() {
        assert_eq!(fmt_tokens(980), "980");
        assert_eq!(fmt_tokens(7_600), "7.6K");
        assert_eq!(fmt_tokens(200_000), "200K");
        assert_eq!(fmt_tokens(166_400), "166.4K");
        assert_eq!(fmt_tokens(1_000_000), "1M");
        let d = ctx_demo();
        assert_eq!(ctx_used(&d), 166_400);
        assert_eq!(ctx_pct(166_400, 200_000), 83);
        assert_eq!(fill_level(166_400, 200_000, 92), "warm");
        assert_eq!(fill_level(190_000, 200_000, 92), "hot");
        assert_eq!(fill_level(10_000, 200_000, 92), "calm");
    }

    #[test]
    fn compaction_and_field() {
        let d = ctx_demo();
        let c = ctx_compact(&d, 180);
        assert_eq!(c[0].kind, "summary");
        assert_eq!(c[0].tokens, 17_352);
        assert!(ctx_used(&c) < ctx_used(&d));
        let f = field_cells(&d, 200_000, 100);
        assert_eq!(f.len(), 100);
        assert_eq!(f[0], Some(0));
        assert_eq!(f[99], None);
        assert_eq!(f.iter().filter(|c| c.is_none()).count(), 17);
    }

    #[test]
    fn turns_and_edits() {
        assert_eq!(turns_left(166_400, 200_000, 92, 6_000), Some(2));
        assert_eq!(turns_left(190_000, 200_000, 92, 6_000), None);
        let mut d = ctx_demo();
        ctx_add_turn(&mut d, 6_000);
        assert_eq!(d[0].tokens, 102_400);
        ctx_drop_item(&mut d, 1, 0);
        assert_eq!(d[1].tokens, 23_600);
        let h = slab_heights(&d, 200_000, 200.0, 14.0);
        assert!(h[4] >= 14.0);
        assert_eq!(ctx_load_order(&d)[0].kind, "system");
        assert_eq!(
            ctx_said(166_400, 200_000),
            "Context: 166.4K of 200K tokens, 83 percent"
        );
    }
}
