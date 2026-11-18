//! Parity models (plan #28): the behaviour that closes the gaps between Oxyn and the affectUI,
//! Arc UI and Beautiful UI components listed in `docs/parity/status.csv`, plus the X-post
//! references. Behaviour-only rebuilds: nothing here is copied from those libraries.
//!
//! - [`compose`]: AI composer triggers, the rich-text block editor, run controls, effort levels.
//! - [`fields`]: number field, scrub input, elastic slider and morph select.
//! - [`meters`]: gauge, usage meter, split bar and both context windows.
//! - [`upload`]: dropzone rows, accept and size checks, and the avatar cropper.
//! - [`home`]: thread groups, heat maps, token usage, uploads that land in turn, run modes and
//!   experiments, document views and polling cost (the chunk 6 X-post references).
//! - [`confirm`]: confirm morph, hold to confirm, confirm popover (destructive buttons that ask).
//! - [`edit`]: a text buffer with selection and IME composition (GPUI text entry).
//! - [`input`]: text area rows and counter, changed-word rows, ghost-text autocomplete.
//! - [`gates`]: approval gates, permission requests and the agent approvals block.
//! - [`text`]: loaders and moving text (morph, slot reels, phases, action swap, streams).
//! - [`diff`]: line and word diffs, hunk review, change summaries, table and revision diffs,
//!   code suggestions, inline edits, image compare, comment threads and the review center.
//! - [`merge`]: the 3-way merge-conflict editor (markers, picks, result, keymap) and Conflict
//!   Radar (agents on parallel branches, overlapping edits, picks and merge order).
//! - [`fidelity`]: the affectUI fidelity pass (plan #32): the exact Chat Composer's approval
//!   modes, attachment kinds and recording trace; Comment Thread and Comments Panel.
//! - [`chatfid`]: fidelity chunk 2: Model Picker rows and tray, Run Controls turns and phases.
//! - [`agentsfid`]: fidelity batch, family B: Agent Findings progress and feed, Agent Inbox tasks.
//! - [`crewfid`]: fidelity batch, family B part 2: Agent Crew tree, handoffs, steering, projects.
//! - [`cursorsfid`]: fidelity batch, family B part 3: Agent Cursors document, pointers,
//!   suggestions, comments and the demo loop.
//! - [`graphfid`]: fidelity batch, family B part 3: Agent Graph nodes, calls, orbit layout, links.
//! - [`handofffid`]: fidelity batch, family B part 3: Agent Handoff relay, items, tasks, assign.
//! - [`lanesfid`]: fidelity batch, family B part 3: Agent Lanes tasks, window, hand-offs, counts.
//! - [`ledgerfid`]: fidelity batch, family B part 4: Agent Ledger changes, undo chains, demo.
//! - [`monitorfid`]: fidelity batch, family B part 4: Agent Monitor states, steps, strip, demo.
//! - [`queuefid`]: fidelity batch, family B part 4: Agent Queue simulation, stats, depth chart.
//! - [`autonomyfid`]: fidelity batch, family B part 6: Autonomy Dial levels, locks and the asks-first rule.
//! - [`awayfid`]: fidelity batch, family B part 6: Away Summary items, order, counts and folding.
//! - [`collabfid`]: fidelity batch, family B part 6: Collaboration Stream messages, quotes, mentions.
//! - [`ctxmapfid`]: fidelity batch, family B part 6: Context Map blocks, ordered treemap, compaction.
//! - [`dtreefid`]: fidelity batch, family B part 8: Decision Tree decisions, branches, follow-ups, flags and keys.
//! - [`fallbackfid`]: fidelity batch, family B part 8: Fallback Chain attempts, states and timing.
//! - [`delegatefid`]: fidelity batch, family B part 8: Delegate Board agents, tasks, fit and run status.
//! - [`costfid`]: fidelity batch, family B part 6: Cost Waterfall steps, rows, axis and formats.
//! - [`recapfid`]: fidelity batch, family B part 4: Agent Recap events, curve, counts.
//! - [`seqfid`]: fidelity batch, family B part 4: Agent Sequence rows, frames, bars, demo run.
//! - [`parallelfid`]: fidelity batch, family B part 9: Parallel Attempts race, scores, best and pick.
//! - [`judgefid`]: fidelity batch, family B part 9: Judge Bracket seeds, verdicts, current match and winner path.
//! - [`memoryfid`]: fidelity batch, family B part 9: Memory Panel groups, filter, search, status and undo quote.
//! - [`runningfid`]: fidelity batch, family B part 9: Running Now tasks, steps, counts and pill label.
//! - [`planfid`]: fidelity batch, family B part 9: Plan vs Actual schedule, slips, knock-on shifts and ETA.
//! - [`rootfid`]: fidelity batch, family B part 9: Root Cause Tree checks, gates, found route and evidence.
//! - [`toolmapfid`]: fidelity batch, family B part 9: Tool Access Map per-agent tool states, counts, calls and replay.
//! - [`schedfid`]: Scheduled Runs: a week of usage-limited sessions, auto or hand booking, split tasks.
//! - [`ratefid`]: Rate Limit Planner: the day's token load against a per-minute limit, draggable runs, spread out, fit.
//! - [`tracefid`]: Trace Waterfall: an agent run as a live span tree on a time axis, folding, details per span.
//! - [`fanfid`]: Fan-out Grid: a job's tasks as a live grid of status cells, filters, details, retry.
//! - [`difftlfid`]: the exact Diff Timeline run view (affectUI, family B part 9)
//! - [`thinkfid`]: Thinking Selector family: effort scales, stepping and snapping, tick tint, the dotted max fill.
//! - [`reasonfid`]: Thinking Selector: Reasoning panel keys, context labels
//! - [`effmenufid`]: Thinking Selector menus: roving highlight, context windows, models with effort.
//! - [`nozzlefid`]: Thinking Selector: Nozzle line gradient, glow and stops (Oxyn original)
//! - [`stairsfid`]: Thinking Selector: Effort stairs budgets (Oxyn original)
//! - [`ctxvisfid`]: Context Visualizer: kinds, shares, field of dots, slabs, compaction and turns left (variants 2)
//! - [`storagefid`]: Storage: categories, dot runs, clearing, unused skills and a fill forecast (variants 2)
//! - [`attachfid`]: Attachments: file types and badges, size checks, upload timing, drop lanes and the print fan (variants 2)
//! - [`gooeyfid`]: Menu Motion: gooey menu rows, heights, stagger, open side, liquid tabs and drops (variants 2)
//! - [`themefid`]: Theme Selector family (variants 3): Background picker, Theme grid, Theme split, Accent and density.
//! - [`screensfid`]: Screens 1: app frames (Inset, Flat, three-pane) and the AI Chat screens
//! - [`screens2fid`]: Screens 2: the agent screens (Orchestration, Thread and the rest) on the Screens 1 frame
//! - [`workscr`]: Screens S2: the workspace screens (Folders, Integrations, Kanban, CRM, Analytics)
//! - [`agentscr`]: Screens S3: agent ops I (Thread, Inbox, Inbox Board, Tasks, Activity, Memory)
//! - [`buildscr`]: Screens S5: orchestration and builders (Evals; Orchestration and Workflow Builder states)
//! - [`opsscr`]: Screens S4: agent ops II (Agents, Skills, Schedules, Usage, Permissions, Run Detail)
//! - [`setscr`]: Screens S6: Settings I (Profile, Preferences, Notifications, Privacy, General)
//! - [`setscr2`]: Screens S7: Settings II (Members, Models, Security, Billing, Get started)
//! - [`onbscr`]: Screens S8: Onboarding wizard (Website, Brand, Tone, Ready)
//! - [`taskdeck`]: Screens S9 (Agent apps I): Taskdeck (Start, Task, Board)
//! - [`cmdstudio`]: Screens S10 (Agent apps II): Command Center and Session Studio
//! - [`fleetwin`]: Screens S11 (Agent apps III): Fleet Window
//! - [`wscoder`]: Screens S11 (Agent apps III): Workspace Coder
//! - [`cmdboard`]: Screens S11 (Agent apps III): Command Board
//! - [`answers`]: fidelity batch, family A: AI Answer markup and streaming, Cited Answer
//!   sentences and quotes, Message Actions, Thinking Loader phases, Thinking State steps and the
//!   Writing Assistant's sample writer and dictation.
//! - [`tools`]: tool calls, the tool-kind map, JSON trees and link unfurls.
//!
//! Like the rest of the core, time is passed in as [`crate::ClockMs`]; nothing reads a clock.

pub mod agentscr;
pub mod agentsfid;
pub mod answers;
pub mod attachfid;
pub mod autonomyfid;
pub mod awayfid;
pub mod buildscr;
pub mod chatfid;
pub mod cmdboard;
pub mod cmdstudio;
pub mod collabfid;
pub mod compose;
pub mod confirm;
pub mod costfid;
pub mod crewfid;
pub mod ctxmapfid;
pub mod ctxvisfid;
pub mod cursorsfid;
pub mod delegatefid;
pub mod diff;
pub mod difftlfid;
pub mod dtreefid;
pub mod edit;
pub mod effmenufid;
pub mod fallbackfid;
pub mod fanfid;
pub mod fidelity;
pub mod fields;
pub mod fleet;
pub mod fleetwin;
pub mod gates;
pub mod gooeyfid;
pub mod graphfid;
pub mod handofffid;
pub mod home;
pub mod input;
pub mod judgefid;
pub mod lanesfid;
pub mod ledgerfid;
pub mod memoryfid;
pub mod merge;
pub mod meters;
pub mod monitorfid;
pub mod nozzlefid;
pub mod onbscr;
pub mod opsscr;
pub mod parallelfid;
pub mod plan;
pub mod planfid;
pub mod queuefid;
pub mod ratefid;
pub mod reasonfid;
pub mod recapfid;
pub mod rootfid;
pub mod runningfid;
pub mod schedfid;
pub mod screens2fid;
pub mod screensfid;
pub mod seqfid;
pub mod setscr;
pub mod setscr2;
pub mod stairsfid;
pub mod storagefid;
pub mod taskdeck;
pub mod text;
pub mod themefid;
pub mod thinkfid;
pub mod toolmapfid;
pub mod tools;
pub mod tracefid;
pub mod upload;
pub mod workscr;
pub mod wscoder;

/// Smoothstep ease used for rewinds and fills (0..1 in, 0..1 out).
pub fn smooth(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// `m:ss` for a number of seconds.
pub fn mss(secs: u64) -> String {
    format!("{}:{:02}", secs / 60, secs % 60)
}

/// Splits `text` on backticks: `(segment, is_code)` pairs, so a title like
/// ``"Run `vercel deploy --prod`"`` sets its command in the mono face.
pub fn code_spans(text: &str) -> Vec<(String, bool)> {
    let mut out = Vec::new();
    for (i, part) in text.split('`').enumerate() {
        if !part.is_empty() {
            out.push((part.to_string(), i % 2 == 1));
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spans_and_time() {
        assert_eq!(
            code_spans("Run `ls -la` now"),
            vec![
                ("Run ".to_string(), false),
                ("ls -la".to_string(), true),
                (" now".to_string(), false)
            ]
        );
        assert_eq!(mss(62), "1:02");
        assert!((smooth(0.5) - 0.5).abs() < 1e-6);
    }
}
