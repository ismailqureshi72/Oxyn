//! Attachments family (variants 2): files going into a prompt – the Upload dropzone, Attachment
//! chips with ring progress and error states, and two Oxyn originals, the Polaroid stack and Drop
//! lanes. Behaviour only: file names, sizes and every line of copy are our own.

/// A file offered to an upload.
#[derive(Clone, Debug, PartialEq)]
pub struct DemoFile {
    /// File name with extension.
    pub name: &'static str,
    /// Size in bytes.
    pub bytes: u64,
    /// The demo makes this one fail mid-upload (a dropped connection).
    pub flaky: bool,
}

/// The demo files.
pub fn demo_files() -> Vec<DemoFile> {
    let f = |name, bytes, flaky| DemoFile { name, bytes, flaky };
    vec![
        f("Q3 report.pdf", 2_400_000, false),
        f("moodboard.png", 1_800_000, false),
        f("pipeline.xlsx", 640_000, false),
        f("brief.docx", 310_000, false),
        f("launch-film.mp4", 48_200_000, false),
        f("pitch.pptx", 5_200_000, true),
        f("assets.zip", 12_600_000, false),
        f("router.rs", 18_400, false),
        f("notes.md", 6_100, false),
        f("voice-memo.mp3", 3_100_000, false),
    ]
}

/// The lower-case extension of a file name (`""` when none).
pub fn ext_of(name: &str) -> String {
    name.rsplit_once('.')
        .map(|(_, e)| e.to_ascii_lowercase())
        .unwrap_or_default()
}

/// The family a file belongs to: `pdf`, `image`, `sheet`, `doc`, `slides`, `video`, `audio`,
/// `archive`, `code`, `text` or `file`.
pub fn file_family(name: &str) -> &'static str {
    match ext_of(name).as_str() {
        "pdf" => "pdf",
        "png" | "jpg" | "jpeg" | "gif" | "webp" | "heic" | "svg" | "avif" => "image",
        "xls" | "xlsx" | "csv" | "tsv" | "numbers" | "ods" => "sheet",
        "doc" | "docx" | "rtf" | "odt" | "pages" => "doc",
        "ppt" | "pptx" | "key" | "odp" => "slides",
        "mp4" | "mov" | "webm" | "mkv" => "video",
        "mp3" | "wav" | "m4a" | "flac" | "ogg" => "audio",
        "zip" | "tar" | "gz" | "7z" | "rar" => "archive",
        "rs" | "ts" | "tsx" | "js" | "py" | "go" | "swift" | "kt" | "java" | "rb" | "c" | "cpp"
        | "json" | "toml" | "yaml" | "yml" | "sh" | "sql" | "html" | "css" => "code",
        "txt" | "md" | "log" => "text",
        _ => "file",
    }
}

/// The badge on a file's icon (`PDF`, `XLS`, `MP4`, the code extension) and its colour
/// (`RRGGBB`). File-type colour is the one hue on an icon.
pub fn file_badge(name: &str) -> (String, &'static str) {
    let e = ext_of(name);
    let fam = file_family(name);
    let hex = match fam {
        "pdf" => "E5322D",
        "image" => "8B5CF6",
        "sheet" => "1D9A5B",
        "doc" => "2F6FEB",
        "slides" => "E2622B",
        "video" => "0EA5E9",
        "audio" => "DB2777",
        "archive" => "C98A0B",
        "code" => "64748B",
        "text" => "6B7280",
        _ => "8A8A93",
    };
    let badge = match fam {
        "sheet" if e == "csv" => "CSV".to_string(),
        "sheet" => "XLS".into(),
        "doc" => "DOC".into(),
        "slides" => "PPT".into(),
        "image" if e == "jpeg" => "JPG".into(),
        _ if e.is_empty() => "FILE".into(),
        _ => e.to_ascii_uppercase().chars().take(4).collect(),
    };
    (badge, hex)
}

/// A size: `840 KB`, `2.4 MB`, `48.2 MB`, `6 KB`, `1.1 GB`.
pub fn fmt_bytes(b: u64) -> String {
    let f = b as f64;
    if f >= 1e9 {
        trim1(f / 1e9, " GB")
    } else if f >= 1e6 {
        trim1(f / 1e6, " MB")
    } else if f >= 1e3 {
        format!("{} KB", (f / 1e3).round() as u64)
    } else {
        format!("{b} B")
    }
}

fn trim1(v: f64, unit: &str) -> String {
    let s = format!("{v:.1}");
    let s = s.strip_suffix(".0").unwrap_or(&s).to_string();
    format!("{s}{unit}")
}

/// Why a file was turned away.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Refusal {
    /// Over the size limit.
    TooBig,
    /// A type the drop zone does not take.
    Type,
}

/// Checks a file against a limit (MB) and the accepted families (empty = any).
pub fn check_file(f: &DemoFile, max_mb: f64, accept: &[&str]) -> Result<(), Refusal> {
    if !accept.is_empty() && !accept.contains(&file_family(f.name)) {
        return Err(Refusal::Type);
    }
    if f.bytes as f64 > max_mb * 1e6 {
        return Err(Refusal::TooBig);
    }
    Ok(())
}

/// The refusal as one line: "Too big – the limit is 10 MB" / "Not a type this box takes".
pub fn refusal_line(r: Refusal, max_mb: f64) -> String {
    match r {
        Refusal::TooBig => format!("Too big – the limit is {}", trim1(max_mb, " MB")),
        Refusal::Type => "Not a type this box takes".into(),
    }
}

/// How long the demo upload takes, in ms: size over `mbps` megabytes a second, kept between
/// 0.7 and 4.5 s so every upload is visible but none drags.
pub fn upload_ms(bytes: u64, mbps: f64) -> u32 {
    let s = bytes as f64 / 1e6 / mbps.max(0.1);
    ((s * 1000.0).round() as u32).clamp(700, 4_500)
}

/// A screen-reader line for a chip: "brief.docx, Word document, 310 KB".
pub fn file_said(f: &DemoFile) -> String {
    let kind = match file_family(f.name) {
        "pdf" => "PDF",
        "image" => "image",
        "sheet" => "spreadsheet",
        "doc" => "Word document",
        "slides" => "slides",
        "video" => "video",
        "audio" => "audio",
        "archive" => "archive",
        "code" => "code file",
        "text" => "text file",
        _ => "file",
    };
    format!("{}, {kind}, {}", f.name, fmt_bytes(f.bytes))
}

/// The lanes a drop can go to (Drop lanes): key, label, hint.
pub const LANES: [(&str, &str, &str); 3] = [
    ("message", "This message", "Read once, with this prompt"),
    (
        "project",
        "Project files",
        "Kept for every chat in the project",
    ),
    (
        "style",
        "Style reference",
        "Matched for look, not read for facts",
    ),
];

/// Which lane a pointer at `x` of `w` px falls in (three equal lanes).
pub fn lane_at(x: f64, w: f64) -> usize {
    if w <= 0.0 {
        return 0;
    }
    ((x / w * 3.0).floor().max(0.0) as usize).min(2)
}

/// The fan-out angle (degrees) and shift (px) of card `i` of `n` in the Polaroid stack: closed
/// cards lean a little each way; open cards spread across `spread` degrees.
pub fn fan(i: usize, n: usize, open: bool, spread: f32) -> (f32, f32) {
    if n <= 1 {
        return (0.0, 0.0);
    }
    let t = i as f32 / (n - 1) as f32 - 0.5;
    if open {
        (t * spread, t * spread * 4.2)
    } else {
        let lean = [-4.0, 3.0, -1.5, 5.0, -3.0, 2.0];
        (lean[i % lean.len()], 0.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn types_and_sizes() {
        assert_eq!(file_family("Q3 report.pdf"), "pdf");
        assert_eq!(file_badge("pipeline.xlsx"), ("XLS".into(), "1D9A5B"));
        assert_eq!(file_badge("data.csv").0, "CSV");
        assert_eq!(file_badge("router.rs").0, "RS");
        assert_eq!(file_badge("README").0, "FILE");
        assert_eq!(fmt_bytes(2_400_000), "2.4 MB");
        assert_eq!(fmt_bytes(640_000), "640 KB");
        assert_eq!(fmt_bytes(48_200_000), "48.2 MB");
        assert_eq!(fmt_bytes(1_000_000), "1 MB");
        assert_eq!(fmt_bytes(512), "512 B");
    }

    #[test]
    fn checks() {
        let d = demo_files();
        assert_eq!(check_file(&d[0], 10.0, &[]), Ok(()));
        assert_eq!(check_file(&d[4], 10.0, &[]), Err(Refusal::TooBig));
        assert_eq!(
            check_file(&d[6], 50.0, &["pdf", "image"]),
            Err(Refusal::Type)
        );
        assert_eq!(
            refusal_line(Refusal::TooBig, 10.0),
            "Too big – the limit is 10 MB"
        );
        assert_eq!(upload_ms(2_400_000, 1.5), 1_600);
        assert_eq!(upload_ms(6_100, 1.5), 700);
        assert_eq!(upload_ms(48_200_000, 1.5), 4_500);
        assert_eq!(file_said(&d[3]), "brief.docx, Word document, 310 KB");
    }

    #[test]
    fn lanes_and_fan() {
        assert_eq!(lane_at(10.0, 300.0), 0);
        assert_eq!(lane_at(150.0, 300.0), 1);
        assert_eq!(lane_at(299.0, 300.0), 2);
        assert_eq!(lane_at(400.0, 300.0), 2);
        let (a, x) = fan(0, 3, true, 30.0);
        assert_eq!(a, -15.0);
        assert!((x + 63.0).abs() < 1e-3);
        assert_eq!(fan(1, 3, true, 30.0), (0.0, 0.0));
        assert_eq!(fan(0, 3, false, 30.0), (-4.0, 0.0));
    }
}
