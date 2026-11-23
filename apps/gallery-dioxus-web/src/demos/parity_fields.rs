//! Plan #28 parity demos, chunk 4b: number entry, choice, meters, uploads, composers and model
//! menus.
#![cfg_attr(not(oxyn_pro), allow(dead_code, unused_imports, unused_variables))]

use dioxus::prelude::*;
use oxyn_core::{
    ClockMs,
    kit::{
        compose::Effort,
        fields::Choice,
        meters::{Band, Cat, GridItem, Part, PartItem, Share, Tone},
    },
};
use oxyn_dioxus::FileDropzone;
#[cfg(oxyn_pro)]
use oxyn_dioxus::{
    AiComposer, AvatarUpload, ContextField, ContextWindowGrid, CtxModel, Gauge, ModelPill,
    NumberField, PillModel, RichTextEditor, RubberSlider, ScrubInput, SplitBar, UnfoldSelect,
    UsageMeter,
};

use super::{Code, mk, rng, sel, tog, txt, var};
use crate::{Demo, Vals};

fn status(msg: Signal<String>) -> Element {
    rsx! { span { style: "font-size:12px;color:var(--oxyn-color-fg-muted);min-height:16px", role: "status", "{msg}" } }
}

// ---------- file dropzone ----------

fn dropzone(v: &Vals, now: ClockMs) -> Element {
    let mut msg = use_signal(String::new);
    let preset = if v.b("preset") {
        vec![
            ("brand-guidelines.pdf".to_string(), 3_400_000),
            ("logo.svg".to_string(), 24_000),
        ]
    } else {
        vec![]
    };
    rsx! {
        div { style: "width:440px;max-width:100%;display:flex;flex-direction:column;gap:6px",
            FileDropzone { variant: v.string("variant"), accept: v.string("accept"), max_size: Some(25 * 1024 * 1024),
                max_files: Some(v.u("max_files")), list_placement: v.string("placement"),
                compact_at: if v.b("compact") { Some(2) } else { None }, preset, autoplay: v.b("autoplay"), multiple: v.b("multiple"),
                disabled: v.b("disabled"), now,
                on_change: move |n: Vec<String>| msg.set(format!("{} file(s)", n.len())) }
            {status(msg)}
        }
    }
}

fn dropzone_code(v: &Vals) -> String {
    Code::new("FileDropzone")
        .s("variant", &v.string("variant"), "plain")
        .raw("accept", super::q(&v.string("accept")))
        .raw("max_size", "Some(25 * 1024 * 1024)")
        .raw("max_files", format!("Some({})", v.u("max_files")))
        .s("list_placement", &v.string("placement"), "below")
        .b("autoplay", v.b("autoplay"))
        .raw("now", "clock.now()")
        .done()
}

pub fn demos() -> Vec<Demo> {
    #[allow(unused_mut)]
    let mut v = vec![Demo {
        controls: vec![
            sel("variant", "Variant", &["plain", "fan"], "plain"),
            txt("accept", "Accept", ".pdf,.csv,.docx,image/*"),
            rng("max_files", "Max files", 1.0, 8.0, 1.0, "5"),
            sel("placement", "List", &["below", "inside"], "below"),
            tog("compact", "Fold at 2 files", false),
            tog("preset", "Earlier uploads", false),
            tog("autoplay", "Sample drop", true),
            tog("multiple", "Multiple", true),
            tog("disabled", "Disabled", false),
        ],
        variants: vec![
            var(
                "autoplay",
                "A sample drop of four files: progress, time left, one failure with Retry.",
                &[],
            ),
            var(
                "fan",
                "Documents fan out, type badges.",
                &[("variant", "fan")],
            ),
            var(
                "folding",
                "Rows inside the edge; the prompt folds at two files.",
                &[
                    ("placement", "inside"),
                    ("compact", "true"),
                    ("autoplay", "false"),
                    ("preset", "true"),
                ],
            ),
            var(
                "single",
                "One file: a new one replaces the old.",
                &[
                    ("multiple", "false"),
                    ("autoplay", "false"),
                    ("max_files", "1"),
                ],
            ),
        ],
        render: dropzone,
        code: dropzone_code,
        keyboard: vec![
            ("Tab", "The file picker, then each row's Retry and Remove"),
            ("Enter / Space", "Open the picker"),
            ("↑ / ↓", "Same action on the next row"),
            ("Delete / Backspace", "Remove the focused row"),
        ],
        a11y: vec![
            "The picker input carries the label, description and limits.",
            "Failed rows announce their reason (role=alert); progress bars are role=progressbar.",
        ],
        responsive: vec!["Fills its container; names truncate."],
        height: 420,
        live: true,
        ..mk(
            "file-dropzone",
            "File dropzone",
            "forms",
            "A generous target for files: rows upload with progress and time left, fail with a reason and Retry, collapse when removed; rejected types and sizes keep their reason; the prompt can fold to a slim bar.",
            "FileDropzone",
            "FileDropzoneEl",
            "file-dropzone",
            "kit/upload.rs",
        )
    }];
    #[cfg(oxyn_pro)]
    v.extend(pro::demos());
    super::in_order(&mut v, &ORDER);
    v
}

/// Page order of this file's demos (free and Pro).
const ORDER: [&str; 14] = [
    "number-field",
    "scrub-input",
    "rubber-slider",
    "unfold-select",
    "gauge",
    "usage-meter",
    "split-bar",
    "context-field",
    "context-grid",
    "file-dropzone",
    "avatar-upload",
    "rich-text-editor",
    "ai-composer",
    "model-pill",
];

// Oxyn Pro demos of this file (private repo ismailqureshi72/oxyn-pro; overlaid by CI).
#[cfg(oxyn_pro)]
#[rustfmt::skip]
mod pro;
