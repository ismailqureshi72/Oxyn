//! affectUI fidelity pass (plan #32, overnight batch, family B part 6): the model behind the exact
//! **Context Map** – what fills an agent's context window as a treemap: system prompt, tools,
//! memory, every file, the chat and tool results as tiles in a fixed order, the free space dashed,
//! and a compaction that squeezes old parts into a summary. Behaviour only.

/// What a part of the context is (picks its tint).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CtxKind {
    /// The system prompt.
    System,
    /// Tool definitions.
    Tools,
    /// Project memory.
    Memory,
    /// A file.
    File,
    /// The conversation.
    Chat,
    /// A tool result.
    Result,
    /// A compaction summary.
    Summary,
}

impl CtxKind {
    /// `system` · `tools` · `memory` · `file` · `chat` · `result` · `summary`.
    pub fn id(self) -> &'static str {
        match self {
            Self::System => "system",
            Self::Tools => "tools",
            Self::Memory => "memory",
            Self::File => "file",
            Self::Chat => "chat",
            Self::Result => "result",
            Self::Summary => "summary",
        }
    }
    /// The kind in words.
    pub fn word(self) -> &'static str {
        match self {
            Self::System => "System prompt",
            Self::Tools => "Tools",
            Self::Memory => "Memory",
            Self::File => "File",
            Self::Chat => "Chat",
            Self::Result => "Tool result",
            Self::Summary => "Summary",
        }
    }
}

/// One part of the context.
#[derive(Clone, Debug, PartialEq)]
pub struct CtxBlock {
    /// Id.
    pub id: String,
    /// Label.
    pub label: String,
    /// Kind.
    pub kind: CtxKind,
    /// Tokens.
    pub tokens: u32,
    /// Brand mark shown before the label (empty for none).
    pub brand: String,
    /// What it holds, listed in its card: (label, brand).
    pub items: Vec<(String, String)>,
}

/// An agent to switch between.
#[derive(Clone, Debug, PartialEq)]
pub struct CtxAgent {
    /// Id.
    pub id: String,
    /// Name.
    pub name: String,
    /// Model.
    pub model: String,
    /// Maker's mark.
    pub brand: String,
    /// Its context window, tokens.
    pub limit: u32,
}

/// The last compaction: the blocks just before it, and which were summarized or dropped.
#[derive(Clone, Debug, PartialEq)]
pub struct CtxCompaction {
    /// The blocks just before.
    pub before: Vec<CtxBlock>,
    /// Ids squeezed into the summary.
    pub summarized: Vec<String>,
    /// Ids dropped.
    pub dropped: Vec<String>,
}

/// "14.8k", "200k", "1.2M", "640".
pub fn ctx_fmt(t: u32) -> String {
    if t >= 1_000_000 {
        let v = t as f32 / 1_000_000.0;
        return trim(format!("{v:.1}")) + "M";
    }
    if t >= 1_000 {
        let v = t as f32 / 1000.0;
        return if v >= 100.0 {
            format!("{}k", v.round() as u32)
        } else {
            trim(format!("{v:.1}")) + "k"
        };
    }
    t.to_string()
}

fn trim(s: String) -> String {
    s.strip_suffix(".0").map(str::to_string).unwrap_or(s)
}

/// Tokens used.
pub fn ctx_used(blocks: &[CtxBlock]) -> u32 {
    blocks.iter().map(|b| b.tokens).sum()
}

/// Percent used, rounded.
pub fn ctx_pct(used: u32, limit: u32) -> u32 {
    ((used as f64 / limit.max(1) as f64) * 100.0).round() as u32
}

/// A tile: index into the weights, then x, y, w, h in 0–1 of the area.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CtxRect {
    /// Index of the weight.
    pub i: usize,
    /// Left.
    pub x: f32,
    /// Top.
    pub y: f32,
    /// Width.
    pub w: f32,
    /// Height.
    pub h: f32,
}

fn worst(row: &[f32], side: f32) -> f32 {
    let s: f32 = row.iter().sum();
    let (mx, mn) = row
        .iter()
        .fold((0f32, f32::MAX), |(a, b), &r| (a.max(r), b.min(r)));
    let s2 = s * s;
    let side2 = side * side;
    (side2 * mx / s2).max(s2 / (side2 * mn))
}

/// An ordered squarified treemap of `weights` (kept in order) over an area `aspect` wide per 1
/// tall; returns rects in 0–1 units. Zero weights get no tile.
pub fn ctx_layout(weights: &[f32], aspect: f32) -> Vec<CtxRect> {
    let total: f32 = weights.iter().filter(|w| **w > 0.0).sum();
    if total <= 0.0 {
        return vec![];
    }
    // Work in an area aspect × 1 whose total is aspect.
    let scale = aspect / total;
    let items: Vec<(usize, f32)> = weights
        .iter()
        .enumerate()
        .filter(|(_, w)| **w > 0.0)
        .map(|(i, w)| (i, w * scale))
        .collect();
    let (mut x, mut y, mut w, mut h) = (0f32, 0f32, aspect, 1f32);
    let mut out = vec![];
    let mut k = 0;
    while k < items.len() {
        let side = w.min(h);
        let mut row: Vec<f32> = vec![items[k].1];
        let mut end = k + 1;
        while end < items.len() {
            let mut next = row.clone();
            next.push(items[end].1);
            if worst(&next, side) <= worst(&row, side) {
                row = next;
                end += 1;
            } else {
                break;
            }
        }
        let s: f32 = row.iter().sum();
        if w >= h {
            // A column on the left.
            let cw = s / h;
            let mut yy = y;
            for (j, r) in row.iter().enumerate() {
                let rh = r / cw;
                out.push(CtxRect {
                    i: items[k + j].0,
                    x,
                    y: yy,
                    w: cw,
                    h: rh,
                });
                yy += rh;
            }
            x += cw;
            w -= cw;
        } else {
            // A row on top.
            let rh = s / w;
            let mut xx = x;
            for (j, r) in row.iter().enumerate() {
                let cw = r / rh;
                out.push(CtxRect {
                    i: items[k + j].0,
                    x: xx,
                    y,
                    w: cw,
                    h: rh,
                });
                xx += cw;
            }
            y += rh;
            h -= rh;
        }
        k = end;
    }
    for r in out.iter_mut() {
        r.x /= aspect;
        r.w /= aspect;
    }
    out
}

/// What a compaction did.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CtxCompactionSum {
    /// Tokens before.
    pub before: u32,
    /// Tokens after.
    pub after: u32,
    /// Parts kept and their tokens.
    pub kept: (usize, u32),
    /// Parts summarized and their tokens.
    pub summarized: (usize, u32),
    /// The summary's tokens.
    pub summary: u32,
    /// Parts dropped and their tokens.
    pub dropped: (usize, u32),
}

/// Sums up a compaction against the blocks after it.
pub fn ctx_compaction_sum(c: &CtxCompaction, after: &[CtxBlock]) -> CtxCompactionSum {
    let mut kept = (0, 0);
    let mut summ = (0, 0);
    let mut drop = (0, 0);
    for b in &c.before {
        let slot = if c.summarized.contains(&b.id) {
            &mut summ
        } else if c.dropped.contains(&b.id) {
            &mut drop
        } else {
            &mut kept
        };
        slot.0 += 1;
        slot.1 += b.tokens;
    }
    CtxCompactionSum {
        before: ctx_used(&c.before),
        after: ctx_used(after),
        kept,
        summarized: summ,
        summary: after
            .iter()
            .filter(|b| b.kind == CtxKind::Summary)
            .map(|b| b.tokens)
            .sum(),
        dropped: drop,
    }
}

fn blk(id: &str, label: &str, kind: CtxKind, tokens: u32, brand: &str) -> CtxBlock {
    CtxBlock {
        id: id.into(),
        label: label.into(),
        kind,
        tokens,
        brand: brand.into(),
        items: vec![],
    }
}

/// The demo agents (our own).
pub fn ctx_demo_agents() -> Vec<CtxAgent> {
    let a = |id: &str, name: &str, model: &str, brand: &str, limit| CtxAgent {
        id: id.into(),
        name: name.into(),
        model: model.into(),
        brand: brand.into(),
        limit,
    };
    vec![
        a("release", "Release helper", "Opus 5.5", "claude", 200_000),
        a("mapper", "Repo mapper", "Gemini Pro", "gemini", 500_000),
        a("triage", "Inbox triage", "GPT-5", "openai", 128_000),
    ]
}

/// The demo context at `grow` 0–1 (0 = a fresh session, 1 = nearly full, before compaction).
pub fn ctx_demo(grow: f32) -> Vec<CtxBlock> {
    use CtxKind::*;
    let g = grow.clamp(0.0, 1.0);
    let lerp = |a: f32, b: f32| (a + (b - a) * g) as u32;
    let mut tools = blk("tools", "Tool definitions", Tools, 14_800, "");
    tools.items = [
        ("GitHub", "github"),
        ("Terminal", ""),
        ("Search", ""),
        ("Read file", ""),
        ("Edit file", ""),
        ("Vercel", "vercel"),
        ("Sentry", "sentry"),
        ("Linear", "linear"),
    ]
    .iter()
    .map(|(a, b)| (a.to_string(), b.to_string()))
    .collect();
    let mut v = vec![
        blk("system", "System prompt", System, 6_100, ""),
        tools,
        blk("memory", "Project memory", Memory, 9_200, ""),
        blk(
            "changelog",
            "CHANGELOG.md",
            File,
            lerp(1_200.0, 7_900.0),
            "",
        ),
        blk("release", "release.ts", File, lerp(3_100.0, 16_900.0), ""),
        blk("chat", "Chat", Chat, lerp(900.0, 44_600.0), ""),
        blk("ci", "CI run · 14×", Result, lerp(0.0, 21_700.0), ""),
        blk(
            "deploy",
            "vercel.json",
            File,
            lerp(600.0, 10_400.0),
            "vercel",
        ),
        blk(
            "issues",
            "Issue search",
            Result,
            lerp(0.0, 16_200.0),
            "linear",
        ),
        blk("notes", "notes.md", File, lerp(4_500.0, 24_800.0), ""),
    ];
    v.retain(|b| b.tokens > 0);
    v
}

/// The demo after a compaction: the full context squeezed (three summarized, one dropped).
pub fn ctx_demo_compacted() -> (Vec<CtxBlock>, CtxCompaction) {
    let before = ctx_demo(1.0);
    let summarized: Vec<String> = vec!["chat".into(), "ci".into(), "notes".into()];
    let dropped: Vec<String> = vec!["issues".into()];
    let mut after: Vec<CtxBlock> = before
        .iter()
        .filter(|b| !summarized.contains(&b.id) && !dropped.contains(&b.id))
        .cloned()
        .collect();
    after.push(blk("summary", "Summary", CtxKind::Summary, 8_400, ""));
    (
        after,
        CtxCompaction {
            before,
            summarized,
            dropped,
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn numbers_layout_and_compaction() {
        assert_eq!(ctx_fmt(14_800), "14.8k");
        assert_eq!(ctx_fmt(200_000), "200k");
        assert_eq!(ctx_fmt(9_000), "9k");
        assert_eq!(ctx_fmt(640), "640");
        assert_eq!(ctx_fmt(1_200_000), "1.2M");
        assert_eq!(ctx_pct(30_400, 200_000), 15);
        let r = ctx_layout(&[1.0, 2.0, 0.0, 3.0, 10.0], 2.4);
        assert_eq!(r.len(), 4);
        let area: f32 = r.iter().map(|t| t.w * t.h).sum();
        assert!((area - 1.0).abs() < 1e-3);
        assert!(
            r.iter()
                .all(|t| t.x >= -1e-4 && t.x + t.w <= 1.0001 && t.y + t.h <= 1.0001)
        );
        // Order is kept: the first tile starts at the top left.
        assert_eq!((r[0].i, r[0].x, r[0].y), (0, 0.0, 0.0));
        let (after, c) = ctx_demo_compacted();
        let s = ctx_compaction_sum(&c, &after);
        assert_eq!(s.kept.0 + s.summarized.0 + s.dropped.0, c.before.len());
        assert_eq!((s.summarized.0, s.dropped.0, s.summary), (3, 1, 8_400));
        assert!(s.after < s.before);
        assert!(ctx_used(&ctx_demo(0.0)) < ctx_used(&ctx_demo(1.0)));
    }
}
