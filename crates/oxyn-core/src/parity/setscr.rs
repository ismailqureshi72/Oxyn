//! Screens S6: Settings I (Profile, Preferences, Notifications, Privacy, General) in the settings
//! shell: the settings sidebar (Back to app, search, Personal and Workspace pages, the account) in
//! the Screens 1 frame, and a centred 696 px column per page. Behaviour follows the affectUI
//! settings screens; the data, names and copy are fictional and written for Oxyn. Every rule (what
//! counts as a change, field checks, the password meter, quiet hours, the digest line, the
//! notification grid, sessions and shared links, the export, the delete check) is a pure function
//! so Dioxus and GPUI draw the same thing.

pub use super::screensfid::{PERSON, PERSON_EMAIL, PERSON_INI, ScreenState, WORKSPACE};

/// A state bar: (key, label).
pub type StateList = &'static [(&'static str, &'static str)];

/// Profile states.
pub const PROFILE_STATES: StateList = &[
    ("sample", "Data"),
    ("loading", "Loading"),
    ("empty", "Empty"),
    ("unsaved", "Unsaved"),
];
/// Preferences states.
pub const PREFS_STATES: StateList = &[
    ("sample", "Data"),
    ("loading", "Loading"),
    ("empty", "Empty"),
    ("memoff", "Memory off"),
];
/// Notifications states.
pub const NOTIF_STATES: StateList = &[
    ("sample", "Data"),
    ("loading", "Loading"),
    ("empty", "Empty"),
    ("quiet", "Quiet now"),
];
/// Privacy states.
pub const PRIVACY_STATES: StateList = &[
    ("sample", "Data"),
    ("loading", "Loading"),
    ("empty", "Empty"),
    ("exporting", "Exporting"),
];
/// General states.
pub const GENERAL_STATES: StateList = &[
    ("sample", "Data"),
    ("loading", "Loading"),
    ("empty", "Empty"),
    ("deleting", "Deleting"),
];

/// The states of a screen by its slug.
pub fn states_for(slug: &str) -> StateList {
    match slug {
        "settings-profile" => PROFILE_STATES,
        "settings-preferences" => PREFS_STATES,
        "settings-notifications" => NOTIF_STATES,
        "settings-privacy" => PRIVACY_STATES,
        _ => GENERAL_STATES,
    }
}

/// Split a state key into the data state and the screen's own state ("" for none).
pub fn split(k: &str) -> (ScreenState, &'static str) {
    const OWN: [&str; 5] = ["unsaved", "memoff", "quiet", "exporting", "deleting"];
    match OWN.iter().find(|o| **o == k) {
        Some(o) => (ScreenState::Sample, o),
        None => (ScreenState::from_key(k), ""),
    }
}

/// A page of the settings nav: key, label, group (`personal` / `workspace`), glyph, and whether it
/// is one of the S6 pages (the others draw their rows from the shell's model).
pub type NavPage = (&'static str, &'static str, &'static str, &'static str, bool);

/// The nav, in order.
pub const NAV: [NavPage; 9] = [
    ("profile", "Profile", "personal", "user", true),
    ("preferences", "Preferences", "personal", "gear", true),
    ("notifications", "Notifications", "personal", "bell", true),
    ("privacy", "Privacy", "personal", "shield", true),
    ("general", "General", "workspace", "building", true),
    ("members", "Members", "workspace", "people", false),
    ("models", "Models", "workspace", "spark", false),
    ("security", "Security", "workspace", "lock", false),
    ("billing", "Billing", "workspace", "card", false),
];

/// The two nav groups: key, label.
pub const GROUPS: [(&str, &str); 2] = [("personal", "Personal"), ("workspace", "Workspace")];

/// The page key of a slug (`settings-privacy` → `privacy`).
pub fn page_of(slug: &str) -> &'static str {
    NAV.iter()
        .find(|p| slug.strip_prefix("settings-") == Some(p.0))
        .map(|p| p.0)
        .unwrap_or("profile")
}

/// The one-line lead under a page heading.
pub fn lead(page: &str) -> &'static str {
    match page {
        "profile" => "How you appear to the people and agents you work with.",
        "preferences" => {
            "How the assistant answers you: models, tools, look and what it remembers."
        }
        "notifications" => "What reaches you, where, and when it stays quiet.",
        "privacy" => "What is kept, what is shared, and where you are signed in.",
        "general" => "The workspace everyone here shares: its name, address and context.",
        "members" => "Who is in the workspace and what they can do.",
        "models" => "Which models the workspace can use.",
        "security" => "How people sign in.",
        _ => "Your plan and invoices.",
    }
}

/// A search over the S6 pages: (page, label) of every setting whose label or words match `q`.
pub fn find(q: &str) -> Vec<(&'static str, &'static str)> {
    let q = q.trim().to_lowercase();
    if q.is_empty() {
        return Vec::new();
    }
    SEARCH
        .iter()
        .filter(|(_, l, w)| l.to_lowercase().contains(&q) || w.contains(q.as_str()))
        .map(|(p, l, _)| (*p, *l))
        .collect()
}

/// Searchable settings: page, label, extra words.
const SEARCH: [(&str, &str, &str); 30] = [
    ("profile", "Picture", "photo avatar image"),
    ("profile", "Full name", "name"),
    ("profile", "Role", "job title"),
    ("profile", "Email", "address mail"),
    ("profile", "Time zone", "timezone clock"),
    ("profile", "Time format", "24 12 hour clock"),
    ("profile", "First day of the week", "monday sunday calendar"),
    ("profile", "Personal context", "about me bio"),
    ("profile", "Password", "sign in login"),
    ("preferences", "Default model", "auto ai llm"),
    ("preferences", "Language", "locale english"),
    ("preferences", "Appearance", "theme dark light mode"),
    ("preferences", "Chat tools", "web search image code canvas"),
    ("preferences", "Custom instructions", "prompt style"),
    ("preferences", "Memory", "remember forget clear"),
    ("notifications", "Daily digest", "summary email morning"),
    ("notifications", "Approvals", "needs you"),
    ("notifications", "Finished runs", "done complete"),
    ("notifications", "Failed runs", "error"),
    ("notifications", "Mentions", "comment"),
    ("notifications", "Quiet hours", "do not disturb night mute"),
    ("privacy", "Improve the models", "training data"),
    ("privacy", "Product analytics", "usage tracking"),
    ("privacy", "Cookies", "consent"),
    ("privacy", "Active sessions", "devices sign out"),
    ("privacy", "Shared links", "public share"),
    ("privacy", "Export your data", "download"),
    ("general", "Workspace name", "company"),
    ("general", "Workspace address", "url slug domain"),
    ("general", "Delete workspace", "danger remove"),
];

// ───────────────────────────────────────────────────────────── Model ──

/// Roles a person picks on Profile.
pub const ROLES: [&str; 6] = [
    "Founder",
    "Design",
    "Engineering",
    "Marketing",
    "Operations",
    "Other",
];
/// Time zones (label).
pub const ZONES: [&str; 5] = [
    "Lisbon (GMT+0)",
    "Berlin (GMT+1)",
    "Karachi (GMT+5)",
    "Singapore (GMT+8)",
    "New York (GMT−5)",
];
/// First day of the week.
pub const WEEK_START: [&str; 2] = ["Monday", "Sunday"];
/// The default model choices: label and the brand id of the official mark ("" for Auto).
pub const DEFAULT_MODELS: [(&str, &str, &str); 4] = [
    ("Auto", "", "Picks a fast or a deep model per message"),
    ("Opus 5.5", "claude", "Careful, long tasks"),
    ("GPT-6 mini", "openai", "Quick everyday answers"),
    ("Gemini 3.8 Pro", "gemini", "Long files and images"),
];
/// Languages.
pub const LANGS: [&str; 5] = [
    "English (UK)",
    "English (US)",
    "Deutsch",
    "Español",
    "Français",
];
/// Appearance.
pub const THEMES: [&str; 3] = ["System", "Light", "Dark"];
/// Chat tools: key, label, line, glyph.
pub const TOOLS: [(&str, &str, &str, &str); 4] = [
    (
        "web",
        "Web search",
        "Looks things up and cites them",
        "globe",
    ),
    (
        "image",
        "Image generation",
        "Draws from a description",
        "image",
    ),
    (
        "code",
        "Code runner",
        "Runs code to check its answers",
        "term",
    ),
    ("canvas", "Canvas", "Opens long drafts side by side", "doc"),
];
/// Answer styles.
pub const STYLES: [&str; 3] = ["Balanced", "Short", "Detailed"];
/// Digest times.
pub const DIGEST_AT: [&str; 4] = ["07:00", "08:00", "09:00", "18:00"];
/// Day letters, Monday first.
pub const DAYS: [&str; 7] = ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"];
/// Notification channels.
pub const CHANNELS: [&str; 3] = ["Email", "Mobile", "Inbox"];
/// Notification kinds: label, line, glyph, tone.
pub const KINDS: [(&str, &str, &str, &str); 6] = [
    ("Approvals", "An agent waits for your yes", "hand", "warn"),
    ("Finished runs", "A run you started is done", "check", "ok"),
    ("Failed runs", "A run stopped on an error", "bug", "bad"),
    ("Mentions", "Someone names you in a chat", "chat", "run"),
    ("Weekly usage", "Spend and runs, every Monday", "chart", ""),
    ("Product news", "New features, now and then", "mega", ""),
];
/// Quiet-hours times (whole hours).
pub const HOURS: [&str; 8] = [
    "18:00", "20:00", "21:00", "22:00", "23:00", "06:00", "07:00", "08:00",
];
/// Cookie choices.
pub const COOKIES: [&str; 2] = ["Essential only", "All cookies"];
/// Workspace icon hues (identity hues).
pub const WS_HUES: [&str; 6] = ["blue", "purple", "teal", "orange", "pink", "green"];
/// Default sharing for new chats.
pub const SHARING: [&str; 3] = ["Only me", "Members", "Anyone with the link"];

/// A signed-in session.
#[derive(Clone, Debug, PartialEq)]
pub struct Session {
    /// Stable id.
    pub id: u32,
    /// Device.
    pub device: &'static str,
    /// Glyph (`laptop`, `phone`, `term`).
    pub glyph: &'static str,
    /// App or browser.
    pub app: &'static str,
    /// Place.
    pub place: &'static str,
    /// Last seen.
    pub seen: &'static str,
    /// This device.
    pub current: bool,
}

/// A shared chat link.
#[derive(Clone, Debug, PartialEq)]
pub struct Link {
    /// Stable id.
    pub id: u32,
    /// Chat title.
    pub title: &'static str,
    /// `workspace` or `public`.
    pub scope: &'static str,
    /// Views.
    pub views: u32,
    /// Shared when.
    pub when: &'static str,
}

/// Everything the five pages hold.
#[derive(Clone, Debug, PartialEq)]
pub struct Prefs {
    /// Has a picture (else initials).
    pub picture: bool,
    /// Full name.
    pub name: String,
    /// Role index in [`ROLES`].
    pub role: usize,
    /// Email.
    pub email: String,
    /// Time zone index.
    pub zone: usize,
    /// 24-hour clock.
    pub clock24: bool,
    /// First day of the week index.
    pub week: usize,
    /// Personal context.
    pub context: String,
    /// A password is set (else sign-in is by Google only).
    pub password: bool,
    /// Default model index.
    pub model: usize,
    /// Language index.
    pub lang: usize,
    /// Appearance index.
    pub theme: usize,
    /// Chat tools on.
    pub tools: [bool; 4],
    /// Custom instructions.
    pub instructions: String,
    /// Answer style index.
    pub style: usize,
    /// Memory on.
    pub memory: bool,
    /// Kept memories.
    pub memories: Vec<&'static str>,
    /// Daily digest on.
    pub digest: bool,
    /// Digest time index.
    pub digest_at: usize,
    /// Digest days.
    pub digest_days: [bool; 7],
    /// Kinds × channels.
    pub grid: [[bool; 3]; 6],
    /// Quiet hours on.
    pub quiet: bool,
    /// Quiet from (index in [`HOURS`]).
    pub quiet_from: usize,
    /// Quiet to.
    pub quiet_to: usize,
    /// Approvals still come through quiet hours.
    pub quiet_urgent: bool,
    /// Chats may improve the models.
    pub train: bool,
    /// Product analytics.
    pub analytics: bool,
    /// Ad measurement.
    pub ads: bool,
    /// Cookies index.
    pub cookies: usize,
    /// Product emails.
    pub product_mail: bool,
    /// Sessions.
    pub sessions: Vec<Session>,
    /// Shared links.
    pub links: Vec<Link>,
    /// Workspace name.
    pub ws_name: String,
    /// Workspace address (the part before `.oxyn.app`).
    pub ws_slug: String,
    /// Workspace icon hue index.
    pub ws_hue: usize,
    /// Company context.
    pub company: String,
    /// Default sharing index.
    pub sharing: usize,
}

/// The sample memories.
pub const MEMORIES: [&str; 5] = [
    "Writes in British English",
    "Leads design at Kestrel Studio",
    "Prefers tables over long lists",
    "Weekly review is on Friday afternoons",
    "Keeps launch budgets in the shared sheet",
];

/// The sample sessions.
pub fn sessions() -> Vec<Session> {
    vec![
        Session {
            id: 1,
            device: "MacBook Pro",
            glyph: "laptop",
            app: "Browser",
            place: "Lisbon",
            seen: "Now",
            current: true,
        },
        Session {
            id: 2,
            device: "Phone",
            glyph: "phone",
            app: "Mobile app",
            place: "Lisbon",
            seen: "2 hours ago",
            current: false,
        },
        Session {
            id: 3,
            device: "Studio desktop",
            glyph: "laptop",
            app: "Desktop app",
            place: "Porto",
            seen: "Yesterday",
            current: false,
        },
        Session {
            id: 4,
            device: "Build server",
            glyph: "term",
            app: "Command line",
            place: "Frankfurt",
            seen: "5 days ago",
            current: false,
        },
    ]
}

/// The sample shared links.
pub fn links() -> Vec<Link> {
    vec![
        Link {
            id: 1,
            title: "Autumn launch plan",
            scope: "workspace",
            views: 14,
            when: "3 days ago",
        },
        Link {
            id: 2,
            title: "Field Overshirt ad angles",
            scope: "workspace",
            views: 6,
            when: "Last week",
        },
        Link {
            id: 3,
            title: "Fabric guide for customers",
            scope: "public",
            views: 212,
            when: "2 weeks ago",
        },
    ]
}

/// Everything with the sample filled in, or a new account (`sample` false): blank text, the
/// defaults, no memories, one session (this one) and no links.
pub fn prefs(sample: bool) -> Prefs {
    let s = |v: &str| if sample { v.to_string() } else { String::new() };
    Prefs {
        picture: sample,
        name: if sample {
            PERSON.to_string()
        } else {
            String::new()
        },
        role: if sample { 1 } else { 5 },
        email: PERSON_EMAIL.to_string(),
        zone: 0,
        clock24: true,
        week: 0,
        context: s(
            "I lead design at a small outdoor-clothing studio. Short answers, tables for comparisons, British spelling.",
        ),
        password: sample,
        model: 0,
        lang: 0,
        theme: 0,
        tools: [true, sample, sample, false],
        instructions: s(
            "Ask before long research. Name your sources. Keep product copy under 60 words.",
        ),
        style: 0,
        memory: true,
        memories: if sample {
            MEMORIES.to_vec()
        } else {
            Vec::new()
        },
        digest: sample,
        digest_at: 1,
        digest_days: [true, true, true, true, true, false, false],
        grid: if sample {
            [
                [true, true, true],
                [false, false, true],
                [true, true, true],
                [false, true, true],
                [true, false, false],
                [false, false, false],
            ]
        } else {
            [
                [true, false, true],
                [false, false, true],
                [true, false, true],
                [false, false, true],
                [false, false, false],
                [false, false, false],
            ]
        },
        quiet: sample,
        quiet_from: 3,
        quiet_to: 6,
        quiet_urgent: true,
        train: false,
        analytics: true,
        ads: false,
        cookies: if sample { 1 } else { 0 },
        product_mail: sample,
        sessions: if sample {
            sessions()
        } else {
            sessions().into_iter().take(1).collect()
        },
        links: if sample { links() } else { Vec::new() },
        ws_name: if sample {
            WORKSPACE.to_string()
        } else {
            String::new()
        },
        ws_slug: s("kestrel-studio"),
        ws_hue: 0,
        company: s(
            "Kestrel Studio designs and sells outdoor clothing online. Customers are commuters who cycle; launches run each season.",
        ),
        sharing: 1,
    }
}

/// How many settings differ between two copies (each field, each memory, each grid cell and day
/// counts once). Sessions and links act at once and are not counted.
pub fn changes(a: &Prefs, b: &Prefs) -> usize {
    let mut n = 0;
    let mut c = |x: bool| n += usize::from(x);
    c(a.picture != b.picture);
    c(a.name != b.name);
    c(a.role != b.role);
    c(a.email != b.email);
    c(a.zone != b.zone);
    c(a.clock24 != b.clock24);
    c(a.week != b.week);
    c(a.context != b.context);
    c(a.password != b.password);
    c(a.model != b.model);
    c(a.lang != b.lang);
    c(a.theme != b.theme);
    for i in 0..4 {
        c(a.tools[i] != b.tools[i]);
    }
    c(a.instructions != b.instructions);
    c(a.style != b.style);
    c(a.memory != b.memory);
    c(a.memories != b.memories);
    c(a.digest != b.digest);
    c(a.digest_at != b.digest_at);
    for i in 0..7 {
        c(a.digest_days[i] != b.digest_days[i]);
    }
    for r in 0..6 {
        for k in 0..3 {
            c(a.grid[r][k] != b.grid[r][k]);
        }
    }
    c(a.quiet != b.quiet);
    c(a.quiet_from != b.quiet_from);
    c(a.quiet_to != b.quiet_to);
    c(a.quiet_urgent != b.quiet_urgent);
    c(a.train != b.train);
    c(a.analytics != b.analytics);
    c(a.ads != b.ads);
    c(a.cookies != b.cookies);
    c(a.product_mail != b.product_mail);
    c(a.ws_name != b.ws_name);
    c(a.ws_slug != b.ws_slug);
    c(a.ws_hue != b.ws_hue);
    c(a.company != b.company);
    c(a.sharing != b.sharing);
    n
}

/// "1 unsaved change" / "3 unsaved changes".
pub fn changes_words(n: usize) -> String {
    if n == 1 {
        "1 unsaved change".to_string()
    } else {
        format!("{n} unsaved changes")
    }
}

/// Caps of the long fields.
pub const CONTEXT_CAP: usize = 600;
/// Custom instructions cap.
pub const INSTR_CAP: usize = 1500;
/// Company context cap.
pub const COMPANY_CAP: usize = 1000;

/// What is wrong with a name, if anything.
pub fn name_error(v: &str) -> Option<&'static str> {
    let t = v.trim();
    if t.is_empty() {
        Some("Add your name")
    } else if t.chars().count() > 60 {
        Some("Keep it under 60 characters")
    } else {
        None
    }
}

/// What is wrong with an email, if anything.
pub fn email_error(v: &str) -> Option<&'static str> {
    let t = v.trim();
    let ok = t.split_once('@').is_some_and(|(a, d)| {
        !a.is_empty() && d.contains('.') && !d.starts_with('.') && !d.ends_with('.')
    }) && !t.contains(' ');
    if t.is_empty() {
        Some("Add an email")
    } else if ok {
        None
    } else {
        Some("That does not look like an email")
    }
}

/// A workspace address made from a name: lower case, letters and digits, single hyphens.
pub fn slug_from(name: &str) -> String {
    let mut s = String::new();
    for ch in name.trim().to_lowercase().chars() {
        if ch.is_ascii_alphanumeric() {
            s.push(ch);
        } else if !s.is_empty() && !s.ends_with('-') {
            s.push('-');
        }
    }
    s.trim_end_matches('-').chars().take(32).collect()
}

/// Addresses taken by other workspaces (sample).
pub const TAKEN: [&str; 3] = ["kestrel", "studio", "northwind"];

/// What is wrong with a workspace address, if anything.
pub fn slug_error(v: &str) -> Option<&'static str> {
    let ok_chars = v
        .chars()
        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-');
    if v.is_empty() {
        Some("Add an address")
    } else if !ok_chars {
        Some("Use lower-case letters, digits and hyphens")
    } else if v.starts_with('-') || v.ends_with('-') {
        Some("It cannot start or end with a hyphen")
    } else if v.chars().count() < 3 {
        Some("Use at least 3 characters")
    } else if TAKEN.contains(&v) {
        Some("Another workspace has this address")
    } else {
        None
    }
}

/// What is wrong with the workspace name, if anything.
pub fn ws_name_error(v: &str) -> Option<&'static str> {
    let t = v.trim();
    if t.is_empty() {
        Some("Name the workspace")
    } else if t.chars().count() > 48 {
        Some("Keep it under 48 characters")
    } else {
        None
    }
}

/// Any field on a page that blocks saving.
pub fn page_errors(p: &Prefs, page: &str) -> usize {
    let n = |o: Option<&'static str>| usize::from(o.is_some());
    match page {
        "profile" => {
            n(name_error(&p.name))
                + n(email_error(&p.email))
                + usize::from(p.context.chars().count() > CONTEXT_CAP)
        }
        "preferences" => usize::from(p.instructions.chars().count() > INSTR_CAP),
        "general" => {
            n(ws_name_error(&p.ws_name))
                + n(slug_error(&p.ws_slug))
                + usize::from(p.company.chars().count() > COMPANY_CAP)
        }
        _ => 0,
    }
}

/// Password strength: score 0–4 and its word. Length, mixed case, digits and symbols each add.
pub fn strength(pw: &str) -> (u8, &'static str) {
    let n = pw.chars().count();
    if n == 0 {
        return (0, "");
    }
    let mut s = 0u8;
    if n >= 8 {
        s += 1;
    }
    if n >= 12 {
        s += 1;
    }
    if pw.chars().any(|c| c.is_lowercase()) && pw.chars().any(|c| c.is_uppercase()) {
        s += 1;
    }
    if pw.chars().any(|c| c.is_ascii_digit()) && pw.chars().any(|c| !c.is_alphanumeric()) {
        s += 1;
    }
    let s = if n < 8 { s.min(1) } else { s };
    (
        s,
        match s {
            0 | 1 => "Too weak",
            2 => "Fair",
            3 => "Good",
            _ => "Strong",
        },
    )
}

/// Whether a new password may be saved: at least "Fair" and both copies match.
pub fn password_ok(pw: &str, again: &str) -> bool {
    strength(pw).0 >= 2 && pw == again
}

/// Hours in a quiet window (it may cross midnight).
pub fn quiet_len(from: usize, to: usize) -> u32 {
    let h = |i: usize| -> u32 { HOURS.get(i).and_then(|s| s[..2].parse().ok()).unwrap_or(0) };
    (h(to) + 24 - h(from)) % 24
}

/// "Quiet from 22:00 to 07:00 – 9 hours, approvals still come through."
pub fn quiet_words(p: &Prefs) -> String {
    if !p.quiet {
        return "Off: notifications arrive at any hour.".to_string();
    }
    let n = quiet_len(p.quiet_from, p.quiet_to);
    let from = HOURS.get(p.quiet_from).copied().unwrap_or("");
    let to = HOURS.get(p.quiet_to).copied().unwrap_or("");
    let tail = if p.quiet_urgent {
        ", approvals still come through"
    } else {
        ", nothing comes through"
    };
    if n == 0 {
        return "Pick an end time after the start.".to_string();
    }
    format!(
        "Quiet from {from} to {to} – {n} hour{}{tail}.",
        if n == 1 { "" } else { "s" }
    )
}

/// "Weekdays at 08:00" / "Every day at 07:00" / "Mon, Wed at 09:00" / "Off".
pub fn digest_words(p: &Prefs) -> String {
    if !p.digest {
        return "Off".to_string();
    }
    let at = DIGEST_AT.get(p.digest_at).copied().unwrap_or("");
    let d = p.digest_days;
    let n = d.iter().filter(|x| **x).count();
    let days = if n == 7 {
        "Every day".to_string()
    } else if d == [true, true, true, true, true, false, false] {
        "Weekdays".to_string()
    } else if d == [false, false, false, false, false, true, true] {
        "Weekends".to_string()
    } else if n == 0 {
        return "No days picked".to_string();
    } else {
        DAYS.iter()
            .zip(d)
            .filter(|(_, on)| *on)
            .map(|(l, _)| *l)
            .collect::<Vec<_>>()
            .join(", ")
    };
    format!("{days} at {at}")
}

/// A grid column is all on.
pub fn col_all(g: &[[bool; 3]; 6], c: usize) -> bool {
    g.iter().all(|r| r[c])
}

/// Turn a whole grid column on, or off when it is all on already.
pub fn toggle_col(g: &mut [[bool; 3]; 6], c: usize) {
    let v = !col_all(g, c);
    for r in g.iter_mut() {
        r[c] = v;
    }
}

/// The channels a kind reaches, as words ("Email and Inbox", "Nowhere").
pub fn reach_words(row: &[bool; 3]) -> String {
    let on: Vec<&str> = CHANNELS
        .iter()
        .zip(row)
        .filter(|(_, b)| **b)
        .map(|(l, _)| *l)
        .collect();
    match on.len() {
        0 => "Nowhere".to_string(),
        1 => on[0].to_string(),
        2 => format!("{} and {}", on[0], on[1]),
        _ => "Everywhere".to_string(),
    }
}

/// Sign every other session out; returns how many.
pub fn sign_out_others(s: &mut Vec<Session>) -> usize {
    let n = s.iter().filter(|x| !x.current).count();
    s.retain(|x| x.current);
    n
}

/// Links in a scope.
pub fn links_in<'a>(l: &'a [Link], scope: &str) -> Vec<&'a Link> {
    l.iter().filter(|x| x.scope == scope).collect()
}

/// The fictional public URL of a link.
pub fn link_url(l: &Link) -> String {
    format!("https://share.oxyn.app/c/{}{}", slug_from(l.title), l.id)
}

/// Export progress: (percent, step label, done). Five steps of 20 %.
pub fn export_step(pct: u32) -> (u32, &'static str, bool) {
    let p = pct.min(100);
    let l = match p {
        0..=19 => "Collecting chats",
        20..=39 => "Collecting files",
        40..=59 => "Collecting memories",
        60..=79 => "Packing the archive",
        80..=99 => "Writing the link",
        _ => "Ready: the link is on its way to your email",
    };
    (p, l, p >= 100)
}

/// A destructive action is confirmed when the typed text equals the expected one (trimmed, case
/// kept).
pub fn confirm_ok(typed: &str, expect: &str) -> bool {
    !expect.is_empty() && typed.trim() == expect
}

/// Initials of a name ("Rowan Vale" → "RV"; "" → "?").
pub fn initials(name: &str) -> String {
    let s: String = name
        .split_whitespace()
        .filter_map(|w| w.chars().next())
        .take(2)
        .flat_map(|c| c.to_uppercase())
        .collect();
    if s.is_empty() { "?".to_string() } else { s }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn s6_settings() {
        assert_eq!(split("quiet"), (ScreenState::Sample, "quiet"));
        assert_eq!(split("loading"), (ScreenState::Loading, ""));
        assert_eq!(states_for("settings-privacy")[3].0, "exporting");
        assert_eq!(page_of("settings-general"), "general");
        assert_eq!(page_of("nope"), "profile");
        assert_eq!(NAV.iter().filter(|p| p.4).count(), 5);
        assert!(find("dark").iter().any(|h| h.0 == "preferences"));
        assert!(find("  ").is_empty());
        // Changes.
        let a = prefs(true);
        let mut b = a.clone();
        assert_eq!(changes(&a, &b), 0);
        b.name = "Rowan V.".into();
        b.grid[0][0] = false;
        b.digest_days[6] = true;
        assert_eq!(changes(&a, &b), 3);
        b.sessions.clear();
        assert_eq!(changes(&a, &b), 3);
        assert_eq!(changes_words(1), "1 unsaved change");
        // Checks.
        assert_eq!(name_error("  "), Some("Add your name"));
        assert_eq!(email_error("rowan@kestrel.example"), None);
        assert!(email_error("rowan@kestrel").is_some());
        assert!(email_error("ro wan@a.b").is_some());
        assert_eq!(slug_from("  Kestrel Studio!! "), "kestrel-studio");
        assert_eq!(
            slug_error("kestrel"),
            Some("Another workspace has this address")
        );
        assert_eq!(slug_error("ab"), Some("Use at least 3 characters"));
        assert!(slug_error("Kes").is_some());
        assert_eq!(slug_error("kestrel-2"), None);
        assert_eq!(page_errors(&prefs(true), "profile"), 0);
        assert_eq!(page_errors(&prefs(false), "profile"), 1);
        assert_eq!(page_errors(&prefs(false), "general"), 2);
        // Password.
        assert_eq!(strength(""), (0, ""));
        assert_eq!(strength("abc").1, "Too weak");
        assert_eq!(strength("abcdefgh").0, 1);
        assert_eq!(strength("Abcdefghijkl").1, "Good");
        assert_eq!(strength("Abcdefghij1!").1, "Strong");
        assert!(password_ok("Abcdefgh1", "Abcdefgh1"));
        assert!(!password_ok("Abcdefgh1", "Abcdefgh2"));
        // Quiet hours and digest.
        assert_eq!(quiet_len(3, 6), 9);
        assert_eq!(quiet_len(6, 7), 1);
        let mut p = prefs(true);
        assert_eq!(
            quiet_words(&p),
            "Quiet from 22:00 to 07:00 – 9 hours, approvals still come through."
        );
        assert_eq!(digest_words(&p), "Weekdays at 08:00");
        p.digest_days = [true; 7];
        assert_eq!(digest_words(&p), "Every day at 08:00");
        p.digest_days = [true, false, true, false, false, false, false];
        assert_eq!(digest_words(&p), "Mon, Wed at 08:00");
        p.digest = false;
        assert_eq!(digest_words(&p), "Off");
        // Grid.
        let mut g = prefs(true).grid;
        assert!(!col_all(&g, 0));
        toggle_col(&mut g, 2);
        assert!(col_all(&g, 2));
        toggle_col(&mut g, 2);
        assert!(g.iter().all(|r| !r[2]));
        assert_eq!(reach_words(&[true, false, true]), "Email and Inbox");
        assert_eq!(reach_words(&[false; 3]), "Nowhere");
        // Sessions, links, export, delete.
        let mut s = sessions();
        assert_eq!(sign_out_others(&mut s), 3);
        assert_eq!(s.len(), 1);
        assert_eq!(links_in(&links(), "public").len(), 1);
        assert_eq!(
            link_url(&links()[2]),
            "https://share.oxyn.app/c/fabric-guide-for-customers3"
        );
        assert_eq!(export_step(45).1, "Collecting memories");
        assert!(export_step(130).2);
        assert!(confirm_ok(" Kestrel Studio ", WORKSPACE));
        assert!(!confirm_ok("kestrel studio", WORKSPACE));
        assert_eq!(initials("Rowan Vale"), "RV");
        assert_eq!(initials(""), "?");
    }
}
