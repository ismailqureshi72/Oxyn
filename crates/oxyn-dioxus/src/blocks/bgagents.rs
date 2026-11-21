//! Background agents (Dioxus).

use dioxus::prelude::*;
use oxyn_core::ClockMs;
use oxyn_core::blocks::bgagents::{BgAgents, FILTERS, MODELS, Phase, REPOS, STEPS, files};

use super::status;
use crate::aikit::{cls, cmd, ico, key_of, live, mark};

/// A chip that opens a small menu of options (radio items, arrow keys, Escape closes).
#[allow(clippy::too_many_arguments)]
fn chip_menu(
    label: &'static str,
    value: String,
    icon: Element,
    opts: Vec<(String, Option<&'static str>)>,
    sel: usize,
    open: bool,
    on_toggle: EventHandler<()>,
    on_pick: EventHandler<usize>,
) -> Element {
    rsx! {
        div { class: "oxbk-cm",
            button { class: "oxaik-chip", r#type: "button", "aria-haspopup": "menu", "aria-expanded": "{open}", "aria-label": "{label}: {value}",
                onclick: move |_| on_toggle.call(()),
                {icon} span { class: "oxbk-cm-v", "{value}" } {ico("chevron", 11.0)}
            }
            if open {
                ul { class: "oxbk-menu", role: "menu", "aria-label": "{label}",
                    onkeydown: move |e: KeyboardEvent| if key_of(&e) == "Escape" { on_toggle.call(()) },
                    for (i, (o, m)) in opts.into_iter().enumerate() {
                        li { key: "{i}", role: "none",
                            button { class: cls("oxbk-mi", i == sel), r#type: "button", role: "menuitemradio", "aria-checked": "{i == sel}",
                                onclick: move |_| on_pick.call(i),
                                if let Some(m) = m { {mark(m, 13.0)} }
                                span { "{o}" }
                                if i == sel { {ico("check", 12.0)} }
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Background agents: coding jobs handed to agents that run off-screen. The job field takes the
/// task plus the repo, branch and model to use (chips with menus; Cmd/Ctrl+Enter starts). Each card
/// goes queued, setting up, running (its current step has a moving sheen), then lands ready for
/// review with lines added/removed and checks; Create pull request swaps to a
/// GitHub-marked PR link, then Merge. Filter tabs with counts split the list; cards expand to their step
/// log and changed files; running tasks cancel, failed or cancelled ones retry, finished ones
/// archive with Undo.
#[component]
pub fn BackgroundAgents(
    /// Host clock.
    now: ClockMs,
    /// Start filter (0 all, 1 running, 2 ready, 3 merged).
    #[props(default)]
    filter: usize,
    /// The next started task fails its tests (error scenario).
    #[props(default)]
    fail_next: bool,
    /// Open this task's details at the start (task id).
    #[props(default)]
    open: Option<usize>,
) -> Element {
    let mut ba = use_signal(move || {
        let mut b = BgAgents::new(fail_next, now.0);
        b.filter = filter.min(3);
        b.open = open;
        b
    });
    let mut said = use_signal(String::new);
    let mut draft = use_signal(String::new);
    let mut menu = use_signal(|| 0u8);
    let n = now.0;
    let b = ba.read().clone();
    let mut act = move |id: String| {
        let x = ba.write().press(&id, n);
        if let Some(x) = x {
            said.set(x);
        }
    };
    let mut start = move || {
        let t = draft.read().clone();
        let x = ba.write().start(&t, n);
        if let Some(x) = x {
            said.set(x);
            draft.set(String::new());
        }
    };
    let counts = b.counts(n);
    let (r, br, md) = b.pick;
    let mut toggle = move |k: u8| {
        let cur = *menu.read();
        menu.set(if cur == k { 0 } else { k });
    };
    rsx! {
        section { class: "oxaik oxbk oxbk-ba", "aria-label": "Background agents",
            header { class: "oxaik-hd", div { class: "oxaik-hl", h3 { class: "oxaik-h", "Background agents" } p { class: "oxaik-sub", "Tasks run on their own machines; you review the result." } } }
            div { class: "oxbk-ba-comp",
                textarea { class: "oxaik-ta oxbk-ba-ta", rows: "2", value: "{draft}", placeholder: "Describe a task for an agent…", "aria-label": "Task",
                    oninput: move |e| draft.set(e.value()),
                    onkeydown: move |e: KeyboardEvent| if key_of(&e) == "Enter" && cmd(&e) { e.prevent_default(); start() },
                }
                div { class: "oxaik-row oxbk-ba-chips",
                    {chip_menu("Repository", REPOS[r].0.to_string(), mark("github", 12.0), REPOS.iter().map(|x| (x.0.to_string(), None)).collect(), r, *menu.read() == 1,
                        EventHandler::new(move |_| toggle(1)), EventHandler::new(move |i| { menu.set(0); act(format!("ba-repo-{i}")) }))}
                    {chip_menu("Branch", REPOS[r].1[br].to_string(), ico("branch", 12.0), REPOS[r].1.iter().map(|x| (x.to_string(), None)).collect(), br, *menu.read() == 2,
                        EventHandler::new(move |_| toggle(2)), EventHandler::new(move |i| { menu.set(0); act(format!("ba-branch-{i}")) }))}
                    {chip_menu("Model", MODELS[md].1.to_string(), mark(MODELS[md].0, 12.0), MODELS.iter().map(|x| (x.1.to_string(), Some(x.0))).collect(), md, *menu.read() == 3,
                        EventHandler::new(move |_| toggle(3)), EventHandler::new(move |i| { menu.set(0); act(format!("ba-model-{i}")) }))}
                    span { class: "oxaik-mute oxbk-ba-hint", "⌘ Enter" }
                    button { class: "oxaik-btn sm pri oxbk-ba-go", r#type: "button", disabled: draft.read().trim().is_empty(), onclick: move |_| start(), "Start" }
                }
            }
            div { class: "oxbk-filter oxaik-scroll", role: "tablist", "aria-label": "Filter tasks",
                onkeydown: move |e: KeyboardEvent| {
                    let k = key_of(&e);
                    if matches!(k.as_str(), "ArrowLeft" | "ArrowRight" | "Home" | "End") {
                        e.prevent_default();
                        let x = ba.write().key(&k, n);
                        if let Some(x) = x { said.set(x) }
                    }
                },
                for (i, f) in FILTERS.iter().enumerate() {
                    button { key: "{i}", class: cls("oxaik-sg", b.filter == i), r#type: "button", role: "tab", "aria-selected": "{b.filter == i}", tabindex: if b.filter == i { "0" } else { "-1" },
                        onclick: move |_| act(format!("ba-f-{i}")),
                        "{f}" span { class: "oxbk-cnt oxaik-num", "{counts[i]}" }
                    }
                }
            }
            ul { class: "oxbk-ba-list",
                if b.visible(n).is_empty() { li { class: "oxaik-mute oxbk-empty", "No tasks here yet." } }
                for t in b.visible(n).into_iter().cloned() {
                    {
                        let ph = BgAgents::phase(&t, n);
                        let (lab, hue) = ph.label();
                        let (el, el_words) = BgAgents::time(&t, n);
                        let open = b.open == Some(t.id);
                        let id = t.id;
                        let pr = BgAgents::pr(&t, n);
                        let archived = t.archived.is_some();
                        let k = match ph { Phase::Running(k) | Phase::Failed(k) => k, Phase::Ready | Phase::Merged => STEPS.len(), _ => 0 };
                        rsx! {
                            li { key: "{id}", class: "oxbk-ba-card", "data-ph": "{lab}", "data-arch": "{archived}",
                                if archived {
                                    div { class: "oxaik-row oxaik-between oxbk-ba-undo",
                                        span { class: "oxaik-mute", "Archived “{t.title}”" }
                                        button { class: "oxaik-btn sm", r#type: "button", onclick: move |_| act(format!("ba-undo-{id}")), {ico("undo", 12.0)} "Undo" }
                                    }
                                } else {
                                    button { class: "oxbk-ba-head", r#type: "button", "aria-expanded": "{open}", "aria-controls": "oxbk-ba-d-{id}",
                                        onclick: move |_| act(format!("ba-open-{id}")),
                                        span { class: "oxbk-ba-title", "{t.title}" }
                                        {ico(if open { "chevron" } else { "chevron-right" }, 12.0)}
                                    }
                                    p { class: "oxaik-row oxaik-mute oxbk-ba-meta",
                                        {mark("github", 12.0)} span { class: "oxbk-ba-repo", "{REPOS[t.repo].0}" }
                                        {ico("branch", 12.0)} span { class: "oxbk-ba-br", "{REPOS[t.repo].1[t.branch.min(REPOS[t.repo].1.len() - 1)]}" }
                                        {mark(MODELS[t.model].0, 12.0)} span { "{MODELS[t.model].1}" }
                                    }
                                    div { class: "oxaik-row oxbk-ba-state",
                                        {status(lab, hue)}
                                        if ph.busy() { span { class: "oxaik-shim", "{BgAgents::step_line(&t, n)}" } } else { span { class: "oxaik-mute", "{BgAgents::step_line(&t, n)}" } }
                                        span { class: "oxaik-mute oxaik-num", "aria-label": "{el_words}", "{el}" }
                                        if matches!(ph, Phase::Ready | Phase::Merged) && t.diff.2 > 0 {
                                            span { class: "oxbk-diff oxaik-num", "aria-label": "{t.diff.0} lines added, {t.diff.1} removed, {t.diff.2} files changed",
                                                b { class: "oxbk-add", "+{t.diff.0}" } b { class: "oxbk-del", "−{t.diff.1}" } span { class: "oxaik-mute", "· {t.diff.2} files" } }
                                        }
                                    }
                                    div { class: "oxaik-row oxbk-ba-acts",
                                        if ph.busy() { button { class: "oxaik-btn sm", r#type: "button", onclick: move |_| act(format!("ba-cancel-{id}")), "Cancel" } }
                                        if matches!(ph, Phase::Failed(_) | Phase::Cancelled) { button { class: "oxaik-btn sm", r#type: "button", onclick: move |_| act(format!("ba-retry-{id}")), {ico("retry", 12.0)} "Retry" } }
                                        if ph == Phase::Ready {
                                            {match pr {
                                                None => rsx! { button { class: "oxaik-btn sm pri", r#type: "button", onclick: move |_| act(format!("ba-pr-{id}")), {ico("merge", 12.0)} "Create pull request" } },
                                                Some(Err(())) => rsx! { button { class: "oxaik-btn sm", r#type: "button", disabled: true, "aria-busy": "true", span { class: "oxaik-shim", "Creating pull request" } } },
                                                Some(Ok(p)) => rsx! {
                                                    span { class: "oxaik-pill", {mark("github", 12.0)} "#{p}" }
                                                    button { class: "oxaik-btn sm pri", r#type: "button", onclick: move |_| act(format!("ba-merge-{id}")), "Merge" }
                                                },
                                            }}
                                        }
                                        if ph == Phase::Merged { span { class: "oxaik-pill", "data-hue": "violet", {mark("github", 12.0)} "#{t.pr.map_or(0, |p| p.0)} merged" } }
                                        if !ph.busy() { button { class: "oxaik-btn sm oxbk-ghost", r#type: "button", onclick: move |_| act(format!("ba-archive-{id}")), {ico("archive", 12.0)} "Archive" } }
                                    }
                                    if open {
                                        div { class: "oxbk-ba-det oxaik-fade", id: "oxbk-ba-d-{id}",
                                            ol { class: "oxbk-ba-steps", "aria-label": "Steps",
                                                for (s, st) in STEPS.iter().enumerate() {
                                                    {
                                                        let (ic, h, w) = if matches!(ph, Phase::Failed(f) if f == s) { ("x-circle", "red", "failed") }
                                                            else if s < k { ("check-circle", "green", "done") }
                                                            else if s == k && ph.busy() { ("play", "blue", "running") }
                                                            else { ("minus", "neutral", "waiting") };
                                                        rsx! { li { key: "{s}", span { class: "oxbk-st", "data-hue": "{h}", {ico(ic, 12.0)} } span { class: "oxaik-sr", "{w}: " } "{st}" } }
                                                    }
                                                }
                                            }
                                            ul { class: "oxbk-ba-files", "aria-label": "Changed files",
                                                if files(&t).is_empty() { li { class: "oxaik-mute", "No changes yet" } }
                                                for (f, a, d) in files(&t) {
                                                    li { key: "{f}", span { class: "oxaik-sr", "changed: " } span { class: "oxaik-mono", "{f}" } b { class: "oxbk-add oxaik-num", "+{a}" } b { class: "oxbk-del oxaik-num", "−{d}" } }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            {live(&said.read())}
        }
    }
}
