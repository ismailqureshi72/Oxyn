//! Tool stacks (`tools.md` §3, §4): consecutive calls grouped into one header, density levels,
//! and the end-of-turn "Worked for" fold.

use std::time::Duration;

use super::{Tone, ToolCall, ToolCallState, ToolKind, format_call_duration, plural};
use crate::{ClockMs, VerbPack, verbs};

/// Stack variants (§6.1, free).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ToolStackVariant {
    /// Rows and groups inline.
    #[default]
    Transcript,
    /// As transcript; a finished turn folds into "Worked for 31s ›".
    WorkedFor,
    /// TUI tree.
    Terminal,
}

impl ToolStackVariant {
    /// Every free variant.
    pub const ALL: [Self; 3] = [Self::Transcript, Self::WorkedFor, Self::Terminal];

    /// Registry id `tool-stack/<id>`.
    pub fn id(self) -> &'static str {
        match self {
            Self::Transcript => "transcript",
            Self::WorkedFor => "worked-for",
            Self::Terminal => "terminal",
        }
    }
}

/// Density (§4.2); a setting, not a variant.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ToolDensity {
    /// One live line only.
    Hidden,
    /// Grouped, always collapsed.
    Compact,
    /// One row per ungrouped call (default).
    #[default]
    Normal,
    /// Everything expanded, no grouping.
    Verbose,
}

/// What groups with what (§3.1).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum GroupClass {
    /// Read, list, search.
    Explore,
    /// Web search and fetch.
    Web,
    /// One MCP server.
    Mcp(String),
    /// File edits (rollup).
    Edit,
    /// Started together while running.
    Parallel,
}

impl GroupClass {
    /// The class of a call, or `None` for kinds that never group.
    pub fn of(call: &ToolCall) -> Option<Self> {
        match call.kind {
            ToolKind::Read | ToolKind::List | ToolKind::Search => Some(Self::Explore),
            ToolKind::WebSearch | ToolKind::Fetch => Some(Self::Web),
            ToolKind::Edit | ToolKind::Create | ToolKind::Delete => Some(Self::Edit),
            ToolKind::Mcp => Some(Self::Mcp(
                call.server
                    .as_ref()
                    .map(|s| s.name.clone())
                    .unwrap_or_default(),
            )),
            _ => None,
        }
    }
}

/// A group header's state (§3.2).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GroupState {
    /// A child is live.
    Live,
    /// A child waits for the user.
    Waiting,
    /// All done, none failed.
    Settled,
    /// Done with failures.
    SettledWithErrors,
    /// Stopped.
    Cancelled,
}

/// One thing in the transcript between two pieces of prose.
#[derive(Clone, Debug, PartialEq)]
pub enum StackItem {
    /// A single row: index into the calls slice.
    Call(usize),
    /// A group of calls.
    Group {
        /// Class.
        class: GroupClass,
        /// Indices into the calls slice.
        calls: Vec<usize>,
        /// Header state.
        state: GroupState,
        /// Header label ("Read 6 files, searched 2 patterns · 1 failed").
        label: String,
        /// Header tone.
        tone: Tone,
        /// Total or live duration.
        duration: Option<String>,
        /// Failed children kept visible when collapsed.
        pinned: Vec<usize>,
    },
    /// `Hidden` density: one live line.
    Progress(String),
    /// `WorkedFor`: the turn folded ("Worked for 31s"), with errors kept visible below it.
    Fold {
        /// Label.
        label: String,
        /// Calls still shown (failed, waiting).
        pinned: Vec<usize>,
        /// Folded calls.
        calls: Vec<usize>,
    },
}

/// One slot in the turn: a call, or something that breaks a group (prose, reasoning, approval).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Entry {
    /// The call at this index.
    Call(usize),
    /// A break.
    Break,
}

/// Window within which calls count as started together.
pub const PARALLEL_WINDOW: Duration = Duration::from_millis(50);

fn group_state(calls: &[&ToolCall]) -> GroupState {
    if calls
        .iter()
        .any(|c| c.state == ToolCallState::AwaitingApproval)
    {
        GroupState::Waiting
    } else if calls.iter().any(|c| c.state.is_live()) {
        GroupState::Live
    } else if calls
        .iter()
        .any(|c| matches!(c.state, ToolCallState::Failed { .. }))
    {
        GroupState::SettledWithErrors
    } else if calls
        .iter()
        .all(|c| matches!(c.state, ToolCallState::Cancelled { .. }))
    {
        GroupState::Cancelled
    } else {
        GroupState::Settled
    }
}

/// The composed header label for a group.
pub fn group_label(class: &GroupClass, calls: &[&ToolCall], pack: &VerbPack) -> String {
    let st = group_state(calls);
    let live = matches!(st, GroupState::Live | GroupState::Waiting);
    let count = |k: &[ToolKind]| calls.iter().filter(|c| k.contains(&c.kind)).count() as u32;
    let failed = calls
        .iter()
        .filter(|c| matches!(c.state, ToolCallState::Failed { .. }))
        .count() as u32;
    let mut label = match class {
        GroupClass::Parallel => format!("Running {}", plural(calls.len() as u32, "tool", "tools")),
        GroupClass::Explore => {
            let reads = count(&[ToolKind::Read]);
            let searches = count(&[ToolKind::Search]);
            let lists = count(&[ToolKind::List]);
            let mut parts = vec![];
            if live {
                // Present tense names the dominant kind only: "Reading 4 files".
                let (r, s, l) = (reads, searches, lists);
                let p = if r >= s && r >= l {
                    format!(
                        "{} {}",
                        ToolKind::Read.verbs(pack).0,
                        plural(r, "file", "files")
                    )
                } else if s >= l {
                    format!(
                        "{} {}",
                        ToolKind::Search.verbs(pack).0,
                        plural(s, "pattern", "patterns")
                    )
                } else {
                    format!("Listing {}", plural(l, "directory", "directories"))
                };
                parts.push(p);
            } else {
                if reads > 0 {
                    parts.push(format!(
                        "{} {}",
                        ToolKind::Read.verbs(pack).1,
                        plural(reads, "file", "files")
                    ));
                }
                if searches > 0 {
                    let v = ToolKind::Search.verbs(pack).1;
                    let v = if parts.is_empty() {
                        v
                    } else {
                        v.to_lowercase()
                    };
                    parts.push(format!("{v} {}", plural(searches, "pattern", "patterns")));
                }
                if lists > 0 {
                    let v = if parts.is_empty() { "Listed" } else { "listed" };
                    parts.push(format!("{v} {}", plural(lists, "directory", "directories")));
                }
            }
            parts.join(", ")
        }
        GroupClass::Web => {
            let s = count(&[ToolKind::WebSearch]);
            let f = count(&[ToolKind::Fetch]);
            let mut parts = vec![];
            if s > 0 {
                parts.push(format!(
                    "{} {}",
                    if live {
                        "Searching the web"
                    } else {
                        "Searched the web"
                    },
                    plural(s, "time", "times")
                ));
            }
            if f > 0 {
                let v = if live { "fetching" } else { "fetched" };
                let v = if parts.is_empty() {
                    let mut c = v.chars();
                    c.next()
                        .map(|x| x.to_uppercase().collect::<String>() + c.as_str())
                        .unwrap_or_default()
                } else {
                    v.to_string()
                };
                parts.push(format!("{v} {}", plural(f, "page", "pages")));
            }
            parts.join(", ")
        }
        GroupClass::Mcp(server) => format!(
            "{} {server} {}",
            if live { "Calling" } else { "Called" },
            plural(calls.len() as u32, "time", "times")
        ),
        GroupClass::Edit => {
            let files: std::collections::BTreeSet<&str> = calls
                .iter()
                .filter_map(|c| c.target.as_ref().map(|t| t.full()))
                .collect();
            let (a, r) = calls
                .iter()
                .filter_map(|c| c.lines)
                .fold((0, 0), |(a, r), (x, y)| (a + x, r + y));
            let n = files.len().max(1) as u32;
            let v = if live { "Editing" } else { "Edited" };
            if a + r > 0 {
                format!("{v} {} +{a} \u{2212}{r}", plural(n, "file", "files"))
            } else {
                format!("{v} {}", plural(n, "file", "files"))
            }
        }
    };
    match st {
        GroupState::Waiting => {
            let i = calls
                .iter()
                .position(|c| c.state == ToolCallState::AwaitingApproval)
                .unwrap_or(0);
            label = format!("Waiting for you · {} of {}", i + 1, calls.len());
        }
        GroupState::Cancelled => {
            let done = calls
                .iter()
                .filter(|c| matches!(c.state, ToolCallState::Succeeded { .. }))
                .count();
            label = format!("Stopped · {done} of {} done", calls.len());
        }
        _ => {}
    }
    if failed > 0 && !live {
        label.push_str(&format!(" · {failed} failed"));
    }
    label
}

fn make_group(
    class: GroupClass,
    idx: Vec<usize>,
    calls: &[ToolCall],
    pack: &VerbPack,
    now: ClockMs,
) -> StackItem {
    let refs: Vec<&ToolCall> = idx.iter().map(|i| &calls[*i]).collect();
    let state = group_state(&refs);
    let label = group_label(&class, &refs, pack);
    let start = refs.iter().filter_map(|c| c.started_at).min();
    let end = refs
        .iter()
        .map(|c| c.ended_at)
        .collect::<Option<Vec<_>>>()
        .and_then(|v| v.into_iter().max());
    let duration = start
        .map(|s| end.unwrap_or(now).since(s))
        .filter(|d| state != GroupState::Live || *d >= Duration::from_secs(1))
        .map(format_call_duration);
    let pinned = idx
        .iter()
        .copied()
        .filter(|i| {
            matches!(
                calls[*i].state,
                ToolCallState::Failed { .. } | ToolCallState::AwaitingApproval
            )
        })
        .collect();
    let tone = match state {
        GroupState::Live => Tone::Default,
        GroupState::Waiting => Tone::Waiting,
        GroupState::SettledWithErrors => Tone::Danger,
        GroupState::Settled | GroupState::Cancelled => Tone::Muted,
    };
    StackItem::Group {
        class,
        calls: idx,
        state,
        label,
        tone,
        duration,
        pinned,
    }
}

/// Lays out one turn (§3.1). `entries` interleave calls and breaks in transcript order.
pub fn layout(
    calls: &[ToolCall],
    entries: &[Entry],
    density: ToolDensity,
    variant: ToolStackVariant,
    turn_done: Option<Duration>,
    pack: &VerbPack,
    now: ClockMs,
) -> Vec<StackItem> {
    if density == ToolDensity::Hidden {
        let live = entries.iter().rev().find_map(|e| match e {
            Entry::Call(i) if calls[*i].state.is_live() => Some(*i),
            _ => None,
        });
        let mut out: Vec<StackItem> = live
            .map(|i| StackItem::Progress(super::call_label(&calls[i], pack)))
            .into_iter()
            .collect();
        out.extend(entries.iter().filter_map(|e| match e {
            Entry::Call(i)
                if matches!(
                    calls[*i].state,
                    ToolCallState::Failed { .. } | ToolCallState::AwaitingApproval
                ) =>
            {
                Some(StackItem::Call(*i))
            }
            _ => None,
        }));
        return out;
    }
    if let (Some(elapsed), ToolStackVariant::WorkedFor) = (turn_done, variant)
        && !entries.is_empty()
    {
        let idx: Vec<usize> = entries
            .iter()
            .filter_map(|e| match e {
                Entry::Call(i) => Some(*i),
                Entry::Break => None,
            })
            .collect();
        let pinned = idx
            .iter()
            .copied()
            .filter(|i| {
                matches!(
                    calls[*i].state,
                    ToolCallState::Failed { .. } | ToolCallState::AwaitingApproval
                )
            })
            .collect();
        return vec![StackItem::Fold {
            label: format!("Worked for {}", verbs::format_summary(elapsed)),
            pinned,
            calls: idx,
        }];
    }
    let group = density != ToolDensity::Verbose;
    let mut out = vec![];
    let mut run: Vec<usize> = vec![];
    let mut run_class: Option<GroupClass> = None;
    let mut last_call: Option<usize> = None;
    let flush = |run: &mut Vec<usize>, class: &mut Option<GroupClass>, out: &mut Vec<StackItem>| {
        match (run.len(), class.take()) {
            (0, _) => {}
            (1, _) | (_, None) => out.extend(run.drain(..).map(StackItem::Call)),
            (_, Some(c)) => out.push(make_group(c, std::mem::take(run), calls, pack, now)),
        }
        run.clear();
    };
    for e in entries {
        let Entry::Call(i) = *e else {
            flush(&mut run, &mut run_class, &mut out);
            last_call = None;
            continue;
        };
        let prev_call = last_call.replace(i);
        let c = &calls[i];
        // Parallel: started within the window of the previous call while any of them runs.
        let parallel = group
            && prev_call.is_some_and(|p| {
                let prev = &calls[p];
                matches!((prev.started_at, c.started_at), (Some(a), Some(b)) if b.since(a) <= PARALLEL_WINDOW && a.since(b) <= PARALLEL_WINDOW)
                    && (prev.state.is_live() || c.state.is_live())
            });
        let class = if parallel {
            Some(GroupClass::Parallel)
        } else {
            GroupClass::of(c).filter(|_| group)
        };
        let joins = match (&run_class, &class) {
            (Some(GroupClass::Parallel), _) if parallel => true,
            (Some(a), Some(b)) => a == b,
            _ => false,
        };
        if parallel && run_class != Some(GroupClass::Parallel) {
            // The previous call joins this parallel burst.
            let prev = if run.last() == prev_call.as_ref() {
                run.pop()
            } else if out.last() == prev_call.map(StackItem::Call).as_ref() {
                out.pop();
                prev_call
            } else {
                None
            };
            flush(&mut run, &mut run_class, &mut out);
            run.extend(prev);
            run_class = Some(GroupClass::Parallel);
            run.push(i);
            continue;
        }
        if !joins {
            flush(&mut run, &mut run_class, &mut out);
            run_class = class.clone();
        }
        run.push(i);
        if class.is_none() {
            flush(&mut run, &mut run_class, &mut out);
        }
    }
    flush(&mut run, &mut run_class, &mut out);
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::run::{Target, ToolCall, ToolCallState as S, ToolErrorKind};

    fn done(id: &str, name: &str, t: &str, at: u64) -> ToolCall {
        ToolCall::new(id, name, ClockMs(at))
            .with_target(Target::Path(t.into()))
            .with_state(S::Succeeded { summary: None })
            .timed(ClockMs(at), Some(ClockMs(at + 400)))
    }

    #[test]
    fn groups_explore_and_breaks() {
        let p = VerbPack::neutral();
        let mut calls = vec![
            done("1", "read_file", "a.rs", 0),
            done("2", "read_file", "b.rs", 1_000),
            done("3", "grep", "auth", 2_000),
            done("4", "bash", "cargo test", 3_000),
            done("5", "read_file", "c.rs", 4_000),
        ];
        calls[2].kind = ToolKind::Search;
        calls[3].kind = ToolKind::Shell;
        let e: Vec<Entry> = (0..5).map(Entry::Call).collect();
        let l = layout(
            &calls,
            &e,
            ToolDensity::Normal,
            ToolStackVariant::Transcript,
            None,
            p,
            ClockMs(9_000),
        );
        assert_eq!(l.len(), 3);
        match &l[0] {
            StackItem::Group { label, calls, .. } => {
                assert_eq!(label, "Read 2 files, searched 1 pattern");
                assert_eq!(calls.len(), 3);
            }
            x => panic!("{x:?}"),
        }
        assert_eq!(l[1], StackItem::Call(3));
        assert_eq!(l[2], StackItem::Call(4));
        // A break splits groups.
        let e = vec![Entry::Call(0), Entry::Break, Entry::Call(1)];
        let l = layout(
            &calls,
            &e,
            ToolDensity::Normal,
            ToolStackVariant::Transcript,
            None,
            p,
            ClockMs(9_000),
        );
        assert_eq!(l, vec![StackItem::Call(0), StackItem::Call(1)]);
        // Verbose never groups.
        let e: Vec<Entry> = (0..3).map(Entry::Call).collect();
        let l = layout(
            &calls,
            &e,
            ToolDensity::Verbose,
            ToolStackVariant::Transcript,
            None,
            p,
            ClockMs(9_000),
        );
        assert_eq!(l.len(), 3);
    }

    #[test]
    fn failures_parallel_and_fold() {
        let p = VerbPack::neutral();
        let mut calls = vec![
            done("1", "read_file", "a.rs", 0),
            done("2", "read_file", "b.rs", 1_000),
        ];
        calls[1].state = S::Failed {
            kind: ToolErrorKind::Tool,
            code: None,
            message: None,
        };
        let e = vec![Entry::Call(0), Entry::Call(1)];
        let l = layout(
            &calls,
            &e,
            ToolDensity::Normal,
            ToolStackVariant::Transcript,
            None,
            p,
            ClockMs(9_000),
        );
        let StackItem::Group {
            label,
            pinned,
            state,
            ..
        } = &l[0]
        else {
            panic!()
        };
        assert_eq!(
            (label.as_str(), pinned.as_slice(), *state),
            (
                "Read 2 files · 1 failed",
                &[1usize][..],
                GroupState::SettledWithErrors
            )
        );

        let par = vec![
            ToolCall::new("a", "bash", ClockMs(0))
                .with_state(S::Running)
                .timed(ClockMs(0), None),
            ToolCall::new("b", "web_search", ClockMs(10))
                .with_state(S::Running)
                .timed(ClockMs(10), None),
            ToolCall::new("c", "read_file", ClockMs(20))
                .with_state(S::Running)
                .timed(ClockMs(20), None),
        ];
        let e: Vec<Entry> = (0..3).map(Entry::Call).collect();
        let l = layout(
            &par,
            &e,
            ToolDensity::Normal,
            ToolStackVariant::Transcript,
            None,
            p,
            ClockMs(500),
        );
        let StackItem::Group { label, class, .. } = &l[0] else {
            panic!("{l:?}")
        };
        assert_eq!(
            (label.as_str(), class),
            ("Running 3 tools", &GroupClass::Parallel)
        );

        let l = layout(
            &calls,
            &e[..2],
            ToolDensity::Normal,
            ToolStackVariant::WorkedFor,
            Some(Duration::from_secs(31)),
            p,
            ClockMs(9_000),
        );
        assert_eq!(
            l,
            vec![StackItem::Fold {
                label: "Worked for 31s".into(),
                pinned: vec![1],
                calls: vec![0, 1]
            }]
        );
        let l = layout(
            &par,
            &e,
            ToolDensity::Hidden,
            ToolStackVariant::Transcript,
            None,
            p,
            ClockMs(500),
        );
        assert_eq!(
            l,
            vec![StackItem::Progress("Reading c.rs".replace(" c.rs", ""))]
        );
    }
}
