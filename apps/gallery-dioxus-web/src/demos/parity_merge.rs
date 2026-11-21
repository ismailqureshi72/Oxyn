//! Plan #31 demos: the 3-way merge-conflict editor and Conflict Radar. Sample code, branches and
//! agents are invented.

use dioxus::prelude::*;
use oxyn_core::{
    ClockMs,
    kit::merge::{MergeFile, Pick, RadarAgent, RadarEdit, RadarFile, Seg, Suggestion},
};
use oxyn_dioxus::{ConflictRadar, MergeConflict};

use super::{Code, mk, rng, sel, tog, var};
use crate::{Align, Demo, Vals};

fn status(msg: Signal<String>) -> Element {
    rsx! { span { style: "font-size:12px;color:var(--oxyn-color-fg-muted);min-height:16px;overflow-wrap:anywhere", role: "status", "{msg}" } }
}

// ---------- merge conflict ----------

const SESSION: &str = "use std::time::Duration;

use crate::store::TokenStore;

pub struct Session {
    pub user: UserId,
    pub token: Token,
    pub expires_at: Instant,
}

<<<<<<< main
const TTL: Duration = Duration::from_secs(300);
||||||| 9f8e7d6
const TTL: Duration = Duration::from_secs(600);
=======
const TTL: Duration = config::session_ttl();
>>>>>>> feature/token-refresh

impl Session {
    pub fn is_expired(&self, now: Instant) -> bool {
        now >= self.expires_at
    }

<<<<<<< main
    pub fn refresh(&mut self, store: &TokenStore) -> Result<(), AuthError> {
        self.token = store.rotate(&self.token)?;
        log::info!(\"session refreshed for {}\", self.user);
||||||| 9f8e7d6
    pub fn refresh(&mut self, store: &TokenStore) -> Result<(), AuthError> {
        self.token = store.rotate(&self.token)?;
=======
    pub async fn refresh(&mut self, store: &TokenStore) -> Result<(), AuthError> {
        self.token = store.rotate_async(&self.token).await?;
>>>>>>> feature/token-refresh
        self.expires_at = Instant::now() + TTL;
        Ok(())
    }

    pub fn revoke(self, store: &TokenStore) {
<<<<<<< main
        store.revoke(&self.token);
||||||| 9f8e7d6
        store.revoke(self.token);
=======
        store.revoke(self.token);
        metrics::counter!(\"session.revoked\").increment(1);
>>>>>>> feature/token-refresh
    }
}
";

const LIMITS: &str = "[server]
port = 8080
workers = 4

[limits]
<<<<<<< main
requests_per_minute = 120
burst = 20
=======
requests_per_minute = 300
>>>>>>> feature/token-refresh

[log]
level = \"info\"
";

fn sample_files(two: bool) -> Vec<MergeFile> {
    let mut f = MergeFile::from_markers("src/auth/session.rs", SESSION);
    let sugs = [
        Some(Suggestion {
            lines: vec!["const TTL: Duration = config::session_ttl();".into()],
            rationale: "Both sides moved off 600 s. Incoming reads it from config, which can still default to 300 s, so take incoming.".into(),
            confidence: Some("high"),
        }),
        Some(Suggestion {
            lines: vec![
                "    pub async fn refresh(&mut self, store: &TokenStore) -> Result<(), AuthError> {".into(),
                "        self.token = store.rotate_async(&self.token).await?;".into(),
                "        log::info!(\"session refreshed for {}\", self.user);".into(),
            ],
            rationale: "Keep the async rotation from incoming and the log line from current; neither change undoes the other.".into(),
            confidence: Some("medium"),
        }),
        None,
    ];
    let mut k = 0;
    for s in f.segs.iter_mut() {
        if let Seg::Conflict(c) = s {
            c.suggestion = sugs.get(k).cloned().flatten();
            k += 1;
        }
    }
    let mut v = vec![f];
    if two {
        v.push(MergeFile::from_markers("config/limits.toml", LIMITS));
    }
    v
}

fn merge_conflict(v: &Vals, _: ClockMs) -> Element {
    let mut msg = use_signal(String::new);
    let two = v.s("files") == "two";
    let picks: Vec<(usize, usize, Pick)> = match v.s("start") {
        "partly" => vec![(0, 0, Pick::Theirs)],
        "resolved" => {
            let mut p = vec![
                (0, 0, Pick::Theirs),
                (0, 1, Pick::Agent),
                (0, 2, Pick::Theirs),
            ];
            if two {
                p.push((1, 0, Pick::Ours));
            }
            p
        }
        _ => vec![],
    };
    rsx! {
        div { style: "width:100%;display:flex;flex-direction:column;gap:6px",
            for k in [format!("{}{}{}{}{}{}{}{}", v.s("layout"), v.s("files"), v.s("labels"), v.b("base"), v.b("suggest"), v.b("advance"), v.s("fold"), v.s("start"))] {
                MergeConflict { key: "{k}", files: sample_files(two), ours_ref: "main", theirs_ref: "feature/token-refresh", base_ref: "9f8e7d6",
                    layout: v.string("layout"), show_base: v.b("base"), labels: v.string("labels"), auto_advance: v.b("advance"),
                    suggestions: v.b("suggest"), fold_after: v.u("fold"), default_picks: picks.clone(),
                    on_resolve: move |(p, n, pick): (String, usize, Option<Pick>)| msg.set(match pick {
                        Some(x) => format!("onResolve({p}, conflict {}, {})", n + 1, x.tag()),
                        None => format!("onResolve({p}, conflict {}, reopened)", n + 1),
                    }),
                    on_mark: move |p: String| msg.set(format!("onMark({p})")),
                    on_complete: move |_| msg.set("onComplete()".to_string()),
                    on_abort: move |_| msg.set("onAbort()".to_string()),
                }
            }
            {status(msg)}
        }
    }
}

fn merge_conflict_code(v: &Vals) -> String {
    let mut c = Code::new("MergeConflict")
        .pre("// files with git conflict markers (merge.conflictStyle = diff3 adds the base)")
        .pre("let text = std::fs::read_to_string(\"src/auth/session.rs\")?;");
    if v.s("files") == "two" {
        c = c.pre("let toml = std::fs::read_to_string(\"config/limits.toml\")?;").raw(
            "files",
            "vec![MergeFile::from_markers(\"src/auth/session.rs\", &text), MergeFile::from_markers(\"config/limits.toml\", &toml)]",
        );
    } else {
        c = c.raw(
            "files",
            "vec![MergeFile::from_markers(\"src/auth/session.rs\", &text)]",
        );
    }
    c = c
        .raw("ours_ref", "\"main\"")
        .raw("theirs_ref", "\"feature/token-refresh\"")
        .raw("base_ref", "\"9f8e7d6\"")
        .s("layout", v.s("layout"), "three-pane")
        .s("labels", v.s("labels"), "git");
    if !v.b("base") {
        c = c.raw("show_base", "false");
    }
    if !v.b("suggest") {
        c = c.raw("suggestions", "false");
    }
    if !v.b("advance") {
        c = c.raw("auto_advance", "false");
    }
    if v.s("fold") != "6" {
        c = c.raw("fold_after", v.s("fold"));
    }
    c.raw(
        "on_resolve",
        "move |(path, n, pick)| host.resolve(path, n, pick)",
    )
    .raw("on_complete", "move |_| host.commit_merge()")
    .done()
}

// ---------- conflict radar ----------

fn radar_data(now: ClockMs, live: bool, many: bool) -> (Vec<RadarAgent>, Vec<RadarFile>) {
    // in the live variant the third agent finishes six seconds in
    let working = live && now.0 < crate::BASE.0 + 6_000;
    let mut agents = vec![
        RadarAgent::new("atlas", "Atlas", "agent/atlas · gpt-5"),
        RadarAgent::new("birch", "Birch", "agent/birch · claude"),
        RadarAgent::new("cove", "Cove", "agent/cove · gemini").running(working),
    ];
    let mut files = vec![
        RadarFile::new(
            "src/auth/session.rs",
            vec![
                RadarEdit::new("atlas", 12, 48)
                    .way("Refresh tokens in middleware before every request."),
                RadarEdit::new("birch", 30, 64)
                    .way("Refresh tokens on demand when a call returns 401."),
            ],
        ),
        RadarFile::new(
            "src/api/routes.rs",
            vec![
                RadarEdit::new("atlas", 4, 18),
                RadarEdit::new("cove", 60, 92).active(working),
            ],
        ),
        RadarFile::new(
            "src/ui/settings.tsx",
            vec![
                RadarEdit::new("birch", 20, 44).way("Group the session options under Security."),
                RadarEdit::new("cove", 36, 70)
                    .way("Move session options into a new Sessions tab.")
                    .active(working),
            ],
        ),
        RadarFile::new("docs/auth.md", vec![RadarEdit::new("birch", 1, 26)]),
        RadarFile::new("Cargo.toml", vec![RadarEdit::new("atlas", 30, 33)]),
    ];
    if many {
        agents.push(RadarAgent::new("drift", "Drift", "agent/drift · local"));
        files.push(RadarFile::new(
            "src/store/tokens.rs",
            vec![
                RadarEdit::new("drift", 8, 40).way("Store refresh tokens hashed with a pepper."),
                RadarEdit::new("atlas", 22, 30).way("Add an index on the token expiry column."),
            ],
        ));
    }
    (agents, files)
}

fn conflict_radar(v: &Vals, now: ClockMs) -> Element {
    let mut msg = use_signal(String::new);
    let live = v.b("live");
    let (agents, files) = radar_data(now, live, v.s("agents") == "4");
    rsx! {
        div { style: "width:100%;max-width:880px;display:flex;flex-direction:column;gap:6px",
            for k in [format!("{}{}{}{}{}", v.s("agents"), v.b("heat"), v.b("order"), v.b("only"), v.b("open"))] {
                ConflictRadar { key: "{k}", agents: agents.clone(), files: files.clone(), title: v.string("title"),
                    heat: v.b("heat"), order: v.b("order"), conflicts_only: v.b("only"), default_open: v.b("open"),
                    on_resolve: move |(p, a): (String, String)| msg.set(format!("onResolve({p}, {a})")),
                    on_merge: move |_| msg.set("onMerge()".to_string()),
                }
            }
            {status(msg)}
        }
    }
}

fn conflict_radar_code(v: &Vals) -> String {
    let mut c = Code::new("ConflictRadar")
        .pre("let agents = vec![RadarAgent::new(\"atlas\", \"Atlas\", \"agent/atlas\"), RadarAgent::new(\"birch\", \"Birch\", \"agent/birch\")];")
        .pre("let files = vec![RadarFile::new(\"src/auth/session.rs\", vec![")
        .pre("    RadarEdit::new(\"atlas\", 12, 48).way(\"Refresh in middleware.\"),")
        .pre("    RadarEdit::new(\"birch\", 30, 64).way(\"Refresh on 401.\"),")
        .pre("])];")
        .raw("agents", "agents")
        .raw("files", "files")
        .s("title", v.s("title"), "Parallel work")
        .b("heat", v.b("heat"))
        .b("order", v.b("order"))
        .b("conflicts_only", v.b("only"));
    c = c
        .raw(
            "on_resolve",
            "move |(path, agent)| picks.write().insert(path, agent)",
        )
        .raw("on_merge", "move |_| host.merge_branches()");
    c.done()
}

pub(crate) fn demos() -> Vec<Demo> {
    vec![
        Demo {
            controls: vec![
                sel(
                    "layout",
                    "Layout",
                    &["three-pane", "result-center", "inline"],
                    "three-pane",
                ),
                sel("files", "Files", &["one", "two"], "one"),
                sel("labels", "Labels", &["git", "editor"], "git"),
                sel("start", "Start", &["fresh", "partly", "resolved"], "fresh"),
                tog("base", "Show base", true),
                tog("suggest", "Agent suggestions", true),
                tog("advance", "Auto-advance", true),
                rng("fold", "Fold clean runs after", 0.0, 12.0, 2.0, "6"),
            ],
            variants: vec![
                var(
                    "three-pane",
                    "Ours · Base · Theirs above an editable Result.",
                    &[],
                ),
                var(
                    "result-center",
                    "Ours · Result · Theirs side by side; push a side into the middle.",
                    &[("layout", "result-center")],
                ),
                var(
                    "inline-stacked",
                    "One column: each open conflict stacks its sides above the actions.",
                    &[("layout", "inline")],
                ),
                var(
                    "multi-file",
                    "Two files: a file strip, Mark resolved per file, then Complete merge.",
                    &[("files", "two")],
                ),
                var(
                    "two-way",
                    "Base hidden: compare the two sides only.",
                    &[("base", "false")],
                ),
                var(
                    "editor-labels",
                    "Current / Incoming wording instead of Ours / Theirs.",
                    &[("labels", "editor")],
                ),
                var(
                    "partly-resolved",
                    "One conflict already taken from theirs.",
                    &[("start", "partly")],
                ),
                var(
                    "ready-to-mark",
                    "Every conflict settled; Mark resolved is enabled.",
                    &[("start", "resolved")],
                ),
                var(
                    "no-agent",
                    "No suggestions: pick a side or edit by hand.",
                    &[("suggest", "false")],
                ),
            ],
            render: merge_conflict,
            code: merge_conflict_code,
            keyboard: vec![
                ("Alt + ↓ / ]", "Next unresolved conflict (wraps)"),
                ("Alt + ↑ / [", "Previous unresolved conflict"),
                ("Alt + 1", "Accept ours (current)"),
                ("Alt + 2", "Accept theirs (incoming)"),
                ("Alt + 3", "Accept both, ours first"),
                ("Alt + 4", "Keep the base"),
                ("Alt + 5", "Accept the agent's suggestion"),
                ("Alt + R", "Reopen the focused conflict"),
                ("E", "Edit the focused conflict by hand"),
                ("⌘ + Enter / Ctrl + Enter", "Apply a manual edit"),
                ("Esc", "Cancel a manual edit"),
                ("⌘ + Z / Ctrl + Z", "Undo the last pick, reopen or mark"),
                (
                    "⌘ + Shift + Enter / Ctrl + Shift + Enter",
                    "Mark the file resolved",
                ),
            ],
            a11y: vec![
                "The editor is a labelled region (“Merge conflicts: feature/token-refresh into main”); each pane and each conflict is a labelled group, and each conflict's actions are a toolbar.",
                "Every conflict says its state in text (“unresolved”, “ours”, “⚑ markers left”) and by a coloured edge; colour is never the only cue.",
                "A polite live region reads “Conflict 2 of 3 in src/auth/session.rs, resolved, took theirs. 1 left.” after each move or pick.",
                "Buttons carry aria-keyshortcuts; Mark resolved and Complete merge stay focusable when disabled and show why next to them (“1 conflict left”).",
                "Abort asks inline (role=alertdialog) and has no shortcut.",
            ],
            responsive: vec![
                "Container query: under 600 px the three-pane inputs become tabs (Ours · Base · Theirs) above one column with the Result below; result-center turns into Ours · Result · Theirs tabs under 760 px, opening on Result.",
                "Under 480 px line numbers narrow, the action buttons share the width two per row and Mark resolved spans the footer.",
                "Long lines wrap inside each pane; the panes never scroll sideways.",
            ],
            align: Align::Start,
            height: 820,
            ..mk(
                "merge-conflict",
                "Merge conflict",
                "checkpoints-git",
                "A 3-way merge editor: Ours, Base and Theirs panes plus a Result; accept ours, theirs, both or base per conflict, take the agent's suggestion or edit by hand, step through conflicts, then mark the file resolved and complete the merge.",
                "MergeConflict",
                "MergeConflictEl",
                "merge-conflict",
                "kit/merge.rs",
            )
        },
        Demo {
            controls: vec![
                sel("agents", "Agents", &["3", "4"], "3"),
                tog("live", "Live (one agent still writing)", true),
                tog("heat", "Overlap heat", false),
                tog("order", "Suggested merge order", false),
                tog("only", "Conflicts only", false),
                tog("open", "Open first conflict", true),
                sel(
                    "title",
                    "Title",
                    &["Parallel work", "Agent branches"],
                    "Parallel work",
                ),
            ],
            variants: vec![
                var(
                    "radar",
                    "Rows per file, a column per agent; one agent still writing.",
                    &[],
                ),
                var(
                    "settled",
                    "Everyone done: pick a way per conflict, then Merge.",
                    &[("live", "false")],
                ),
                var(
                    "heat",
                    "Conflicting cells tinted by how many lines overlap.",
                    &[("heat", "true"), ("live", "false")],
                ),
                var(
                    "merge-order",
                    "Least-contested branch first.",
                    &[("order", "true"), ("live", "false")],
                ),
                var(
                    "conflicts-only",
                    "Only the files where agents clash, four agents.",
                    &[("only", "true"), ("agents", "4"), ("live", "false")],
                ),
                var(
                    "collapsed",
                    "Ways stay folded until a file is opened.",
                    &[("open", "false"), ("live", "false")],
                ),
            ],
            render: conflict_radar,
            code: conflict_radar_code,
            keyboard: vec![
                (
                    "Tab",
                    "Moves through Merge, the agents, the conflicting files and the ways to settle the one that is open",
                ),
                (
                    "Enter / Space",
                    "On a conflicting file: show or hide its ways; on a way: keep it",
                ),
                (
                    "↑ / ↓ / ← / →",
                    "Move between the ways of the open conflict (radio group)",
                ),
            ],
            a11y: vec![
                "The grid is a table: agents are column headers, files are row headers, and every cell says who changed which lines and whether it was kept (“Birch changed lines 30 to 64, kept”).",
                "Conflicts and settled files are named in text (“Conflict”, “✓ Birch”), not only by colour.",
                "The ways to settle a conflict are a radio group.",
                "The detail line under the table is a polite live region: pointing at or focusing a bar or an agent reads its numbers (files, open conflicts, kept).",
            ],
            responsive: vec![
                "Container query: below 440 px agent names and folders drop (monograms stay), the columns shrink and a way's sentence moves under its name.",
                "Merge is disabled with the reason beside it (“2 conflicts · 1 working”).",
            ],
            align: Align::Start,
            height: 560,
            live: true,
            ..mk(
                "conflict-radar",
                "Conflict radar",
                "checkpoints-git",
                "Agents working on their own branches at once and where they get in each other's way: a row per file, a column per agent, a line between agents that changed the same lines, each agent's way in one sentence to pick from, and Merge once everything is settled.",
                "ConflictRadar",
                "ConflictRadarEl",
                "conflict-radar",
                "kit/merge.rs",
            )
        },
    ]
}
