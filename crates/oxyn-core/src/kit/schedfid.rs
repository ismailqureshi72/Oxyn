//! affectUI fidelity pass, family B part 9 (plan #32): the **Scheduled Runs** model. Agent tasks
//! are booked into a week of fixed-length sessions, each with a usage limit (100 %). Auto
//! booking places the biggest tasks first and puts each one in the fullest session that still
//! has room, so small tasks fill the gaps; a task bigger than one session is cut into equal
//! parts that land in different sessions. Manual booking keeps only the hand-booked runs and
//! leaves the rest in the backlog. Behaviour only: the views live in `oxyn-dioxus` / `oxyn-gpui`
//! (`kit/schedfid.rs`).

/// A figure shown on a task card ("63 files").
#[derive(Clone, Debug, PartialEq)]
pub struct SrMetric {
    /// `files`, `tests`, `pages` or `endpoints` (picks the glyph).
    pub kind: String,
    /// The number.
    pub n: u32,
}

/// One agent task.
#[derive(Clone, Debug, PartialEq)]
pub struct SrTask {
    /// Stable id.
    pub id: String,
    /// Title.
    pub title: String,
    /// One-line description.
    pub desc: String,
    /// Agent id (also its brand mark): `claude`, `openai`, `gemini`.
    pub agent: String,
    /// Share of one session it needs, percent (over 100 means it must be split).
    pub need: u32,
    /// Figures for the card.
    pub metrics: Vec<SrMetric>,
    /// Booked by hand into this cell (day * slots + slot), kept in Manual.
    pub pin: Option<usize>,
}

/// An agent in the filter.
#[derive(Clone, Debug, PartialEq)]
pub struct SrAgent {
    /// Id (brand mark).
    pub id: String,
    /// Name shown in the pill.
    pub name: String,
}

/// One booked piece of a task.
#[derive(Clone, Debug, PartialEq)]
pub struct SrPart {
    /// Index into the task list.
    pub task: usize,
    /// Part number, 1-based.
    pub k: u32,
    /// How many parts the task was cut into.
    pub of: u32,
    /// Share of the session this piece takes, percent.
    pub share: u32,
    /// Cell index (day * slots + slot).
    pub cell: usize,
}

/// A week's plan.
#[derive(Clone, Debug, PartialEq, Default)]
pub struct SrPlan {
    /// Booked pieces, in booking order.
    pub parts: Vec<SrPart>,
    /// Tasks still to schedule (indices), in list order.
    pub backlog: Vec<usize>,
    /// Booked percent per cell (may pass 100 when a hand booking overfills it).
    pub load: Vec<u32>,
}

/// How the backlog is booked.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SrMode {
    /// Book everything that fits.
    Auto,
    /// Keep hand-booked runs only.
    Manual,
}

impl SrMode {
    /// `auto` / `manual`.
    pub fn id(self) -> &'static str {
        match self {
            Self::Auto => "auto",
            Self::Manual => "manual",
        }
    }
}

/// The session limit, percent.
pub const SR_LIMIT: u32 = 100;

/// Plan the week. `pins` are the hand bookings (task id → cell) and win over each task's own
/// `pin`; `unpinned` lists tasks whose own pin was taken back. In Auto everything else is booked
/// best-fit, biggest first; a task over the limit is cut into ⌈need / limit⌉ equal parts placed in
/// distinct cells. A task (or part set) that cannot fit stays in the backlog.
pub fn sr_plan(
    tasks: &[SrTask],
    cells: usize,
    mode: SrMode,
    pins: &[(String, usize)],
    unpinned: &[String],
) -> SrPlan {
    let mut load = vec![0u32; cells];
    let mut parts = Vec::new();
    let mut placed = vec![false; tasks.len()];
    // Hand bookings first: a pinned task takes its whole need in one cell.
    for (i, t) in tasks.iter().enumerate() {
        let cell = pins
            .iter()
            .rev()
            .find(|(id, _)| *id == t.id)
            .map(|p| p.1)
            .or_else(|| t.pin.filter(|_| !unpinned.contains(&t.id)));
        if let Some(c) = cell.filter(|&c| c < cells) {
            load[c] += t.need;
            parts.push(SrPart {
                task: i,
                k: 1,
                of: 1,
                share: t.need,
                cell: c,
            });
            placed[i] = true;
        }
    }
    if mode == SrMode::Auto {
        let mut order: Vec<usize> = (0..tasks.len()).filter(|&i| !placed[i]).collect();
        order.sort_by(|&a, &b| tasks[b].need.cmp(&tasks[a].need).then(a.cmp(&b)));
        for i in order {
            let need = tasks[i].need.max(1);
            let of = need.div_ceil(SR_LIMIT);
            let mut shares: Vec<u32> = vec![need / of; of as usize];
            for s in shares.iter_mut().take((need % of) as usize) {
                *s += 1;
            }
            let mut trial = load.clone();
            let mut got = Vec::new();
            for &sh in &shares {
                let best = (0..cells)
                    .filter(|c| !got.contains(c) && trial[*c] + sh <= SR_LIMIT)
                    .max_by(|&a, &b| trial[a].cmp(&trial[b]).then(b.cmp(&a)));
                match best {
                    Some(c) => {
                        trial[c] += sh;
                        got.push(c);
                    }
                    None => break,
                }
            }
            if got.len() == shares.len() {
                load = trial;
                // Parts in time order.
                let mut cs = got.clone();
                cs.sort();
                for (k, (c, sh)) in cs.into_iter().zip(shares).enumerate() {
                    parts.push(SrPart {
                        task: i,
                        k: k as u32 + 1,
                        of,
                        share: sh,
                        cell: c,
                    });
                }
                placed[i] = true;
            }
        }
    }
    SrPlan {
        parts,
        backlog: (0..tasks.len()).filter(|&i| !placed[i]).collect(),
        load,
    }
}

/// Pieces in one cell, in booking order.
pub fn sr_in_cell(plan: &SrPlan, cell: usize) -> Vec<&SrPart> {
    plan.parts.iter().filter(|p| p.cell == cell).collect()
}

/// Cells (in order) where a task's whole need still fits.
pub fn sr_fits(plan: &SrPlan, need: u32) -> Vec<usize> {
    (0..plan.load.len())
        .filter(|&c| plan.load[c] + need <= SR_LIMIT)
        .collect()
}

/// How many tasks an agent owns (all agents when `None`).
pub fn sr_count(tasks: &[SrTask], agent: Option<&str>) -> usize {
    tasks
        .iter()
        .filter(|t| agent.is_none_or(|a| t.agent == a))
        .count()
}

/// Day headers of the demo week.
pub const SR_DAYS: [(&str, u32); 3] = [("Mon", 16), ("Tue", 17), ("Wed", 18)];
/// Session start times of the demo week.
pub const SR_SLOTS: [&str; 3] = ["08:00", "13:00", "18:00"];

/// "Mon 16 · 13:00".
pub fn sr_cell_label(cell: usize) -> String {
    let n = SR_SLOTS.len();
    let (d, s) = (cell / n, cell % n);
    let (dn, dd) = SR_DAYS.get(d).copied().unwrap_or(("Day", 0));
    format!(
        "{dn} {dd} · {}",
        SR_SLOTS.get(s).copied().unwrap_or("--:--")
    )
}

/// The demo agents.
pub fn sr_agents() -> Vec<SrAgent> {
    [
        ("claude", "Claude Code"),
        ("openai", "Codex"),
        ("gemini", "Gemini CLI"),
    ]
    .iter()
    .map(|(i, n)| SrAgent {
        id: i.to_string(),
        name: n.to_string(),
    })
    .collect()
}

fn tk(
    id: &str,
    agent: &str,
    title: &str,
    desc: &str,
    need: u32,
    m: &[(&str, u32)],
    pin: Option<usize>,
) -> SrTask {
    SrTask {
        id: id.into(),
        title: title.into(),
        desc: desc.into(),
        agent: agent.into(),
        need,
        metrics: m
            .iter()
            .map(|(k, n)| SrMetric {
                kind: k.to_string(),
                n: *n,
            })
            .collect(),
        pin,
    }
}

/// The demo backlog: eleven tasks over three agents, five of them booked by hand, one needing
/// one and a half sessions.
pub fn sr_demo() -> Vec<SrTask> {
    vec![
        tk(
            "sessions",
            "claude",
            "Move sign-in to server sessions",
            "Drop the token cookie, add a session store",
            150,
            &[("files", 48), ("tests", 132)],
            None,
        ),
        tk(
            "webhooks",
            "openai",
            "Retry failed payment webhooks",
            "Back-off queue with a dead-letter view",
            55,
            &[("files", 12), ("tests", 40)],
            Some(0),
        ),
        tk(
            "apiref",
            "gemini",
            "Document the orders API",
            "Reference pages from the route schema",
            45,
            &[("pages", 14), ("endpoints", 22)],
            None,
        ),
        tk(
            "flaky",
            "claude",
            "Fix flaky checkout tests",
            "Stabilise timers in the cart suite",
            30,
            &[("files", 6), ("tests", 58)],
            Some(1),
        ),
        tk(
            "i18n",
            "gemini",
            "Extract strings for French",
            "Pull UI copy into message files",
            60,
            &[("files", 37)],
            Some(4),
        ),
        tk(
            "lint",
            "openai",
            "Turn on strict lint rules",
            "Fix the warnings the new rules raise",
            25,
            &[("files", 71)],
            None,
        ),
        tk(
            "search",
            "claude",
            "Add search to the help centre",
            "Index articles and wire the search box",
            70,
            &[("files", 19), ("tests", 26)],
            Some(5),
        ),
        tk(
            "deps",
            "gemini",
            "Update front-end packages",
            "Bump majors one at a time, run the suite",
            35,
            &[("files", 9), ("tests", 310)],
            None,
        ),
        tk(
            "a11y",
            "claude",
            "Audit forms for screen readers",
            "Labels, errors and focus order",
            40,
            &[("pages", 11)],
            Some(7),
        ),
        tk(
            "csv",
            "gemini",
            "Export invoices as CSV",
            "Streamed export with a progress toast",
            20,
            &[("files", 5), ("tests", 14)],
            None,
        ),
        tk(
            "cache",
            "claude",
            "Cache product images at the edge",
            "Resize once, serve from the CDN",
            15,
            &[("files", 4), ("endpoints", 3)],
            None,
        ),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn auto_and_manual() {
        let t = sr_demo();
        assert_eq!(sr_count(&t, None), 11);
        assert_eq!(sr_count(&t, Some("openai")), 2);
        let m = sr_plan(&t, 9, SrMode::Manual, &[], &[]);
        assert_eq!(m.parts.len(), 5);
        assert_eq!(m.backlog.len(), 6);
        assert_eq!(m.load[0], 55);
        let a = sr_plan(&t, 9, SrMode::Auto, &[], &[]);
        assert!(a.backlog.is_empty());
        assert!(a.load.iter().all(|&l| l <= SR_LIMIT));
        let total: u32 = t.iter().map(|x| x.need).sum();
        assert_eq!(a.load.iter().sum::<u32>(), total);
        // The 150 % task is cut in two 75 % parts in different sessions.
        let sp: Vec<_> = a.parts.iter().filter(|p| p.task == 0).collect();
        assert_eq!(sp.len(), 2);
        assert_eq!((sp[0].share, sp[1].share, sp[0].of), (75, 75, 2));
        assert_ne!(sp[0].cell, sp[1].cell);
        // A hand drop overfills a session.
        let d = sr_plan(&t, 9, SrMode::Manual, &[("sessions".into(), 0)], &[]);
        assert_eq!(d.load[0], 205);
        // Taking back a pin returns it to the backlog.
        let u = sr_plan(&t, 9, SrMode::Manual, &[], &["webhooks".into()]);
        assert_eq!(u.backlog.len(), 7);
        assert_eq!(sr_fits(&m, 50).len(), 6);
        assert_eq!(sr_cell_label(4), "Tue 17 · 13:00");
    }
}
