//! Attachments family (variants 2) demos: a drop zone, chips with upload progress and errors,
//! and two Oxyn originals – a stack of image prints and a drop target that splits into lanes.

use dioxus::prelude::*;
use oxyn_core::{ClockMs, parity::attachfid::demo_files};
use oxyn_dioxus::{AttachmentChips, DropLanes, PolaroidStack, UploadDropzone};

use super::{Code, mk, rng, sel, tog, var};
use crate::{Align, Demo, Vals};

const CAT: &str = "attachments";

fn status(msg: Signal<String>) -> Element {
    rsx! { span { class: "oxd-status", style: "font-size:12px;color:var(--oxyn-color-fg-muted);min-height:16px", role: "status", "{msg}" } }
}

// ---------- Upload dropzone ----------

fn upload_dropzone(v: &Vals, _now: ClockMs) -> Element {
    let mut msg = use_signal(String::new);
    let key = format!(
        "{}{}{}{}{}",
        v.s("limit"),
        v.s("accept"),
        v.s("stroke"),
        v.s("corner"),
        v.s("speed")
    );
    let accept: Vec<String> = match v.s("accept") {
        "pdf + images" => vec!["pdf".into(), "image".into()],
        _ => Vec::new(),
    };
    let d = demo_files();
    let files = match v.s("accept") {
        "pdf + images" => vec![d[0].clone(), d[1].clone(), d[6].clone()],
        _ => vec![d[0].clone(), d[1].clone(), d[4].clone()],
    };
    rsx! {
        div { key: "{key}", "data-k": "{key}", style: "width:340px;max-width:100%;display:flex;flex-direction:column;align-items:center;gap:10px",
            UploadDropzone { files: Some(files), max_mb: v.f("limit"), accept, stroke: v.b("stroke"), corner: v.f("corner") as f32,
                speed: v.f("speed"), on_upload: move |n: String| msg.set(format!("onUpload({n})")) }
            {status(msg)}
        }
    }
}

fn upload_dropzone_code(v: &Vals) -> String {
    Code::new("UploadDropzone")
        .s("max_mb", v.s("limit"), "10")
        .b("stroke", v.b("stroke"))
        .raw("on_upload", "move |name| attach(name)")
        .done()
}

fn upload_dropzone_demo() -> Demo {
    Demo {
        controls: vec![
            rng("limit", "Limit (MB)", 1.0, 60.0, 1.0, "10"),
            sel(
                "accept",
                "Accepts",
                &["anything", "pdf + images"],
                "anything",
            ),
            rng("speed", "Speed (MB/s)", 0.5, 6.0, 0.5, "1.5"),
            rng("corner", "Corner", 8.0, 32.0, 2.0, "20"),
            tog("stroke", "Hairline", false),
        ],
        variants: vec![
            var(
                "default",
                "Drag a file under the box into it – or press it – to watch it upload; the 48 MB film is refused.",
                &[],
            ),
            var(
                "hairline",
                "With a hairline and tighter corners, and a 60 MB limit so the film goes through.",
                &[("stroke", "true"), ("corner", "12"), ("limit", "60")],
            ),
            var(
                "typed",
                "Only PDFs and images: the archive is turned away by type, not size.",
                &[("accept", "pdf + images"), ("limit", "20")],
            ),
        ],
        render: upload_dropzone,
        code: upload_dropzone_code,
        keyboard: vec![
            ("Tab", "The box, then each file"),
            ("Enter / Space on the box", "Adds the next file"),
            ("Enter on a file", "Uploads it"),
        ],
        a11y: vec![
            "The box is a button; each file names its type and size and says how to upload it.",
            "Progress, success and refusals are said in a polite status line; the refusal is red text, not colour alone.",
        ],
        responsive: vec!["340 px wide; the sample files wrap under the box."],
        height: 320,
        live: false,
        align: Align::Center,
        ..mk(
            "upload-dropzone",
            "Upload dropzone",
            CAT,
            "Drag a file into the box to upload it – a progress bar and percent, then Uploaded; one over the limit is refused in red. Real file-type icons on every file.",
            "UploadDropzone",
            "UploadDropzoneEl",
            "upload-dropzone",
            "parity/attachfid.rs",
        )
    }
}

// ---------- Attachment chips ----------

fn attachment_chips(v: &Vals, _now: ClockMs) -> Element {
    let mut msg = use_signal(String::new);
    let key = format!("{}{}{}", v.s("max"), v.s("limit"), v.s("speed"));
    rsx! {
        div { key: "{key}", "data-k": "{key}", style: "width:520px;max-width:100%;display:flex;flex-direction:column;gap:10px",
            AttachmentChips { max_visible: v.u("max"), max_mb: v.f("limit"), speed: v.f("speed"),
                on_remove: move |n: String| msg.set(format!("onRemove({n})")) }
            {status(msg)}
        }
    }
}

fn attachment_chips_code(v: &Vals) -> String {
    Code::new("AttachmentChips")
        .raw("files", "Some(picked())")
        .s("max_visible", v.s("max"), "5")
        .raw("on_remove", "move |name| drop_file(name)")
        .done()
}

fn attachment_chips_demo() -> Demo {
    Demo {
        controls: vec![
            rng("max", "Chips before +N", 2.0, 8.0, 1.0, "5"),
            rng("limit", "Limit (MB)", 5.0, 60.0, 5.0, "25"),
            rng("speed", "Speed (MB/s)", 0.5, 6.0, 0.5, "1.5"),
        ],
        variants: vec![
            var(
                "default",
                "Six files: rings fill as they upload, the slides fail with Retry, the film is too big; the last one folds into +1.",
                &[],
            ),
            var(
                "compact",
                "Three chips before +N, slow uploads.",
                &[("max", "3"), ("speed", "0.5")],
            ),
            var(
                "roomy",
                "A 60 MB limit and room for eight: nothing is refused or folded.",
                &[("max", "8"), ("limit", "60")],
            ),
        ],
        render: attachment_chips,
        code: attachment_chips_code,
        keyboard: vec![
            ("Tab", "Retry and remove on each chip, +N, Add file"),
            ("Enter on +N", "Shows every chip"),
        ],
        a11y: vec![
            "The tray is a list; each chip's label names the file, its type and size; remove and Retry buttons name the file.",
            "Upload results are said in a status line; failed and too-big chips say so in text as well as red.",
        ],
        responsive: vec!["520 px composer; chips wrap and long names ellipsize at 150 px."],
        height: 300,
        live: false,
        align: Align::Center,
        ..mk(
            "attachment-chips",
            "Attachment chips",
            CAT,
            "Files on their way into a prompt: a ring of progress round each icon, then type and size; failed uploads offer Retry, files over the limit stay red, and extras fold into +N.",
            "AttachmentChips",
            "AttachmentChipsEl",
            "attachment-chips",
            "parity/attachfid.rs",
        )
    }
}

// ---------- Polaroid stack ----------

fn polaroid_stack(v: &Vals, _now: ClockMs) -> Element {
    let mut msg = use_signal(String::new);
    let key = format!("{}{}{}", v.s("count"), v.s("spread"), v.s("open"));
    rsx! {
        div { key: "{key}", "data-k": "{key}", style: "width:340px;max-width:100%;display:flex;flex-direction:column;align-items:center;gap:10px",
            PolaroidStack { count: v.u("count"), spread: v.f("spread") as f32, default_open: v.b("open"),
                on_remove: move |n: String| msg.set(format!("onRemove({n})")) }
            {status(msg)}
        }
    }
}

fn polaroid_stack_code(v: &Vals) -> String {
    Code::new("PolaroidStack")
        .s("count", v.s("count"), "4")
        .s("spread", v.s("spread"), "36")
        .raw("on_remove", "move |name| drop_image(name)")
        .done()
}

fn polaroid_stack_demo() -> Demo {
    Demo {
        controls: vec![
            rng("count", "Images", 1.0, 6.0, 1.0, "4"),
            rng("spread", "Fan (°)", 12.0, 60.0, 4.0, "36"),
            tog("open", "Start fanned", false),
        ],
        variants: vec![
            var(
                "stacked",
                "Four prints leaning on each other; point at them to fan them out, press one to bring it forward.",
                &[],
            ),
            var(
                "fanned",
                "Six prints, fanned wide from the start.",
                &[("count", "6"), ("spread", "52"), ("open", "true")],
            ),
            var(
                "pair",
                "Two prints and a tight fan.",
                &[("count", "2"), ("spread", "20")],
            ),
        ],
        render: polaroid_stack,
        code: polaroid_stack_code,
        keyboard: vec![
            (
                "Tab",
                "Each print and its remove button (focus fans the stack), then Add image",
            ),
            ("Enter on a print", "Brings it to the front"),
        ],
        a11y: vec![
            "The stack is a group labelled with the count; each print is a button named after its file.",
            "The caption names the front print; adding and removing are said in a status line.",
        ],
        responsive: vec!["340 px stage; the fan stays inside it up to 60°."],
        height: 320,
        live: false,
        align: Align::Center,
        ..mk(
            "polaroid-stack",
            "Polaroid stack",
            CAT,
            "An Oxyn original: image attachments as a little stack of prints that fans out when you point at it; press one to bring it forward, take any out.",
            "PolaroidStack",
            "PolaroidStackEl",
            "polaroid-stack",
            "parity/attachfid.rs",
        )
    }
}

// ---------- Drop lanes ----------

fn drop_lanes(v: &Vals, _now: ClockMs) -> Element {
    let mut msg = use_signal(String::new);
    let key = v.string("set");
    let d = demo_files();
    let files = match v.s("set") {
        "media" => vec![d[1].clone(), d[4].clone(), d[9].clone()],
        "code" => vec![d[7].clone(), d[8].clone(), d[2].clone(), d[6].clone()],
        _ => vec![d[0].clone(), d[1].clone(), d[3].clone(), d[2].clone()],
    };
    rsx! {
        div { key: "{key}", "data-k": "{key}", style: "width:560px;max-width:100%;display:flex;flex-direction:column;gap:10px",
            DropLanes { files: Some(files), on_file: move |(n, l): (String, String)| msg.set(format!("onFile({n}, {l})")) }
            {status(msg)}
        }
    }
}

fn drop_lanes_code(_: &Vals) -> String {
    Code::new("DropLanes")
        .raw("files", "Some(dropped())")
        .raw("on_file", "move |(name, lane)| route(name, lane)")
        .done()
}

fn drop_lanes_demo() -> Demo {
    Demo {
        controls: vec![sel(
            "set",
            "Files",
            &["documents", "media", "code"],
            "documents",
        )],
        variants: vec![
            var(
                "documents",
                "Four documents waiting: drag one onto a lane, or press it and then a lane.",
                &[],
            ),
            var(
                "media",
                "An image, a film and a voice memo.",
                &[("set", "media")],
            ),
            var(
                "code",
                "Source, notes, a sheet and an archive.",
                &[("set", "code")],
            ),
        ],
        render: drop_lanes,
        code: drop_lanes_code,
        keyboard: vec![
            ("Enter on a file", "Picks it up (pressed)"),
            ("Enter on a lane", "Files the picked file there"),
            ("Enter on ×", "Sends a filed file back"),
        ],
        a11y: vec![
            "Files are toggle buttons (aria-pressed); lane headings are buttons, enabled once a file is picked.",
            "Every move is said in a status line, with the lane's name.",
        ],
        responsive: vec!["560 px card; lanes stack under 480 px."],
        height: 330,
        live: false,
        align: Align::Center,
        ..mk(
            "drop-lanes",
            "Drop lanes",
            CAT,
            "An Oxyn original: one drop target that splits into where a file should go – this message, the project's files or a style reference.",
            "DropLanes",
            "DropLanesEl",
            "drop-lanes",
            "parity/attachfid.rs",
        )
    }
}

/// Every Attachments (variants 2) demo, in sidebar order.
pub(crate) fn demos() -> Vec<Demo> {
    vec![
        upload_dropzone_demo(),
        attachment_chips_demo(),
        polaroid_stack_demo(),
        drop_lanes_demo(),
    ]
}
