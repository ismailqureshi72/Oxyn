//! Uploads (plan #28, chunk 4b): `FileDropzone` (affectUI Dropzone, Arc UI File dropzone and the
//! Arc File upload block) and `AvatarUpload` (Arc UI Avatar upload). Behaviour-only rebuilds;
//! models in `oxyn_core::kit::upload`.
#![cfg_attr(not(oxyn_pro), allow(dead_code, unused_imports))]

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

// Oxyn Pro part of this module (private repo ismailqureshi72/oxyn-pro; overlaid by CI).
#[cfg(oxyn_pro)]
#[rustfmt::skip]
mod pro;
#[cfg(oxyn_pro)]
#[allow(unused_imports)]
pub use pro::*;
