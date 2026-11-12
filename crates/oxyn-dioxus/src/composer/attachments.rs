//! `AttachmentRow` and `DropZone` (`docs/specs/composer-attachments.md`).

use std::{cell::RefCell, collections::HashMap, rc::Rc};

use dioxus::prelude::*;
use oxyn_core::{
    ClockMs,
    composer::{
        AttachmentItem, AttachmentKind, AttachmentState, AttachmentVariant, FileType, UploadRing,
        attachments::{RingFrame, block_bar, overflow},
    },
};

use super::icons::{Icon, file_icon};

/// The attachments above the text, in one of the free layouts. Controlled: the host owns the
/// items and their upload state; the row draws them and reports removals.
#[component]
pub fn AttachmentRow(
    /// Items in the draft.
    items: Vec<AttachmentItem>,
    /// Shared clock time.
    now: ClockMs,
    /// `tile-ring` · `chip` · `chip-bar` · `thumb-strip` · `context-chip` · `paste-chip` ·
    /// `list` · `terminal` (the two drop variants are [`DropZone`]).
    #[props(default = "tile-ring".to_string())]
    variant: String,
    /// Slots before "+k more".
    #[props(default = 6)]
    max_visible: usize,
    /// Reduced motion: static % label instead of the ring.
    #[props(default)]
    reduced: bool,
    /// × pressed.
    #[props(default)]
    on_remove: EventHandler<u64>,
    /// ⟲ pressed.
    #[props(default)]
    on_retry: EventHandler<u64>,
) -> Element {
    let variant = AttachmentVariant::from_id(&variant).unwrap_or_default();
    let rings = use_hook(|| Rc::new(RefCell::new(HashMap::<u64, UploadRing>::new())));
    let mut expanded = use_signal(|| false);
    if items.is_empty() {
        return rsx! {};
    }
    let (shown, more) = overflow(items.len(), max_visible, expanded());
    let frames: Vec<RingFrame> = {
        let mut r = rings.borrow_mut();
        r.retain(|id, _| items.iter().any(|i| i.id == *id));
        items
            .iter()
            .map(|i| r.entry(i.id).or_default().update(&i.state, now))
            .collect()
    };
    let class = format!("oxyn-att {}", variant.id());
    rsx! {
        div { class: "{class}", role: "list", "aria-label": "Attachments",
            for (item, frame) in items.iter().zip(frames).take(shown) {
                match variant {
                    AttachmentVariant::TileRing | AttachmentVariant::ThumbStrip | AttachmentVariant::DropOverlay | AttachmentVariant::DropInline => rsx! {
                        Tile { key: "{item.id}", item: item.clone(), frame, big: variant == AttachmentVariant::ThumbStrip, reduced, on_remove, on_retry }
                    },
                    AttachmentVariant::List => rsx! { ListRow { key: "{item.id}", item: item.clone(), on_remove } },
                    AttachmentVariant::Terminal => rsx! { TermRow { key: "{item.id}", item: item.clone() } },
                    _ => rsx! {
                        Chip { key: "{item.id}", item: item.clone(), bar: variant == AttachmentVariant::ChipBar, on_remove, on_retry }
                    },
                }
            }
            if more > 0 {
                button { class: "oxyn-att-more", onclick: move |_| expanded.set(true), "+{more} more" }
            }
        }
    }
}

fn progress(item: &AttachmentItem) -> Option<f32> {
    match item.state {
        AttachmentState::Uploading { progress } => progress,
        AttachmentState::Paused { progress } => Some(progress),
        _ => None,
    }
}

fn badge(t: FileType) -> Option<&'static str> {
    match t {
        FileType::Code(b) => Some(b),
        FileType::Pdf => Some("PDF"),
        FileType::Spreadsheet => Some("CSV"),
        _ => None,
    }
}

#[component]
fn Tile(
    item: AttachmentItem,
    frame: RingFrame,
    big: bool,
    reduced: bool,
    on_remove: EventHandler<u64>,
    on_retry: EventHandler<u64>,
) -> Element {
    let id = item.id;
    let t = item.file_type();
    let state = state_class(&item);
    let size = if big { 72 } else { 56 };
    let thumb = match &item.kind {
        AttachmentKind::Image {
            thumb: Some(src), ..
        } => Some(src.clone()),
        _ => None,
    };
    let pct = progress(&item).map(|p| format!("{:.0}", p * 100.0));
    let ring = match frame {
        RingFrame::Arc { fraction, alpha } if !reduced => Some((fraction * 100.0, alpha, false)),
        RingFrame::Indeterminate { .. } if !reduced => Some((25.0, 1.0, true)),
        _ => None,
    };
    let spin = match frame {
        RingFrame::Indeterminate { angle } => angle * 360.0,
        _ => 0.0,
    };
    let title = format!("{} · {}", item.name, item.meta());
    let ext = badge(t)
        .map(str::to_string)
        .unwrap_or_else(|| t.word(&item.name));
    rsx! {
        div { class: "oxyn-tile {state}", role: "listitem", title: "{title}", tabindex: "0",
            style: "width:{size}px;height:{size}px",
            if let Some(src) = thumb {
                img { class: "thumb", src: "{src}", alt: "{item.name}" }
            } else {
                div { class: "glyph",
                    Icon { name: file_icon(t), size: if big { 24.0 } else { 20.0 } }
                    span { class: "ext", "{ext}" }
                }
            }
            if let Some((p, a, ind)) = ring {
                svg { class: "ring", view_box: "0 0 {size} {size}", style: "opacity:{a};transform:rotate({spin}deg)",
                    rect { x: "1", y: "1", width: "{size - 2}", height: "{size - 2}", rx: "10", "pathLength": "100",
                        class: if ind { "arc ind" } else { "arc" }, "stroke-dasharray": "{p} 100" }
                }
            }
            match &item.state {
                AttachmentState::Uploading { .. } | AttachmentState::Paused { .. } if pct.is_some() => rsx! {
                    span { class: "pct", "{pct.clone().unwrap_or_default()}%" }
                },
                AttachmentState::Reading { label } => rsx! { span { class: "reading", "{label}" } },
                AttachmentState::Error { retryable: true, .. } => rsx! {
                    button { class: "act retry", "aria-label": "Retry {item.name}", onclick: move |_| on_retry.call(id), Icon { name: "retry", size: 12.0, stroke: 2.0 } }
                },
                _ => rsx! {},
            }
            button { class: "act x", "aria-label": "Remove {item.name}", onclick: move |_| on_remove.call(id),
                Icon { name: "x", size: 10.0, stroke: 2.0 }
            }
        }
    }
}

fn state_class(item: &AttachmentItem) -> &'static str {
    match item.state {
        AttachmentState::Validating | AttachmentState::Queued => "queued",
        AttachmentState::Uploading { .. } => "uploading",
        AttachmentState::Paused { .. } => "paused",
        AttachmentState::Reading { .. } => "reading",
        AttachmentState::Ready { .. } | AttachmentState::Sent => "ready",
        AttachmentState::Error { .. } | AttachmentState::Rejected(_) => "error",
        AttachmentState::Removing => "removing",
    }
}

#[component]
fn Chip(
    item: AttachmentItem,
    bar: bool,
    on_remove: EventHandler<u64>,
    on_retry: EventHandler<u64>,
) -> Element {
    let id = item.id;
    let state = state_class(&item);
    let t = item.file_type();
    let (name, tail) = match &item.kind {
        AttachmentKind::Context(r) => r.label(),
        _ => (item.name.clone(), item.meta()),
    };
    let p = progress(&item);
    let retry = matches!(
        item.state,
        AttachmentState::Error {
            retryable: true,
            ..
        }
    );
    rsx! {
        div { class: "oxyn-chip {state}", role: "listitem", tabindex: "0", title: "{item.name}",
            span { class: "ic", Icon { name: file_icon(t), size: 14.0 } }
            span { class: "nm", "{name}" }
            if !tail.is_empty() { span { class: "tail", "{tail}" } }
            if retry {
                button { class: "act", "aria-label": "Retry", onclick: move |_| on_retry.call(id), Icon { name: "retry", size: 12.0 } }
            }
            button { class: "act", "aria-label": "Remove {item.name}", onclick: move |_| on_remove.call(id), Icon { name: "x", size: 11.0, stroke: 2.0 } }
            if bar {
                if let Some(p) = p {
                    span { class: "bar", style: "width:{p * 100.0:.1}%" }
                }
            }
        }
    }
}

#[component]
fn ListRow(item: AttachmentItem, on_remove: EventHandler<u64>) -> Element {
    let id = item.id;
    let state = state_class(&item);
    let size = match item.kind {
        AttachmentKind::File { bytes } | AttachmentKind::Image { bytes, .. } => {
            oxyn_core::composer::attachments::format_bytes(bytes)
        }
        _ => String::new(),
    };
    rsx! {
        div { class: "oxyn-lrow {state}", role: "listitem",
            Icon { name: file_icon(item.file_type()), size: 14.0 }
            span { class: "nm", "{item.name}" }
            span { class: "sz", "{size}" }
            span { class: "st", "{item.meta()}" }
            button { class: "act", "aria-label": "Remove {item.name}", onclick: move |_| on_remove.call(id), Icon { name: "x", size: 11.0 } }
        }
    }
}

#[component]
fn TermRow(item: AttachmentItem) -> Element {
    let kind = match item.file_type() {
        FileType::Image => "image",
        FileType::Folder => "dir",
        FileType::Url => "url",
        _ => "file",
    };
    let state = state_class(&item);
    let tail = match progress(&item) {
        Some(p) => format!("{:>3.0} % {}", p * 100.0, block_bar(p, 8)),
        None => item.meta(),
    };
    rsx! {
        div { class: "oxyn-trow {state}", role: "listitem",
            span { class: "k", "[{kind}]" }
            " {item.name} "
            span { class: "t", "{tail}" }
        }
    }
}

/// The drop target: `inline` (composer outline) or `overlay` (full window).
#[component]
pub fn DropZone(
    /// `armed` · `valid` · `invalid`.
    state: String,
    /// `Drop to attach · 3 files`, `2 of 3 files can't be attached`.
    label: String,
    /// `inline` or `overlay`.
    #[props(default = "inline".to_string())]
    variant: String,
) -> Element {
    rsx! {
        div { class: "oxyn-drop {variant} {state}", "aria-live": "polite",
            div { class: "target",
                Icon { name: "clip", size: if variant == "overlay" { 28.0 } else { 18.0 } }
                span { "{label}" }
            }
        }
    }
}
