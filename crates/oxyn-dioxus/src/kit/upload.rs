//! Uploads (plan #28, chunk 4b): `FileDropzone` (affectUI Dropzone, Arc UI File dropzone and the
//! Arc File upload block) and `AvatarUpload` (Arc UI Avatar upload). Behaviour-only rebuilds;
//! models in `oxyn_core::kit::upload`.

use std::rc::Rc;

use dioxus::prelude::*;
use oxyn_core::{
    ClockMs,
    kit::upload::{
        Crop, Status, Uploads, data_url, ext, image_size, kind, progress, size, time_left,
    },
};

use crate::composer::icons::Icon;

fn focus(d: Option<Rc<MountedData>>) {
    if let Some(d) = d {
        spawn(async move {
            let _ = d.set_focus(true).await;
        });
    }
}

fn kind_icon(name: &str) -> &'static str {
    match kind(name) {
        "image" => "image",
        "sheet" => "table",
        "archive" => "archive",
        "audio" => "audio",
        "video" => "video",
        "code" => "code",
        _ => "file",
    }
}

/// File dropzone: a generous target for one or more files, then a list of rows that upload with
/// progress and time left, fail with a reason and Retry, and collapse when removed.
///
/// - `variant: "fan"` (affectUI Dropzone): documents fan out over the target while files hover,
///   rows carry type badges, `autoplay` plays a sample drop of four files.
/// - `variant: "plain"` (Arc File dropzone / File upload): the icon lifts while files hover,
///   `list_placement` puts rows below or inside the target's edge and `compact_at` folds the prompt
///   to a slim bar once the list holds that many files.
///
/// Files that fail `accept` or `max_size` keep a row with the reason and never upload. Enter or
/// Space on the target opens the picker; in the list, Up/Down move to the same action on the
/// next row and Delete removes the focused row. Uploads are simulated on the clock (a name
/// containing "fail" fails halfway on its first attempt).
#[component]
pub fn FileDropzone(
    /// Main prompt.
    #[props(default = "Add files".to_string())]
    label: String,
    /// Second line.
    #[props(default = "Drop files here or choose from your device".to_string())]
    description: String,
    /// Small line under it (defaults to the accepted types, size and file limit).
    #[props(default)]
    note: Option<String>,
    /// Label while files hover.
    #[props(default = "Drop to upload".to_string())]
    drop_label: String,
    /// Accept list (".pdf, image/*").
    #[props(default)]
    accept: String,
    /// Largest file in bytes.
    #[props(default)]
    max_size: Option<u64>,
    /// Most files kept.
    #[props(default)]
    max_files: Option<usize>,
    /// Several files (a new file replaces the old one when false).
    #[props(default = true)]
    multiple: bool,
    /// `plain` · `fan`.
    #[props(default = "plain".to_string())]
    variant: String,
    /// `below` · `inside`.
    #[props(default = "below".to_string())]
    list_placement: String,
    /// Fold the prompt to a slim bar once the list holds this many files.
    #[props(default)]
    compact_at: Option<usize>,
    /// Rows present at mount (name, bytes), already uploaded.
    #[props(default)]
    preset: Vec<(String, u64)>,
    /// Play a sample drop of four files.
    #[props(default)]
    autoplay: bool,
    /// Disabled.
    #[props(default)]
    disabled: bool,
    /// Shared clock (progress).
    now: ClockMs,
    /// Names in the list whenever it changes.
    #[props(default)]
    on_change: EventHandler<Vec<String>>,
) -> Element {
    let accept0 = accept.clone();
    let mut u = use_signal(move || {
        let mut x = Uploads::new(&accept0, max_size, max_files);
        x.single = !multiple;
        let files: Vec<(&str, u64)> = preset.iter().map(|(n, b)| (n.as_str(), *b)).collect();
        x.add(&files, now);
        for r in &mut x.rows {
            r.status = Status::Done;
            r.preset = true;
        }
        if autoplay {
            x.add(
                &[
                    ("Q3 board deck.pdf", 4_200_000),
                    ("revenue-by-region.csv", 380_000),
                    ("hero-shot.png", 2_600_000),
                    ("contract-final-fail.docx", 1_100_000),
                ],
                now,
            );
            if let Some(r) = x.rows.iter_mut().find(|r| r.name.contains("fail")) {
                r.fail_at = Some(0.55);
                r.error = Some("Connection dropped".into());
            }
        }
        x
    });
    let mut hover = use_signal(|| false);
    let mut refused = use_signal(|| 0usize);
    let mut rowbtn = use_signal(Vec::<(u64, Option<Rc<MountedData>>)>::new);
    // Settle a copy on the clock for drawing; handlers settle the real one.
    let mut view = u();
    view.tick(now);
    let rows = view.rows.clone();
    let compact = view.compact(compact_at);
    let hint = note.clone().unwrap_or_else(|| view.hint());
    let mut add = move |files: Vec<(String, u64)>| {
        let refs: Vec<(&str, u64)> = files.iter().map(|(n, b)| (n.as_str(), *b)).collect();
        let mut w = u.write();
        w.tick(now);
        let r = w.add(&refs, now);
        if let Some(row) = w
            .rows
            .iter_mut()
            .rev()
            .take(refs.len())
            .find(|r| r.name.to_lowercase().contains("fail"))
        {
            row.fail_at = Some(0.5);
            row.error = Some("Upload failed · server timed out".into());
        }
        refused.set(r);
        on_change.call(w.rows.iter().map(|r| r.name.clone()).collect());
    };
    let inside = list_placement == "inside";
    let fan = variant == "fan";
    let n = rows.len();
    let list = rsx! {
        if !rows.is_empty() {
            ul { class: "oxu-list", "aria-label": "Files",
                for (ri, r) in rows.iter().enumerate() {
                    {
                        let p = progress(r, now);
                        let id = r.id;
                        let el = now.since(r.started).as_millis() as u64;
                        let st = match r.status { Status::Done => "done", Status::Failed => "failed", Status::Uploading => "up", Status::Queued => "queued" };
                        let line = match r.status {
                            Status::Done => format!("{} · Uploaded", size(r.bytes)),
                            Status::Failed => r.error.clone().unwrap_or_else(|| "Failed".into()),
                            _ => match time_left(p, el) {
                                Some(t) => format!("{} of {} · {t}", size((r.bytes as f32 * p) as u64), size(r.bytes)),
                                None => format!("{} · Starting…", size(r.bytes)),
                            },
                        };
                        let can_retry = r.status == Status::Failed && !r.rejected;
                        rsx! {
                            li { key: "{id}", class: if r.preset { "oxu-row preset" } else { "oxu-row" }, "data-s": "{st}",
                                if fan {
                                    span { class: "oxu-badge", "data-k": "{kind(&r.name)}", "{ext(&r.name)}" }
                                } else {
                                    span { class: "oxu-ico", Icon { name: kind_icon(&r.name), size: 14.0 } }
                                }
                                div { class: "oxu-main",
                                    div { class: "oxu-top",
                                        span { class: "oxu-name", title: "{r.name}", "{r.name}" }
                                        if r.status == Status::Done { span { class: "oxu-ok", Icon { name: "check", size: 11.0 } } }
                                    }
                                    span { class: "oxu-line", role: if r.status == Status::Failed { "alert" }, "{line}" }
                                    if r.status == Status::Uploading || (r.status == Status::Failed && !r.rejected) {
                                        span { class: "oxu-bar", role: "progressbar", "aria-valuenow": "{(p * 100.0) as u32}", "aria-valuemin": "0", "aria-valuemax": "100", "aria-label": "{r.name}",
                                            span { style: "width:{p * 100.0}%" } }
                                    }
                                }
                                if can_retry {
                                    button { class: "oxu-act", r#type: "button", "aria-label": "Retry {r.name}",
                                        onclick: move |_| { let mut w = u.write(); w.tick(now); w.retry(id, now); },
                                        Icon { name: "retry", size: 12.0 }
                                    }
                                }
                                button { class: "oxu-act", r#type: "button", "aria-label": "Remove {r.name}",
                                    onmounted: move |e| rowbtn.write().push((id, Some(e.data()))),
                                    onclick: move |_| { u.write().remove(id); on_change.call(u.peek().rows.iter().map(|r| r.name.clone()).collect()); },
                                    onkeydown: move |e: KeyboardEvent| match e.key() {
                                        Key::Delete | Key::Backspace => { e.prevent_default(); u.write().remove(id); on_change.call(u.peek().rows.iter().map(|r| r.name.clone()).collect()); }
                                        Key::ArrowDown | Key::ArrowUp => {
                                            e.prevent_default();
                                            let d: i64 = if e.key() == Key::ArrowDown { 1 } else { -1 };
                                            let t = (ri as i64 + d).clamp(0, n as i64 - 1) as usize;
                                            let tid = u.peek().rows.get(t).map_or(0, |r| r.id);
                                            focus(rowbtn.peek().iter().rev().find(|x| x.0 == tid).and_then(|x| x.1.clone()));
                                        }
                                        _ => {}
                                    },
                                    Icon { name: "x", size: 12.0 }
                                }
                            }
                        }
                    }
                }
            }
        }
    };
    rsx! {
        div { class: "oxu-dz {variant}", "data-hover": "{hover()}", "data-compact": "{compact}", "data-inside": "{inside}", "data-disabled": "{disabled}",
            label { class: "oxu-target", "aria-disabled": "{disabled}",
                ondragover: move |e| { e.prevent_default(); if !disabled && !hover() { hover.set(true) } },
                ondragleave: move |_| hover.set(false),
                ondrop: move |e: DragEvent| {
                    e.prevent_default();
                    hover.set(false);
                    if disabled { return }
                    let files: Vec<(String, u64)> = e.data_transfer().files().iter().map(|f| (f.name(), f.size())).collect();
                    add(files);
                },
                if fan {
                    div { class: "oxu-fan", "aria-hidden": "true",
                        span { class: "oxu-doc a", Icon { name: "image", size: 14.0 } }
                        span { class: "oxu-doc b", Icon { name: "file", size: 14.0 } }
                        span { class: "oxu-doc c", Icon { name: "table", size: 14.0 } }
                    }
                } else {
                    span { class: "oxu-up", "aria-hidden": "true", Icon { name: "download", size: 16.0 } }
                }
                div { class: "oxu-copy",
                    span { class: "oxu-title", if hover() { "{drop_label}" } else { "{label}" } }
                    if !compact {
                        span { class: "oxu-desc", "{description}" }
                        span { class: "oxu-note", "{hint}" }
                    }
                }
                span { class: "oxu-browse", "aria-hidden": "true", "Browse" }
                input { r#type: "file", class: "oxu-file", multiple, accept: "{accept}", disabled,
                    "aria-label": "{label}. {description}. {hint}",
                    onchange: move |e: FormEvent| {
                        let files: Vec<(String, u64)> = e.files().iter().map(|f| (f.name(), f.size())).collect();
                        add(files);
                    },
                }
                if inside { {list.clone()} }
            }
            if refused() > 0 {
                div { class: "oxu-refused", role: "alert", "{refused()} file(s) not added · the list holds {max_files.unwrap_or(0)}" }
            }
            if !inside { {list} }
        }
    }
}

/// Avatar upload (Arc behaviour): click or drop a photo onto the avatar to crop it in place. The
/// avatar grows into a 184 px crop circle; drag the photo (past an edge it follows at a falling
/// rate, never more than a quarter of the circle, and springs back on release), zoom with the
/// wheel toward the pointer or the slider, arrows move it 2 % (Shift 10 %), + / − zoom, 0 resets,
/// Enter saves and Escape cancels. Saving uploads through a progress ring and settles; type, size
/// and upload errors show clearly with Retry. The host gets the crop square to cut.
#[component]
pub fn AvatarUpload(
    /// Heading beside the avatar.
    #[props(default = "Profile photo".to_string())]
    label: String,
    /// Helper copy (defaults to the accepted types and size limit).
    #[props(default)]
    hint: Option<String>,
    /// Current photo (URL or data URL).
    #[props(default)]
    src: Option<String>,
    /// Initials when there is no photo.
    #[props(default = "AK".to_string())]
    initials: String,
    /// Accepted MIME types.
    #[props(default = vec!["image/png".to_string(), "image/jpeg".to_string(), "image/webp".to_string()])]
    accept: Vec<String>,
    /// Largest file in bytes.
    #[props(default = 5_242_880)]
    max_size: u64,
    /// Avatar diameter at rest.
    #[props(default = 88.0)]
    size_px: f32,
    /// The first upload attempt fails (demos the error and Retry).
    #[props(default)]
    fail_first: bool,
    /// Open the cropper on mount with `src` (demos the crop state).
    #[props(default)]
    start_cropping: bool,
    /// Show Remove.
    #[props(default = true)]
    removable: bool,
    /// Disabled.
    #[props(default)]
    disabled: bool,
    /// Shared clock (the progress ring).
    now: ClockMs,
    /// Saved: the crop square as (centre x, centre y, side) fractions of the image.
    #[props(default)]
    on_upload: EventHandler<(f32, f32, f32)>,
    /// Removed.
    #[props(default)]
    on_remove: EventHandler<()>,
) -> Element {
    let start_src = src.clone();
    let mut photo = use_signal(move || start_src);
    let mut draft = use_signal(move || {
        if start_cropping {
            src.clone().map(|s| (s, 1.4f32))
        } else {
            None
        }
    });
    let mut crop = use_signal(move || Crop::new(if start_cropping { 1.4 } else { 1.0 }));
    let mut saved = use_signal(|| None::<Crop>);
    let mut drag = use_signal(|| None::<(f64, f64, f32, f32)>);
    let mut shown = use_signal(|| None::<(f32, f32)>);
    let mut uploading = use_signal(|| None::<(ClockMs, u32)>);
    let mut error = use_signal(|| None::<String>);
    let mut stage = use_signal(|| None::<Rc<MountedData>>);
    let words: Vec<String> = accept
        .iter()
        .map(|m| {
            m.rsplit('/')
                .next()
                .unwrap_or("")
                .to_uppercase()
                .replace("JPEG", "JPG")
        })
        .collect();
    let hint_txt = hint
        .clone()
        .unwrap_or_else(|| format!("{} up to {}", words.join(", "), size(max_size)));
    let fid = format!(
        "oxu-av-{}",
        label
            .chars()
            .filter(|c| c.is_alphanumeric())
            .collect::<String>()
            .to_lowercase()
    );
    let accept_attr = accept.join(",");
    let circle_px = if draft().is_some() { 184.0 } else { size_px };
    let c = crop();
    let cropping = draft().is_some();
    let (sx, sy) = shown().unwrap_or((c.x, c.y));
    let up = uploading().map(|(t, attempt)| {
        let p = (now.since(t).as_millis() as f32 / 1400.0).min(1.0);
        (p, attempt)
    });
    // Upload settles on the clock: a copy decides, the handlers below commit.
    let phase = match up {
        Some((p, 1)) if fail_first && p >= 0.6 => "failed",
        Some((p, _)) if p >= 1.0 => "done",
        Some(_) => "up",
        None => "idle",
    };
    let ring = up.map_or(
        0.0,
        |(p, a)| if fail_first && a == 1 { p.min(0.6) } else { p },
    );
    let circ = std::f32::consts::TAU * 46.0;
    let accept2 = use_signal(|| accept.clone());
    let mut pick = move |name: String, bytes: u64, mime: Option<String>, data: Option<Vec<u8>>| {
        let m = mime.unwrap_or_else(|| oxyn_core::kit::upload::mime(&name));
        if !accept2.peek().contains(&m) {
            error.set(Some(format!(
                "{} isn't a supported image. Use {}.",
                ext(&name),
                hint_txt_short(&accept2.peek())
            )));
            return;
        }
        if bytes > max_size {
            error.set(Some(format!(
                "That photo is {}. The limit is {}.",
                size(bytes),
                size(max_size)
            )));
            return;
        }
        let Some(d) = data else { return };
        let (w, h) = image_size(&d).unwrap_or((1, 1));
        if w == 0 || h == 0 {
            error.set(Some("That image couldn't be read.".into()));
            return;
        }
        error.set(None);
        crop.set(Crop::new(w as f32 / h as f32));
        draft.set(Some((data_url(&m, &d), w as f32 / h as f32)));
        focus(stage());
    };
    let show_src = if cropping {
        draft().map(|d| d.0)
    } else {
        photo()
    };
    let sv = saved();
    let rest_style = match (&sv, cropping) {
        (Some(s), false) => {
            let (w, h) = if s.aspect >= 1.0 {
                (s.aspect, 1.0)
            } else {
                (1.0, 1.0 / s.aspect)
            };
            format!(
                "width:{}%;height:{}%;transform:translate(calc(-50% + {}%), calc(-50% + {}%))",
                w * s.zoom * 100.0,
                h * s.zoom * 100.0,
                s.x * 100.0,
                s.y * 100.0
            )
        }
        _ => {
            let (w, h) = if c.aspect >= 1.0 {
                (c.aspect, 1.0)
            } else {
                (1.0, 1.0 / c.aspect)
            };
            format!(
                "width:{}%;height:{}%;transform:translate(calc(-50% + {}%), calc(-50% + {}%))",
                w * c.zoom * 100.0,
                h * c.zoom * 100.0,
                sx * 100.0,
                sy * 100.0
            )
        }
    };
    let mut save = move || {
        let k = crop();
        saved.set(Some(k));
        if let Some((s, _)) = draft() {
            photo.set(Some(s));
        }
        draft.set(None);
        let attempt = uploading().map_or(1, |u| u.1);
        uploading.set(Some((now, attempt)));
        on_upload.call(k.source());
    };
    rsx! {
        div { class: "oxu-av", "data-crop": "{cropping}", "data-drag": "{drag().is_some()}", "data-phase": "{phase}", role: "group", "aria-label": "{label}",
            div { class: "oxu-avrow",
                div { class: "oxu-circle", style: "--sz:{circle_px}px",
                    tabindex: if cropping { "0" } else if disabled { "-1" } else { "0" },
                    role: if cropping { "application" } else { "button" },
                    "aria-label": if cropping { "Crop area. Drag or use arrow keys to move, plus and minus to zoom, Enter to save, Escape to cancel." } else { "Change {label}" },
                    onmounted: move |e| stage.set(Some(e.data())),
                    ondragover: move |e| e.prevent_default(),
                    ondrop: move |e: DragEvent| {
                        e.prevent_default();
                        if disabled { return }
                        if let Some(f) = e.data_transfer().files().into_iter().next() {
                            let (n, b, m) = (f.name(), f.size(), f.content_type());
                            spawn(async move { let d = f.read_bytes().await.ok().map(|x| x.to_vec()); pick(n, b, m, d); });
                        }
                    },
                    onpointerdown: move |e: PointerEvent| if cropping {
                        let p = e.client_coordinates();
                        drag.set(Some((p.x, p.y, crop().x, crop().y)));
                    },
                    onpointermove: move |e: PointerEvent| if let Some((x0, y0, cx, cy)) = drag() {
                        let p = e.client_coordinates();
                        let nx = cx + ((p.x - x0) / 184.0) as f32;
                        let ny = cy + ((p.y - y0) / 184.0) as f32;
                        let k = crop();
                        shown.set(Some(k.shown(nx, ny)));
                        let mut w = crop.write();
                        w.x = nx;
                        w.y = ny;
                    },
                    onpointerup: move |_| if drag().is_some() { drag.set(None); shown.set(None); crop.write().clamp(); },
                    onpointerleave: move |_| if drag().is_some() { drag.set(None); shown.set(None); crop.write().clamp(); },
                    onwheel: move |e: WheelEvent| if cropping {
                        e.prevent_default();
                        let dy = e.delta().strip_units().y;
                        let p = e.element_coordinates();
                        let f = if dy < 0.0 { 1.08 } else { 1.0 / 1.08 };
                        crop.write().zoom_at(f, (p.x / 184.0 - 0.5) as f32, (p.y / 184.0 - 0.5) as f32);
                    },
                    onkeydown: move |e: KeyboardEvent| if cropping {
                        let sh = e.modifiers().shift();
                        match e.key() {
                            Key::ArrowLeft => { e.prevent_default(); crop.write().nudge(-1, 0, sh) }
                            Key::ArrowRight => { e.prevent_default(); crop.write().nudge(1, 0, sh) }
                            Key::ArrowUp => { e.prevent_default(); crop.write().nudge(0, -1, sh) }
                            Key::ArrowDown => { e.prevent_default(); crop.write().nudge(0, 1, sh) }
                            Key::Character(ch) if ch == "+" || ch == "=" => crop.write().zoom_at(1.15, 0.0, 0.0),
                            Key::Character(ch) if ch == "-" => crop.write().zoom_at(1.0 / 1.15, 0.0, 0.0),
                            Key::Character(ch) if ch == "0" => crop.write().reset(),
                            Key::Enter => { e.prevent_default(); save() }
                            Key::Escape => { e.prevent_default(); draft.set(None) }
                            _ => {}
                        }
                    },
                    if let Some(s) = &show_src {
                        img { class: "oxu-img", src: "{s}", alt: "", draggable: "false", style: "{rest_style}" }
                        if cropping {
                            span { class: "oxu-clip", "aria-hidden": "true",
                                img { class: "oxu-img", src: "{s}", alt: "", draggable: "false", style: "{rest_style}" }
                            }
                        }
                    } else {
                        span { class: "oxu-ini", "{initials}" }
                    }
                    if cropping { span { class: "oxu-mask", "aria-hidden": "true" } }
                    if !cropping && !disabled { label { class: "oxu-hit", r#for: "{fid}", "aria-hidden": "true" } }
                    if phase == "up" || phase == "failed" {
                        svg { class: "oxu-ring", view_box: "0 0 100 100", "aria-hidden": "true",
                            circle { cx: "50", cy: "50", r: "46", fill: "none", stroke_width: "4", class: "oxu-rtrack" }
                            circle { cx: "50", cy: "50", r: "46", fill: "none", stroke_width: "4", class: "oxu-rfill",
                                stroke_dasharray: "{circ * ring} {circ}", transform: "rotate(-90 50 50)", stroke_linecap: "round" }
                        }
                    }
                    if phase == "done" && up.is_some_and(|u| now.since(uploading().map_or(now, |x| x.0)).as_millis() < 2400 && u.0 >= 1.0) {
                        span { class: "oxu-done", Icon { name: "check", size: 14.0 } }
                    }
                }
                if !cropping {
                    div { class: "oxu-avcopy",
                        span { class: "oxu-title", "{label}" }
                        span { class: "oxu-note", "{hint_txt}" }
                        div { class: "oxu-avbtns",
                            label { class: "oxu-btn", "aria-disabled": "{disabled}",
                                if photo().is_some() { "Replace photo" } else { "Upload photo" }
                                input { id: "{fid}", r#type: "file", class: "oxu-file", accept: "{accept_attr}", disabled,
                                    onchange: move |e: FormEvent| {
                                        if let Some(f) = e.files().into_iter().next() {
                                            let (n, b, m) = (f.name(), f.size(), f.content_type());
                                            spawn(async move { let d = f.read_bytes().await.ok().map(|x| x.to_vec()); pick(n, b, m, d); });
                                        }
                                    },
                                }
                            }
                            if removable && photo().is_some() {
                                button { class: "oxu-btn quiet", r#type: "button", disabled,
                                    onclick: move |_| { photo.set(None); saved.set(None); uploading.set(None); on_remove.call(()) },
                                    "Remove"
                                }
                            }
                        }
                    }
                }
            }
            if cropping {
                div { class: "oxu-tools",
                    button { class: "oxu-zb", r#type: "button", "aria-label": "Zoom out", onclick: move |_| crop.write().zoom_at(1.0 / 1.15, 0.0, 0.0), Icon { name: "minus", size: 12.0 } }
                    input { class: "oxu-zoom", r#type: "range", min: "1", max: "4", step: "0.01", value: "{c.zoom}", "aria-label": "Zoom",
                        oninput: move |e| if let Ok(z) = e.value().parse::<f32>() { let z0 = crop().zoom; crop.write().zoom_at(z / z0, 0.0, 0.0) } }
                    button { class: "oxu-zb", r#type: "button", "aria-label": "Zoom in", onclick: move |_| crop.write().zoom_at(1.15, 0.0, 0.0), Icon { name: "plus", size: 12.0 } }
                }
                div { class: "oxu-avbtns center",
                    button { class: "oxu-btn solid", r#type: "button", onclick: move |_| save(), "Save photo" }
                    button { class: "oxu-btn quiet", r#type: "button", onclick: move |_| draft.set(None), "Cancel" }
                }
            }
            if phase == "failed" {
                div { class: "oxu-err", role: "alert",
                    span { "Upload failed · the server didn't respond." }
                    button { class: "oxu-btn", r#type: "button", onclick: move |_| { uploading.set(Some((now, 2))); }, Icon { name: "retry", size: 11.0 } "Retry" }
                }
            }
            if let Some(e) = error() {
                div { class: "oxu-err", role: "alert", span { "{e}" } }
            }
        }
    }
}

fn hint_txt_short(accept: &[String]) -> String {
    accept
        .iter()
        .map(|m| {
            m.rsplit('/')
                .next()
                .unwrap_or("")
                .to_uppercase()
                .replace("JPEG", "JPG")
        })
        .collect::<Vec<_>>()
        .join(", ")
}
