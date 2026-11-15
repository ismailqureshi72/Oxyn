//! affectUI fidelity pass (plan #32): the models behind the exact Chat Composer (approval modes,
//! the plus menu's attachment kinds, the recording trace) and the Comment Thread / Comments Panel
//! pair (commands, filters, order, edit runs). Behaviour-only rebuilds; nothing is copied.

/// How the agent's actions are approved (the composer's permission chip).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Approval {
    /// Ask before editing outside files or using the internet ("Request approval").
    Ask,
    /// Only ask for actions that may be unsafe ("Approve for me").
    Auto,
    /// Unrestricted access to the internet and all files ("Unrestricted access", orange).
    Full,
}

impl Approval {
    /// Every mode, in menu order.
    pub const ALL: [Approval; 3] = [Approval::Ask, Approval::Auto, Approval::Full];

    /// Parses `ask` / `auto` / `full` (anything else is `Ask`).
    pub fn parse(s: &str) -> Self {
        match s {
            "auto" => Self::Auto,
            "full" => Self::Full,
            _ => Self::Ask,
        }
    }
    /// Stable key.
    pub fn key(self) -> &'static str {
        match self {
            Self::Ask => "ask",
            Self::Auto => "auto",
            Self::Full => "full",
        }
    }
    /// Menu label.
    pub fn label(self) -> &'static str {
        match self {
            Self::Ask => "Request approval",
            Self::Auto => "Approve for me",
            Self::Full => "Full access",
        }
    }
    /// What the chip says once picked (Full access reads as a warning).
    pub fn chip(self) -> &'static str {
        match self {
            Self::Full => "Unrestricted access",
            m => m.label(),
        }
    }
    /// One-line description in the menu.
    pub fn desc(self) -> &'static str {
        match self {
            Self::Ask => "Always ask before editing outside files or using the internet",
            Self::Auto => "Only ask for potentially unsafe actions",
            Self::Full => "Unrestricted access to the internet and all files on your computer",
        }
    }
}

/// What the plus menu attaches.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FileKind {
    /// A PDF.
    Pdf,
    /// An image (shown as a thumbnail).
    Image,
    /// A spreadsheet (Excel).
    Sheet,
    /// A document (Word).
    Doc,
    /// A source file.
    Code,
    /// A folder.
    Folder,
}

impl FileKind {
    /// Every kind, in menu order.
    pub const ALL: [FileKind; 6] = [
        FileKind::Pdf,
        FileKind::Image,
        FileKind::Sheet,
        FileKind::Doc,
        FileKind::Code,
        FileKind::Folder,
    ];
    /// Stable key (`pdf`, `image`, `sheet`, `doc`, `code`, `folder`).
    pub fn key(self) -> &'static str {
        match self {
            Self::Pdf => "pdf",
            Self::Image => "image",
            Self::Sheet => "sheet",
            Self::Doc => "doc",
            Self::Code => "code",
            Self::Folder => "folder",
        }
    }
    /// Parses a key.
    pub fn parse(s: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|k| k.key() == s)
    }
    /// Menu label.
    pub fn label(self) -> &'static str {
        match self {
            Self::Pdf => "PDF",
            Self::Image => "Image",
            Self::Sheet => "Spreadsheet",
            Self::Doc => "Document",
            Self::Code => "Code",
            Self::Folder => "Folder",
        }
    }
    /// Menu hint.
    pub fn hint(self) -> &'static str {
        match self {
            Self::Pdf => "Read, summarise and cite",
            Self::Image => "PNG, JPG or a screenshot",
            Self::Sheet => "Excel or CSV",
            Self::Doc => "Word or Google Docs",
            Self::Code => "A file from your repo",
            Self::Folder => "Everything inside it",
        }
    }
    /// The file-type colour (`RRGGBB`), as file icons draw it.
    pub fn hex(self) -> &'static str {
        match self {
            Self::Pdf => "E5322D",
            Self::Image => "8B5CF6",
            Self::Sheet => "1D9A5B",
            Self::Doc => "2F6FEB",
            Self::Code => "64748B",
            Self::Folder => "3BA7F5",
        }
    }
    /// The short badge on the icon.
    pub fn badge(self) -> &'static str {
        match self {
            Self::Pdf => "PDF",
            Self::Image => "",
            Self::Sheet => "XLS",
            Self::Doc => "DOC",
            Self::Code => "</>",
            Self::Folder => "",
        }
    }
    /// A sample name for the n-th attachment of this kind (demo data).
    pub fn sample(self, n: usize) -> String {
        let base = match self {
            Self::Pdf => ["Q3 board report.pdf", "Pricing research.pdf"],
            Self::Image => ["screenshot.png", "moodboard.jpg"],
            Self::Sheet => ["Launch budget.xlsx", "Pipeline.csv"],
            Self::Doc => ["Press brief.docx", "Interview notes.docx"],
            Self::Code => ["router.rs", "checkout.ts"],
            Self::Folder => ["brand-assets", "research"],
        };
        let s = base[n % 2].to_string();
        if n < 2 {
            s
        } else {
            format!("{} ({})", s, n / 2 + 1)
        }
    }
}

/// One attachment chip in the composer.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Attachment {
    /// Unique id within the composer.
    pub id: u32,
    /// Kind (icon, colour, thumbnail).
    pub kind: FileKind,
    /// File or folder name.
    pub name: String,
}

/// Adds an attachment of `kind` with the next sample name and returns its id.
pub fn attach(list: &mut Vec<Attachment>, kind: FileKind) -> u32 {
    let id = list.iter().map(|a| a.id).max().map_or(1, |m| m + 1);
    let n = list.iter().filter(|a| a.kind == kind).count();
    list.push(Attachment {
        id,
        kind,
        name: kind.sample(n),
    });
    id
}

/// Levels (0..1) for the recording trace: `n` bars ending at time `t_ms`, the newest last. The
/// trace scrolls left by one bar every 90 ms; quiet stretches sit near zero (drawn as dots).
pub fn waveform(t_ms: u64, n: usize) -> Vec<f32> {
    let head = t_ms / 90;
    (0..n as u64)
        .map(|i| {
            let k = head.saturating_sub(n as u64 - 1 - i);
            if k < 4 {
                return 0.0;
            }
            // Deterministic speech-like envelope: words (on) and gaps (off).
            let word = (k / 7).wrapping_mul(2_654_435_761) % 5;
            if word == 0 {
                return 0.04;
            }
            let h = k.wrapping_mul(40_503).wrapping_add(k / 3 * 977) % 100;
            0.15 + (h as f32 / 100.0) * 0.85
        })
        .collect()
}

/// A comment's leading command (`/edit`), split from its text. The command keeps its case for
/// display ("/Edit"); `None` when the text does not start with one.
pub fn split_command(text: &str) -> (Option<&str>, &str) {
    let t = text.trim_start();
    if let Some(rest) = t.strip_prefix('/') {
        let end = rest.find(char::is_whitespace).unwrap_or(rest.len());
        if end > 0 && rest[..end].chars().all(|c| c.is_ascii_alphabetic()) {
            return (Some(&t[..end + 1]), rest[end..].trim_start());
        }
    }
    (None, t)
}

/// True for a runnable `/edit` comment.
pub fn is_edit(text: &str) -> bool {
    matches!(split_command(text).0, Some(c) if c.eq_ignore_ascii_case("/edit"))
}

/// True when the comment asks something (ends with "?" and is not an edit).
pub fn is_question(text: &str) -> bool {
    !is_edit(text) && text.trim_end().ends_with('?')
}

/// "just now", "4min ago", "2h ago".
pub fn ago(minutes: u32) -> String {
    match minutes {
        0 => "just now".to_string(),
        m if m < 60 => format!("{m}min ago"),
        m => format!("{}h ago", m / 60),
    }
}

/// Where an `/edit` stands in the Comments Panel.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum EditRun {
    /// Not run yet: "Run edit".
    #[default]
    Idle,
    /// Running: a spinner and "Running…".
    Running,
    /// Applied: "Edit applied" and Undo.
    Applied,
}

/// One comment in the Comments Panel.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PanelComment {
    /// Id.
    pub id: u32,
    /// Author name.
    pub author: String,
    /// Avatar seed (picks the drawn portrait).
    pub avatar: u8,
    /// Minutes ago.
    pub minutes: u32,
    /// Text; a leading `/edit` makes it runnable.
    pub text: String,
    /// Resolved (dimmed and struck through).
    pub resolved: bool,
    /// Edit state.
    pub run: EditRun,
}

impl PanelComment {
    /// A comment.
    pub fn new(id: u32, author: &str, avatar: u8, minutes: u32, text: &str) -> Self {
        Self {
            id,
            author: author.to_string(),
            avatar,
            minutes,
            text: text.to_string(),
            resolved: false,
            run: EditRun::Idle,
        }
    }
}

/// Comments Panel tabs.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PanelFilter {
    /// Everything.
    All,
    /// Questions only.
    Questions,
    /// `/edit` comments only.
    Edits,
}

/// The comments a tab shows, newest first unless `oldest_first`. Edits that ran stay listed.
pub fn panel_view(c: &[PanelComment], f: PanelFilter, oldest_first: bool) -> Vec<usize> {
    let mut v: Vec<usize> = (0..c.len())
        .filter(|&i| match f {
            PanelFilter::All => true,
            PanelFilter::Questions => is_question(&c[i].text),
            PanelFilter::Edits => is_edit(&c[i].text),
        })
        .collect();
    v.sort_by_key(|&i| (c[i].minutes, std::cmp::Reverse(c[i].id)));
    if oldest_first {
        v.reverse();
    }
    v
}

/// The Edits tab's badge: edits not applied yet.
pub fn open_edits(c: &[PanelComment]) -> usize {
    c.iter()
        .filter(|x| is_edit(&x.text) && x.run != EditRun::Applied && !x.resolved)
        .count()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn approval_modes() {
        assert_eq!(Approval::parse("full").chip(), "Unrestricted access");
        assert_eq!(Approval::parse("x"), Approval::Ask);
        assert_eq!(Approval::Auto.key(), "auto");
    }

    #[test]
    fn attachments_get_ids_and_names() {
        let mut v = vec![];
        assert_eq!(attach(&mut v, FileKind::Pdf), 1);
        assert_eq!(attach(&mut v, FileKind::Pdf), 2);
        attach(&mut v, FileKind::Pdf);
        assert_eq!(v[0].name, "Q3 board report.pdf");
        assert_eq!(v[1].name, "Pricing research.pdf");
        assert_eq!(v[2].name, "Q3 board report.pdf (2)");
        assert_eq!(FileKind::parse("sheet"), Some(FileKind::Sheet));
    }

    #[test]
    fn waveform_scrolls() {
        let a = waveform(9_000, 24);
        let b = waveform(9_090, 24);
        assert_eq!(a.len(), 24);
        assert_eq!(a[1..], b[..23]);
        assert!(waveform(0, 8).iter().all(|x| *x == 0.0));
    }

    #[test]
    fn commands_and_filters() {
        assert_eq!(
            split_command("/Edit Rename it"),
            (Some("/Edit"), "Rename it")
        );
        assert_eq!(split_command("Hello /edit"), (None, "Hello /edit"));
        assert!(is_edit("/edit x") && is_edit("/Edit x") && !is_edit("/plan x"));
        assert!(is_question("Can we ship?") && !is_question("/edit ship?"));
        let c = vec![
            PanelComment::new(1, "A", 0, 10, "/Edit one"),
            PanelComment::new(2, "B", 1, 2, "Why?"),
            PanelComment::new(3, "C", 2, 30, "Note"),
        ];
        assert_eq!(panel_view(&c, PanelFilter::All, false), vec![1, 0, 2]);
        assert_eq!(panel_view(&c, PanelFilter::All, true), vec![2, 0, 1]);
        assert_eq!(panel_view(&c, PanelFilter::Edits, false), vec![0]);
        assert_eq!(panel_view(&c, PanelFilter::Questions, false), vec![1]);
        assert_eq!(open_edits(&c), 1);
        assert_eq!(ago(0), "just now");
        assert_eq!(ago(12), "12min ago");
        assert_eq!(ago(130), "2h ago");
    }
}

/// Levels (0..1) for affectUI's recording meter: `n` capsules that sit as dots when quiet and
/// stretch with the voice. Most bars rest as dots; a few rise per frame (frames every 110 ms).
pub fn meter_levels(t_ms: u64, n: usize) -> Vec<f32> {
    let f = t_ms / 110;
    // Words and pauses: a pause frame keeps every bar a dot.
    let speaking = !(f / 6).wrapping_mul(2_654_435_761).is_multiple_of(4) && t_ms > 300;
    (0..n as u64)
        .map(|i| {
            if !speaking {
                return 0.0;
            }
            let h = (f.wrapping_mul(97).wrapping_add(i.wrapping_mul(131)))
                .wrapping_mul(2_246_822_519)
                % 100;
            // A soft hump around the middle of the row, the rest mostly dots.
            let mid = 1.0 - ((i as f32 - n as f32 / 2.0).abs() / (n as f32 / 2.0)).min(1.0);
            let v = h as f32 / 100.0 * (0.35 + 0.65 * mid);
            if v < 0.32 { 0.0 } else { v.min(1.0) }
        })
        .collect()
}

/// Six portrait photos of fictional people (generated, no real person), 72 px WebP, for the
/// comment avatars. `seed` wraps.
pub fn avatar_webp(seed: u8) -> &'static [u8] {
    const A: [&[u8]; 6] = [
        include_bytes!("../../assets/avatars/avatar-0.webp"),
        include_bytes!("../../assets/avatars/avatar-1.webp"),
        include_bytes!("../../assets/avatars/avatar-2.webp"),
        include_bytes!("../../assets/avatars/avatar-3.webp"),
        include_bytes!("../../assets/avatars/avatar-4.webp"),
        include_bytes!("../../assets/avatars/avatar-5.webp"),
    ];
    A[seed as usize % A.len()]
}

/// The avatar photo as a `data:image/webp;base64,…` URI.
pub fn avatar_data_uri(seed: u8) -> String {
    const T: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let b = avatar_webp(seed);
    let mut s = String::with_capacity(23 + b.len() * 4 / 3 + 4);
    s.push_str("data:image/webp;base64,");
    for c in b.chunks(3) {
        let n = (u32::from(c[0]) << 16)
            | (u32::from(*c.get(1).unwrap_or(&0)) << 8)
            | u32::from(*c.get(2).unwrap_or(&0));
        s.push(T[(n >> 18) as usize & 63] as char);
        s.push(T[(n >> 12) as usize & 63] as char);
        s.push(if c.len() > 1 {
            T[(n >> 6) as usize & 63] as char
        } else {
            '='
        });
        s.push(if c.len() > 2 {
            T[n as usize & 63] as char
        } else {
            '='
        });
    }
    s
}

#[cfg(test)]
mod fidelity2_tests {
    use super::*;

    #[test]
    fn meter_rests_as_dots_and_rises() {
        assert!(meter_levels(0, 16).iter().all(|v| *v == 0.0));
        let any = (0..80u64).any(|k| meter_levels(400 + k * 110, 16).iter().any(|v| *v > 0.5));
        assert!(any);
        assert_eq!(meter_levels(5_000, 16).len(), 16);
    }

    #[test]
    fn avatars_are_webp_data_uris() {
        assert_eq!(&avatar_webp(0)[..4], b"RIFF");
        let u = avatar_data_uri(7);
        assert!(u.starts_with("data:image/webp;base64,UklGR"));
        assert_eq!((u.len() - 23) % 4, 0);
    }
}
