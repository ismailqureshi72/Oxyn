//! Attachments model (`docs/specs/composer-attachments.md`): item states, validation, file-type
//! icons and meta, the upload ring (smoothed, never backwards, skipped for fast uploads),
//! overflow, the terminal block bar and send gating.

use std::time::Duration;

use crate::ClockMs;

/// Attachment row variants (`composer-attachments.md` §6, free set).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum AttachmentVariant {
    /// 56 px tiles with a clockwise upload ring; % → × in the corner.
    #[default]
    TileRing,
    /// One-line chips (icon · name · meta).
    Chip,
    /// Chips with a progress bar along the bottom edge.
    ChipBar,
    /// 72 px image thumbnails in a scrolling strip.
    ThumbStrip,
    /// File / folder / URL / repo / symbol / thread / selection references.
    ContextChip,
    /// Pasted-text chips with preview and Expand.
    PasteChip,
    /// Vertical rows for dense IDE panels.
    List,
    /// Full-window drop overlay.
    DropOverlay,
    /// Drop target confined to the composer outline.
    DropInline,
    /// `[file] report.pdf 64 % ▓▓▓░░` rows.
    Terminal,
}

impl AttachmentVariant {
    /// Every free variant.
    pub const ALL: [AttachmentVariant; 10] = [
        Self::TileRing,
        Self::Chip,
        Self::ChipBar,
        Self::ThumbStrip,
        Self::ContextChip,
        Self::PasteChip,
        Self::List,
        Self::DropOverlay,
        Self::DropInline,
        Self::Terminal,
    ];

    /// Short id; registry id is `attachment/<id>`.
    pub fn id(self) -> &'static str {
        match self {
            Self::TileRing => "tile-ring",
            Self::Chip => "chip",
            Self::ChipBar => "chip-bar",
            Self::ThumbStrip => "thumb-strip",
            Self::ContextChip => "context-chip",
            Self::PasteChip => "paste-chip",
            Self::List => "list",
            Self::DropOverlay => "drop-overlay",
            Self::DropInline => "drop-inline",
            Self::Terminal => "terminal",
        }
    }

    /// Parses `chip` or `attachment/chip`.
    pub fn from_id(id: &str) -> Option<Self> {
        let id = id.strip_prefix("attachment/").unwrap_or(id);
        Self::ALL.into_iter().find(|v| v.id() == id)
    }
}

/// What an attachment is.
#[derive(Clone, Debug, PartialEq)]
pub enum AttachmentKind {
    /// A file with its size.
    File {
        /// Size in bytes.
        bytes: u64,
    },
    /// An image; `thumb` is a URL or path the renderer can load.
    Image {
        /// Size in bytes.
        bytes: u64,
        /// Thumbnail source.
        thumb: Option<String>,
    },
    /// A pasted-text chip.
    PastedText {
        /// Line count.
        lines: usize,
    },
    /// A reference; no upload.
    Context(ContextRef),
}

/// A context reference (`attachment/context-chip`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ContextRef {
    /// A file path.
    File(String),
    /// A folder path.
    Folder(String),
    /// A URL.
    Url(String),
    /// A repo on a branch.
    Repo {
        /// Branch name.
        branch: String,
        /// Changed files.
        changes: u32,
    },
    /// A code symbol.
    Symbol(String),
    /// Another thread.
    Thread(String),
    /// Lines of a file.
    Selection {
        /// File path.
        path: String,
        /// First line (1-based).
        from: u32,
        /// Last line.
        to: u32,
    },
}

impl ContextRef {
    /// Short name and dim tail (`auth.rs`, `src/middleware`).
    pub fn label(&self) -> (String, String) {
        fn split(path: &str) -> (String, String) {
            match path.rsplit_once('/') {
                Some((dir, name)) => (name.to_string(), dir.to_string()),
                None => (path.to_string(), String::new()),
            }
        }
        match self {
            Self::File(p) | Self::Folder(p) => split(p),
            Self::Url(u) => {
                let host = u
                    .split("://")
                    .nth(1)
                    .unwrap_or(u)
                    .split('/')
                    .next()
                    .unwrap_or(u);
                (host.trim_start_matches("www.").to_string(), String::new())
            }
            Self::Repo { branch, changes } => (
                branch.clone(),
                match changes {
                    0 => "clean".into(),
                    1 => "1 change".into(),
                    n => format!("{n} changes"),
                },
            ),
            Self::Symbol(s) | Self::Thread(s) => (s.clone(), String::new()),
            Self::Selection { path, from, to } => {
                let (name, _) = split(path);
                (name, format!("lines {from}–{to}"))
            }
        }
    }

    /// Icon for the reference.
    pub fn file_type(&self) -> FileType {
        match self {
            Self::File(p) => FileType::from_name(p),
            Self::Folder(_) => FileType::Folder,
            Self::Url(_) => FileType::Url,
            Self::Repo { .. } => FileType::Repo,
            Self::Symbol(_) => FileType::Symbol,
            Self::Thread(_) => FileType::Thread,
            Self::Selection { .. } => FileType::Selection,
        }
    }
}

/// Our own file-type icon set (outlined, 1.5 px stroke). Type is always also written in text.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum FileType {
    /// Raster or vector image.
    Image,
    /// PDF.
    Pdf,
    /// Word-processor document.
    Document,
    /// Spreadsheet or CSV.
    Spreadsheet,
    /// Slides.
    Slides,
    /// Source code; the badge is the language (`RS`, `TS`).
    Code(&'static str),
    /// Archive.
    Archive,
    /// Audio.
    Audio,
    /// Video.
    Video,
    /// Plain text or log.
    Text,
    /// Folder.
    Folder,
    /// Repository.
    Repo,
    /// URL.
    Url,
    /// Code symbol.
    Symbol,
    /// Thread.
    Thread,
    /// Selection.
    Selection,
    /// Unknown.
    Unknown,
}

impl FileType {
    /// From a file name's extension.
    pub fn from_name(name: &str) -> Self {
        let ext = name
            .rsplit_once('.')
            .map(|(_, e)| e.to_ascii_lowercase())
            .unwrap_or_default();
        match ext.as_str() {
            "png" | "jpg" | "jpeg" | "gif" | "webp" | "svg" | "avif" | "heic" => Self::Image,
            "pdf" => Self::Pdf,
            "doc" | "docx" | "odt" | "rtf" | "pages" => Self::Document,
            "csv" | "tsv" | "xls" | "xlsx" | "ods" | "numbers" | "parquet" => Self::Spreadsheet,
            "ppt" | "pptx" | "key" | "odp" => Self::Slides,
            "zip" | "tar" | "gz" | "tgz" | "7z" | "rar" => Self::Archive,
            "mp3" | "wav" | "m4a" | "flac" | "ogg" => Self::Audio,
            "mp4" | "mov" | "webm" | "mkv" => Self::Video,
            "txt" | "log" | "md" => Self::Text,
            "rs" => Self::Code("RS"),
            "ts" | "tsx" => Self::Code("TS"),
            "js" | "jsx" | "mjs" => Self::Code("JS"),
            "py" => Self::Code("PY"),
            "go" => Self::Code("GO"),
            "swift" => Self::Code("SW"),
            "kt" => Self::Code("KT"),
            "java" => Self::Code("JV"),
            "c" | "h" => Self::Code("C"),
            "cpp" | "cc" | "hpp" => Self::Code("C++"),
            "rb" => Self::Code("RB"),
            "toml" | "yaml" | "yml" | "json" => Self::Code("CFG"),
            "sql" => Self::Code("SQL"),
            "sh" | "zsh" | "bash" => Self::Code("SH"),
            "html" | "css" => Self::Code("WEB"),
            _ => Self::Unknown,
        }
    }

    /// The word written in the meta text (`PDF`, `CSV`, `Rust`).
    pub fn word(self, name: &str) -> String {
        let ext = name
            .rsplit_once('.')
            .map(|(_, e)| e.to_ascii_uppercase())
            .unwrap_or_default();
        match self {
            Self::Image | Self::Spreadsheet | Self::Archive | Self::Audio | Self::Video
                if !ext.is_empty() =>
            {
                ext
            }
            Self::Pdf => "PDF".into(),
            Self::Document => "Document".into(),
            Self::Slides => "Slides".into(),
            Self::Code(_) | Self::Text if !ext.is_empty() => ext,
            Self::Folder => "Folder".into(),
            Self::Repo => "Repo".into(),
            Self::Url => "Link".into(),
            Self::Symbol => "Symbol".into(),
            Self::Thread => "Thread".into(),
            Self::Selection => "Selection".into(),
            _ => "File".into(),
        }
    }

    /// A one-glyph fallback for text renderers and the terminal variant.
    pub fn glyph(self) -> &'static str {
        match self {
            Self::Image => "▣",
            Self::Pdf | Self::Document | Self::Text => "▤",
            Self::Spreadsheet => "▦",
            Self::Slides => "▭",
            Self::Code(_) => "‹›",
            Self::Archive => "▥",
            Self::Audio => "♪",
            Self::Video => "▶",
            Self::Folder => "▱",
            Self::Repo => "⑂",
            Self::Url => "↗",
            Self::Symbol => "ƒ",
            Self::Thread => "❝",
            Self::Selection => "⌶",
            Self::Unknown => "◇",
        }
    }
}

/// `2.4 MB`, `812 KB`, `64 B` (decimal units, one decimal under 10).
pub fn format_bytes(b: u64) -> String {
    const U: [&str; 4] = ["KB", "MB", "GB", "TB"];
    if b < 1_000 {
        return format!("{b} B");
    }
    let mut v = b as f64 / 1_000.0;
    let mut i = 0;
    while v >= 1_000.0 && i < U.len() - 1 {
        v /= 1_000.0;
        i += 1;
    }
    if v < 10.0 {
        format!("{v:.1} {}", U[i])
    } else {
        format!("{} {}", v.round() as u64, U[i])
    }
}

/// An attachment's lifecycle (`composer-attachments.md` §2).
#[derive(Clone, Debug, PartialEq)]
pub enum AttachmentState {
    /// Checking type and size (≤ 1 frame normally).
    Validating,
    /// Waiting for an upload slot.
    Queued,
    /// Uploading; `None` when the size is unknown (indeterminate arc).
    Uploading {
        /// Fraction `0..=1`.
        progress: Option<f32>,
    },
    /// Offline; frozen at `progress`.
    Paused {
        /// Fraction `0..=1`.
        progress: f32,
    },
    /// Parsing on the agent side.
    Reading {
        /// `Reading…`, `Analyzing image`, `Indexing 214 files`.
        label: String,
    },
    /// Ready; `meta` like `PDF · 12 pages`. Empty meta means the default (`type · size`).
    Ready {
        /// Meta text.
        meta: String,
    },
    /// Failed.
    Error {
        /// `Upload failed`, `Couldn't read file`.
        reason: String,
        /// Retry is offered.
        retryable: bool,
    },
    /// Rejected before upload.
    Rejected(Reject),
    /// Shrinking out; Undo is offered for 6 s.
    Removing,
    /// Locked into a sent message.
    Sent,
}

/// Why an item was refused before upload.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Reject {
    /// Over the size limit.
    TooLarge {
        /// The item's size.
        bytes: u64,
        /// The limit.
        limit: u64,
    },
    /// Type not accepted.
    Unsupported {
        /// The extension, with a dot.
        ext: String,
    },
    /// Already attached (the existing tile pulses; no new tile).
    Duplicate,
    /// Too many files.
    TooMany {
        /// The limit.
        max: usize,
    },
}

impl Reject {
    /// Reason text (`32 MB · max 20 MB`, `Can't read .exe`).
    pub fn label(&self) -> String {
        match self {
            Self::TooLarge { bytes, limit } => {
                format!("{} · max {}", format_bytes(*bytes), format_bytes(*limit))
            }
            Self::Unsupported { ext } => format!("Can't read {ext}"),
            Self::Duplicate => "Already attached".into(),
            Self::TooMany { max } => format!("Up to {max} files"),
        }
    }
}

/// One attachment in the draft.
#[derive(Clone, Debug, PartialEq)]
pub struct AttachmentItem {
    /// Stable id.
    pub id: u64,
    /// File name or reference label.
    pub name: String,
    /// What it is.
    pub kind: AttachmentKind,
    /// Where it is in its lifecycle.
    pub state: AttachmentState,
}

impl AttachmentItem {
    /// A file at `state`.
    pub fn file(id: u64, name: impl Into<String>, bytes: u64, state: AttachmentState) -> Self {
        let name = name.into();
        let kind = if FileType::from_name(&name) == FileType::Image {
            AttachmentKind::Image { bytes, thumb: None }
        } else {
            AttachmentKind::File { bytes }
        };
        Self {
            id,
            name,
            kind,
            state,
        }
    }

    /// A ready context reference.
    pub fn context(id: u64, r: ContextRef) -> Self {
        Self {
            id,
            name: r.label().0,
            kind: AttachmentKind::Context(r),
            state: AttachmentState::Ready {
                meta: String::new(),
            },
        }
    }

    /// Icon.
    pub fn file_type(&self) -> FileType {
        match &self.kind {
            AttachmentKind::Context(r) => r.file_type(),
            AttachmentKind::PastedText { .. } => FileType::Text,
            AttachmentKind::Image { .. } => FileType::Image,
            AttachmentKind::File { .. } => FileType::from_name(&self.name),
        }
    }

    /// Meta line for the item's state (`CSV · 2.4 MB`, `Waiting…`, `64 %`).
    pub fn meta(&self) -> String {
        match &self.state {
            AttachmentState::Validating | AttachmentState::Queued => "Waiting…".into(),
            AttachmentState::Uploading { progress: Some(p) } => format!("{:.0} %", p * 100.0),
            AttachmentState::Uploading { progress: None } => "Uploading…".into(),
            AttachmentState::Paused { .. } => "Paused · offline".into(),
            AttachmentState::Reading { label } => label.clone(),
            AttachmentState::Error { reason, .. } => reason.clone(),
            AttachmentState::Rejected(r) => r.label(),
            AttachmentState::Removing => "Removed".into(),
            AttachmentState::Ready { .. } | AttachmentState::Sent => {
                let meta = match &self.state {
                    AttachmentState::Ready { meta } => meta.as_str(),
                    _ => "",
                };
                if !meta.is_empty() {
                    return meta.to_string();
                }
                match &self.kind {
                    AttachmentKind::File { bytes } | AttachmentKind::Image { bytes, .. } => {
                        format!(
                            "{} · {}",
                            self.file_type().word(&self.name),
                            format_bytes(*bytes)
                        )
                    }
                    AttachmentKind::PastedText { lines } => format!("{lines} lines"),
                    AttachmentKind::Context(r) => r.label().1,
                }
            }
        }
    }

    /// The tile shows a danger treatment.
    pub fn is_error(&self) -> bool {
        matches!(
            self.state,
            AttachmentState::Error { .. } | AttachmentState::Rejected(_)
        )
    }

    /// Still on its way (blocks send under [`PendingPolicy::Block`]).
    pub fn is_pending(&self) -> bool {
        matches!(
            self.state,
            AttachmentState::Validating
                | AttachmentState::Queued
                | AttachmentState::Uploading { .. }
                | AttachmentState::Paused { .. }
                | AttachmentState::Reading { .. }
        )
    }
}

/// What the composer accepts.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AcceptRule {
    /// Accepted extensions, lowercase without the dot. Empty accepts everything.
    pub extensions: Vec<String>,
    /// Size limit per file.
    pub max_bytes: u64,
    /// Files per message.
    pub max_files: usize,
}

impl Default for AcceptRule {
    fn default() -> Self {
        Self {
            extensions: Vec::new(),
            max_bytes: 20_000_000,
            max_files: 10,
        }
    }
}

impl AcceptRule {
    /// Validates a new file against the rule and the items already attached.
    pub fn validate(
        &self,
        name: &str,
        bytes: u64,
        existing: &[AttachmentItem],
    ) -> Result<(), Reject> {
        let ext = name
            .rsplit_once('.')
            .map(|(_, e)| e.to_ascii_lowercase())
            .unwrap_or_default();
        if !self.extensions.is_empty() && !self.extensions.contains(&ext) {
            return Err(Reject::Unsupported {
                ext: format!(".{ext}"),
            });
        }
        if bytes > self.max_bytes {
            return Err(Reject::TooLarge {
                bytes,
                limit: self.max_bytes,
            });
        }
        let live: Vec<_> = existing
            .iter()
            .filter(|i| {
                !matches!(
                    i.state,
                    AttachmentState::Removing | AttachmentState::Rejected(_)
                )
            })
            .collect();
        let same = |i: &&&AttachmentItem| {
            i.name == name
                && matches!(i.kind, AttachmentKind::File { bytes: b } | AttachmentKind::Image { bytes: b, .. } if b == bytes)
        };
        if live.iter().any(|i| same(&i)) {
            return Err(Reject::Duplicate);
        }
        if live.len() >= self.max_files {
            return Err(Reject::TooMany {
                max: self.max_files,
            });
        }
        Ok(())
    }
}

/// What happens when the user sends while items are still pending.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum PendingPolicy {
    /// Send waits ("Waiting for 2 uploads").
    #[default]
    Block,
    /// Pending items are dropped with a confirmation line.
    SendWithout,
    /// The message is accepted and waits in the queue.
    QueueUntilReady,
}

/// Whether the attachments let the draft go.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SendGate {
    /// Nothing pending.
    Ready {
        /// Failed or rejected items that won't be sent (`1 file won't be sent`).
        excluded: usize,
    },
    /// Waiting for `n` uploads.
    Waiting(usize),
}

impl SendGate {
    /// Gate for `items` under `policy`.
    pub fn of(items: &[AttachmentItem], policy: PendingPolicy) -> Self {
        let pending = items.iter().filter(|i| i.is_pending()).count();
        let excluded = items.iter().filter(|i| i.is_error()).count();
        if pending > 0 && policy == PendingPolicy::Block {
            Self::Waiting(pending)
        } else {
            Self::Ready { excluded }
        }
    }

    /// Hint-row text, if any.
    pub fn hint(&self) -> Option<String> {
        match self {
            Self::Waiting(1) => Some("Waiting for 1 upload".into()),
            Self::Waiting(n) => Some(format!("Waiting for {n} uploads")),
            Self::Ready { excluded: 0 } => None,
            Self::Ready { excluded: 1 } => Some("1 file won't be sent".into()),
            Self::Ready { excluded } => Some(format!("{excluded} files won't be sent")),
        }
    }
}

/// The upload ring: follows real progress smoothly, never goes backwards, and is skipped when an
/// upload finishes in under 300 ms (no flash).
#[derive(Clone, Debug, Default)]
pub struct UploadRing {
    started: Option<ClockMs>,
    shown: f32,
    target: f32,
    target_at: Option<ClockMs>,
    from: f32,
    done_at: Option<ClockMs>,
}

/// Minimum time an upload must take before its ring is drawn.
pub const MIN_UPLOADING: Duration = Duration::from_millis(300);
/// How long the ring eases to a new progress report.
const RING_EASE: Duration = Duration::from_millis(100);
/// Ring fade after 100 % (with 120 ms delay).
pub const RING_FADE: Duration = Duration::from_millis(160);

/// What the ring draws this frame.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum RingFrame {
    /// Nothing (not uploading, or a fast upload).
    None,
    /// A determinate arc: fraction drawn and its opacity.
    Arc {
        /// Fraction `0..=1`.
        fraction: f32,
        /// Opacity.
        alpha: f32,
    },
    /// A 90° arc spinning once per second; `angle` in turns.
    Indeterminate {
        /// Rotation in turns.
        angle: f32,
    },
}

impl UploadRing {
    /// Feeds the item's state; call each frame with the shared clock.
    pub fn update(&mut self, state: &AttachmentState, now: ClockMs) -> RingFrame {
        match state {
            AttachmentState::Uploading { .. } | AttachmentState::Paused { .. } => {
                let p = match state {
                    AttachmentState::Paused { progress } => Some(*progress),
                    AttachmentState::Uploading { progress } => *progress,
                    _ => None,
                };
                let started = *self.started.get_or_insert(now);
                let Some(p) = p else {
                    if now.since(started) < MIN_UPLOADING {
                        return RingFrame::None;
                    }
                    return RingFrame::Indeterminate {
                        angle: (now.0 % 1000) as f32 / 1000.0,
                    };
                };
                let p = p.clamp(0.0, 1.0);
                if p > self.target {
                    self.from = self.current(now);
                    self.target = p;
                    self.target_at = Some(now);
                }
                self.shown = self.current(now);
                if now.since(started) < MIN_UPLOADING {
                    return RingFrame::None;
                }
                RingFrame::Arc {
                    fraction: self.shown,
                    alpha: 1.0,
                }
            }
            AttachmentState::Reading { .. } | AttachmentState::Ready { .. } => {
                let Some(started) = self.started else {
                    return RingFrame::None;
                };
                let done = *self.done_at.get_or_insert(now);
                if done.since(started) < MIN_UPLOADING {
                    return RingFrame::None;
                }
                let t = now.since(done).as_millis() as f32 - 120.0;
                if t >= RING_FADE.as_millis() as f32 {
                    return RingFrame::None;
                }
                RingFrame::Arc {
                    fraction: 1.0,
                    alpha: 1.0 - (t.max(0.0) / RING_FADE.as_millis() as f32),
                }
            }
            _ => RingFrame::None,
        }
    }

    fn current(&self, now: ClockMs) -> f32 {
        let Some(at) = self.target_at else {
            return self.shown;
        };
        let t = (now.since(at).as_millis() as f32 / RING_EASE.as_millis() as f32).min(1.0);
        self.from + (self.target - self.from) * t
    }
}

/// `▓▓▓░░` for the terminal variant.
pub fn block_bar(fraction: f32, cells: usize) -> String {
    let filled = ((fraction.clamp(0.0, 1.0) * cells as f32).round() as usize).min(cells);
    "▓".repeat(filled) + &"░".repeat(cells - filled)
}

/// Splits a row into the visible items and the `+k more` count.
pub fn overflow(len: usize, max_visible: usize, expanded: bool) -> (usize, usize) {
    if expanded || len <= max_visible {
        (len, 0)
    } else {
        // The last visible slot becomes the "+k more" chip.
        let shown = max_visible.saturating_sub(1);
        (shown, len - shown)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn meta_and_types() {
        let a = AttachmentItem::file(
            1,
            "q3.csv",
            2_400_000,
            AttachmentState::Ready {
                meta: String::new(),
            },
        );
        assert_eq!(a.meta(), "CSV · 2.4 MB");
        assert_eq!(FileType::from_name("main.rs"), FileType::Code("RS"));
        assert_eq!(format_bytes(812_000), "812 KB");
        assert_eq!(format_bytes(64), "64 B");
        let r = ContextRef::Selection {
            path: "src/auth.rs".into(),
            from: 12,
            to: 40,
        };
        assert_eq!(r.label(), ("auth.rs".into(), "lines 12–40".into()));
        assert_eq!(
            ContextRef::Url("https://www.github.com/x".into()).label().0,
            "github.com"
        );
    }

    #[test]
    fn validation() {
        let rule = AcceptRule {
            extensions: vec!["pdf".into(), "png".into()],
            ..AcceptRule::default()
        };
        assert_eq!(
            rule.validate("a.exe", 10, &[]),
            Err(Reject::Unsupported { ext: ".exe".into() })
        );
        let big = rule.validate("a.pdf", 32_000_000, &[]).unwrap_err();
        assert_eq!(big.label(), "32 MB · max 20 MB");
        let have = [AttachmentItem::file(
            1,
            "a.pdf",
            10,
            AttachmentState::Queued,
        )];
        assert_eq!(rule.validate("a.pdf", 10, &have), Err(Reject::Duplicate));
        assert!(rule.validate("b.pdf", 10, &have).is_ok());
    }

    #[test]
    fn send_gate() {
        let items = [
            AttachmentItem::file(
                1,
                "a.pdf",
                10,
                AttachmentState::Uploading {
                    progress: Some(0.4),
                },
            ),
            AttachmentItem::file(
                2,
                "b.pdf",
                10,
                AttachmentState::Error {
                    reason: "Upload failed".into(),
                    retryable: true,
                },
            ),
        ];
        assert_eq!(
            SendGate::of(&items, PendingPolicy::Block),
            SendGate::Waiting(1)
        );
        assert_eq!(
            SendGate::of(&items, PendingPolicy::SendWithout)
                .hint()
                .as_deref(),
            Some("1 file won't be sent")
        );
    }

    #[test]
    fn ring_skips_fast_uploads_and_never_goes_back() {
        let mut fast = UploadRing::default();
        let up = |p| AttachmentState::Uploading { progress: Some(p) };
        assert_eq!(fast.update(&up(0.5), ClockMs(0)), RingFrame::None);
        let ready = AttachmentState::Ready {
            meta: String::new(),
        };
        assert_eq!(fast.update(&ready, ClockMs(200)), RingFrame::None);

        let mut slow = UploadRing::default();
        slow.update(&up(0.2), ClockMs(0));
        let RingFrame::Arc { fraction, .. } = slow.update(&up(0.6), ClockMs(400)) else {
            panic!()
        };
        assert!(fraction < 0.6);
        let RingFrame::Arc { fraction: f2, .. } = slow.update(&up(0.3), ClockMs(600)) else {
            panic!()
        };
        assert!((f2 - 0.6).abs() < 1e-4, "never backwards: {f2}");
        assert!(matches!(
            slow.update(&ready, ClockMs(700)),
            RingFrame::Arc { alpha, .. } if alpha == 1.0
        ));
        assert_eq!(slow.update(&ready, ClockMs(1_000)), RingFrame::None);
    }

    #[test]
    fn bars_and_overflow() {
        assert_eq!(block_bar(0.64, 5), "▓▓▓░░");
        assert_eq!(overflow(9, 6, false), (5, 4));
        assert_eq!(overflow(4, 6, false), (4, 0));
        for v in AttachmentVariant::ALL {
            assert_eq!(AttachmentVariant::from_id(v.id()), Some(v));
        }
    }
}
