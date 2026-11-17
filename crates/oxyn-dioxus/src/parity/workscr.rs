//! Screens S2: the workspace screens on the Screens 1 frame. Folders Overview and Detail,
//! Integrations, Kanban, CRM and Analytics, each with its persona's sidebar
//! (`oxyn_core::parity::workscr::persona`), full-view states (Data · Loading · Empty plus one of
//! its own) and real interactions. Styles in `workscr.css` (`oxv-` shared, `oxvf-` Folders,
//! `oxvd-` Detail, `oxvi-` Integrations, `oxvk-` Kanban, `oxvc-` CRM, `oxva-` Analytics). Chrome
//! stays monochrome; colour is for official marks, file and folder glyphs and status.

use dioxus::prelude::*;
use oxyn_core::parity::screensfid::PERSON;
use oxyn_core::parity::workscr::{
    ANALYTICS_STATES, CRM_COLS, CRM_OWNERS, CRM_STATES, CRM_TABS, CardStatus, Company, CrmStage,
    DocFile, FILES_STATES, FOLDER_SORTS, FOLDER_STATES, FOLDER_TABS, FileKind, FolderTile,
    INTEG_CATS, INTEG_STATES, INTEG_TABS, Integ, KANBAN_STATES, KB_LABELS, KB_LANES, KB_SORTS,
    KB_TABS, POPULAR_SHOWN, RANGES, ScreenState, TEMPLATE_TABS, add_card, add_company, axis_days,
    axis_top, base_state, breakdown, cards, cat_tone, channel_total, channels, companies, crm_view,
    dollars, dollars_k, folder_docs, folder_tiles, integ_view, integrations, kind_for, kpis,
    lane_step, lane_view, money, more_words, move_card, polyline, set_cell, stage_sums, tab_counts,
    templates, templates_view, tile_line, top_products, weekly, when_words,
};

use super::efflistfid::check_glyph;
use super::screensfid::{AppFrame, MenuButton, nico, skel, stage_scale, use_threads};
use crate::brand::BrandIcon;

/// The state bar entries of each S2 screen (for a host's `ScreenStage { states }`).
pub fn s2_states(screen: &str) -> Vec<(&'static str, &'static str)> {
    match screen {
        "folders-detail" => FOLDER_STATES.to_vec(),
        "integrations" => INTEG_STATES.to_vec(),
        "kanban" => KANBAN_STATES.to_vec(),
        "crm" => CRM_STATES.to_vec(),
        "analytics" => ANALYTICS_STATES.to_vec(),
        _ => FILES_STATES.to_vec(),
    }
}

/// A row of tabs (`role=tablist`, ←/→ move) with optional counts.
fn tabs(
    aria: &'static str,
    items: Vec<(&'static str, String, Option<usize>)>,
    mut cur: Signal<&'static str>,
) -> Element {
    let keys: Vec<&'static str> = items.iter().map(|(k, _, _)| *k).collect();
    rsx! {
        div { class: "oxv-tabs", role: "tablist", "aria-label": "{aria}",
            for (k, l, n) in items {
                {
                    let on = cur() == k;
                    let ks = keys.clone();
                    rsx! {
                        button { key: "{k}", r#type: "button", role: "tab", class: "oxv-tab", "aria-selected": "{on}", tabindex: if on { "0" } else { "-1" },
                            onclick: move |_| cur.set(k),
                            onkeydown: move |e: KeyboardEvent| {
                                let i = ks.iter().position(|x| *x == k).unwrap_or(0);
                                let n = ks.len();
                                let j = match e.key() { Key::ArrowRight => (i + 1) % n, Key::ArrowLeft => (i + n - 1) % n, _ => return };
                                e.prevent_default();
                                cur.set(ks[j]);
                            },
                            span { class: "oxv-tab-l", "{l}" }
                            if let Some(n) = n { span { class: "oxv-tab-n", "{n}" } }
                        }
                    }
                }
            }
        }
    }
}

/// A search field with Escape to clear.
fn find(
    aria: &'static str,
    ph: &'static str,
    mut q: Signal<String>,
    class: &'static str,
) -> Element {
    rsx! {
        label { class: "oxv-find {class}",
            {nico("search")}
            input { r#type: "search", placeholder: "{ph}", "aria-label": "{aria}", value: "{q}",
                oninput: move |e| q.set(e.value()),
                onkeydown: move |e: KeyboardEvent| if e.key() == Key::Escape { q.set(String::new()) } }
        }
    }
}

/// A filled folder glyph in the folder tone.
fn folder_glyph(size: u32) -> Element {
    rsx! {
        svg { class: "oxv-folder", width: "{size}", height: "{size}", view_box: "0 0 24 24", "aria-hidden": "true",
            path { d: "M2.5 6.5A1.5 1.5 0 0 1 4 5h5.2l2 2H20a1.5 1.5 0 0 1 1.5 1.5v1H2.5z", fill: "currentColor", opacity: ".72" }
            path { d: "M2.5 9.5h19v8A1.5 1.5 0 0 1 20 19H4a1.5 1.5 0 0 1-1.5-1.5z", fill: "currentColor" }
        }
    }
}

/// A page preview: a sheet with lines and the kind's badge.
fn page_preview(kind: FileKind) -> Element {
    let k = kind.key();
    let b = kind.badge();
    rsx! {
        div { class: "oxv-prev", "aria-hidden": "true",
            div { class: "oxv-sheet", "data-kind": "{k}",
                if kind == FileKind::Pdf || kind == FileKind::Slides || kind == FileKind::Image { span { class: "oxv-page-img" } }
                for i in 0..5 { span { key: "{i}", class: "oxv-page-l", "data-kind": "{k}" } }
                span { class: "oxv-badge", "data-kind": "{k}", "{b}" }
            }
        }
    }
}

/// The base data state and the screen's own state key (one of the S2 extra states, else "").
fn split_state(state: &str) -> (ScreenState, &'static str) {
    let own = [
        "creating",
        "uploading",
        "failed",
        "dragging",
        "editing",
        "compare",
    ]
    .into_iter()
    .find(|k| *k == state)
    .unwrap_or("");
    (base_state(state), own)
}

// ───────────────────────────────────────────────────────────── Folders ──

/// Screen – Folders: Overview (folders with tabs, search, sort, grid or list, Create folder, then
/// file templates) and Detail (one folder's files with Share and Upload files). `detail` opens
/// the first folder; opening a tile or a sidebar project goes there too, and the crumb or ← comes
/// back. States: `sample`, `loading`, `empty`, `creating` (the new-folder field open) and
/// `uploading` (two files on their way in).
#[component]
pub fn FoldersScreen(
    /// `inset` or `flat`.
    #[props(default = "inset".to_string())]
    layout: String,
    /// A state key.
    #[props(default = "sample".to_string())]
    state: String,
    /// Height px.
    #[props(default = 640)]
    height: u32,
    /// Start inside the first folder.
    #[props(default)]
    detail: bool,
    /// `grid` or `list`.
    #[props(default = "grid".to_string())]
    view: String,
    /// Oxyn: star a folder to keep it first (starts with one starred).
    #[props(default)]
    starred: bool,
) -> Element {
    let st = base_state(&state);
    let thr = use_threads(st);
    let active = use_signal(|| 0u32);
    let list = use_signal(move || {
        let mut v = folder_tiles(st == ScreenState::Sample || st == ScreenState::Loading);
        if starred && let Some(f) = v.get_mut(2) {
            f.starred = true;
        }
        v
    });
    let mut open = use_signal(move || if detail { Some(3u32) } else { None });
    let as_list = use_signal(move || view == "list");
    let cur = open().and_then(|id| list().into_iter().find(|f| f.id == id));
    let nav_on = if cur.is_some() { "autumn" } else { "files" };
    let (crumb, title) = match &cur {
        Some(f) => ("Files".to_string(), f.name.clone()),
        None => (String::new(), String::new()),
    };
    rsx! {
        AppFrame { layout, state: st.key().to_string(), height, threads: thr, active, side: "files", nav_on, crumb, title, bare: cur.is_none(),
            on_nav: move |k: String| match k.as_str() {
                "files" => open.set(None),
                "autumn" => open.set(Some(3)),
                "brand" => open.set(Some(1)),
                "hiring" => open.set(Some(2)),
                "research" => open.set(Some(4)),
                _ => {}
            },
            if let Some(f) = cur {
                FolderDetail { key: "{f.id}", state: state.clone(), name: f.name.clone(), shared: f.shared, as_list, on_back: move |_| open.set(None) }
            } else {
                FolderOverview { state: state.clone(), list, as_list, on_open: move |id: u32| open.set(Some(id)) }
            }
        }
    }
}

#[component]
fn FolderOverview(
    state: String,
    list: Signal<Vec<FolderTile>>,
    as_list: Signal<bool>,
    on_open: EventHandler<u32>,
) -> Element {
    let (st, sk) = split_state(&state);
    let tab = use_signal(|| "all");
    let q = use_signal(String::new);
    let mut sort = use_signal(|| "manual".to_string());
    let mut adding = use_signal(move || sk == "creating");
    let mut draft = use_signal(String::new);
    let mut said = use_signal(String::new);
    let ttab = use_signal(|| "all");
    let tq = use_signal(String::new);
    let mut tkind = use_signal(String::new);
    let mut tdesc = use_signal(|| false);
    let tpl = use_signal(move || templates(st == ScreenState::Sample));
    let mut as_list = as_list;
    let mut list = list;
    let all = list();
    let view = oxyn_core::parity::workscr::tiles_view(&all, tab(), &q(), &sort());
    let kind = if tkind().is_empty() {
        None
    } else {
        Some(FileKind::from_key(&tkind()))
    };
    let tv = templates_view(&tpl(), ttab(), kind, &tq(), tdesc());
    let mut create = move || {
        let name = draft.peek().trim().to_string();
        if name.is_empty() {
            return;
        }
        let id = list.peek().iter().map(|f| f.id).max().unwrap_or(0) + 1;
        list.write().insert(
            0,
            FolderTile {
                id,
                name: name.clone(),
                shared: false,
                files: 0,
                mins: 0,
                starred: false,
            },
        );
        draft.set(String::new());
        adding.set(false);
        said.set(format!("Created {name}"));
    };
    let lay = if as_list() { "list" } else { "grid" };
    let loading = st == ScreenState::Loading;
    let kind_l = if tkind().is_empty() {
        "Type".to_string()
    } else {
        FileKind::from_key(&tkind()).label().to_string()
    };
    let mut kind_items: Vec<(&'static str, &'static str, &'static str)> =
        vec![("", "Any type", "")];
    kind_items.extend(FileKind::ALL.iter().map(|k| (k.key(), k.label(), "")));
    rsx! {
        div { class: "oxv-page oxvf",
            h1 { class: "oxv-h1", "Files" }
            p { class: "oxv-lead", "Everything your chats made or used, and the folders you keep it in. "
                button { r#type: "button", class: "oxv-link", "How folders work" } }
            section { class: "oxv-sec", "aria-labelledby": "oxvf-fh",
                h2 { id: "oxvf-fh", class: "oxv-cap", "Folders" span { class: "oxv-cap-n", "{all.len()}" } }
                div { class: "oxv-bar",
                    {tabs("Folders shown", FOLDER_TABS.iter().map(|(k, l)| (*k, l.to_string(), None)).collect(), tab)}
                    span { class: "oxscn-grow" }
                    MenuButton { label: "", aria: "Sort folders", icon: "sort", right: true, class: "oxv-sq",
                        items: FOLDER_SORTS.to_vec().into_iter().map(|(k, l)| (k, l, "")).collect::<Vec<_>>(), checked: sort(),
                        on_pick: move |k: String| sort.set(k) }
                    div { class: "oxv-seg", role: "group", "aria-label": "View",
                        button { r#type: "button", class: "oxv-seg-b", "aria-label": "Grid", "aria-pressed": "{!as_list()}", onclick: move |_| as_list.set(false), {nico("apps")} }
                        button { r#type: "button", class: "oxv-seg-b", "aria-label": "List", "aria-pressed": "{as_list()}", onclick: move |_| as_list.set(true), {nico("list")} }
                    }
                    {find("Search folders", "Search folders…", q, "")}
                    button { r#type: "button", class: "oxv-pri", onclick: move |_| adding.set(true), {nico("plus")} span { "Create folder" } }
                }
                if loading {
                    div { class: "oxvf-tiles", "data-view": "grid", "aria-hidden": "true",
                        for i in 0..4 { div { key: "{i}", class: "oxvf-tile oxv-sk", span { class: "oxv-skb" } span { class: "oxv-skl" } } }
                    }
                } else {
                    if all.is_empty() && !adding() {
                        div { class: "oxv-none",
                            span { class: "oxv-none-g", {folder_glyph(28)} }
                            p { class: "oxv-none-t", "No folders yet" }
                            p { class: "oxv-none-l", "Group the files for one project, then share the folder with people or agents." }
                            button { r#type: "button", class: "oxscn-btn", onclick: move |_| adding.set(true), {nico("plus")} span { "Create folder" } }
                        }
                    }
                    ul { class: "oxvf-tiles", "data-view": "{lay}", "aria-label": "Folders",
                        if adding() {
                            li { class: "oxvf-tile oxvf-add",
                                span { class: "oxvf-fg", {folder_glyph(26)} }
                                input { class: "oxv-in", r#type: "text", placeholder: "Folder name", "aria-label": "New folder name", value: "{draft}",
                                    onmounted: move |e: MountedEvent| { let d = e.data(); spawn(async move { let _ = d.set_focus(true).await; }); },
                                    oninput: move |e| draft.set(e.value()),
                                    onkeydown: move |e: KeyboardEvent| match e.key() { Key::Enter => create(), Key::Escape => { adding.set(false); draft.set(String::new()); } _ => {} } }
                                button { r#type: "button", class: "oxscn-ib", "aria-label": "Cancel", onclick: move |_| { adding.set(false); draft.set(String::new()); }, {nico("close")} }
                                button { r#type: "button", class: "oxv-pri oxv-pri-s", disabled: draft().trim().is_empty(), onclick: move |_| create(), "Create" }
                            }
                        }
                        for f in view {
                            {
                                let id = f.id;
                                let star = f.starred;
                                let line = tile_line(&f);
                                let when = when_words(f.mins);
                                let nm = f.name.clone();
                                rsx! {
                                    li { key: "{id}", class: "oxvf-tile", "data-star": "{star}",
                                        button { r#type: "button", class: "oxvf-open", "aria-label": "Open {nm}, {line}", onclick: move |_| on_open.call(id),
                                            span { class: "oxvf-fg", {folder_glyph(26)} }
                                            span { class: "oxvf-t",
                                                span { class: "oxvf-n", title: "{f.name}", "{f.name}" }
                                                span { class: "oxvf-l", {nico(if f.shared { "people" } else { "lock" })} span { class: "oxv-ell", "{line}" } }
                                            }
                                            span { class: "oxvf-when", "{when}" }
                                        }
                                        button { r#type: "button", class: "oxscn-ib oxvf-star", "aria-label": "Star {nm}", "aria-pressed": "{star}",
                                            onclick: move |_| { if let Some(x) = list.write().iter_mut().find(|x| x.id == id) { x.starred = !x.starred; } },
                                            {nico("star")} }
                                    }
                                }
                            }
                        }
                    }
                    if !all.is_empty() && oxyn_core::parity::workscr::tiles_view(&all, tab(), &q(), "").is_empty() {
                        p { class: "oxscn-hint", role: "status", "No folder matches." }
                    }
                }
            }
            section { class: "oxv-sec", "aria-labelledby": "oxvf-th",
                h2 { id: "oxvf-th", class: "oxv-cap", "File templates" span { class: "oxv-cap-n", "{tpl().len()}" } }
                div { class: "oxv-bar",
                    {tabs("Templates shown", TEMPLATE_TABS.iter().map(|(k, l)| (*k, l.to_string(), None)).collect(), ttab)}
                    span { class: "oxscn-grow" }
                    MenuButton { label: kind_l, aria: "Filter by type", icon: "filter", right: true, caret: true, items: kind_items, checked: tkind(),
                        on_pick: move |k: String| tkind.set(k) }
                    button { r#type: "button", class: "oxscn-btn oxv-sq", "aria-label": if tdesc() { "Sort A to Z" } else { "Sort Z to A" }, "aria-pressed": "{tdesc()}", onclick: move |_| tdesc.toggle(), {nico("sort")} }
                    {find("Search templates", "Search templates…", tq, "")}
                    button { r#type: "button", class: "oxscn-btn", onclick: move |_| said.set("Pick a file to save as a template".to_string()), {nico("plus")} span { "Add template" } }
                }
                if loading {
                    div { class: "oxvf-tpls", "aria-hidden": "true",
                        for i in 0..4 { div { key: "{i}", class: "oxvf-tpl oxv-sk", div { class: "oxv-prev" } span { class: "oxv-skl" } } }
                    }
                } else if tpl().is_empty() {
                    p { class: "oxv-quiet", "Templates you save show up here, ready to copy into any folder." }
                } else {
                    ul { class: "oxvf-tpls", "aria-label": "File templates",
                        for t in tv {
                            {
                                let nm = t.name;
                                rsx! {
                                    li { key: "{t.id}", class: "oxvf-tpl",
                                        button { r#type: "button", class: "oxvf-tpl-b", "aria-label": "Use {nm}", onclick: move |_| said.set(format!("Copied {nm} into a new file")),
                                            {page_preview(t.kind)}
                                            span { class: "oxvf-tn", "{t.name}" }
                                            span { class: "oxvf-tm",
                                                span { class: "oxv-ell", "{t.kind.label()} · {t.owner}" }
                                                span { class: "oxv-dotsep", "·" }
                                                {nico(if t.shared { "people" } else { "lock" })}
                                                span { if t.shared { "Shared" } else { "Private" } }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            p { class: "oxscn-sr", role: "status", "{said}" }
        }
    }
}

#[component]
fn FolderDetail(
    state: String,
    name: String,
    shared: bool,
    as_list: Signal<bool>,
    on_back: EventHandler<()>,
) -> Element {
    let (st, sk) = split_state(&state);
    let mut files = use_signal(move || {
        let mut v = folder_docs(st == ScreenState::Sample);
        if sk == "uploading" {
            v.insert(
                0,
                DocFile {
                    id: 90,
                    name: "Field-Photos.png".into(),
                    kind: FileKind::Image,
                    mins: 0,
                },
            );
            v.insert(
                0,
                DocFile {
                    id: 91,
                    name: "Survey-Answers.csv".into(),
                    kind: FileKind::Sheet,
                    mins: 0,
                },
            );
        }
        v
    });
    let mut up = use_signal(move || {
        if sk == "uploading" {
            vec![(91u32, 64u32), (90, 22)]
        } else {
            vec![]
        }
    });
    let mut share = use_signal(|| false);
    let mut invite = use_signal(String::new);
    let mut people = use_signal(move || {
        if shared {
            vec!["Tobi Lund".to_string(), "Mira Sol".to_string()]
        } else {
            vec![]
        }
    });
    let mut said = use_signal(String::new);
    let mut sort = use_signal(|| "recent".to_string());
    let mut n_up = use_signal(|| 0u32);
    let mut as_list = as_list;
    let loading = st == ScreenState::Loading;
    let mut v = files();
    if sort() == "name" {
        v.sort_by(|a, b| a.name.cmp(&b.name));
    }
    let lay = if as_list() { "list" } else { "grid" };
    let ups = up();
    let mut upload = move || {
        let k = *n_up.peek() % 3;
        n_up += 1;
        let nm = ["Notes-Draft.md", "Moodboard.png", "Quote.pdf"][k as usize];
        let id = files.peek().iter().map(|f| f.id).max().unwrap_or(0) + 1;
        files.write().insert(
            0,
            DocFile {
                id,
                name: nm.into(),
                kind: kind_for(nm),
                mins: 0,
            },
        );
        up.write().retain(|(i, _)| *i != id);
        said.set(format!("Uploaded {nm}"));
    };
    let nm2 = name.clone();
    rsx! {
        div { class: "oxv-page oxvd",
            button { r#type: "button", class: "oxv-back", onclick: move |_| on_back.call(()), {nico("back")} span { "Files" } }
            div { class: "oxvd-head",
                span { class: "oxvd-fg", {folder_glyph(30)} }
                h1 { class: "oxv-h1 oxvd-t", title: "{name}", "{name}" }
                span { class: "oxscn-grow" }
                MenuButton { label: "", aria: "More for {name}", icon: "more", right: true, class: "oxv-ghost",
                    items: vec![("rename", "Rename", ""), ("copy", "Duplicate", ""), ("move", "Move to…", ""), ("delete", "Delete folder", "")],
                    on_pick: move |k: String| said.set(match k.as_str() { "rename" => "Rename: type a new name".to_string(), "copy" => format!("Duplicated {nm2}"), "delete" => "Deleting asks first".to_string(), _ => "Pick where to move it".to_string() }) }
                MenuButton { label: if sort() == "name" { "Name".to_string() } else { "Recent".to_string() }, aria: "Sort files", icon: "sort", right: true, class: "oxvd-sort",
                    items: vec![("recent", "Recent first", ""), ("name", "Name", "")], checked: sort(), on_pick: move |k: String| sort.set(k) }
                div { class: "oxv-seg", role: "group", "aria-label": "View",
                    button { r#type: "button", class: "oxv-seg-b", "aria-label": "Grid", "aria-pressed": "{!as_list()}", onclick: move |_| as_list.set(false), {nico("apps")} }
                    button { r#type: "button", class: "oxv-seg-b", "aria-label": "List", "aria-pressed": "{as_list()}", onclick: move |_| as_list.set(true), {nico("list")} }
                }
                div { class: "oxv-pop-w",
                    button { r#type: "button", class: "oxscn-btn", "aria-haspopup": "dialog", "aria-expanded": "{share}", onclick: move |_| share.toggle(), {nico("share")} span { "Share" } }
                    if share() {
                        div { class: "oxscn-catch", onclick: move |_| share.set(false) }
                        div { class: "oxv-pop oxvd-share", role: "dialog", "aria-label": "Share {name}",
                            onkeydown: move |e: KeyboardEvent| if e.key() == Key::Escape { share.set(false) },
                            p { class: "oxv-pop-h", "Share “{name}”" }
                            div { class: "oxvd-inv",
                                input { class: "oxv-in", r#type: "email", placeholder: "Name or email", "aria-label": "Invite by name or email", value: "{invite}",
                                    oninput: move |e| invite.set(e.value()),
                                    onkeydown: move |e: KeyboardEvent| if e.key() == Key::Enter { let v = invite.peek().trim().to_string(); if !v.is_empty() { people.write().push(v); invite.set(String::new()); } } }
                                button { r#type: "button", class: "oxv-pri oxv-pri-s", disabled: invite().trim().is_empty(),
                                    onclick: move |_| { let v = invite.peek().trim().to_string(); if !v.is_empty() { people.write().push(v); invite.set(String::new()); } }, "Invite" }
                            }
                            div { class: "oxvd-ppl",
                                div { class: "oxvd-p", span { class: "oxscn-av", "RV" } span { class: "oxv-ell", "{PERSON} (you)" } span { class: "oxv-q", "Owner" } }
                                for (i, p) in people().into_iter().enumerate() {
                                    div { key: "{i}", class: "oxvd-p",
                                        span { class: "oxscn-av", {p.chars().next().map(|c| c.to_string()).unwrap_or_default()} }
                                        span { class: "oxv-ell", "{p}" }
                                        button { r#type: "button", class: "oxv-link", "aria-label": "Remove {p}", onclick: move |_| { people.write().remove(i); }, "Remove" }
                                    }
                                }
                            }
                            button { r#type: "button", class: "oxv-link oxvd-copy", onclick: move |_| { said.set("Link copied".to_string()); share.set(false); }, {nico("link")} span { "Copy link" } }
                        }
                    }
                }
                button { r#type: "button", class: "oxv-pri", onclick: move |_| upload(), {nico("upload")} span { "Upload files" } }
            }
            if loading {
                div { class: "oxvd-files", "data-view": "grid", "aria-hidden": "true",
                    for i in 0..5 { div { key: "{i}", class: "oxvd-file oxv-sk", div { class: "oxv-prev" } span { class: "oxv-skl" } } }
                }
            } else if v.is_empty() {
                div { class: "oxvd-drop", role: "group", "aria-label": "Empty folder",
                    span { class: "oxv-none-g", {nico("upload")} }
                    p { class: "oxv-none-t", "This folder is empty" }
                    p { class: "oxv-none-l", "Drop files here or upload them. Chats working in this folder save what they make here too." }
                    button { r#type: "button", class: "oxv-pri", onclick: move |_| upload(), {nico("upload")} span { "Upload files" } }
                }
            } else {
                ul { class: "oxvd-files", "data-view": "{lay}", "aria-label": "Files in {name}",
                    for f in v {
                        {
                            let p = ups.iter().find(|(i, _)| *i == f.id).map(|(_, p)| *p);
                            let meta = if p.is_some() { "Uploading…".to_string() } else { format!("{} · {}", f.kind.label(), when_words(f.mins)) };
                            let fname = f.name.clone();
                            rsx! {
                                li { key: "{f.id}", class: "oxvd-file", "data-up": "{p.is_some()}",
                                    button { r#type: "button", class: "oxvd-file-b", "aria-label": "Open {fname}", onclick: move |_| said.set(format!("Opened {fname}")),
                                        {page_preview(f.kind)}
                                        span { class: "oxvf-tn", title: "{f.name}", "{f.name}" }
                                        span { class: "oxvf-tm", span { class: "oxv-ell", "{meta}" } }
                                    }
                                    if let Some(p) = p {
                                        div { class: "oxvd-prog", role: "progressbar", "aria-label": "Uploading {f.name}", "aria-valuenow": "{p}", "aria-valuemin": "0", "aria-valuemax": "100",
                                            span { style: "width:{p}%" } }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            p { class: "oxscn-sr", role: "status", "{said}" }
        }
    }
}

// ───────────────────────────────────────────────────────────── Integrations ──

/// Screen – Integrations: a sticky header (tabs, search, Request), apps grouped by category with
/// their official marks; + asks before linking, a linked app's ✓ opens Settings / Unlink, and
/// Popular folds the rest behind "… and N more". States: `sample`, `loading`, `empty`, `failed`
/// (one link that didn't go through, with Try again).
#[component]
pub fn IntegrationsScreen(
    /// `inset` or `flat`.
    #[props(default = "inset".to_string())]
    layout: String,
    /// A state key.
    #[props(default = "sample".to_string())]
    state: String,
    /// Height px.
    #[props(default = 640)]
    height: u32,
    /// Oxyn: a category menu that narrows the page to one group.
    #[props(default)]
    cat_menu: bool,
) -> Element {
    let (st, sk) = split_state(&state);
    let thr = use_threads(st);
    let active = use_signal(|| 0u32);
    let mut list = use_signal(move || integrations(st == ScreenState::Sample));
    let tab = use_signal(|| "all");
    let q = use_signal(String::new);
    let mut cat = use_signal(String::new);
    let mut ask = use_signal(|| None::<&'static str>);
    let mut menu = use_signal(|| None::<&'static str>);
    let mut failed = use_signal(move || if sk == "failed" { Some("figma") } else { None });
    let mut more = use_signal(|| false);
    let mut req = use_signal(|| false);
    let mut req_t = use_signal(String::new);
    let mut said = use_signal(String::new);
    let all = list();
    let groups = integ_view(&all, tab(), &q(), &cat());
    let loading = st == ScreenState::Loading;
    let on_n = all.iter().filter(|i| i.on).count();
    let mut cat_items: Vec<(&'static str, &'static str, &'static str)> =
        vec![("", "All categories", "")];
    cat_items.extend(INTEG_CATS.iter().map(|(k, l)| (*k, *l, "")));
    let cat_l = INTEG_CATS
        .iter()
        .find(|(k, _)| *k == cat())
        .map(|(_, l)| l.to_string())
        .unwrap_or_else(|| "All categories".to_string());
    let mut send_req = move || {
        let t = req_t.peek().trim().to_string();
        if t.is_empty() {
            return;
        }
        said.set(format!(
            "Asked for {t}. We'll let you know when it's ready."
        ));
        req_t.set(String::new());
        req.set(false);
    };
    let card = move |i: Integ| {
        let id = i.id;
        let nm = i.name;
        let asking = ask() == Some(id);
        let open_m = menu() == Some(id);
        let fail = failed() == Some(id);
        rsx! {
            li { key: "{id}", class: "oxvi-card", "data-on": "{i.on}", "data-fail": "{fail}",
                span { class: "oxvi-logo", BrandIcon { name: id.to_string(), size: 18.0, coloured: true, fallback: true } }
                span { class: "oxvi-t",
                    span { class: "oxvi-n", span { class: "oxv-ell", "{nm}" } if i.verified { span { class: "oxvi-ver", title: "Verified publisher", {nico("verified")} } } }
                    if fail { span { class: "oxvi-err", role: "alert", "Couldn't link {nm}. The sign-in window closed." } }
                    else { span { class: "oxvi-b", title: "{i.blurb}", "{i.blurb}" } }
                }
                div { class: "oxv-pop-w",
                    if fail {
                        button { r#type: "button", class: "oxscn-btn oxvi-retry", onclick: move |_| { failed.set(None); ask.set(Some(id)); }, "Try again" }
                    } else if i.on {
                        button { r#type: "button", class: "oxvi-act", "data-on": "true", "aria-label": "{nm} is linked. Options", "aria-haspopup": "menu", "aria-expanded": "{open_m}",
                            onclick: move |_| menu.set(if open_m { None } else { Some(id) }), {check_glyph()} }
                    } else {
                        button { r#type: "button", class: "oxvi-act", "aria-label": "Link {nm}", "aria-haspopup": "dialog", "aria-expanded": "{asking}",
                            onclick: move |_| ask.set(if asking { None } else { Some(id) }), {nico("plus")} }
                    }
                    if open_m {
                        div { class: "oxscn-catch", onclick: move |_| menu.set(None) }
                        div { class: "oxscn-menu oxvi-menu", role: "menu", "aria-label": "{nm}",
                            onkeydown: move |e: KeyboardEvent| if e.key() == Key::Escape { menu.set(None) },
                            button { r#type: "button", class: "oxscn-mi", role: "menuitem", onclick: move |_| { menu.set(None); said.set(format!("{nm} settings")); }, {nico("gear")} span { class: "oxscn-mi-l", "Settings" } }
                            button { r#type: "button", class: "oxscn-mi oxv-danger", role: "menuitem",
                                onclick: move |_| { menu.set(None); if let Some(x) = list.write().iter_mut().find(|x| x.id == id) { x.on = false; } said.set(format!("Unlinked {nm}")); },
                                {nico("close")} span { class: "oxscn-mi-l", "Unlink" } }
                        }
                    }
                    if asking {
                        div { class: "oxscn-catch", onclick: move |_| ask.set(None) }
                        div { class: "oxv-pop oxvi-ask", role: "dialog", "aria-label": "Link {nm}",
                            onkeydown: move |e: KeyboardEvent| if e.key() == Key::Escape { ask.set(None) },
                            p { class: "oxv-pop-h", BrandIcon { name: id.to_string(), size: 16.0, coloured: true } span { "Link {nm}?" } }
                            p { class: "oxv-pop-l", "Agents can read from {nm} and ask before they change anything there." }
                            div { class: "oxv-pop-a",
                                button { r#type: "button", class: "oxscn-btn", onclick: move |_| ask.set(None), "Cancel" }
                                button { r#type: "button", class: "oxv-pri oxv-pri-s",
                                    onmounted: move |e: MountedEvent| { let d = e.data(); spawn(async move { let _ = d.set_focus(true).await; }); },
                                    onclick: move |_| { ask.set(None); if let Some(x) = list.write().iter_mut().find(|x| x.id == id) { x.on = true; } said.set(format!("Linked {nm}")); }, "Link" }
                            }
                        }
                    }
                }
            }
        }
    };
    rsx! {
        AppFrame { layout, state: st.key().to_string(), height, threads: thr, active, side: "files", nav_on: "apps", bare: true,
            div { class: "oxv-page oxvi",
                div { class: "oxvi-in",
                    h1 { class: "oxv-h1", "Integrations" }
                    p { class: "oxv-lead", "Link the tools your team already works in, then use them from any chat." }
                    div { class: "oxvi-sticky",
                        {tabs("Show", INTEG_TABS.iter().map(|(k, l)| (*k, l.to_string(), if *k == "on" && on_n > 0 { Some(on_n) } else { None })).collect(), tab)}
                        span { class: "oxscn-grow" }
                        if cat_menu {
                            MenuButton { label: cat_l, aria: "Category", icon: "filter", right: true, caret: true, items: cat_items, checked: cat(), on_pick: move |k: String| cat.set(k) }
                        }
                        {find("Search integrations", "Search integrations", q, "oxvi-q")}
                        div { class: "oxv-pop-w",
                            button { r#type: "button", class: "oxv-pri", "aria-haspopup": "dialog", "aria-expanded": "{req}", onclick: move |_| req.toggle(), {nico("plus")} span { "Request" } }
                            if req() {
                                div { class: "oxscn-catch", onclick: move |_| req.set(false) }
                                div { class: "oxv-pop oxvi-req", role: "dialog", "aria-label": "Request an integration",
                                    onkeydown: move |e: KeyboardEvent| if e.key() == Key::Escape { req.set(false) },
                                    p { class: "oxv-pop-h", "Which tool should we add?" }
                                    input { class: "oxv-in", r#type: "text", placeholder: "App name", "aria-label": "App name", value: "{req_t}",
                                        onmounted: move |e: MountedEvent| { let d = e.data(); spawn(async move { let _ = d.set_focus(true).await; }); },
                                        oninput: move |e| req_t.set(e.value()),
                                        onkeydown: move |e: KeyboardEvent| if e.key() == Key::Enter { send_req() } }
                                    div { class: "oxv-pop-a",
                                        button { r#type: "button", class: "oxscn-btn", onclick: move |_| req.set(false), "Cancel" }
                                        button { r#type: "button", class: "oxv-pri oxv-pri-s", disabled: req_t().trim().is_empty(), onclick: move |_| send_req(), "Send request" }
                                    }
                                }
                            }
                        }
                    }
                    if loading {
                        for g in 0..2 {
                            section { key: "{g}", class: "oxvi-sec", "aria-hidden": "true",
                                span { class: "oxv-skl oxvi-skh" }
                                div { class: "oxvi-grid",
                                    for i in 0..4 { div { key: "{i}", class: "oxvi-card oxv-sk", span { class: "oxvi-logo" } span { class: "oxv-skl" } } }
                                }
                            }
                        }
                    } else if all.is_empty() {
                        div { class: "oxv-none",
                            span { class: "oxv-none-g", {nico("plug")} }
                            p { class: "oxv-none-t", "No integrations to show" }
                            p { class: "oxv-none-l", "Your admin hasn't allowed any apps yet. Ask for the one you need and we'll pass it on." }
                            button { r#type: "button", class: "oxscn-btn", onclick: move |_| req.set(true), {nico("plus")} span { "Request" } }
                        }
                    } else if groups.is_empty() {
                        p { class: "oxscn-hint", role: "status", "Nothing matches. Try another word, or request the app." }
                    } else {
                        for (k, t, items) in groups {
                            {
                                let fold = k == "popular" && !more() && items.len() > POPULAR_SHOWN && q().is_empty();
                                let rest: Vec<Integ> = if fold { items[POPULAR_SHOWN..].to_vec() } else { vec![] };
                                let shown: Vec<Integ> = if fold { items[..POPULAR_SHOWN].to_vec() } else { items };
                                let words = more_words(&rest);
                                rsx! {
                                    section { key: "{k}", class: "oxvi-sec", "aria-labelledby": "oxvi-{k}",
                                        h2 { id: "oxvi-{k}", class: "oxvi-h", "{t}" }
                                        ul { class: "oxvi-grid", "aria-label": "{t}",
                                            for i in shown { {card(i)} }
                                        }
                                        if fold {
                                            button { r#type: "button", class: "oxvi-more", "aria-expanded": "false", onclick: move |_| more.set(true),
                                                span { class: "oxvi-stack", "aria-hidden": "true",
                                                    for r in rest.iter().take(3) { span { key: "{r.id}", class: "oxvi-mini", BrandIcon { name: r.id.to_string(), size: 13.0, coloured: true } } }
                                                }
                                                span { "{words}" } {nico("next")}
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            p { class: "oxscn-sr", role: "status", "{said}" }
        }
    }
}

// ───────────────────────────────────────────────────────────── Kanban ──

/// A card being dragged: id, pointer start and offset (canvas px).
#[derive(Clone, Copy, PartialEq)]
struct Drag {
    id: u32,
    x0: f64,
    y0: f64,
    dx: f64,
    dy: f64,
}

fn status_ico(s: CardStatus) -> Element {
    let k = s.key();
    rsx! {
        svg { class: "oxvk-st", "data-st": "{k}", width: "14", height: "14", view_box: "0 0 16 16", "aria-hidden": "true",
            match s {
                CardStatus::Attention => rsx! { path { d: "M8 2.2 14.4 13.5H1.6z", fill: "currentColor" } },
                CardStatus::Fixed => rsx! { circle { cx: "8", cy: "8", r: "3.4", fill: "currentColor" } },
                CardStatus::Review => rsx! { circle { cx: "8", cy: "8", r: "5.2", fill: "none", stroke: "currentColor", stroke_width: "1.6" } path { d: "M8 2.8a5.2 5.2 0 0 1 0 10.4z", fill: "currentColor" } },
                CardStatus::Progress => rsx! { circle { cx: "8", cy: "8", r: "5.2", fill: "none", stroke: "currentColor", stroke_width: "1.6", stroke_dasharray: "3 2" } path { d: "M8 2.8a5.2 5.2 0 0 1 5.2 5.2H8z", fill: "currentColor" } },
            }
        }
    }
}

fn pri_ico(lane: &str) -> Element {
    let n = match lane {
        "low" => 1,
        "med" => 2,
        "high" => 3,
        _ => 0,
    };
    rsx! {
        if n == 0 {
            span { class: "oxvk-pri", "data-lane": "done", {nico("check")} }
        } else {
            svg { class: "oxvk-pri", "data-lane": "{lane}", width: "14", height: "14", view_box: "0 0 16 16", "aria-hidden": "true",
                for i in 0..3 {
                    rect { key: "{i}", x: "{2 + i * 5}", y: "{10 - i * 3}", width: "3", height: "{4 + i * 3}", rx: "1", "data-on": "{i < n}" }
                }
            }
        }
    }
}

/// Screen – Kanban: Inbox / Overview with Filter, Sort by and Board | List, task tabs with counts,
/// New task, and four 276 px lanes (Low, Medium, High priority, Completed). Cards drag between
/// lanes with a tilt; from the keyboard, Space lifts a card, ← / → carry it and Space or Escape
/// put it down. States: `sample`, `loading`, `empty`, `dragging` (a card held over High).
#[component]
pub fn KanbanScreen(
    /// `inset` or `flat`.
    #[props(default = "inset".to_string())]
    layout: String,
    /// A state key.
    #[props(default = "sample".to_string())]
    state: String,
    /// Height px.
    #[props(default = 640)]
    height: u32,
    /// `board` or `list`.
    #[props(default = "board".to_string())]
    view: String,
    /// Oxyn: show each card's label chip (Bug, Feature, Design).
    #[props(default = true)]
    labels: bool,
) -> Element {
    let (st, sk) = split_state(&state);
    let thr = use_threads(st);
    let active = use_signal(|| 0u32);
    let mut list = use_signal(move || cards(st == ScreenState::Sample));
    let tab = use_signal(|| "all");
    let mut sort = use_signal(|| "manual".to_string());
    let mut label = use_signal(String::new);
    let mut as_list = use_signal(move || view == "list");
    let mut star = use_signal(|| false);
    let mut drag = use_signal(move || {
        if sk == "dragging" {
            Some(Drag {
                id: 5,
                x0: 0.0,
                y0: 0.0,
                dx: 292.0,
                dy: 46.0,
            })
        } else {
            None
        }
    });
    let mut over = use_signal(move || if sk == "dragging" { Some("high") } else { None });
    let mut lift = use_signal(|| None::<u32>);
    let mut adding = use_signal(|| false);
    let mut draft = use_signal(String::new);
    let mut said = use_signal(String::new);
    let all = list();
    let (n_all, n_open, n_done) = tab_counts(&all);
    let loading = st == ScreenState::Loading;
    let lab = label();
    let (all2, lab2) = (all.clone(), lab.clone());
    let lane_cards = move |lane: &str| -> Vec<oxyn_core::parity::workscr::Card> {
        lane_view(&all2, lane, tab(), "", &sort())
            .into_iter()
            .filter(|c| lab2.is_empty() || c.label == lab2)
            .collect()
    };
    let mut add = move || {
        let t = draft.peek().trim().to_string();
        if t.is_empty() {
            return;
        }
        add_card(&mut list.write(), &t, "low");
        draft.set(String::new());
        adding.set(false);
        said.set(format!("Added {t} to Low priority"));
    };
    let mut drop_it = move || {
        if let (Some(d), Some(l)) = (*drag.peek(), *over.peek()) {
            let moved = d.dx.abs() + d.dy.abs() > 4.0;
            if moved && move_card(&mut list.write(), d.id, l) {
                let ln = KB_LANES
                    .iter()
                    .find(|x| x.0 == l)
                    .map(|x| x.1)
                    .unwrap_or("");
                said.set(format!("Moved to {ln}"));
            }
        }
        drag.set(None);
        over.set(None);
    };
    let filt_items: Vec<(&'static str, &'static str, &'static str)> =
        std::iter::once(("", "Any label", ""))
            .chain(KB_LABELS.iter().map(|(k, l)| (*k, *l, "")))
            .collect();
    let filt_l = KB_LABELS
        .iter()
        .find(|(k, _)| *k == lab)
        .map(|(_, l)| l.to_string())
        .unwrap_or_else(|| "Filter".to_string());
    let sort_l = if sort() == "manual" {
        "Sort by".to_string()
    } else {
        KB_SORTS
            .iter()
            .find(|(k, _)| *k == sort())
            .map(|(_, l)| l.to_string())
            .unwrap_or_default()
    };
    let card_el = move |c: oxyn_core::parity::workscr::Card| {
        let id = c.id;
        let d = drag();
        let lifted = d.map(|d| d.id == id).unwrap_or(false);
        let kb = lift() == Some(id);
        let style = match d {
            Some(d) if d.id == id => format!("--dx:{:.0}px;--dy:{:.0}px", d.dx, d.dy),
            _ => String::new(),
        };
        let lb = KB_LABELS
            .iter()
            .find(|(k, _)| *k == c.label)
            .map(|(_, l)| *l);
        let t = c.title.clone();
        let lane = c.lane;
        rsx! {
            li { key: "{id}", class: "oxvk-card", "data-lift": "{lifted}", "data-kb": "{kb}", style: "{style}",
                onpointerdown: move |e: PointerEvent| {
                    if e.trigger_button() == Some(dioxus::html::input_data::MouseButton::Primary) {
                        let p = e.client_coordinates();
                        drag.set(Some(Drag { id, x0: p.x, y0: p.y, dx: 0.0, dy: 0.0 }));
                        over.set(Some(lane));
                    }
                },
                button { r#type: "button", class: "oxvk-card-b", "aria-roledescription": "Draggable task", "aria-pressed": "{kb}",
                    "aria-label": "{t}, {c.status.label()}. Space to pick up.",
                    onkeydown: move |e: KeyboardEvent| {
                        let k = e.key();
                        if k == Key::Character(" ".into()) || k == Key::Enter {
                            e.prevent_default();
                            if lift() == Some(id) { lift.set(None); said.set("Put down".to_string()); } else { lift.set(Some(id)); said.set(format!("Picked up {t}. Use left and right arrows.")); }
                        } else if lift() == Some(id) && (k == Key::ArrowLeft || k == Key::ArrowRight) {
                            e.prevent_default();
                            let cur = list.peek().iter().find(|x| x.id == id).map(|x| x.lane).unwrap_or("low");
                            let to = lane_step(cur, if k == Key::ArrowLeft { -1 } else { 1 });
                            if move_card(&mut list.write(), id, to) {
                                let ln = KB_LANES.iter().find(|x| x.0 == to).map(|x| x.1).unwrap_or("");
                                said.set(format!("Moved to {ln}"));
                            }
                        } else if k == Key::Escape && lift() == Some(id) {
                            lift.set(None);
                        }
                    },
                    span { class: "oxvk-ct",
                        {pri_ico(lane)}
                        span { class: "oxvk-title", title: "{c.title}", "{c.title}" }
                    }
                    if !c.body.is_empty() { span { class: "oxvk-body", "{c.body}" } }
                    span { class: "oxvk-foot",
                        {status_ico(c.status)}
                        span { class: "oxvk-sl", "{c.status.label()}" }
                        if labels && let Some(l) = lb { span { class: "oxvk-lab", "data-l": "{c.label}", "{l}" } }
                        span { class: "oxscn-grow" }
                        span { class: "oxvk-n", title: "Comments", {nico("chat")} "{c.talk}" }
                        if c.links > 0 { span { class: "oxvk-n", title: "Links", {nico("clip")} "{c.links}" } }
                    }
                }
            }
        }
    };
    rsx! {
        AppFrame { layout, state: st.key().to_string(), height, threads: thr, active, side: "board", nav_on: "board", crumb: "Inbox", title: "Overview",
            badge: rsx! { button { r#type: "button", class: "oxscn-ib oxvk-star", "aria-label": "Star this view", "aria-pressed": "{star}", onclick: move |_| star.toggle(), {nico("star")} } },
            actions: rsx! {
                MenuButton { label: filt_l, aria: "Filter by label", icon: "filter", right: true, class: "oxv-ghost", items: filt_items, checked: lab.clone(), on_pick: move |k: String| label.set(k) }
                MenuButton { label: sort_l, aria: "Sort by", icon: "sort", right: true, caret: true, class: "oxv-ghost", items: KB_SORTS.to_vec().into_iter().map(|(k, l)| (k, l, "")).collect::<Vec<_>>(), checked: sort(), on_pick: move |k: String| sort.set(k) }
                div { class: "oxv-seg oxvk-view", role: "group", "aria-label": "Layout",
                    button { r#type: "button", class: "oxv-seg-t", "aria-pressed": "{!as_list()}", onclick: move |_| as_list.set(false), "Board" }
                    button { r#type: "button", class: "oxv-seg-t", "aria-pressed": "{as_list()}", onclick: move |_| as_list.set(true), "List" }
                }
            },
            div { class: "oxvk",
                onpointermove: move |e: PointerEvent| {
                    if let Some(mut d) = drag() {
                        let p = e.client_coordinates();
                        let s = stage_scale();
                        d.dx = (p.x - d.x0) / s;
                        d.dy = (p.y - d.y0) / s;
                        drag.set(Some(d));
                    }
                },
                onpointerup: move |_| drop_it(),
                onpointerleave: move |_| if drag.peek().is_some() && sk != "dragging" { drag.set(None); over.set(None); },
                div { class: "oxvk-bar",
                    {tabs("Tasks", KB_TABS.iter().map(|(k, l)| (*k, l.to_string(), Some(match *k { "open" => n_open, "done" => n_done, _ => n_all }))).collect(), tab)}
                    span { class: "oxscn-grow" }
                    button { r#type: "button", class: "oxv-pri", onclick: move |_| adding.set(true), {nico("plus")} span { "New task" } }
                }
                if adding() {
                    div { class: "oxvk-new",
                        input { class: "oxv-in", r#type: "text", placeholder: "Task title", "aria-label": "New task title", value: "{draft}",
                            onmounted: move |e: MountedEvent| { let d = e.data(); spawn(async move { let _ = d.set_focus(true).await; }); },
                            oninput: move |e| draft.set(e.value()),
                            onkeydown: move |e: KeyboardEvent| match e.key() { Key::Enter => add(), Key::Escape => { adding.set(false); draft.set(String::new()); } _ => {} } }
                        span { class: "oxv-q", "Adds to Low priority" }
                        button { r#type: "button", class: "oxscn-btn", onclick: move |_| { adding.set(false); draft.set(String::new()); }, "Cancel" }
                        button { r#type: "button", class: "oxv-pri oxv-pri-s", disabled: draft().trim().is_empty(), onclick: move |_| add(), "Add task" }
                    }
                }
                if loading {
                    div { class: "oxvk-lanes", "aria-hidden": "true",
                        for i in 0..4 { div { key: "{i}", class: "oxvk-lane", span { class: "oxv-skl oxvk-skh" } for j in 0..(3 - i % 2) { div { key: "{j}", class: "oxvk-card oxv-sk", {skel(3)} } } } }
                    }
                } else if all.is_empty() && !adding() {
                    div { class: "oxv-none",
                        span { class: "oxv-none-g", {nico("kanban")} }
                        p { class: "oxv-none-t", "No tasks on this board" }
                        p { class: "oxv-none-l", "Add the first one. Its priority picks the lane, and you can drag it later." }
                        button { r#type: "button", class: "oxscn-btn", onclick: move |_| adding.set(true), {nico("plus")} span { "New task" } }
                    }
                } else if as_list() {
                    div { class: "oxvk-list", role: "table", "aria-label": "Tasks",
                        for (lk, ll) in KB_LANES {
                            {
                                let cs = lane_cards(lk);
                                rsx! {
                                    div { key: "{lk}", role: "rowgroup", class: "oxvk-lg",
                                        div { role: "row", class: "oxvk-lh", span { role: "columnheader", class: "oxvk-lh-t", {pri_ico(lk)} "{ll}" span { class: "oxv-tab-n", "{cs.len()}" } } }
                                        for c in cs {
                                            div { key: "{c.id}", role: "row", class: "oxvk-lr",
                                                span { role: "cell", class: "oxvk-lr-t", span { class: "oxv-ell", "{c.title}" } }
                                                span { role: "cell", class: "oxvk-lr-s", {status_ico(c.status)} span { class: "oxv-ell", "{c.status.label()}" } }
                                                span { role: "cell", class: "oxvk-lr-n", {nico("chat")} "{c.talk}" }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                } else {
                    div { class: "oxvk-lanes",
                        for (lk, ll) in KB_LANES {
                            {
                                let cs = lane_cards(lk);
                                let n = all.iter().filter(|c| c.lane == lk).count();
                                let hot = over() == Some(lk) && drag().is_some();
                                rsx! {
                                    section { key: "{lk}", class: "oxvk-lane", "data-over": "{hot}", "aria-label": "{ll}, {n} tasks",
                                        onpointerenter: move |_| if drag.peek().is_some() { over.set(Some(lk)) },
                                        header { class: "oxvk-lane-h",
                                            {pri_ico(lk)}
                                            h2 { class: "oxvk-lane-t", "{ll}" }
                                            span { class: "oxvk-cnt", "{n}" }
                                        }
                                        ul { class: "oxvk-cards",
                                            for c in cs { {card_el(c)} }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                p { class: "oxscn-sr", role: "status", "aria-live": "polite", "{said}" }
            }
        }
    }
}

// ───────────────────────────────────────────────────────────── CRM ──

/// Screen – CRM: Sales / Head of Sales with its open views as tabs (marks on each), pipeline
/// filters (stage, owner, search) and the Company CRM table: sortable headers, row select with
/// Delete, Add row, and cells you edit with a double-click or Enter (Escape cancels, Tab moves
/// on). States: `sample`, `loading`, `empty`, `editing` (a Value cell open).
#[component]
pub fn CrmScreen(
    /// `inset` or `flat`.
    #[props(default = "inset".to_string())]
    layout: String,
    /// A state key.
    #[props(default = "sample".to_string())]
    state: String,
    /// Height px.
    #[props(default = 640)]
    height: u32,
    /// Oxyn: a pipeline strip over the table (rows and value per stage; press one to filter).
    #[props(default)]
    totals: bool,
) -> Element {
    let (st, sk) = split_state(&state);
    let thr = use_threads(st);
    let active = use_signal(|| 0u32);
    let mut rows = use_signal(move || companies(st == ScreenState::Sample));
    let mut view = use_signal(|| "companies");
    let mut stage = use_signal(String::new);
    let mut owner = use_signal(String::new);
    let q = use_signal(String::new);
    let mut sort = use_signal(|| ("", false));
    let mut sel = use_signal(Vec::<u32>::new);
    let mut edit = use_signal(move || {
        if sk == "editing" {
            Some((4u32, "value"))
        } else {
            None
        }
    });
    let mut draft = use_signal(move || {
        if sk == "editing" {
            "124000".to_string()
        } else {
            String::new()
        }
    });
    let mut bad = use_signal(|| false);
    let mut role = use_signal(|| "head".to_string());
    let mut star = use_signal(|| true);
    let mut said = use_signal(String::new);
    let all = rows();
    let vstage = match view() {
        "deals" => "proposal",
        "renewals" => "won",
        _ => "",
    };
    let fstage = if stage().is_empty() {
        vstage.to_string()
    } else {
        stage()
    };
    let shown = crm_view(&all, &fstage, &owner(), &q(), sort());
    let loading = st == ScreenState::Loading;
    let n_sel = sel().len();
    let all_on = !shown.is_empty() && shown.iter().all(|r| sel().contains(&r.id));
    let ids: Vec<u32> = shown.iter().map(|r| r.id).collect();
    let mut commit = move || -> bool {
        let Some((id, col)) = *edit.peek() else {
            return true;
        };
        let v = draft.peek().clone();
        if set_cell(&mut rows.write(), id, col, &v) {
            edit.set(None);
            bad.set(false);
            said.set("Saved".to_string());
            true
        } else {
            bad.set(true);
            said.set("That isn't a number".to_string());
            false
        }
    };
    let cell_text = |r: &Company, col: &str| -> String {
        match col {
            "name" => r.name.clone(),
            "cat" => r.cat.clone(),
            "domain" => r.domain.clone(),
            "owner" => r.owner.clone(),
            "value" => r.value.to_string(),
            "stage" => r.stage.key().to_string(),
            _ => r.last.clone(),
        }
    };
    let mut stage_items: Vec<(&'static str, &'static str, &'static str)> =
        vec![("", "All stages", "")];
    stage_items.extend(CrmStage::ALL.iter().map(|s| (s.key(), s.label(), "")));
    let mut owner_items: Vec<(&'static str, &'static str, &'static str)> = vec![("", "Anyone", "")];
    owner_items.extend(CRM_OWNERS.iter().map(|o| (*o, *o, "")));
    let stage_l = if stage().is_empty() {
        "Stage".to_string()
    } else {
        CrmStage::from_key(&stage()).label().to_string()
    };
    let owner_l = if owner().is_empty() {
        "Owner".to_string()
    } else {
        owner()
    };
    let title = if role() == "head" {
        "Head of Sales"
    } else {
        "Account owner"
    };
    let sums = stage_sums(&all);
    rsx! {
        AppFrame { layout, state: st.key().to_string(), height, threads: thr, active, side: "sales", nav_on: "companies", crumb: "Sales", crumb_icon: "folder", title,
            badge: rsx! {
                MenuButton { label: "", aria: "Switch view", caret: true, class: "oxv-ghost oxvc-role", items: vec![("head", "Head of Sales", ""), ("owner", "Account owner", "")], checked: role(), on_pick: move |k: String| role.set(k) }
                button { r#type: "button", class: "oxscn-ib", "aria-label": "Star this page", "aria-pressed": "{star}", onclick: move |_| star.toggle(), {nico("star")} }
            },
            div { class: "oxvc",
                div { class: "oxvc-tabs",
                    button { r#type: "button", class: "oxscn-ib", "aria-label": "New view", onclick: move |_| said.set("Pick a table to start a view".to_string()), {nico("plus")} }
                    button { r#type: "button", class: "oxscn-ib", "aria-label": "All views", {nico("apps")} }
                    span { class: "oxvc-sep", "aria-hidden": "true" }
                    div { class: "oxv-tabs oxvc-views", role: "tablist", "aria-label": "Open views",
                        for (k, l, b) in CRM_TABS {
                            button { key: "{k}", r#type: "button", role: "tab", class: "oxvc-tab", "aria-selected": "{view() == k}", tabindex: if view() == k { "0" } else { "-1" },
                                onclick: move |_| { view.set(k); sel.set(vec![]); },
                                onkeydown: move |e: KeyboardEvent| {
                                    let i = CRM_TABS.iter().position(|x| x.0 == k).unwrap_or(0);
                                    let j = match e.key() { Key::ArrowRight => (i + 1) % 4, Key::ArrowLeft => (i + 3) % 4, _ => return };
                                    e.prevent_default();
                                    view.set(CRM_TABS[j].0);
                                },
                                span { class: "oxvc-mk", BrandIcon { name: b.to_string(), size: 14.0, coloured: true } }
                                span { class: "oxv-ell", "{l}" }
                            }
                        }
                    }
                    span { class: "oxscn-grow" }
                    MenuButton { label: stage_l, aria: "Filter by stage", icon: "filter", right: true, class: "oxv-ghost", items: stage_items, checked: stage(), on_pick: move |k: String| stage.set(k) }
                    MenuButton { label: owner_l, aria: "Filter by owner", icon: "user", right: true, class: "oxv-ghost", items: owner_items, checked: owner(), on_pick: move |k: String| owner.set(k) }
                    {find("Search companies", "Search", q, "oxvc-q")}
                }
                if totals && !loading && !all.is_empty() {
                    div { class: "oxvc-strip", role: "group", "aria-label": "Pipeline",
                        for (s, n, v) in sums {
                            button { key: "{s.key()}", r#type: "button", class: "oxvc-chip", "data-tone": "{s.key()}", "aria-pressed": "{stage() == s.key()}",
                                onclick: move |_| { let cur = stage.peek().clone(); stage.set(if cur == s.key() { String::new() } else { s.key().to_string() }); },
                                span { class: "oxvc-chip-l", "{s.label()}" }
                                span { class: "oxvc-chip-n", "{n} · ${money(v)}" }
                            }
                        }
                    }
                }
                if n_sel > 0 {
                    div { class: "oxvc-selbar", role: "toolbar", "aria-label": "Selected rows",
                        span { "{n_sel} selected" }
                        button { r#type: "button", class: "oxscn-btn oxv-danger", onclick: move |_| { let s = sel.peek().clone(); rows.write().retain(|r| !s.contains(&r.id)); sel.set(vec![]); said.set("Deleted".to_string()); }, {nico("trash")} span { "Delete" } }
                        button { r#type: "button", class: "oxv-link", onclick: move |_| sel.set(vec![]), "Clear" }
                    }
                }
                div { class: "oxvc-card",
                    table { class: "oxvc-table", "aria-label": "Company CRM",
                        thead {
                            tr {
                                th { class: "oxvc-ck", input { r#type: "checkbox", "aria-label": "Select all rows", checked: all_on, disabled: shown.is_empty(),
                                    onchange: move |e| sel.set(if e.checked() { ids.clone() } else { vec![] }) } }
                                for (k, l, g) in CRM_COLS {
                                    {
                                        let (sc, sd) = sort();
                                        let aria = if sc == k { if sd { "descending" } else { "ascending" } } else { "none" };
                                        let sortable = matches!(k, "name" | "owner" | "value" | "stage");
                                        rsx! {
                                            th { key: "{k}", "aria-sort": "{aria}", "data-k": "{k}",
                                                if sortable {
                                                    button { r#type: "button", class: "oxvc-th", onclick: move |_| { let (c, d) = *sort.peek(); sort.set(if c == k { (k, !d) } else { (k, false) }); },
                                                        {nico(g)} span { "{l}" } }
                                                } else {
                                                    span { class: "oxvc-th", {nico(g)} span { "{l}" } }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        tbody {
                            if loading {
                                for i in 0..8 { tr { key: "{i}", "aria-hidden": "true", td { class: "oxvc-ck" } td { colspan: "7", span { class: "oxv-skl oxvc-skr" } } } }
                            } else {
                                for r in shown.clone() {
                                    {
                                        let id = r.id;
                                        let on = sel().contains(&id);
                                        rsx! {
                                            tr { key: "{id}", "data-sel": "{on}",
                                                td { class: "oxvc-ck", input { r#type: "checkbox", "aria-label": "Select {r.name}", checked: on,
                                                    onchange: move |e| { if e.checked() { sel.write().push(id); } else { sel.write().retain(|x| *x != id); } } } }
                                                for (k, _, _) in CRM_COLS {
                                                    {
                                                        let editing = edit() == Some((id, k));
                                                        let txt = cell_text(&r, k);
                                                        let ed = k != "last";
                                                        rsx! {
                                                            td { key: "{k}", "data-k": "{k}", "data-edit": "{editing}", tabindex: if ed { "0" } else { "-1" },
                                                                ondoubleclick: move |_| if ed { edit.set(Some((id, k))); draft.set(txt.clone()); bad.set(false); },
                                                                onkeydown: move |e: KeyboardEvent| if ed && e.key() == Key::Enter && edit.peek().is_none() { edit.set(Some((id, k))); draft.set(cell_text_static(&rows.peek(), id, k)); },
                                                                if editing {
                                                                    if k == "stage" || k == "owner" {
                                                                        select { class: "oxvc-in", "aria-label": "Edit {k}",
                                                                            onmounted: move |e: MountedEvent| { let d = e.data(); spawn(async move { let _ = d.set_focus(true).await; }); },
                                                                            onchange: move |e| { draft.set(e.value()); commit(); },
                                                                            onkeydown: move |e: KeyboardEvent| if e.key() == Key::Escape { edit.set(None) },
                                                                            if k == "stage" {
                                                                                for s in CrmStage::ALL { option { key: "{s.key()}", value: "{s.key()}", selected: r.stage == s, "{s.label()}" } }
                                                                            } else {
                                                                                for o in CRM_OWNERS { option { key: "{o}", value: "{o}", selected: r.owner == o, "{o}" } }
                                                                            }
                                                                        }
                                                                    } else {
                                                                        input { class: "oxvc-in", r#type: "text", "aria-label": "Edit {k}", "aria-invalid": "{bad}", value: "{draft}",
                                                                            onmounted: move |e: MountedEvent| { let d = e.data(); spawn(async move { let _ = d.set_focus(true).await; }); },
                                                                            oninput: move |e| draft.set(e.value()),
                                                                            onkeydown: move |e: KeyboardEvent| match e.key() {
                                                                                Key::Enter => { commit(); }
                                                                                Key::Escape => { edit.set(None); bad.set(false); }
                                                                                Key::Tab => {
                                                                                    e.prevent_default();
                                                                                    if commit() {
                                                                                        let i = CRM_COLS.iter().position(|c| c.0 == k).unwrap_or(0);
                                                                                        let nk = CRM_COLS[(i + 1) % 6].0;
                                                                                        draft.set(cell_text_static(&rows.peek(), id, nk));
                                                                                        edit.set(Some((id, nk)));
                                                                                    }
                                                                                }
                                                                                _ => {}
                                                                            } }
                                                                    }
                                                                } else {
                                                                    {crm_cell(&r, k)}
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
                        }
                    }
                    if !loading && all.is_empty() {
                        div { class: "oxv-none",
                            span { class: "oxv-none-g", {nico("building")} }
                            p { class: "oxv-none-t", "No companies yet" }
                            p { class: "oxv-none-l", "Add a row, or link a CRM in Integrations to bring yours in." }
                        }
                    } else if !loading && shown.is_empty() {
                        p { class: "oxscn-hint oxvc-nohit", role: "status", "No company matches these filters." }
                    }
                    div { class: "oxvc-foot",
                        button { r#type: "button", class: "oxv-link oxvc-add", disabled: loading,
                            onclick: move |_| { let id = add_company(&mut rows.write()); edit.set(Some((id, "name"))); draft.set("Untitled".to_string()); },
                            {nico("plus")} span { "Add row" } }
                        span { class: "oxscn-grow" }
                        span { class: "oxv-q", "Double-click a cell, or press Enter on it, to edit" }
                    }
                }
                p { class: "oxscn-sr", role: "status", "{said}" }
            }
        }
    }
}

fn cell_text_static(rows: &[Company], id: u32, col: &str) -> String {
    rows.iter()
        .find(|r| r.id == id)
        .map(|r| match col {
            "name" => r.name.clone(),
            "cat" => r.cat.clone(),
            "domain" => r.domain.clone(),
            "owner" => r.owner.clone(),
            "value" => r.value.to_string(),
            "stage" => r.stage.key().to_string(),
            _ => r.last.clone(),
        })
        .unwrap_or_default()
}

fn crm_cell(r: &Company, col: &str) -> Element {
    match col {
        "name" => {
            rsx! { span { class: "oxvc-co", BrandIcon { name: r.name.clone(), size: 15.0, coloured: true, fallback: true } span { class: "oxv-ell", "{r.name}" } } }
        }
        "cat" => rsx! { span { class: "oxvc-tag", "data-tone": "{cat_tone(&r.cat)}", "{r.cat}" } },
        "domain" => rsx! { span { class: "oxvc-dom", "{r.domain}" } },
        "value" => rsx! { span { class: "oxvc-num", "{money(r.value)}" } },
        "stage" => {
            rsx! { span { class: "oxvc-tag", "data-tone": "{r.stage.key()}", "{r.stage.label()}" } }
        }
        "owner" => rsx! { span { class: "oxv-ell", "{r.owner}" } },
        _ => rsx! { span { class: "oxv-ell oxv-q2", "{r.last}" } },
    }
}

// ───────────────────────────────────────────────────────────── Analytics ──

/// A line chart: series (name, tone, values, dashed), an axis top, three day labels. ← / → (or
/// hovering a day) move a crosshair whose readout is announced.
#[component]
fn LineChart(
    label: String,
    series: Vec<(String, String, Vec<f64>, bool)>,
    top: f64,
    days: Vec<String>,
    #[props(default = 180)] h: u32,
    #[props(default)] money: bool,
) -> Element {
    let mut at = use_signal(|| None::<usize>);
    let w = 700.0;
    let hh = h as f64;
    let n = series.first().map(|s| s.2.len()).unwrap_or(7).max(2);
    let fmt = move |v: f64| {
        if money {
            dollars_k(v)
        } else if v >= 1000.0 {
            format!("{:.1}k", v / 1000.0)
        } else {
            format!("{v:.0}")
        }
    };
    let ticks = [top, top / 2.0, 0.0];
    let read = at().map(|i| {
        series
            .iter()
            .filter(|s| !s.3)
            .map(|s| format!("{} {}", s.0, fmt(s.2.get(i).copied().unwrap_or(0.0))))
            .collect::<Vec<_>>()
            .join(" · ")
    });
    let cx = at().map(|i| i as f64 / (n - 1) as f64 * 100.0);
    rsx! {
        div { class: "oxva-chart", tabindex: "0", role: "img", "aria-label": "{label}",
            onkeydown: move |e: KeyboardEvent| {
                let cur = at();
                let j = match e.key() { Key::ArrowRight => cur.map(|i| (i + 1).min(n - 1)).unwrap_or(0), Key::ArrowLeft => cur.map(|i| i.saturating_sub(1)).unwrap_or(n - 1), Key::Escape => { at.set(None); return } _ => return };
                e.prevent_default();
                at.set(Some(j));
            },
            onmouseleave: move |_| at.set(None),
            div { class: "oxva-y", "aria-hidden": "true", for (i, t) in ticks.iter().enumerate() { span { key: "{i}", "{fmt(*t)}" } } }
            div { class: "oxva-plot",
                svg { class: "oxva-svg", view_box: "0 0 {w} {hh}", preserve_aspect_ratio: "none", "aria-hidden": "true",
                    for i in 0..3 { line { key: "{i}", class: "oxva-grid", x1: "0", x2: "{w}", y1: "{hh * i as f64 / 2.0}", y2: "{hh * i as f64 / 2.0}" } }
                    for (i, s) in series.iter().enumerate() {
                        polyline { key: "{i}", class: "oxva-line", "data-tone": "{s.1}", "data-dash": "{s.3}", points: "{polyline(&s.2, w, hh, top)}" }
                    }
                    for i in 0..n {
                        rect { key: "h{i}", x: "{w * (i as f64 - 0.5) / (n - 1) as f64}", y: "0", width: "{w / (n - 1) as f64}", height: "{hh}", fill: "transparent",
                            onmouseenter: move |_| at.set(Some(i)) }
                    }
                }
                if let Some(x) = cx {
                    span { class: "oxva-cross", style: "left:{x:.2}%", "aria-hidden": "true" }
                    if let Some(r) = read.clone() { span { class: "oxva-tip", "data-right": "{x > 60.0}", style: "left:{x:.2}%", "{r}" } }
                }
                div { class: "oxva-x", "aria-hidden": "true", for (i, d) in days.iter().enumerate() { span { key: "{i}", "{d}" } } }
            }
            p { class: "oxscn-sr", "aria-live": "polite", if let Some(r) = read { "{r}" } }
        }
    }
}

fn spark(trend: &str) -> Element {
    let pts = match trend {
        "up" => "0,14 8,11 14,13 22,7 30,9 40,3",
        "down" => "0,4 8,9 14,6 22,12 30,10 40,14",
        _ => "0,9 40,9",
    };
    rsx! { svg { class: "oxva-spark", "data-t": "{trend}", width: "42", height: "18", view_box: "0 0 40 18", "aria-hidden": "true", polyline { points: "{pts}" } } }
}

/// Screen – Analytics: Analytics / Revenue with a date range, Export and Save as report; KPI tiles,
/// revenue by channel (legend toggles each line), the revenue breakdown, top products and two
/// weekly trends. Charts take ← / → for a crosshair readout. States: `sample`, `loading`,
/// `empty`, `compare` (the previous period dashed under each line).
#[component]
pub fn AnalyticsScreen(
    /// `inset` or `flat`.
    #[props(default = "inset".to_string())]
    layout: String,
    /// A state key.
    #[props(default = "sample".to_string())]
    state: String,
    /// Height px.
    #[props(default = 640)]
    height: u32,
    /// The opening range: `7d`, `30d` or `90d`.
    #[props(default = "7d".to_string())]
    range: String,
) -> Element {
    let (st, sk) = split_state(&state);
    let thr = use_threads(st);
    let active = use_signal(|| 0u32);
    let mut rg = use_signal(move || range.clone());
    let mut cmp = use_signal(move || sk == "compare");
    let mut saved = use_signal(|| false);
    let mut hide = use_signal(Vec::<String>::new);
    let mut said = use_signal(String::new);
    let r = rg();
    let loading = st == ScreenState::Loading;
    let blank = st == ScreenState::Empty;
    let rl = RANGES
        .iter()
        .find(|x| x.0 == r)
        .map(|x| x.1)
        .unwrap_or("Last 7 days");
    let ch = channels(&r);
    let top = axis_top(&ch.iter().map(|c| c.2).collect::<Vec<_>>());
    let days: Vec<String> = axis_days(&r).iter().map(|s| s.to_string()).collect();
    let hidden = hide();
    let mut series: Vec<(String, String, Vec<f64>, bool)> = vec![];
    for (n, t, v) in ch.iter() {
        if hidden.iter().any(|h| h == n) {
            continue;
        }
        if cmp() {
            series.push((
                format!("{n} before"),
                t.to_string(),
                v.iter().map(|x| x * 0.88).collect(),
                true,
            ));
        }
        series.push((n.to_string(), t.to_string(), v.to_vec(), false));
    }
    let tp = top_products(&r);
    let tp_max = tp.iter().map(|p| p.1).fold(1.0, f64::max);
    let range_items: Vec<(&'static str, &'static str, &'static str)> =
        RANGES.iter().map(|(k, l, _)| (*k, *l, "")).collect();
    rsx! {
        AppFrame { layout, state: st.key().to_string(), height, threads: thr, active, side: "revenue", nav_on: "revenue", crumb: "Analytics", title: "Revenue",
            actions: rsx! {
                MenuButton { label: rl, aria: "Date range", icon: "cal", right: true, caret: true, class: "oxv-ghost", items: range_items, checked: r.clone(), on_pick: move |k: String| rg.set(k) }
                button { r#type: "button", class: "oxscn-btn oxva-cmp", "aria-pressed": "{cmp}", onclick: move |_| cmp.toggle(), {nico("cycle")} span { "Compare" } }
                MenuButton { label: "Export", aria: "Export", icon: "download", right: true, class: "oxv-ghost",
                    items: vec![("csv", "CSV file", ""), ("pdf", "PDF summary", ""), ("sheet", "Google Sheets", "googlesheets")],
                    on_pick: move |k: String| said.set(format!("Exported revenue as {}", match k.as_str() { "csv" => "CSV", "pdf" => "PDF", _ => "a sheet" })) }
                button { r#type: "button", class: "oxv-pri oxv-pri-s", "aria-pressed": "{saved}", onclick: move |_| { saved.toggle(); said.set(if saved() { "Saved to Reports".to_string() } else { "Removed from Reports".to_string() }); },
                    if saved() { {nico("check")} span { "Saved" } } else { span { "Save as report" } } }
            },
            div { class: "oxva",
                div { class: "oxva-kpis",
                    for k in kpis(&r) {
                        div { key: "{k.name}", class: "oxva-kpi oxva-card",
                            p { class: "oxva-cap", "{k.name}" }
                            if loading { span { class: "oxv-skl oxva-skv" } }
                            else if blank { p { class: "oxva-big oxv-q2", "—" } }
                            else {
                                p { class: "oxva-v",
                                    span { class: "oxva-big", "{k.value}" }
                                    span { class: "oxva-d", "data-good": "{k.good}", "{k.delta}" }
                                }
                                span { class: "oxva-bars", "aria-hidden": "true",
                                    for (i, b) in k.bars.iter().enumerate() { span { key: "{i}", "data-last": "{i == 6}", style: "height:{(*b as u32 * 22 / 100).max(4)}px" } }
                                }
                            }
                        }
                    }
                }
                div { class: "oxva-row2",
                    div { class: "oxva-card oxva-chan",
                        p { class: "oxva-cap", "Revenue by channel" }
                        if loading { span { class: "oxv-skl oxva-skv" } div { class: "oxva-skc" } }
                        else if blank { div { class: "oxv-none", p { class: "oxv-none-t", "No sales in this range" } p { class: "oxv-none-l", "Revenue shows up here as soon as the first order is paid." } } }
                        else {
                            p { class: "oxva-v", span { class: "oxva-big", "{dollars_k(channel_total(&r))}" } span { class: "oxva-d oxva-chip", "data-good": "true", if cmp() { "+12% vs before" } else { "+$7.9k" } } }
                            LineChart { label: "Revenue by channel, {rl}", series, top, days: days.clone(), money: true }
                        }
                        div { class: "oxva-foot",
                            span { "Top 4 channels" }
                            span { class: "oxscn-grow" }
                            if !loading && !blank {
                                for (n, t, _) in ch.iter() {
                                    {
                                        let nm = n.to_string();
                                        let off = hidden.contains(&nm);
                                        rsx! {
                                            button { key: "{n}", r#type: "button", class: "oxva-leg", "data-tone": "{t}", "aria-pressed": "{!off}",
                                                onclick: move |_| { let mut h = hide.write(); if let Some(i) = h.iter().position(|x| *x == nm) { h.remove(i); } else { h.push(nm.clone()); } },
                                                span { class: "oxva-sw" } span { "{n}" } }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    div { class: "oxva-card oxva-brk",
                        p { class: "oxva-cap", "Revenue breakdown" }
                        if loading { {skel(6)} }
                        else {
                            ul { class: "oxva-brk-l", "aria-label": "Revenue breakdown",
                                for (i, (n, v, t)) in breakdown(&r).into_iter().enumerate() {
                                    li { key: "{n}", class: "oxva-brk-r", "data-alt": "{i % 2 == 1}", "data-total": "{i == 5}",
                                        span { class: "oxva-brk-n", "{n}" }
                                        span { class: "oxva-brk-v", if blank { "$0" } else { "{dollars(v)}" } }
                                        {spark(if blank { "flat" } else { t })}
                                    }
                                }
                            }
                        }
                    }
                }
                div { class: "oxva-row3",
                    div { class: "oxva-card oxva-top",
                        p { class: "oxva-cap", "Top products" }
                        if loading { {skel(6)} }
                        else if blank { p { class: "oxv-quiet", "Your best sellers will rank here." } }
                        else {
                            ul { class: "oxva-top-l",
                                for (n, v) in tp.iter() {
                                    li { key: "{n}", class: "oxva-top-r",
                                        span { class: "oxva-top-h", span { class: "oxva-q", "{n}" } span { class: "oxva-num", "{dollars(*v)}" } }
                                        span { class: "oxva-meter", role: "meter", "aria-label": "{n}", "aria-valuenow": "{v:.0}", "aria-valuemin": "0", "aria-valuemax": "{tp_max:.0}",
                                            span { style: "width:{v / tp_max * 100.0:.1}%" } }
                                    }
                                }
                            }
                        }
                    }
                    for (t, head, delta, now, before, tp2) in weekly(&r) {
                        div { key: "{t}", class: "oxva-card oxva-wk",
                            p { class: "oxva-cap", if t == "Sessions" { "Sessions over time" } else { "{t}" } }
                            if loading { span { class: "oxv-skl oxva-skv" } div { class: "oxva-skc oxva-skc-s" } }
                            else if blank { p { class: "oxv-quiet", "Nothing to compare yet." } }
                            else {
                                p { class: "oxva-v", span { class: "oxva-big", "{head}" } span { class: "oxva-d oxva-chip", "data-good": "true", "{delta}" } }
                                LineChart { label: "{t}, this week against last", series: vec![("This week".to_string(), "blue".to_string(), now.to_vec(), false), ("Last week".to_string(), "sky".to_string(), before.to_vec(), false)],
                                    top: tp2, days: days.clone(), h: 110, money: t != "Sessions" }
                            }
                            div { class: "oxva-foot", span { "This week vs. last week" } }
                        }
                    }
                }
                p { class: "oxscn-sr", role: "status", "{said}" }
            }
        }
    }
}
