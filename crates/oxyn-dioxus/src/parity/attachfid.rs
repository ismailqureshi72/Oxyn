//! Attachments family (variants 2): [`FileIcon`] (a page with a coloured type badge, a landscape
//! for images), [`UploadDropzone`] (drag a file in – or press it – to upload; too-big files are
//! refused), [`AttachmentChips`] (chips with ring progress, failed and too-big states, Retry and
//! +N overflow), and two Oxyn originals: [`PolaroidStack`] (image attachments that fan out) and
//! [`DropLanes`] (a drop target that splits into where the file should go). Models:
//! `oxyn_core::parity::attachfid`. Styles: `attachfid.css` (`oxfic-`, `oxud-`, `oxac-`, `oxps-`,
//! `oxdrl-`). Timed states end on the CSS animation's end, so no timer is needed.

use dioxus::prelude::*;
use oxyn_core::parity::attachfid::{
    DemoFile, LANES, Refusal, check_file, demo_files, fan, file_badge, file_family, file_said,
    fmt_bytes, refusal_line, upload_ms,
};

/// A file-type icon: a page with a folded corner and the type's badge in its colour (`PDF`,
/// `XLS`, `MP4`, `RS`), or a small landscape for images.
#[component]
pub fn FileIcon(
    /// File name (the extension picks the icon).
    name: String,
    /// Height in px.
    #[props(default = 32.0)]
    size: f32,
) -> Element {
    let (badge, hex) = file_badge(&name);
    let w = size * 0.8;
    let bw = 6.0 + badge.len() as f32 * 5.4;
    let image = file_family(&name) == "image";
    rsx! {
        svg { class: "oxfic", width: "{w}", height: "{size}", view_box: "0 0 32 40", "aria-hidden": "true",
            path { class: "oxfic-page", d: "M5 1.5h15.5L29.5 10.5v26a2 2 0 0 1-2 2h-22.5a2 2 0 0 1-2-2v-33a2 2 0 0 1 2-2z" }
            path { class: "oxfic-fold", d: "M20.5 1.5v7a2 2 0 0 0 2 2h7" }
            if image {
                rect { x: "8", y: "15", width: "17", height: "13", rx: "2.5", fill: "#{hex}", opacity: ".18" }
                circle { cx: "12.5", cy: "19", r: "1.9", fill: "#{hex}" }
                path { d: "M8 26.5 13.5 22l3.5 2.8 3-2.3 5 4", fill: "none", stroke: "#{hex}", stroke_width: "1.8", stroke_linejoin: "round", stroke_linecap: "round" }
            } else {
                path { class: "oxfic-lines", d: "M9 15h12M9 19h14M9 23h9" }
            }
            rect { x: "0.5", y: "26", width: "{bw}", height: "11", rx: "3", fill: "#{hex}" }
            text { x: "{0.5 + bw / 2.0}", y: "33.9", text_anchor: "middle", font_size: "7.4", font_weight: "700", fill: "#fff",
                font_family: "Inter,ui-sans-serif,system-ui,sans-serif", letter_spacing: ".2", "{badge}" }
        }
    }
}

fn upload_glyph() -> Element {
    rsx! {
        svg { width: "20", height: "20", view_box: "0 0 24 24", fill: "none", stroke: "currentColor", stroke_width: "1.8",
            stroke_linecap: "round", stroke_linejoin: "round", "aria-hidden": "true",
            path { d: "M12 15V4M7.5 8.5 12 4l4.5 4.5M4 15v3a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2v-3" }
        }
    }
}

fn x_glyph() -> Element {
    rsx! {
        svg { width: "12", height: "12", view_box: "0 0 24 24", fill: "none", stroke: "currentColor", stroke_width: "2.4",
            stroke_linecap: "round", "aria-hidden": "true", path { d: "M6 6l12 12M18 6 6 18" } }
    }
}

fn check_small() -> Element {
    rsx! {
        svg { width: "14", height: "14", view_box: "0 0 24 24", fill: "none", stroke: "currentColor", stroke_width: "2.4",
            stroke_linecap: "round", stroke_linejoin: "round", "aria-hidden": "true", path { d: "m5 12.5 4.5 4.5L19 7.5" } }
    }
}

/// Attachments – Upload dropzone: a box that takes a file dragged onto it (or the sample chips
/// under it, by drag, click or Enter). It shows the file with a progress bar and percent, says
/// Uploaded, then waits for the next; a file over the limit is refused in red.
#[component]
pub fn UploadDropzone(
    /// Files offered under the box (a PDF, an image and a too-big video when `None`).
    #[props(default)]
    files: Option<Vec<DemoFile>>,
    /// Size limit in MB.
    #[props(default = 10.0)]
    max_mb: f64,
    /// Accepted families (`pdf`, `image`, `doc` …); empty takes any.
    #[props(default)]
    accept: Vec<String>,
    /// Megabytes a second for the demo upload.
    #[props(default = 1.5)]
    speed: f64,
    /// A hairline round the box.
    #[props(default)]
    stroke: bool,
    /// Corner radius in px.
    #[props(default = 20.0)]
    corner: f32,
    /// A file finished uploading (its name).
    #[props(default)]
    on_upload: EventHandler<String>,
) -> Element {
    let list = files.unwrap_or_else(|| {
        let d = demo_files();
        vec![d[0].clone(), d[1].clone(), d[4].clone()]
    });
    let n = list.len().max(1);
    let mut phase = use_signal(|| "idle");
    let mut cur = use_signal(|| 0usize);
    let mut why = use_signal(String::new);
    let mut over = use_signal(|| false);
    let mut drag = use_signal(|| None::<usize>);
    let mut next = use_signal(|| 0usize);
    let mut said = use_signal(String::new);
    let all = list.clone();
    let acc = accept.clone();
    let start = use_callback(move |i: usize| {
        if phase() != "idle" {
            return;
        }
        let f = &all[i % all.len().max(1)];
        let a: Vec<&str> = acc.iter().map(String::as_str).collect();
        cur.set(i);
        match check_file(f, max_mb, &a) {
            Ok(()) => {
                phase.set("up");
                said.set(format!("Uploading {}.", f.name));
            }
            Err(r) => {
                why.set(refusal_line(r, max_mb));
                phase.set("no");
                said.set(format!("{} refused: {}.", f.name, refusal_line(r, max_mb)));
            }
        }
    });
    let f = list[cur() % n].clone();
    let dur = upload_ms(f.bytes, speed);
    let size = fmt_bytes(f.bytes);
    let limit = if accept.is_empty() {
        format!("Up to {}", fmt_bytes((max_mb * 1e6) as u64))
    } else {
        format!(
            "{} · up to {}",
            accept.join(", ").to_uppercase(),
            fmt_bytes((max_mb * 1e6) as u64)
        )
    };
    let box_style = format!("border-radius:{corner}px");
    let ph = phase();
    let fname = f.name;

    rsx! {
        div { class: "oxud", "data-stroke": "{stroke}",
            div { class: "oxud-box", style: "{box_style}", role: "button", tabindex: "0", "data-over": "{over()}", "data-phase": "{ph}",
                "aria-label": "Drop a file here, or press to add the next sample file",
                ondragover: move |e: DragEvent| { e.prevent_default(); over.set(true); },
                ondragleave: move |_| over.set(false),
                ondrop: move |e: DragEvent| { e.prevent_default(); over.set(false); if let Some(i) = drag() { start.call(i); } drag.set(None); },
                onclick: move |_| { let i = next(); next.set((i + 1) % n); start.call(i); },
                onkeydown: move |e: KeyboardEvent| if e.key() == Key::Enter || e.key() == Key::Character(" ".into()) { e.prevent_default(); let i = next(); next.set((i + 1) % n); start.call(i); },
                if ph == "up" || ph == "done" {
                    div { class: "oxud-file",
                        FileIcon { name: fname.to_string(), size: 40.0 }
                        div { class: "oxud-ft",
                            span { class: "oxud-fn", "{fname}" }
                            span { class: "oxud-fm",
                                "{size} · "
                                if ph == "up" { span { class: "oxud-pct", style: "--dur:{dur}ms" } } else { span { class: "oxud-ok", "Uploaded" } }
                            }
                            span { class: "oxud-bar",
                                i { class: "oxud-fill", style: "--dur:{dur}ms",
                                    onanimationend: move |_| { phase.set("done"); said.set(format!("{fname} uploaded.")); on_upload.call(fname.to_string()); } }
                            }
                        }
                        if ph == "done" { span { class: "oxud-wait", "aria-hidden": "true", onanimationend: move |_| phase.set("idle") } }
                    }
                } else {
                    div { class: "oxud-idle",
                        span { class: "oxud-tile", {upload_glyph()} }
                        if ph == "no" {
                            span { class: "oxud-h oxud-no", "{why}" }
                            span { class: "oxud-wait oxud-long", "aria-hidden": "true", onanimationend: move |_| phase.set("idle") }
                        } else {
                            span { class: "oxud-h", if over() { "Let go to upload" } else { "Drop a file here" } }
                        }
                        span { class: "oxud-s", "{limit}" }
                    }
                }
            }
            div { class: "oxud-chips",
                for (i, c) in list.iter().enumerate() {
                    {
                        let busy = ph != "idle" && cur() % n == i;
                        let lab = format!("{}. Drag into the box, or press to upload.", file_said(c));
                        rsx! {
                            button { key: "{c.name}", class: "oxud-chip", r#type: "button", draggable: "true", "data-busy": "{busy}",
                                "aria-label": "{lab}",
                                ondragstart: move |_| drag.set(Some(i)),
                                ondragend: move |_| { over.set(false); },
                                onclick: move |_| start.call(i),
                                FileIcon { name: c.name.to_string(), size: 44.0 }
                                span { class: "oxud-cn", "{c.name}" }
                            }
                        }
                    }
                }
            }
            div { class: "oxud-sr", role: "status", "aria-live": "polite", "{said}" }
        }
    }
}

/// One chip's state in [`AttachmentChips`].
#[derive(Clone, PartialEq)]
struct Chip {
    id: u32,
    file: DemoFile,
    state: &'static str,
    run: u32,
}

fn chip_of(id: u32, f: DemoFile, max_mb: f64) -> Chip {
    let state = match check_file(&f, max_mb, &[]) {
        Err(Refusal::TooBig) => "big",
        _ => "up",
    };
    Chip {
        id,
        file: f,
        state,
        run: 0,
    }
}

/// Attachments – Attachment chips: files on their way into a prompt. Each chip draws a ring of
/// progress round its icon while it uploads, then shows type and size; a dropped connection
/// shows Upload failed with Retry, a file over the limit stays red until removed. Past
/// `max_visible` chips the rest fold into a +N chip.
#[component]
pub fn AttachmentChips(
    /// Starting files (six samples when `None`).
    #[props(default)]
    files: Option<Vec<DemoFile>>,
    /// Size limit in MB.
    #[props(default = 25.0)]
    max_mb: f64,
    /// Chips shown before +N.
    #[props(default = 5)]
    max_visible: usize,
    /// Megabytes a second for the demo upload.
    #[props(default = 1.5)]
    speed: f64,
    /// Removed a chip (file name).
    #[props(default)]
    on_remove: EventHandler<String>,
) -> Element {
    let start = files.unwrap_or_else(|| {
        let d = demo_files();
        vec![
            d[0].clone(),
            d[5].clone(),
            d[4].clone(),
            d[1].clone(),
            d[2].clone(),
            d[3].clone(),
        ]
    });
    let pool = demo_files();
    let mut chips = use_signal(move || {
        start
            .into_iter()
            .enumerate()
            .map(|(i, f)| chip_of(i as u32 + 1, f, max_mb))
            .collect::<Vec<_>>()
    });
    let mut wide = use_signal(|| false);
    let mut add_at = use_signal(|| 6usize);
    let mut said = use_signal(String::new);
    let cs = chips();
    let shown = if wide() || cs.len() <= max_visible {
        cs.len()
    } else {
        max_visible.saturating_sub(1).max(1)
    };
    let more = cs.len() - shown;
    let up = cs.iter().filter(|c| c.state == "up").count();
    let head = match (cs.len(), up) {
        (0, _) => "No files yet".to_string(),
        (n, 0) => format!("{n} file{}", if n == 1 { "" } else { "s" }),
        (n, u) => format!("{n} files · {u} uploading"),
    };

    rsx! {
        div { class: "oxac",
            div { class: "oxac-tray", role: "list", "aria-label": "Attachments",
                for c in cs.iter().take(shown).cloned() {
                    {
                        let dur = upload_ms(c.file.bytes, speed);
                        let id = c.id;
                        let name = c.file.name;
                        let (badge, _) = file_badge(name);
                        let size = fmt_bytes(c.file.bytes);
                        let flaky = c.file.flaky && c.run == 0;
                        let meta = match c.state {
                            "ok" => format!("{badge} · {size}"),
                            "fail" => "Upload failed".to_string(),
                            "big" => format!("Too big · {size}"),
                            _ => String::new(),
                        };
                        let lab = file_said(&c.file);
                        rsx! {
                            div { key: "{id}-{c.run}", class: "oxac-chip", role: "listitem", "data-state": "{c.state}", "aria-label": "{lab}",
                                span { class: "oxac-ic",
                                    FileIcon { name: name.to_string(), size: 30.0 }
                                    if c.state == "up" {
                                        svg { class: "oxac-ring", view_box: "0 0 40 40", "aria-hidden": "true",
                                            circle { class: "oxac-rt", cx: "20", cy: "20", r: "17" }
                                            circle { class: "oxac-rf", cx: "20", cy: "20", r: "17", "data-flaky": "{flaky}", style: "--dur:{dur}ms",
                                                onanimationend: move |_| {
                                                    let mut v = chips.write();
                                                    if let Some(x) = v.iter_mut().find(|x| x.id == id) {
                                                        x.state = if flaky { "fail" } else { "ok" };
                                                    }
                                                    said.set(if flaky { format!("{name}: upload failed.") } else { format!("{name} uploaded.") });
                                                }
                                            }
                                        }
                                    }
                                }
                                span { class: "oxac-txt",
                                    span { class: "oxac-nm", "{name}" }
                                    if c.state == "up" {
                                        span { class: "oxac-meta", "Uploading · " span { class: "oxud-pct", "data-flaky": "{flaky}", style: "--dur:{dur}ms" } }
                                    } else {
                                        span { class: "oxac-meta", "{meta}" }
                                    }
                                }
                                if c.state == "fail" {
                                    button { class: "oxac-retry", r#type: "button", "aria-label": "Retry {name}",
                                        onclick: move |_| {
                                            let mut v = chips.write();
                                            if let Some(x) = v.iter_mut().find(|x| x.id == id) { x.state = "up"; x.run += 1; }
                                            said.set(format!("Retrying {name}."));
                                        },
                                        "Retry"
                                    }
                                }
                                button { class: "oxac-x", r#type: "button", "aria-label": "Remove {name}",
                                    onclick: move |_| {
                                        chips.write().retain(|x| x.id != id);
                                        said.set(format!("Removed {name}."));
                                        on_remove.call(name.to_string());
                                    },
                                    {x_glyph()}
                                }
                            }
                        }
                    }
                }
                if more > 0 {
                    button { class: "oxac-more", r#type: "button", "aria-label": "Show {more} more files", onclick: move |_| wide.set(true), "+{more}" }
                } else if wide() && cs.len() > max_visible {
                    button { class: "oxac-more", r#type: "button", onclick: move |_| wide.set(false), "Fewer" }
                }
            }
            div { class: "oxac-field",
                span { class: "oxac-ph", "Ask about these files…" }
                div { class: "oxac-bar",
                    button { class: "oxac-add", r#type: "button",
                        onclick: move |_| {
                            let i = add_at();
                            let f = pool[i % pool.len()].clone();
                            add_at.set(i + 1);
                            let id = chips.peek().iter().map(|c| c.id).max().unwrap_or(0) + 1;
                            said.set(format!("Added {}.", f.name));
                            chips.write().push(chip_of(id, f, max_mb));
                        },
                        span { "+" } "Add file"
                    }
                    span { class: "oxac-count", "{head}" }
                    span { class: "oxac-send", "data-ready": "{up == 0 && !cs.is_empty()}", "aria-hidden": "true", {check_small()} }
                }
            }
            div { class: "oxud-sr", role: "status", "aria-live": "polite", "{said}" }
        }
    }
}

/// A generated picture for the stack (abstract sky, sun and hills – no real photos).
fn scene(k: usize) -> String {
    let s = [
        ("#9CC9F5", "#E8F1FB", "#F7C873", "#5C9B6B", "#3E7A52"),
        ("#F6B49A", "#FCE3D2", "#FFF1C2", "#B5655B", "#7E4A57"),
        ("#3E4B8C", "#7D86C7", "#F4F1E8", "#2B3366", "#1C2147"),
        ("#BFE3D6", "#F0F8F4", "#F9D776", "#64A88C", "#2F7A63"),
        ("#E3B8E8", "#F7EAF9", "#FFFFFF", "#9A6BB0", "#6C4485"),
        ("#F3D27A", "#FBF0CF", "#FFFFFF", "#C98A4B", "#8C5A33"),
    ][k % 6];
    format!(
        "background:radial-gradient(circle at 72% 30%,{} 0 9%,transparent 10%),radial-gradient(120% 70% at 25% 105%,{} 0 45%,transparent 46%),radial-gradient(130% 80% at 85% 112%,{} 0 48%,transparent 49%),linear-gradient(180deg,{},{})",
        s.2, s.4, s.3, s.0, s.1
    )
}

/// Attachments – Polaroid stack (an Oxyn original): image attachments as a small stack of prints;
/// pointing at the stack (or focusing it) fans them out, a print comes to the front when pressed,
/// and each can be taken out. The caption under the stack names the front print.
#[component]
pub fn PolaroidStack(
    /// How many prints to start with (1–6).
    #[props(default = 4)]
    count: usize,
    /// Fan spread in degrees.
    #[props(default = 36.0)]
    spread: f32,
    /// Starts fanned out (for previews).
    #[props(default)]
    default_open: bool,
    /// A print was removed (its name).
    #[props(default)]
    on_remove: EventHandler<String>,
) -> Element {
    let mut prints = use_signal(move || (0..count.clamp(1, 6)).collect::<Vec<usize>>());
    let mut front = use_signal(|| 0usize);
    let mut hover = use_signal(|| false);
    let mut said = use_signal(String::new);
    let ps = prints();
    let n = ps.len();
    let open = hover() || default_open;
    let fr = ps.iter().position(|p| *p == front()).unwrap_or(0);
    let name_of = |k: usize| format!("photo-{}.jpg", k + 1);
    let cap = if n == 0 {
        "No images".to_string()
    } else {
        format!("{} · {} of {n}", name_of(ps[fr]), fr + 1)
    };

    rsx! {
        div { class: "oxps", "data-open": "{open}",
            div { class: "oxps-stage", role: "group", "aria-label": "{n} images",
                onpointerenter: move |_| hover.set(true),
                onpointerleave: move |_| hover.set(false),
                onfocusin: move |_| hover.set(true),
                onfocusout: move |_| hover.set(false),
                for (i, k) in ps.iter().copied().enumerate() {
                    {
                        let (a, x) = fan(i, n, open, spread);
                        let z = if k == front() { 20 } else { 10 - i };
                        let lift = if open { -10.0 } else { 0.0 };
                        let st = format!("transform:translate({x}px,{lift}px) rotate({a}deg);z-index:{z}");
                        let pic = scene(k);
                        let nm = name_of(k);
                        let nm2 = nm.clone();
                        rsx! {
                            div { key: "{k}", class: "oxps-card", style: "{st}", "data-front": "{k == front()}",
                                button { class: "oxps-pic", r#type: "button", "aria-label": "Bring {nm} to the front",
                                    onclick: move |_| { front.set(k); said.set(format!("{nm} in front.")); },
                                    span { class: "oxps-img", style: "{pic}" }
                                }
                                button { class: "oxps-x", r#type: "button", "aria-label": "Remove {nm2}",
                                    onclick: move |_| {
                                        prints.write().retain(|p| *p != k);
                                        said.set(format!("Removed {nm2}."));
                                        on_remove.call(nm2.clone());
                                    },
                                    {x_glyph()}
                                }
                            }
                        }
                    }
                }
                if n == 0 { span { class: "oxps-empty", "No images attached" } }
            }
            div { class: "oxps-foot",
                span { class: "oxps-cap", "{cap}" }
                button { class: "oxps-add", r#type: "button", disabled: n >= 6,
                    onclick: move |_| {
                        let have = prints();
                        if let Some(k) = (0..6).find(|k| !have.contains(k)) { prints.write().push(k); front.set(k); said.set(format!("Added photo-{}.jpg.", k + 1)); }
                    },
                    "+ Add image"
                }
            }
            div { class: "oxud-sr", role: "status", "aria-live": "polite", "{said}" }
        }
    }
}

/// Attachments – Drop lanes (an Oxyn original): one drop target that splits into where a file
/// should go – this message, the project's files, or a style reference. Drag a chip onto a lane,
/// or press a chip and then a lane. Filed chips can be sent back.
#[component]
pub fn DropLanes(
    /// Files to place (four samples when `None`).
    #[props(default)]
    files: Option<Vec<DemoFile>>,
    /// A file went to a lane: (file name, lane key).
    #[props(default)]
    on_file: EventHandler<(String, String)>,
) -> Element {
    let list = files.unwrap_or_else(|| {
        let d = demo_files();
        vec![d[0].clone(), d[1].clone(), d[3].clone(), d[2].clone()]
    });
    let n = list.len();
    let mut lane_of = use_signal(move || vec![None::<usize>; n]);
    let mut held = use_signal(|| None::<usize>);
    let mut over = use_signal(|| None::<usize>);
    let mut said = use_signal(String::new);
    let names: Vec<&'static str> = list.iter().map(|f| f.name).collect();
    let names2 = names.clone();
    let put = use_callback(move |(f, l): (usize, usize)| {
        lane_of.write()[f] = Some(l);
        held.set(None);
        over.set(None);
        said.set(format!("{} filed under {}.", names[f], LANES[l].1));
        on_file.call((names[f].to_string(), LANES[l].0.to_string()));
    });
    let lo = lane_of();
    let active = held().is_some();
    let loose: Vec<usize> = (0..n).filter(|i| lo[*i].is_none()).collect();

    rsx! {
        div { class: "oxdrl", "data-active": "{active}",
            div { class: "oxdrl-tray",
                span { class: "oxdrl-tl", if loose.is_empty() { "Every file has a place" } else if active { "Now pick a lane" } else { "Drag a file to a lane, or press it" } }
                div { class: "oxdrl-chips",
                    for i in loose.iter().copied() {
                        {
                            let f = list[i].clone();
                            let on = held() == Some(i);
                            rsx! {
                                button { key: "{f.name}", class: "oxdrl-chip", r#type: "button", draggable: "true", "aria-pressed": "{on}",
                                    "aria-label": "{file_said(&f)}",
                                    ondragstart: move |_| held.set(Some(i)),
                                    ondragend: move |_| over.set(None),
                                    onclick: move |_| held.set(if on { None } else { Some(i) }),
                                    FileIcon { name: f.name.to_string(), size: 22.0 }
                                    span { "{f.name}" }
                                }
                            }
                        }
                    }
                }
            }
            div { class: "oxdrl-lanes",
                for (l, (key, label, hint)) in LANES.iter().enumerate() {
                    {
                        let inl: Vec<usize> = (0..n).filter(|i| lo[*i] == Some(l)).collect();
                        let cnt = inl.len();
                        rsx! {
                            div { key: "{key}", class: "oxdrl-lane", "data-over": "{over() == Some(l)}", "data-k": "{key}",
                                ondragover: move |e: DragEvent| { e.prevent_default(); over.set(Some(l)); },
                                ondragleave: move |_| over.set(None),
                                ondrop: move |e: DragEvent| { e.prevent_default(); if let Some(f) = held() { put.call((f, l)); } },
                                button { class: "oxdrl-head", r#type: "button", disabled: !active, "aria-label": "File under {label}",
                                    onclick: move |_| if let Some(f) = held() { put.call((f, l)) },
                                    span { class: "oxdrl-ln", "{label}" }
                                    span { class: "oxdrl-n", "{cnt}" }
                                }
                                span { class: "oxdrl-hint", "{hint}" }
                                ul { class: "oxdrl-in",
                                    for i in inl.iter().copied() {
                                        {
                                            let nm = names2[i];
                                            rsx! {
                                                li { key: "{nm}", class: "oxdrl-item",
                                                    FileIcon { name: nm.to_string(), size: 18.0 }
                                                    span { class: "oxdrl-nm", "{nm}" }
                                                    button { class: "oxac-x", r#type: "button", "aria-label": "Send {nm} back",
                                                        onclick: move |_| { lane_of.write()[i] = None; said.set(format!("{nm} sent back.")); },
                                                        {x_glyph()}
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
            div { class: "oxud-sr", role: "status", "aria-live": "polite", "{said}" }
        }
    }
}
