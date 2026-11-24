//! Semantic colour for generic icons (folders, files, terminal, browser, git, schedules…).
//!
//! An object icon carries one meaningful hue, light and dark, so a folder reads as a folder and a
//! PDF as a PDF at a glance (the convention of macOS Finder, VS Code icon themes and the agent
//! desktop apps). Text stays neutral, and chrome glyphs (search, settings, bell, plus, more) stay
//! neutral too: [`IconTone::for_icon`] returns `None` for them. Brand and model marks are not
//! here; they keep their official colours (`crate::brand`).
//!
//! Renderers read [`IconTone::light`] / [`IconTone::dark`] (GPUI) or the `--ox-ic-<key>` custom
//! properties that `icontone.css` defines per theme (Dioxus). A host opts out with
//! `data-ox-icon-tone="mono"`, which sets every tone back to `currentColor`.

/// One semantic hue for a kind of object icon.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum IconTone {
    /// Folders, projects, workspaces ("Choose folder").
    Folder,
    /// Documents: .md, .txt, .docx.
    Doc,
    /// Spreadsheets: .csv, .xlsx.
    Sheet,
    /// Slides: .pptx, .key.
    Slides,
    /// PDF.
    Pdf,
    /// Images: .png, .jpg, .svg.
    Image,
    /// Video: .mp4, .mov.
    Video,
    /// Audio: .mp3, .wav.
    Audio,
    /// Archives: .zip, .tar.
    Archive,
    /// Data and config: .json, .yaml, .toml.
    Data,
    /// Source code (a generic file in a repo).
    Code,
    /// Terminal, shell, command.
    Terminal,
    /// Browser, preview, the web.
    Browser,
    /// Git: branch, commit, worktree.
    Git,
    /// Schedules, calendar, clocks, automations.
    Schedule,
    /// Skills, plugins, capabilities.
    Skills,
    /// Memory.
    Memory,
    /// Agents and sub-agents.
    Agent,
    /// Approval and authorization when it waits on you.
    Approval,
    /// Running.
    Running,
    /// Done.
    Done,
    /// Failed.
    Failed,
    /// Cloud (the Local/Cloud toggle).
    Cloud,
    /// Danger: delete, remove.
    Danger,
    /// Money, credits, usage.
    Money,
    /// Charts and analytics.
    Chart,
}

impl IconTone {
    /// Every tone, in a stable order.
    pub const ALL: [IconTone; 26] = [
        IconTone::Folder,
        IconTone::Doc,
        IconTone::Sheet,
        IconTone::Slides,
        IconTone::Pdf,
        IconTone::Image,
        IconTone::Video,
        IconTone::Audio,
        IconTone::Archive,
        IconTone::Data,
        IconTone::Code,
        IconTone::Terminal,
        IconTone::Browser,
        IconTone::Git,
        IconTone::Schedule,
        IconTone::Skills,
        IconTone::Memory,
        IconTone::Agent,
        IconTone::Approval,
        IconTone::Running,
        IconTone::Done,
        IconTone::Failed,
        IconTone::Cloud,
        IconTone::Danger,
        IconTone::Money,
        IconTone::Chart,
    ];

    /// Stable key, the `--ox-ic-<key>` custom property.
    pub fn key(self) -> &'static str {
        match self {
            Self::Folder => "folder",
            Self::Doc => "doc",
            Self::Sheet => "sheet",
            Self::Slides => "slides",
            Self::Pdf => "pdf",
            Self::Image => "image",
            Self::Video => "video",
            Self::Audio => "audio",
            Self::Archive => "archive",
            Self::Data => "data",
            Self::Code => "code",
            Self::Terminal => "terminal",
            Self::Browser => "browser",
            Self::Git => "git",
            Self::Schedule => "schedule",
            Self::Skills => "skills",
            Self::Memory => "memory",
            Self::Agent => "agent",
            Self::Approval => "approval",
            Self::Running => "running",
            Self::Done => "done",
            Self::Failed => "failed",
            Self::Cloud => "cloud",
            Self::Danger => "danger",
            Self::Money => "money",
            Self::Chart => "chart",
        }
    }

    /// The hue on the light theme (`RRGGBB`).
    pub fn light(self) -> &'static str {
        match self {
            Self::Folder => "00A1FF",
            Self::Doc => "0190FE",
            Self::Sheet => "1FA463",
            Self::Slides => "F28A2E",
            Self::Pdf => "F04438",
            Self::Image => "EE3E9E",
            Self::Video => "A651F0",
            Self::Audio => "13B3A0",
            Self::Archive => "A2845E",
            Self::Data => "F28A2E",
            Self::Code => "5856D6",
            Self::Terminal => "1F9D55",
            Self::Browser => "007AFF",
            Self::Git => "F03C2E",
            Self::Schedule => "FF9500",
            Self::Skills => "AF52DE",
            Self::Memory => "5856D6",
            Self::Agent => "0190FE",
            Self::Approval => "E8B000",
            Self::Running => "0190FE",
            Self::Done => "1E8A4F",
            Self::Failed => "F04438",
            Self::Cloud => "32ADE6",
            Self::Danger => "F04438",
            Self::Money => "34C759",
            Self::Chart => "5856D6",
        }
    }

    /// The hue on the dark theme (`RRGGBB`): the same hue, about 10 % lighter.
    pub fn dark(self) -> &'static str {
        match self {
            Self::Folder => "4CBDFF",
            Self::Doc => "47B0FF",
            Self::Sheet => "6FD89A",
            Self::Slides => "FF9F45",
            Self::Pdf => "FF5A4E",
            Self::Image => "FF5CB4",
            Self::Video => "BF7BFF",
            Self::Audio => "2ED3BF",
            Self::Archive => "AC8E68",
            Self::Data => "FF9F45",
            Self::Code => "8E8CFF",
            Self::Terminal => "30D158",
            Self::Browser => "0A84FF",
            Self::Git => "FF5A4E",
            Self::Schedule => "FF9F0A",
            Self::Skills => "BF5AF2",
            Self::Memory => "7D7BFF",
            Self::Agent => "47B0FF",
            Self::Approval => "FFD60A",
            Self::Running => "47B0FF",
            Self::Done => "30D158",
            Self::Failed => "FF453A",
            Self::Cloud => "64D2FF",
            Self::Danger => "FF453A",
            Self::Money => "30D158",
            Self::Chart => "7D7BFF",
        }
    }

    /// The hue for a theme.
    pub fn hex(self, dark: bool) -> &'static str {
        if dark { self.dark() } else { self.light() }
    }

    /// Parses a key.
    pub fn parse(key: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|t| t.key() == key)
    }

    /// The tone for an icon name used across Oxyn's icon sets (`folder`, `file`, `term`,
    /// `globe`, `clock`, `branch`…). Chrome glyphs (search, gear, bell, plus, more, chat, home…)
    /// return `None` and stay neutral.
    pub fn for_icon(name: &str) -> Option<Self> {
        Some(match name {
            "folder" | "folder-open" | "project" | "workspace" => Self::Folder,
            "file" | "note" | "doc" | "docs" | "md" | "txt" => Self::Doc,
            "sheet" | "csv" | "xlsx" | "table" => Self::Sheet,
            "slides" | "deck" | "pptx" => Self::Slides,
            "pdf" => Self::Pdf,
            "image" | "img" | "png" | "jpg" | "svg" | "photo" => Self::Image,
            "video" | "mp4" | "mov" => Self::Video,
            "audio" | "mp3" | "wav" => Self::Audio,
            "archive" | "zip" | "tar" => Self::Archive,
            "data" | "json" | "yaml" | "toml" => Self::Data,
            "code" => Self::Code,
            "term" | "terminal" | "shell" | "cmd" => Self::Terminal,
            "globe" | "browser" | "web" | "preview" => Self::Browser,
            "git" | "branch" | "commit" | "worktree" | "tree" => Self::Git,
            "clock" | "schedule" | "calendar" | "cal" => Self::Schedule,
            "spark" | "skills" | "skill" | "plugin" | "plugins" | "bolt" => Self::Skills,
            "memory" | "brain" => Self::Memory,
            "bot" | "agent" | "agents" => Self::Agent,
            "pulse" | "runs" | "running" => Self::Running,
            "tasks" | "done" => Self::Done,
            "failed" | "error" => Self::Failed,
            "cloud" => Self::Cloud,
            "trash" | "delete" | "danger" => Self::Danger,
            "coin" | "credits" | "usage" | "money" => Self::Money,
            "chart" | "analytics" => Self::Chart,
            _ => return None,
        })
    }

    /// The tone for a file name, from its extension (`report.pdf` → [`IconTone::Pdf`]). A name
    /// without a known extension is a generic document; one ending in `/` is a folder.
    pub fn for_file(name: &str) -> Self {
        if name.ends_with('/') {
            return Self::Folder;
        }
        let ext = name
            .rsplit_once('.')
            .map(|(_, e)| e.to_ascii_lowercase())
            .unwrap_or_default();
        match ext.as_str() {
            "pdf" => Self::Pdf,
            "csv" | "xlsx" | "xls" | "numbers" | "tsv" => Self::Sheet,
            "ppt" | "pptx" | "key" => Self::Slides,
            "png" | "jpg" | "jpeg" | "gif" | "webp" | "svg" | "heic" => Self::Image,
            "mp4" | "mov" | "webm" => Self::Video,
            "mp3" | "wav" | "m4a" | "ogg" => Self::Audio,
            "zip" | "tar" | "gz" | "tgz" | "7z" => Self::Archive,
            "json" | "yaml" | "yml" | "toml" | "lock" => Self::Data,
            "rs" | "ts" | "tsx" | "js" | "jsx" | "py" | "go" | "html" | "css" | "swift" | "kt"
            | "java" | "cpp" | "c" | "h" | "rb" | "sh" | "sql" | "vue" | "svelte" => Self::Code,
            _ => Self::Doc,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keys_parse_back_and_are_unique() {
        let mut keys: Vec<_> = IconTone::ALL.iter().map(|t| t.key()).collect();
        for t in IconTone::ALL {
            assert_eq!(IconTone::parse(t.key()), Some(t));
        }
        keys.sort_unstable();
        keys.dedup();
        assert_eq!(keys.len(), IconTone::ALL.len());
    }

    #[test]
    fn folder_is_the_composer_light_blue() {
        assert_eq!(IconTone::Folder.light(), "00A1FF");
        assert_eq!(IconTone::for_icon("folder"), Some(IconTone::Folder));
        assert_eq!(IconTone::for_icon("search"), None);
        assert_eq!(IconTone::for_icon("gear"), None);
        assert_eq!(IconTone::for_file("Q3 report.pdf"), IconTone::Pdf);
        assert_eq!(IconTone::for_file("main.rs"), IconTone::Code);
        assert_eq!(IconTone::for_file("notes"), IconTone::Doc);
        assert_eq!(IconTone::for_file("src/"), IconTone::Folder);
    }

    #[test]
    fn every_hue_is_six_hex_digits_and_dark_differs_or_is_lighter() {
        for t in IconTone::ALL {
            for h in [t.light(), t.dark()] {
                assert_eq!(h.len(), 6, "{}", t.key());
                assert!(u32::from_str_radix(h, 16).is_ok(), "{}", t.key());
            }
        }
    }

    #[test]
    fn css_defines_every_tone_for_both_themes() {
        // The stylesheet lives in oxyn-dioxus, which is not in the published oxyn-core package: read
        // it at run time from the workspace and skip outside it, so `cargo test` works from crates.io.
        let p = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../oxyn-dioxus/src/kit/icontone.css");
        let Ok(css) = std::fs::read_to_string(&p) else {
            eprintln!(
                "skipped: {} not found (not in the Oxyn workspace)",
                p.display()
            );
            return;
        };
        for t in IconTone::ALL {
            let l = format!("--ox-ic-{}:#{}", t.key(), t.light());
            let d = format!("--ox-ic-{}:#{}", t.key(), t.dark());
            assert!(css.contains(&l), "missing {l}");
            assert!(css.contains(&d), "missing {d}");
        }
    }
}
