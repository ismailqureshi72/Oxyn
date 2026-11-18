//! Screens S2: the workspace screens (Folders Overview and Detail, Integrations, Kanban, CRM,
//! Analytics) on the Screens 1 frame. Each screen's sidebar comes from a persona ([`persona`]);
//! the data here is fictional sample data written for Oxyn, and every view rule (filter, sort,
//! group, move) is a pure function so Dioxus and GPUI render the same thing.

pub use super::screensfid::ScreenState;
use super::screensfid::thousands;

// ───────────────────────────────────────────────────────────── Personas ──

/// A sidebar nav entry: key, label, glyph, count (0: none).
pub type NavItem = (&'static str, &'static str, &'static str, u32);

/// A screen family's sidebar: its primary nav, then one labelled group.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Persona {
    /// Stable key (`files`, `board`, `sales`, `revenue`).
    pub key: &'static str,
    /// Primary nav.
    pub nav: &'static [NavItem],
    /// The group label under the nav.
    pub group: &'static str,
    /// The group's entries (key, label, glyph).
    pub items: &'static [(&'static str, &'static str, &'static str)],
}

/// The four personas of the workspace screens.
pub const PERSONAS: [Persona; 4] = [
    Persona {
        key: "files",
        nav: &[
            ("new", "New chat", "pen", 0),
            ("files", "Folders", "folder", 0),
            ("apps", "Integrations", "apps", 0),
        ],
        group: "Projects",
        items: &[
            ("autumn", "Autumn launch", "folderopen"),
            ("brand", "Brand kit", "folderopen"),
            ("hiring", "Hiring", "folderopen"),
            ("research", "Research", "folderopen"),
        ],
    },
    Persona {
        key: "board",
        nav: &[
            ("inbox", "Inbox", "inbox", 3),
            ("activity", "Activity", "bell", 2),
            ("mine", "My issues", "user", 0),
            ("board", "Board", "layers", 0),
            ("projects", "Projects", "box", 0),
        ],
        group: "Problems",
        items: &[
            ("p1", "Sign-in links expire early", "mail"),
            ("p2", "Card payments retry twice", "card"),
            ("p3", "Reports open slowly", "bug"),
        ],
    },
    Persona {
        key: "sales",
        nav: &[
            ("home", "Home", "home", 0),
            ("companies", "Companies", "building", 284),
            ("contacts", "Contacts", "people", 0),
            ("deals", "Deals", "dollar", 0),
            ("reports", "Reports", "chart", 0),
        ],
        group: "Pipelines",
        items: &[
            ("newbiz", "New business", "target"),
            ("renew", "Renewals", "cycle"),
            ("partners", "Partners", "people"),
        ],
    },
    Persona {
        key: "revenue",
        nav: &[
            ("home", "Home", "home", 0),
            ("revenue", "Revenue", "chart", 0),
            ("invoices", "Invoices", "doc", 12),
            ("payouts", "Payouts", "wallet", 0),
            ("customers", "Customers", "people", 0),
        ],
        group: "Channels",
        items: &[
            ("store", "Web store", "store"),
            ("market", "Marketplace", "bag"),
            ("wholesale", "Wholesale", "box"),
            ("subs", "Subscriptions", "cycle"),
        ],
    },
];

/// The persona for a key (the Folders one for anything unknown).
pub fn persona(key: &str) -> &'static Persona {
    PERSONAS
        .iter()
        .find(|p| p.key == key)
        .unwrap_or(&PERSONAS[0])
}

/// A screen state bar: Data · Loading · Empty, then one state of the screen's own.
pub type States = [(&'static str, &'static str); 4];

/// Folders Overview states.
pub const FILES_STATES: States = [
    ("sample", "Data"),
    ("loading", "Loading"),
    ("empty", "Empty"),
    ("creating", "Creating"),
];
/// Folders Detail states.
pub const FOLDER_STATES: States = [
    ("sample", "Data"),
    ("loading", "Loading"),
    ("empty", "Empty"),
    ("uploading", "Uploading"),
];
/// Integrations states.
pub const INTEG_STATES: States = [
    ("sample", "Data"),
    ("loading", "Loading"),
    ("empty", "Empty"),
    ("failed", "Link failed"),
];
/// Kanban states.
pub const KANBAN_STATES: States = [
    ("sample", "Data"),
    ("loading", "Loading"),
    ("empty", "Empty"),
    ("dragging", "Dragging"),
];
/// CRM states.
pub const CRM_STATES: States = [
    ("sample", "Data"),
    ("loading", "Loading"),
    ("empty", "Empty"),
    ("editing", "Editing"),
];
/// Analytics states.
pub const ANALYTICS_STATES: States = [
    ("sample", "Data"),
    ("loading", "Loading"),
    ("empty", "Empty"),
    ("compare", "Compared"),
];

/// The data state behind a state key (a screen's own states show sample data).
pub fn base_state(k: &str) -> ScreenState {
    ScreenState::from_key(k)
}

// ───────────────────────────────────────────────────────────── Files ──

/// A kind of file, with its type badge and semantic tone.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FileKind {
    /// A text document.
    Doc,
    /// A spreadsheet.
    Sheet,
    /// A PDF.
    Pdf,
    /// A slide deck.
    Slides,
    /// An image.
    Image,
}

impl FileKind {
    /// Every kind, in menu order.
    pub const ALL: [FileKind; 5] = [
        FileKind::Doc,
        FileKind::Sheet,
        FileKind::Pdf,
        FileKind::Slides,
        FileKind::Image,
    ];
    /// Stable key, also the `--ox-ic-*` tone.
    pub fn key(self) -> &'static str {
        match self {
            FileKind::Doc => "doc",
            FileKind::Sheet => "sheet",
            FileKind::Pdf => "pdf",
            FileKind::Slides => "slides",
            FileKind::Image => "image",
        }
    }
    /// The badge on the page glyph.
    pub fn badge(self) -> &'static str {
        match self {
            FileKind::Doc => "DOC",
            FileKind::Sheet => "XLS",
            FileKind::Pdf => "PDF",
            FileKind::Slides => "PPT",
            FileKind::Image => "PNG",
        }
    }
    /// The kind in words.
    pub fn label(self) -> &'static str {
        match self {
            FileKind::Doc => "Document",
            FileKind::Sheet => "Spreadsheet",
            FileKind::Pdf => "PDF",
            FileKind::Slides => "Slides",
            FileKind::Image => "Image",
        }
    }
    /// From a key (`doc` for anything unknown).
    pub fn from_key(k: &str) -> FileKind {
        FileKind::ALL
            .into_iter()
            .find(|x| x.key() == k)
            .unwrap_or(FileKind::Doc)
    }
}

/// A folder tile on the Overview.
#[derive(Clone, Debug, PartialEq)]
pub struct FolderTile {
    /// Id.
    pub id: u32,
    /// Name.
    pub name: String,
    /// Shared with people (else private).
    pub shared: bool,
    /// Files inside.
    pub files: u32,
    /// Minutes since it changed.
    pub mins: u32,
    /// Starred (sorts first).
    pub starred: bool,
}

/// Overview folder tabs.
pub const FOLDER_TABS: [(&str, &str); 3] =
    [("all", "All"), ("shared", "Shared"), ("private", "Private")];
/// Template tabs.
pub const TEMPLATE_TABS: [(&str, &str); 3] = [
    ("all", "All"),
    ("mine", "Mine"),
    ("shared", "Shared with me"),
];
/// Folder sort keys (key, label).
pub const FOLDER_SORTS: [(&str, &str); 3] = [
    ("recent", "Last changed"),
    ("name", "Name"),
    ("files", "Most files"),
];

/// The sample folders (none unless `sample`).
pub fn folder_tiles(sample: bool) -> Vec<FolderTile> {
    if !sample {
        return vec![];
    }
    [
        ("Brand kit", true, 3, 40),
        ("Hiring", false, 4, 300),
        ("Autumn launch", true, 5, 12),
        ("Research", false, 2, 2900),
    ]
    .into_iter()
    .enumerate()
    .map(|(i, (n, s, f, m))| FolderTile {
        id: i as u32 + 1,
        name: n.to_string(),
        shared: s,
        files: f,
        mins: m,
        starred: false,
    })
    .collect()
}

/// The folders a tab, a search and a sort leave, starred first.
pub fn tiles_view(list: &[FolderTile], tab: &str, q: &str, sort: &str) -> Vec<FolderTile> {
    let q = q.trim().to_lowercase();
    let mut v: Vec<FolderTile> = list
        .iter()
        .filter(|f| match tab {
            "shared" => f.shared,
            "private" => !f.shared,
            _ => true,
        })
        .filter(|f| q.is_empty() || f.name.to_lowercase().contains(&q))
        .cloned()
        .collect();
    match sort {
        "name" => v.sort_by_key(|f| f.name.to_lowercase()),
        "files" => v.sort_by_key(|f| std::cmp::Reverse(f.files)),
        "recent" => v.sort_by_key(|f| f.mins),
        _ => {}
    }
    v.sort_by_key(|f| !f.starred);
    v
}

/// "Shared · 3 files".
pub fn tile_line(f: &FolderTile) -> String {
    let who = if f.shared { "Shared" } else { "Private" };
    let n = f.files;
    format!("{who} · {n} {}", if n == 1 { "file" } else { "files" })
}

/// A file template.
#[derive(Clone, Debug, PartialEq)]
pub struct Template {
    /// Id.
    pub id: u32,
    /// Name.
    pub name: &'static str,
    /// Kind.
    pub kind: FileKind,
    /// Owner.
    pub owner: &'static str,
    /// Made by the signed-in person.
    pub mine: bool,
    /// Shared (else private).
    pub shared: bool,
}

/// The sample templates.
pub fn templates(sample: bool) -> Vec<Template> {
    if !sample {
        return vec![];
    }
    vec![
        Template {
            id: 1,
            name: "Style-Guide",
            kind: FileKind::Pdf,
            owner: "Tobi Lund",
            mine: false,
            shared: true,
        },
        Template {
            id: 2,
            name: "Launch-Budget",
            kind: FileKind::Sheet,
            owner: "Tobi Lund",
            mine: false,
            shared: true,
        },
        Template {
            id: 3,
            name: "Pitch-Notes",
            kind: FileKind::Doc,
            owner: "Rowan Vale",
            mine: true,
            shared: false,
        },
        Template {
            id: 4,
            name: "Monday-Numbers",
            kind: FileKind::Sheet,
            owner: "Rowan Vale",
            mine: true,
            shared: false,
        },
    ]
}

/// The templates a tab, a type filter, a search and a sort direction leave.
pub fn templates_view(
    list: &[Template],
    tab: &str,
    kind: Option<FileKind>,
    q: &str,
    desc: bool,
) -> Vec<Template> {
    let q = q.trim().to_lowercase();
    let mut v: Vec<Template> = list
        .iter()
        .filter(|t| match tab {
            "mine" => t.mine,
            "shared" => !t.mine,
            _ => true,
        })
        .filter(|t| kind.is_none_or(|k| t.kind == k))
        .filter(|t| q.is_empty() || t.name.to_lowercase().contains(&q))
        .cloned()
        .collect();
    if desc {
        v.sort_by(|a, b| b.name.cmp(a.name));
    }
    v
}

/// A file in an open folder.
#[derive(Clone, Debug, PartialEq)]
pub struct DocFile {
    /// Id.
    pub id: u32,
    /// Name.
    pub name: String,
    /// Kind.
    pub kind: FileKind,
    /// Minutes since it changed.
    pub mins: u32,
}

/// The files in the sample open folder.
pub fn folder_docs(sample: bool) -> Vec<DocFile> {
    if !sample {
        return vec![];
    }
    [
        ("Launch-Brief", FileKind::Doc, 120),
        ("Rollout-Plan", FileKind::Pdf, 1500),
        ("Spend-Tracker", FileKind::Sheet, 1700),
        ("Ad-Lines", FileKind::Doc, 4400),
        ("Spring-Results", FileKind::Sheet, 10_500),
    ]
    .into_iter()
    .enumerate()
    .map(|(i, (n, k, m))| DocFile {
        id: i as u32 + 1,
        name: n.to_string(),
        kind: k,
        mins: m,
    })
    .collect()
}

/// When a file changed, in words ("2 hours ago", "Yesterday", "3 days ago", "Last week").
pub fn when_words(mins: u32) -> String {
    match mins {
        0 => "Just now".to_string(),
        1..=59 => format!("{mins} min ago"),
        60..=119 => "1 hour ago".to_string(),
        120..=1439 => format!("{} hours ago", mins / 60),
        1440..=2879 => "Yesterday".to_string(),
        2880..=10_079 => format!("{} days ago", mins / 1440),
        _ => "Last week".to_string(),
    }
}

/// The kind a new file name implies (`.csv` a sheet, `.pdf` a PDF…).
pub fn kind_for(name: &str) -> FileKind {
    let ext = name.rsplit('.').next().unwrap_or("").to_lowercase();
    match ext.as_str() {
        "csv" | "xls" | "xlsx" | "numbers" => FileKind::Sheet,
        "pdf" => FileKind::Pdf,
        "ppt" | "pptx" | "key" => FileKind::Slides,
        "png" | "jpg" | "jpeg" | "webp" | "gif" => FileKind::Image,
        _ => FileKind::Doc,
    }
}

// ───────────────────────────────────────────────────────────── Integrations ──

/// An app that can be linked.
#[derive(Clone, Debug, PartialEq)]
pub struct Integ {
    /// Brand id (official mark).
    pub id: &'static str,
    /// Name.
    pub name: &'static str,
    /// One line on what linking it does.
    pub blurb: &'static str,
    /// Category key.
    pub cat: &'static str,
    /// Linked.
    pub on: bool,
    /// Verified publisher.
    pub verified: bool,
}

/// Categories, in page order (key, title).
pub const INTEG_CATS: [(&str, &str); 5] = [
    ("popular", "Popular"),
    ("google", "Google"),
    ("talk", "Communication"),
    ("dev", "Engineering"),
    ("money", "Sales and payments"),
];

/// Filter tabs.
pub const INTEG_TABS: [(&str, &str); 3] =
    [("all", "All"), ("on", "Connected"), ("off", "Available")];

/// The sample directory.
pub fn integrations(sample: bool) -> Vec<Integ> {
    if !sample {
        return vec![];
    }
    let v: [(&str, &str, &str, &str, bool, bool); 18] = [
        (
            "slack",
            "Slack",
            "Get a ping in a channel and reply to it from the chat.",
            "popular",
            true,
            true,
        ),
        (
            "notion",
            "Notion",
            "Turn answers into pages and keep project notes current.",
            "popular",
            false,
            true,
        ),
        (
            "linear",
            "Linear",
            "File issues from feedback and follow them to done.",
            "popular",
            false,
            true,
        ),
        (
            "figma",
            "Figma",
            "Show frames inline and tie them to the task they belong to.",
            "popular",
            false,
            true,
        ),
        (
            "github",
            "GitHub",
            "See pull requests and checks next to each task.",
            "popular",
            false,
            true,
        ),
        (
            "hubspot",
            "HubSpot",
            "Keep companies and deals matched in both places.",
            "popular",
            false,
            true,
        ),
        (
            "stripe",
            "Stripe",
            "Read payments and refunds without leaving the page.",
            "popular",
            false,
            true,
        ),
        (
            "zapier",
            "Zapier",
            "Start a zap when an agent finishes a run.",
            "popular",
            false,
            false,
        ),
        (
            "googledrive",
            "Google Drive",
            "Pick files from Drive and preview them in place.",
            "google",
            false,
            true,
        ),
        (
            "googlecalendar",
            "Google Calendar",
            "See the next meetings with each contact on one line.",
            "google",
            false,
            true,
        ),
        (
            "gmail",
            "Gmail",
            "Draft replies from a thread and send them after review.",
            "google",
            false,
            true,
        ),
        (
            "zoom",
            "Zoom",
            "Start a call from any record and keep the recording link.",
            "talk",
            false,
            true,
        ),
        (
            "discord",
            "Discord",
            "Post release notes to a community channel.",
            "talk",
            false,
            false,
        ),
        (
            "intercom",
            "Intercom",
            "Bring support chats in with the customer's history.",
            "talk",
            false,
            true,
        ),
        (
            "gitlab",
            "GitLab",
            "Follow merge requests and pipelines per task.",
            "dev",
            false,
            true,
        ),
        (
            "sentry",
            "Sentry",
            "Link errors to the issue that fixes them.",
            "dev",
            false,
            true,
        ),
        (
            "vercel",
            "Vercel",
            "See each preview deploy beside its change.",
            "dev",
            false,
            true,
        ),
        (
            "shopify",
            "Shopify",
            "Read orders and stock levels by store.",
            "money",
            false,
            true,
        ),
    ];
    v.into_iter()
        .map(|(id, name, blurb, cat, on, verified)| Integ {
            id,
            name,
            blurb,
            cat,
            on,
            verified,
        })
        .collect()
}

/// Popular shows six cards, then a "more" row for the rest.
pub const POPULAR_SHOWN: usize = 6;

/// The directory grouped by category for a tab, a search and an optional category, empty groups
/// dropped. A search looks at names and lines.
pub fn integ_view(
    list: &[Integ],
    tab: &str,
    q: &str,
    cat: &str,
) -> Vec<(&'static str, &'static str, Vec<Integ>)> {
    let q = q.trim().to_lowercase();
    INTEG_CATS
        .iter()
        .filter(|(k, _)| cat.is_empty() || cat == *k)
        .map(|(k, t)| {
            let v: Vec<Integ> = list
                .iter()
                .filter(|i| i.cat == *k)
                .filter(|i| match tab {
                    "on" => i.on,
                    "off" => !i.on,
                    _ => true,
                })
                .filter(|i| {
                    q.is_empty()
                        || i.name.to_lowercase().contains(&q)
                        || i.blurb.to_lowercase().contains(&q)
                })
                .cloned()
                .collect();
            (*k, *t, v)
        })
        .filter(|(_, _, v)| !v.is_empty())
        .collect()
}

/// "Stripe, Zapier and 2 more" for the folded part of a group.
pub fn more_words(rest: &[Integ]) -> String {
    match rest.len() {
        0 => String::new(),
        1 => rest[0].name.to_string(),
        2 => format!("{} and {}", rest[0].name, rest[1].name),
        n => format!("{}, {} and {} more", rest[0].name, rest[1].name, n - 2),
    }
}

// ───────────────────────────────────────────────────────────── Kanban ──

/// A board column: three priorities, then Completed.
pub const KB_LANES: [(&str, &str); 4] = [
    ("low", "Low priority"),
    ("med", "Medium priority"),
    ("high", "High priority"),
    ("done", "Completed"),
];

/// Board tabs.
pub const KB_TABS: [(&str, &str); 3] = [
    ("all", "All tasks"),
    ("open", "Open"),
    ("done", "Completed"),
];

/// Sort keys.
pub const KB_SORTS: [(&str, &str); 3] = [
    ("manual", "Manual"),
    ("talk", "Most comments"),
    ("title", "Title"),
];

/// Where a card stands.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CardStatus {
    /// Waiting for review (blue half dot).
    Review,
    /// Being worked on (purple ring).
    Progress,
    /// Needs a person (amber triangle).
    Attention,
    /// Done (green dot).
    Fixed,
}

impl CardStatus {
    /// Key, also the semantic tone.
    pub fn key(self) -> &'static str {
        match self {
            CardStatus::Review => "review",
            CardStatus::Progress => "progress",
            CardStatus::Attention => "attention",
            CardStatus::Fixed => "fixed",
        }
    }
    /// Words.
    pub fn label(self) -> &'static str {
        match self {
            CardStatus::Review => "Ready for review",
            CardStatus::Progress => "In progress",
            CardStatus::Attention => "Needs attention",
            CardStatus::Fixed => "Fixed",
        }
    }
}

/// A board card.
#[derive(Clone, Debug, PartialEq)]
pub struct Card {
    /// Id.
    pub id: u32,
    /// Title.
    pub title: String,
    /// Two lines of detail.
    pub body: String,
    /// Lane key.
    pub lane: &'static str,
    /// Status.
    pub status: CardStatus,
    /// A label (`bug`, `feature`, `ux`) or empty.
    pub label: &'static str,
    /// Comments.
    pub talk: u32,
    /// Links.
    pub links: u32,
}

/// The labels a card can carry (key, words); their tones are blue, purple and amber.
pub const KB_LABELS: [(&str, &str); 3] = [("bug", "Bug"), ("feature", "Feature"), ("ux", "Design")];

/// The sample board (11 cards).
pub fn cards(sample: bool) -> Vec<Card> {
    if !sample {
        return vec![];
    }
    use CardStatus::*;
    let v: [(&str, &str, &str, CardStatus, &str, u32, u32); 11] = [
        (
            "Password reset loops back",
            "People who joined with a passkey and later set a password land on the sign-in page again.",
            "low",
            Review,
            "bug",
            12,
            1,
        ),
        (
            "Avatar crop is ignored",
            "The editor shows the cropped square, but the saved picture is the full photo.",
            "low",
            Progress,
            "ux",
            8,
            2,
        ),
        (
            "Export settings as CSV",
            "Admins want one file with every workspace setting for their audit.",
            "low",
            Progress,
            "feature",
            6,
            1,
        ),
        (
            "Sessions end too soon",
            "People are signed out after half an hour instead of the full day they chose.",
            "med",
            Attention,
            "bug",
            21,
            1,
        ),
        (
            "Magic link opens a 404",
            "The email goes out on time but its link points at a page we removed.",
            "med",
            Review,
            "bug",
            17,
            1,
        ),
        (
            "Reports page is slow",
            "The monthly report takes several seconds to draw on a large workspace.",
            "med",
            Review,
            "",
            14,
            1,
        ),
        (
            "Checkout fails on one browser",
            "Buyers on an older mobile browser can't finish paying for an order.",
            "high",
            Attention,
            "bug",
            30,
            1,
        ),
        (
            "Calls drop after ten minutes",
            "Group calls with more than four people end without a warning.",
            "high",
            Attention,
            "",
            18,
            2,
        ),
        (
            "Webhook retries pile up",
            "Failed deliveries retry every second instead of backing off.",
            "high",
            Review,
            "feature",
            9,
            1,
        ),
        (
            "Live updates stop",
            "The feed stops after the socket sits idle for a minute.",
            "done",
            Fixed,
            "",
            32,
            0,
        ),
        (
            "App crashes on sign-in",
            "Opening the mobile app with an expired token closed it at once.",
            "done",
            Fixed,
            "bug",
            11,
            0,
        ),
    ];
    v.into_iter()
        .enumerate()
        .map(|(i, (t, b, l, s, lb, c, k))| Card {
            id: i as u32 + 1,
            title: t.to_string(),
            body: b.to_string(),
            lane: l,
            status: s,
            label: lb,
            talk: c,
            links: k,
        })
        .collect()
}

/// The cards of one lane for a tab, a search and a sort.
pub fn lane_view(list: &[Card], lane: &str, tab: &str, q: &str, sort: &str) -> Vec<Card> {
    let q = q.trim().to_lowercase();
    let mut v: Vec<Card> = list
        .iter()
        .filter(|c| c.lane == lane)
        .filter(|c| match tab {
            "open" => c.lane != "done",
            "done" => c.lane == "done",
            _ => true,
        })
        .filter(|c| q.is_empty() || c.title.to_lowercase().contains(&q))
        .cloned()
        .collect();
    match sort {
        "talk" => v.sort_by_key(|c| std::cmp::Reverse(c.talk)),
        "title" => v.sort_by(|a, b| a.title.cmp(&b.title)),
        _ => {}
    }
    v
}

/// Tab counts (all, open, done).
pub fn tab_counts(list: &[Card]) -> (usize, usize, usize) {
    let done = list.iter().filter(|c| c.lane == "done").count();
    (list.len(), list.len() - done, done)
}

/// Moves a card to a lane (to the end); a card moved to Completed turns Fixed, one moved out of
/// it goes back to In progress. Returns false when nothing moved.
pub fn move_card(list: &mut Vec<Card>, id: u32, lane: &'static str) -> bool {
    let Some(i) = list.iter().position(|c| c.id == id) else {
        return false;
    };
    if list[i].lane == lane {
        return false;
    }
    let mut c = list.remove(i);
    if lane == "done" {
        c.status = CardStatus::Fixed;
    } else if c.lane == "done" {
        c.status = CardStatus::Progress;
    }
    c.lane = lane;
    list.push(c);
    true
}

/// The lane left (-1) or right (+1) of `lane`, clamped.
pub fn lane_step(lane: &str, d: i32) -> &'static str {
    let i = KB_LANES.iter().position(|(k, _)| *k == lane).unwrap_or(0) as i32;
    KB_LANES[(i + d).clamp(0, KB_LANES.len() as i32 - 1) as usize].0
}

/// Adds a card at the top of a lane and returns its id.
pub fn add_card(list: &mut Vec<Card>, title: &str, lane: &'static str) -> u32 {
    let id = list.iter().map(|c| c.id).max().unwrap_or(0) + 1;
    list.insert(
        0,
        Card {
            id,
            title: title.trim().to_string(),
            body: String::new(),
            lane,
            status: if lane == "done" {
                CardStatus::Fixed
            } else {
                CardStatus::Progress
            },
            label: "",
            talk: 0,
            links: 0,
        },
    );
    id
}

// ───────────────────────────────────────────────────────────── CRM ──

/// A pipeline stage, with its tone.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CrmStage {
    /// New lead (grey).
    New,
    /// Qualified (blue).
    Qualified,
    /// Proposal sent (purple).
    Proposal,
    /// Negotiating (amber).
    Negotiation,
    /// Won (green).
    Won,
    /// Lost (red).
    Lost,
}

impl CrmStage {
    /// In pipeline order.
    pub const ALL: [CrmStage; 6] = [
        CrmStage::New,
        CrmStage::Qualified,
        CrmStage::Proposal,
        CrmStage::Negotiation,
        CrmStage::Won,
        CrmStage::Lost,
    ];
    /// Key, also the tone.
    pub fn key(self) -> &'static str {
        match self {
            CrmStage::New => "new",
            CrmStage::Qualified => "qualified",
            CrmStage::Proposal => "proposal",
            CrmStage::Negotiation => "negotiation",
            CrmStage::Won => "won",
            CrmStage::Lost => "lost",
        }
    }
    /// Words.
    pub fn label(self) -> &'static str {
        match self {
            CrmStage::New => "New",
            CrmStage::Qualified => "Qualified",
            CrmStage::Proposal => "Proposal",
            CrmStage::Negotiation => "Negotiation",
            CrmStage::Won => "Won",
            CrmStage::Lost => "Lost",
        }
    }
    /// From a key (New for anything unknown).
    pub fn from_key(k: &str) -> CrmStage {
        CrmStage::ALL
            .into_iter()
            .find(|s| s.key() == k)
            .unwrap_or(CrmStage::New)
    }
}

/// A company row.
#[derive(Clone, Debug, PartialEq)]
pub struct Company {
    /// Id.
    pub id: u32,
    /// Name (also the brand mark).
    pub name: String,
    /// Category tag.
    pub cat: String,
    /// Domain.
    pub domain: String,
    /// Owner.
    pub owner: String,
    /// Deal value, whole dollars.
    pub value: u32,
    /// Stage.
    pub stage: CrmStage,
    /// Last contact.
    pub last: String,
}

/// Open views as tabs (key, label, brand mark).
pub const CRM_TABS: [(&str, &str, &str); 4] = [
    ("companies", "Company CRM", "hubspot"),
    ("deals", "Deals", "stripe"),
    ("contacts", "Contacts", "airtable"),
    ("renewals", "Renewals", "notion"),
];

/// The table's columns (key, label, glyph).
pub const CRM_COLS: [(&str, &str, &str); 7] = [
    ("name", "Company", "text"),
    ("cat", "Category", "tag"),
    ("domain", "Domain", "link"),
    ("owner", "Owner", "user"),
    ("value", "Value", "dollar"),
    ("stage", "Stage", "kanban"),
    ("last", "Last contact", "cal"),
];

/// Category tags and their tones.
pub const CRM_CATS: [(&str, &str); 7] = [
    ("Productivity", "grey"),
    ("Planning", "blue"),
    ("Design", "pink"),
    ("Payments", "purple"),
    ("Commerce", "amber"),
    ("Dev tools", "grey"),
    ("CRM", "blue"),
];

/// The tone of a category tag.
pub fn cat_tone(cat: &str) -> &'static str {
    CRM_CATS
        .iter()
        .find(|(c, _)| *c == cat)
        .map(|(_, t)| *t)
        .unwrap_or("grey")
}

/// The sample companies (ten).
pub fn companies(sample: bool) -> Vec<Company> {
    if !sample {
        return vec![];
    }
    use CrmStage::*;
    let v: [(&str, &str, &str, &str, u32, CrmStage, &str); 10] = [
        (
            "Notion",
            "Productivity",
            "notion.so",
            "Ines Hart",
            48_000,
            Negotiation,
            "Today",
        ),
        (
            "Linear",
            "Planning",
            "linear.app",
            "Tobi Lund",
            12_500,
            Proposal,
            "Yesterday",
        ),
        (
            "Figma",
            "Design",
            "figma.com",
            "Ines Hart",
            86_000,
            Won,
            "Mar 6",
        ),
        (
            "Stripe",
            "Payments",
            "stripe.com",
            "Mira Sol",
            124_000,
            Qualified,
            "Mar 5",
        ),
        (
            "Shopify",
            "Commerce",
            "shopify.com",
            "Tobi Lund",
            9_800,
            New,
            "Mar 4",
        ),
        (
            "Vercel",
            "Dev tools",
            "vercel.com",
            "Mira Sol",
            62_000,
            Proposal,
            "Mar 3",
        ),
        (
            "Airtable",
            "Productivity",
            "airtable.com",
            "Ines Hart",
            37_500,
            Qualified,
            "Mar 1",
        ),
        (
            "HubSpot",
            "CRM",
            "hubspot.com",
            "Ezra Pike",
            21_000,
            New,
            "Feb 27",
        ),
        (
            "ClickUp",
            "Planning",
            "clickup.com",
            "Ezra Pike",
            6_400,
            Lost,
            "Feb 25",
        ),
        (
            "Dropbox",
            "Productivity",
            "dropbox.com",
            "Mira Sol",
            18_900,
            Negotiation,
            "Feb 24",
        ),
    ];
    v.into_iter()
        .enumerate()
        .map(|(i, (n, c, d, o, val, s, l))| Company {
            id: i as u32 + 1,
            name: n.to_string(),
            cat: c.to_string(),
            domain: d.to_string(),
            owner: o.to_string(),
            value: val,
            stage: s,
            last: l.to_string(),
        })
        .collect()
}

/// The owners on the team.
pub const CRM_OWNERS: [&str; 4] = ["Ines Hart", "Tobi Lund", "Mira Sol", "Ezra Pike"];

/// The rows a stage filter, an owner filter, a search and a sort (column, descending) leave.
pub fn crm_view(
    list: &[Company],
    stage: &str,
    owner: &str,
    q: &str,
    sort: (&str, bool),
) -> Vec<Company> {
    let q = q.trim().to_lowercase();
    let mut v: Vec<Company> = list
        .iter()
        .filter(|c| stage.is_empty() || c.stage.key() == stage)
        .filter(|c| owner.is_empty() || c.owner == owner)
        .filter(|c| q.is_empty() || c.name.to_lowercase().contains(&q) || c.domain.contains(&q))
        .cloned()
        .collect();
    let (col, desc) = sort;
    match col {
        "name" => v.sort_by(|a, b| a.name.cmp(&b.name)),
        "value" => v.sort_by_key(|c| c.value),
        "owner" => v.sort_by(|a, b| a.owner.cmp(&b.owner)),
        "stage" => v.sort_by_key(|c| CrmStage::ALL.iter().position(|s| *s == c.stage)),
        _ => return v,
    }
    if desc {
        v.reverse();
    }
    v
}

/// Writes an edited cell; returns false (and leaves the row) when the value doesn't parse.
pub fn set_cell(list: &mut [Company], id: u32, col: &str, val: &str) -> bool {
    let Some(c) = list.iter_mut().find(|c| c.id == id) else {
        return false;
    };
    let v = val.trim();
    match col {
        "name" if !v.is_empty() => c.name = v.to_string(),
        "domain" if !v.is_empty() => c.domain = v.to_lowercase(),
        "owner" if !v.is_empty() => c.owner = v.to_string(),
        "cat" if !v.is_empty() => c.cat = v.to_string(),
        "value" => {
            let digits: String = v.chars().filter(|c| c.is_ascii_digit()).collect();
            match digits.parse::<u32>() {
                Ok(n) if !digits.is_empty() => c.value = n,
                _ => return false,
            }
        }
        "stage" => c.stage = CrmStage::from_key(&v.to_lowercase()),
        _ => return false,
    }
    true
}

/// Adds an empty row and returns its id.
pub fn add_company(list: &mut Vec<Company>) -> u32 {
    let id = list.iter().map(|c| c.id).max().unwrap_or(0) + 1;
    list.push(Company {
        id,
        name: "Untitled".to_string(),
        cat: "Productivity".to_string(),
        domain: String::new(),
        owner: CRM_OWNERS[0].to_string(),
        value: 0,
        stage: CrmStage::New,
        last: "Today".to_string(),
    });
    id
}

/// Pipeline totals per stage (stage, rows, value) for the filter strip.
pub fn stage_sums(list: &[Company]) -> Vec<(CrmStage, usize, u32)> {
    CrmStage::ALL
        .into_iter()
        .map(|s| {
            let r: Vec<&Company> = list.iter().filter(|c| c.stage == s).collect();
            (s, r.len(), r.iter().map(|c| c.value).sum())
        })
        .collect()
}

/// "48,000".
pub fn money(v: u32) -> String {
    thousands(v)
}

// ───────────────────────────────────────────────────────────── Analytics ──

/// Date ranges (key, label, days).
pub const RANGES: [(&str, &str, u32); 3] = [
    ("7d", "Last 7 days", 7),
    ("30d", "Last 30 days", 30),
    ("90d", "Last 90 days", 90),
];

/// A KPI tile.
#[derive(Clone, Debug, PartialEq)]
pub struct Kpi {
    /// Name.
    pub name: &'static str,
    /// Value in words.
    pub value: String,
    /// Change in words ("+12.5%").
    pub delta: String,
    /// The change is good news.
    pub good: bool,
    /// Seven bars, 0–100.
    pub bars: [u8; 7],
}

/// The range's multiplier against 7 days.
fn mult(range: &str) -> f64 {
    RANGES
        .iter()
        .find(|r| r.0 == range)
        .map(|r| r.2 as f64 / 7.0)
        .unwrap_or(1.0)
}

/// "$96,412".
pub fn dollars(v: f64) -> String {
    let neg = v < 0.0;
    let s = format!("${}", thousands(v.abs().round() as u32));
    if neg { format!("−{s}") } else { s }
}

/// "$96.4k".
pub fn dollars_k(v: f64) -> String {
    if v >= 1_000_000.0 {
        format!("${:.1}m", v / 1_000_000.0)
    } else if v >= 1000.0 {
        format!("${:.1}k", v / 1000.0)
    } else {
        format!("${v:.0}")
    }
}

/// The four KPI tiles for a range.
pub fn kpis(range: &str) -> Vec<Kpi> {
    let m = mult(range);
    vec![
        Kpi {
            name: "Gross revenue",
            value: dollars(96_412.0 * m),
            delta: "+12.5%".into(),
            good: true,
            bars: [30, 42, 38, 55, 61, 58, 100],
        },
        Kpi {
            name: "Active customers",
            value: thousands((2_847.0 * m.sqrt()) as u32),
            delta: "+8.2%".into(),
            good: true,
            bars: [40, 44, 47, 52, 55, 61, 100],
        },
        Kpi {
            name: "Conversion",
            value: "3.24%".into(),
            delta: "+0.4%".into(),
            good: true,
            bars: [52, 47, 60, 58, 66, 64, 100],
        },
        Kpi {
            name: "Refund rate",
            value: "1.8%".into(),
            delta: "−0.3%".into(),
            good: true,
            bars: [100, 92, 88, 80, 72, 66, 50],
        },
    ]
}

/// Revenue by channel: name, tone, seven daily points in dollars.
pub fn channels(range: &str) -> Vec<(&'static str, &'static str, [f64; 7])> {
    let m = mult(range);
    let s = |a: [f64; 7]| a.map(|x| x * m);
    vec![
        (
            "Web store",
            "blue",
            s([4400., 4800., 4100., 4600., 5300., 5100., 5700.]),
        ),
        (
            "Marketplace",
            "pink",
            s([3600., 3400., 4300., 4700., 3900., 4500., 4300.]),
        ),
        (
            "Subscriptions",
            "purple",
            s([2300., 2700., 2550., 2200., 2750., 2650., 2900.]),
        ),
        (
            "Wholesale",
            "teal",
            s([1500., 1450., 1800., 2000., 1750., 1600., 1700.]),
        ),
    ]
}

/// Sum of the last points of each channel series ("$96.4k" headline is the gross revenue).
pub fn channel_total(range: &str) -> f64 {
    96_412.0 * mult(range)
}

/// Revenue breakdown rows: name, dollars, trend (up, down, flat).
pub fn breakdown(range: &str) -> Vec<(&'static str, f64, &'static str)> {
    let m = mult(range);
    vec![
        ("Gross sales", 104_210.0 * m, "up"),
        ("Discounts", -4_180.0 * m, "down"),
        ("Refunds", -1_734.0 * m, "down"),
        ("Shipping", 2_940.0 * m, "up"),
        ("Taxes", 8_316.0 * m, "flat"),
        ("Net revenue", 96_412.0 * m, "up"),
    ]
}

/// Top products: name, dollars.
pub fn top_products(range: &str) -> Vec<(&'static str, f64)> {
    let m = mult(range);
    vec![
        ("Yearly plan", 38_400.0 * m),
        ("Team seats", 24_150.0 * m),
        ("Monthly plan", 18_920.0 * m),
        ("Add-ons", 9_640.0 * m),
    ]
}

/// A weekly trend: title, headline, change, this week, last week, axis top.
pub type Trend = (&'static str, String, String, [f64; 7], [f64; 7], f64);

/// Weekly trends for a range.
pub fn weekly(range: &str) -> Vec<Trend> {
    let m = mult(range);
    vec![
        (
            "Sessions",
            format!("{:.1}k", 16.4 * m),
            format!("+{:.1}k", 2.3 * m),
            [1700., 2050., 1900., 2400., 2650., 2300., 2950.],
            [1600., 1650., 1800., 1700., 1950., 1900., 2100.],
            3000.,
        ),
        (
            "Average order value",
            "$141".to_string(),
            "+$17".to_string(),
            [120., 124., 122., 130., 128., 134., 136.],
            [116., 118., 119., 118., 121., 122., 124.],
            160.,
        ),
    ]
}

/// Points for an SVG polyline: `w` × `h`, values scaled to `top`.
pub fn polyline(vals: &[f64], w: f64, h: f64, top: f64) -> String {
    let n = vals.len().max(2) - 1;
    vals.iter()
        .enumerate()
        .map(|(i, v)| {
            let x = w * i as f64 / n as f64;
            let y = h - (v / top).clamp(0.0, 1.0) * h;
            format!("{x:.1},{y:.1}")
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// A round axis top over every value.
pub fn axis_top(series: &[[f64; 7]]) -> f64 {
    let max = series
        .iter()
        .flat_map(|s| s.iter().copied())
        .fold(0.0f64, f64::max);
    let step = if max > 30_000.0 {
        10_000.0
    } else if max > 3000.0 {
        1000.0
    } else {
        100.0
    };
    ((max / step).ceil() * step).max(step)
}

/// Days along the axis for a range (first, middle, last).
pub fn axis_days(range: &str) -> [&'static str; 3] {
    match range {
        "30d" => ["Feb 9", "Feb 24", "Mar 9"],
        "90d" => ["Dec 10", "Jan 24", "Mar 9"],
        _ => ["Mar 3", "Mar 6", "Mar 9"],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn s2_workspace_parity() {
        assert_eq!(persona("board").nav.len(), 5);
        assert_eq!(persona("nope").key, "files");
        let t = folder_tiles(true);
        assert_eq!(tiles_view(&t, "shared", "", "name").len(), 2);
        assert_eq!(tiles_view(&t, "all", "hir", "recent")[0].name, "Hiring");
        assert_eq!(tile_line(&t[0]), "Shared · 3 files");
        assert_eq!(
            templates_view(&templates(true), "mine", None, "", false).len(),
            2
        );
        assert_eq!(
            templates_view(&templates(true), "all", Some(FileKind::Sheet), "", false).len(),
            2
        );
        assert_eq!(when_words(120), "2 hours ago");
        assert_eq!(when_words(1500), "Yesterday");
        assert_eq!(kind_for("plan.csv"), FileKind::Sheet);
        let i = integrations(true);
        assert_eq!(integ_view(&i, "on", "", "").len(), 1);
        assert_eq!(more_words(&i[6..8]), "Stripe and Zapier");
        let mut c = cards(true);
        assert_eq!(tab_counts(&c), (11, 9, 2));
        assert!(move_card(&mut c, 1, "done"));
        assert_eq!(c.last().unwrap().status, CardStatus::Fixed);
        assert_eq!(lane_step("low", -1), "low");
        assert_eq!(lane_step("med", 1), "high");
        let mut co = companies(true);
        assert!(set_cell(&mut co, 1, "value", "$50,500"));
        assert_eq!(co[0].value, 50_500);
        assert!(!set_cell(&mut co, 1, "value", "lots"));
        assert_eq!(crm_view(&co, "won", "", "", ("", false)).len(), 1);
        assert_eq!(crm_view(&co, "", "", "", ("value", true))[0].name, "Stripe");
        assert_eq!(dollars(-4180.0), "−$4,180");
        assert_eq!(dollars_k(96_412.0), "$96.4k");
        assert_eq!(kpis("7d")[0].value, "$96,412");
        assert!(polyline(&[0.0, 50.0], 100.0, 10.0, 100.0).starts_with("0.0,10.0"));
    }
}
