//! Screens S7: Settings II (Members, Models, Security, Billing and the Get started checklist) in the
//! S6 settings shell ([`super::setscr`]). Behaviour follows the affectUI settings screens; the data,
//! names and copy are fictional and written for Oxyn. Workspace policies (model switches and access,
//! the defaults, sign-in rules, billing interval and limits, the checklist's visibility) are fields
//! of [`Ws`] and count as unsaved changes; lists (members, keys, passkeys, domains, sessions) act at
//! once. Every rule is a pure function so Dioxus and GPUI draw the same thing.

use super::screensfid::{PERSON, PERSON_EMAIL};

/// A state bar: (key, label).
pub type StateList = &'static [(&'static str, &'static str)];

/// Members states.
pub const MEMBERS_STATES: StateList = &[
    ("sample", "Data"),
    ("loading", "Loading"),
    ("empty", "Empty"),
    ("inviting", "Inviting"),
];
/// Models states.
pub const MODELS_STATES: StateList = &[
    ("sample", "Data"),
    ("loading", "Loading"),
    ("empty", "Empty"),
    ("newkey", "New key"),
];
/// Security states.
pub const SECURITY_STATES: StateList = &[
    ("sample", "Data"),
    ("loading", "Loading"),
    ("empty", "Empty"),
    ("twostep", "Setting up"),
];
/// Billing states.
pub const BILLING_STATES: StateList = &[
    ("sample", "Data"),
    ("loading", "Loading"),
    ("empty", "Empty"),
    ("cancelling", "Cancelling"),
];
/// Get started states.
pub const START_STATES: StateList = &[
    ("sample", "Data"),
    ("loading", "Loading"),
    ("empty", "Empty"),
    ("complete", "Complete"),
];

/// The screens' own state keys.
pub const OWN: [&str; 5] = ["inviting", "newkey", "twostep", "cancelling", "complete"];

/// The states of an S7 screen by its slug, if it is one.
pub fn states_for(slug: &str) -> Option<StateList> {
    Some(match slug {
        "settings-members" => MEMBERS_STATES,
        "settings-models" => MODELS_STATES,
        "settings-security" => SECURITY_STATES,
        "settings-billing" => BILLING_STATES,
        "settings-onboarding" => START_STATES,
        _ => return None,
    })
}

/// Searchable S7 settings: page, label, extra words.
pub const SEARCH: [(&str, &str, &str); 22] = [
    ("members", "Invite members", "add people email seat"),
    ("members", "Member roles", "owner admin viewer permission"),
    ("members", "Teams", "groups"),
    ("members", "Seats", "licence license"),
    ("models", "Chat model", "default auto"),
    ("models", "Image model", "picture"),
    (
        "models",
        "Providers",
        "anthropic openai google mistral deepseek",
    ),
    ("models", "API keys", "token secret provider key rotate"),
    ("security", "Two-step sign-in", "2fa mfa authenticator totp"),
    ("security", "Recovery codes", "backup 2fa"),
    ("security", "Passkeys", "touch id face fingerprint webauthn"),
    ("security", "Single sign-on", "sso saml okta"),
    ("security", "Email domains", "domain verify join"),
    (
        "security",
        "Signed-in devices",
        "sessions sign out everyone",
    ),
    ("security", "Audit log", "history events activity"),
    ("billing", "Plan", "subscription upgrade pro team"),
    ("billing", "Usage", "credits storage runs limits"),
    ("billing", "Payment method", "card"),
    ("billing", "Invoices", "receipts download"),
    ("billing", "Cancel plan", "downgrade end subscription"),
    ("onboarding", "Get started", "checklist setup onboarding"),
    ("onboarding", "Hide the checklist", "dismiss"),
];

// ──────────────────────────────────────────────────────────── Members ──

/// Workspace roles: label and what they can do.
pub const ROLES: [(&str, &str); 4] = [
    (
        "Owner",
        "Everything, including billing and deleting the workspace",
    ),
    (
        "Admin",
        "Invite and remove people, manage models and security",
    ),
    ("Member", "Chat, run agents and share with the workspace"),
    ("Viewer", "Read shared chats and runs"),
];
/// Seats on the plan.
pub const SEATS: usize = 10;
/// Teams: name and identity hue.
pub const TEAMS: [(&str, &str); 3] = [
    ("Design", "purple"),
    ("Engineering", "blue"),
    ("Growth", "teal"),
];

/// A person in the workspace (or invited to it).
#[derive(Clone, Debug, PartialEq)]
pub struct Member {
    /// Stable id.
    pub id: u32,
    /// Name ("" while invited).
    pub name: &'static str,
    /// Email.
    pub email: String,
    /// Index in [`ROLES`].
    pub role: usize,
    /// Index in [`TEAMS`], if any.
    pub team: Option<usize>,
    /// Usage this month, percent of their share.
    pub usage: u8,
    /// Last active.
    pub seen: &'static str,
    /// Invited, not joined yet.
    pub invited: bool,
}

impl Member {
    /// The name, or the email while invited.
    pub fn label(&self) -> &str {
        if self.name.is_empty() {
            &self.email
        } else {
            self.name
        }
    }
    /// This is the signed-in person.
    pub fn is_me(&self) -> bool {
        self.email == PERSON_EMAIL
    }
}

fn m(
    id: u32,
    name: &'static str,
    email: &str,
    role: usize,
    team: Option<usize>,
    usage: u8,
    seen: &'static str,
) -> Member {
    Member {
        id,
        name,
        email: email.to_string(),
        role,
        team,
        usage,
        seen,
        invited: name.is_empty(),
    }
}

/// The sample members, or just you (`sample` false).
pub fn members(sample: bool) -> Vec<Member> {
    let me = m(1, PERSON, PERSON_EMAIL, 0, Some(0), 68, "Now");
    if !sample {
        return vec![Member { usage: 0, ..me }];
    }
    vec![
        me,
        m(
            2,
            "Ines Duarte",
            "ines@kestrel.example",
            1,
            Some(1),
            42,
            "1 hour ago",
        ),
        m(
            3,
            "Tomas Reyna",
            "tomas@kestrel.example",
            2,
            Some(1),
            91,
            "Today",
        ),
        m(
            4,
            "Mika Sato",
            "mika@kestrel.example",
            2,
            Some(0),
            23,
            "Yesterday",
        ),
        m(
            5,
            "Odile Brandt",
            "odile@kestrel.example",
            2,
            Some(2),
            57,
            "3 days ago",
        ),
        m(
            6,
            "Jules Arden",
            "jules@kestrel.example",
            3,
            None,
            5,
            "Last week",
        ),
        m(
            7,
            "",
            "noor@kestrel.example",
            2,
            Some(2),
            0,
            "Invited 2 days ago",
        ),
    ]
}

/// Members whose name or email contains `q` and whose role is `role` (None: any).
pub fn filter<'a>(list: &'a [Member], q: &str, role: Option<usize>) -> Vec<&'a Member> {
    let q = q.trim().to_lowercase();
    list.iter()
        .filter(|x| role.is_none_or(|r| x.role == r))
        .filter(|x| {
            q.is_empty() || x.name.to_lowercase().contains(&q) || x.email.contains(q.as_str())
        })
        .collect()
}

/// Seats used (joined and invited both hold one).
pub fn seats_used(list: &[Member]) -> usize {
    list.len()
}

/// "7 of 10 seats · 3 free".
pub fn seats_words(list: &[Member]) -> String {
    let n = seats_used(list);
    let free = SEATS.saturating_sub(n);
    format!("{n} of {SEATS} seats · {free} free")
}

/// Split what was typed into the invite field into addresses (commas, spaces or new lines).
pub fn split_emails(s: &str) -> Vec<String> {
    s.split(|c: char| c == ',' || c == ';' || c.is_whitespace())
        .map(|x| x.trim().to_lowercase())
        .filter(|x| !x.is_empty())
        .collect()
}

/// What stops an invite, if anything.
pub fn invite_error(typed: &str, list: &[Member]) -> Option<String> {
    let mails = split_emails(typed);
    if mails.is_empty() {
        return Some("Add at least one email".to_string());
    }
    if let Some(bad) = mails
        .iter()
        .find(|e| super::setscr::email_error(e).is_some())
    {
        return Some(format!("{bad} does not look like an email"));
    }
    if let Some(dup) = mails.iter().find(|e| list.iter().any(|x| &x.email == *e)) {
        return Some(format!("{dup} is already in the workspace"));
    }
    let free = SEATS.saturating_sub(seats_used(list));
    if mails.len() > free {
        return Some(format!(
            "Only {free} seat{} left: add seats on Billing",
            if free == 1 { "" } else { "s" }
        ));
    }
    None
}

/// Add invites for every address; returns how many.
pub fn invite(list: &mut Vec<Member>, typed: &str, role: usize) -> usize {
    let mails = split_emails(typed);
    let first = list.iter().map(|x| x.id).max().unwrap_or(0) + 1;
    for (id, e) in (first..).zip(mails.iter()) {
        list.push(Member {
            id,
            name: "",
            email: e.clone(),
            role,
            team: None,
            usage: 0,
            seen: "Invited just now",
            invited: true,
        });
    }
    mails.len()
}

/// Owners in the list.
pub fn owners(list: &[Member]) -> usize {
    list.iter().filter(|x| x.role == 0 && !x.invited).count()
}

/// What stops giving member `id` the role `role`, if anything.
pub fn role_error(list: &[Member], id: u32, role: usize) -> Option<&'static str> {
    let x = list.iter().find(|x| x.id == id)?;
    if x.role == 0 && role != 0 && owners(list) <= 1 {
        Some("The workspace needs at least one owner")
    } else {
        None
    }
}

/// What stops removing member `id`, if anything.
pub fn remove_error(list: &[Member], id: u32) -> Option<&'static str> {
    let x = list.iter().find(|x| x.id == id)?;
    if x.is_me() {
        Some("You cannot remove yourself; another owner can")
    } else if x.role == 0 && owners(list) <= 1 {
        Some("The workspace needs at least one owner")
    } else {
        None
    }
}

/// People in team `t`.
pub fn team_size(list: &[Member], t: usize) -> usize {
    list.iter().filter(|x| x.team == Some(t)).count()
}

// ───────────────────────────────────────────────────────────── Models ──

/// A provider: brand id of its official mark, name, where it is hosted.
pub const PROVIDERS: [(&str, &str, &str); 5] = [
    ("anthropic", "Anthropic", "Hosted in the EU"),
    ("openai", "OpenAI", "Hosted in the EU"),
    ("gemini", "Google", "Hosted in the EU"),
    ("mistral", "Mistral", "Hosted in France"),
    ("deepseek", "DeepSeek", "Hosted by Oxyn in Frankfurt"),
];

/// Who may use a model.
pub const ACCESS: [&str; 3] = ["Everyone", "Admins", "Engineering"];

/// A model: name, provider index, kind (`chat` / `image`), mark id, line.
pub const MODELS: [(&str, usize, &str, &str, &str); 9] = [
    (
        "Opus 5.5",
        0,
        "chat",
        "claude",
        "Careful, long tasks · 400K context",
    ),
    (
        "Sonnet 5.5",
        0,
        "chat",
        "claude",
        "Everyday writing and code · 400K",
    ),
    ("GPT-6", 1, "chat", "openai", "Reasoning and tools · 256K"),
    (
        "GPT-6 mini",
        1,
        "chat",
        "openai",
        "Quick everyday answers · 128K",
    ),
    (
        "Image 3",
        1,
        "image",
        "openai",
        "Pictures from a description",
    ),
    (
        "Gemini 3.8 Pro",
        2,
        "chat",
        "gemini",
        "Long files and images · 2M",
    ),
    (
        "Gemini 3.8 Flash",
        2,
        "chat",
        "gemini",
        "Fast, low cost · 1M",
    ),
    (
        "Mistral Large 3",
        3,
        "chat",
        "mistral",
        "Strong in French and German · 128K",
    ),
    (
        "DeepSeek V4",
        4,
        "chat",
        "deepseek",
        "Open weights, code · 128K",
    ),
];

/// "2/2 on".
pub fn provider_on(on: &[bool], prov: usize) -> (usize, usize) {
    let idx: Vec<usize> = (0..MODELS.len()).filter(|i| MODELS[*i].1 == prov).collect();
    (idx.iter().filter(|i| on[**i]).count(), idx.len())
}

/// Models in a provider that match the kind tab (`all`, `chat`, `image`) and the query.
pub fn models_in(prov: usize, tab: &str, q: &str) -> Vec<usize> {
    let q = q.trim().to_lowercase();
    (0..MODELS.len())
        .filter(|i| MODELS[*i].1 == prov)
        .filter(|i| tab == "all" || MODELS[*i].2 == tab)
        .filter(|i| q.is_empty() || MODELS[*i].0.to_lowercase().contains(&q))
        .collect()
}

/// Chat models that are on (the defaults pick from these).
pub fn chat_on(on: &[bool]) -> Vec<usize> {
    (0..MODELS.len())
        .filter(|i| MODELS[*i].2 == "chat" && on[*i])
        .collect()
}

/// Turn model `i` on or off. A default that is turned off falls back (chat to Auto, image to
/// none). Returns what to announce.
pub fn set_model(w: &mut Ws, i: usize, on: bool) -> String {
    w.models_on[i] = on;
    let name = MODELS[i].0;
    if !on && w.chat_default == Some(i) {
        w.chat_default = None;
        return format!("{name} turned off; new chats start on Auto");
    }
    if !on && w.image_default == Some(i) {
        w.image_default = None;
        return format!("{name} turned off; pictures are off until you pick another model");
    }
    format!("{name} turned {}", if on { "on" } else { "off" })
}

/// An API key for a provider.
#[derive(Clone, Debug, PartialEq)]
pub struct ApiKey {
    /// Stable id.
    pub id: u32,
    /// Provider index.
    pub prov: usize,
    /// Its name.
    pub name: String,
    /// The secret (fictional).
    pub secret: String,
    /// Added.
    pub added: &'static str,
    /// Last used.
    pub used: &'static str,
}

/// The sample keys.
pub fn keys(sample: bool) -> Vec<ApiKey> {
    if !sample {
        return Vec::new();
    }
    vec![
        ApiKey {
            id: 1,
            prov: 0,
            name: "Studio production".to_string(),
            secret: "sk-demo-kestrel-7Q2m9ZtL4pXv81Ha3f9a".to_string(),
            added: "12 Aug",
            used: "2 minutes ago",
        },
        ApiKey {
            id: 2,
            prov: 3,
            name: "Translations".to_string(),
            secret: "mk-demo-kestrel-Vb6Rr0Nq5Lw2Jc8E71c4".to_string(),
            added: "3 Oct",
            used: "Yesterday",
        },
    ]
}

/// The key with all but its first 8 and last 4 characters hidden.
pub fn mask(secret: &str) -> String {
    let c: Vec<char> = secret.chars().collect();
    if c.len() <= 12 {
        return "•".repeat(c.len());
    }
    let head: String = c[..8].iter().collect();
    let tail: String = c[c.len() - 4..].iter().collect();
    format!("{head}••••••••{tail}")
}

/// What is wrong with a key someone pasted, if anything.
pub fn key_error(name: &str, secret: &str) -> Option<&'static str> {
    let s = secret.trim();
    if name.trim().is_empty() {
        Some("Name the key")
    } else if s.is_empty() {
        Some("Paste the key")
    } else if s.contains(char::is_whitespace) {
        Some("A key has no spaces")
    } else if s.chars().count() < 20 {
        Some("That key looks too short")
    } else {
        None
    }
}

/// A new secret for a rotated key: same prefix, a fresh tail (deterministic from the old one).
pub fn rotated(secret: &str, n: u32) -> String {
    const A: &[u8] = b"ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz23456789";
    let prefix: String = secret.chars().take(16).collect();
    let mut h: u64 = 1469598103934665603 ^ u64::from(n);
    for b in secret.bytes() {
        h = (h ^ u64::from(b)).wrapping_mul(1099511628211);
    }
    let mut tail = String::new();
    for _ in 0..20 {
        h = h
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        tail.push(A[(h >> 33) as usize % A.len()] as char);
    }
    format!("{prefix}{tail}")
}

// ─────────────────────────────────────────────────────────── Security ──

/// How long people stay signed in.
pub const SESSION_LEN: [&str; 4] = ["1 day", "7 days", "30 days", "90 days"];

/// A passkey.
#[derive(Clone, Debug, PartialEq)]
pub struct Passkey {
    /// Stable id.
    pub id: u32,
    /// Its name.
    pub name: String,
    /// Glyph (`laptop`, `phone`).
    pub glyph: &'static str,
    /// Added.
    pub added: &'static str,
    /// Last used.
    pub used: &'static str,
}

/// The sample passkeys.
pub fn passkeys(sample: bool) -> Vec<Passkey> {
    if !sample {
        return Vec::new();
    }
    vec![
        Passkey {
            id: 1,
            name: "MacBook Pro fingerprint".to_string(),
            glyph: "laptop",
            added: "2 Sep",
            used: "Today",
        },
        Passkey {
            id: 2,
            name: "Phone face unlock".to_string(),
            glyph: "phone",
            added: "2 Sep",
            used: "Last week",
        },
    ]
}

/// An email domain.
#[derive(Clone, Debug, PartialEq)]
pub struct Domain {
    /// The domain.
    pub name: String,
    /// Verified.
    pub verified: bool,
}

/// The sample domains.
pub fn domains(sample: bool) -> Vec<Domain> {
    if !sample {
        return Vec::new();
    }
    vec![
        Domain {
            name: "kestrel.example".to_string(),
            verified: true,
        },
        Domain {
            name: "kestrel-studio.example".to_string(),
            verified: false,
        },
    ]
}

/// What is wrong with a domain someone typed, if anything.
pub fn domain_error(v: &str, list: &[Domain]) -> Option<&'static str> {
    let t = v.trim().to_lowercase();
    let ok = t.contains('.')
        && !t.starts_with('.')
        && !t.ends_with('.')
        && t.chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '.' || c == '-');
    if t.is_empty() {
        Some("Add a domain")
    } else if t.contains('@') {
        Some("Just the part after the @")
    } else if !ok {
        Some("That does not look like a domain")
    } else if list.iter().any(|d| d.name == t) {
        Some("This domain is already listed")
    } else {
        None
    }
}

/// The DNS record that proves a domain (fictional, stable per domain).
pub fn txt_record(domain: &str) -> String {
    let mut h: u32 = 2166136261;
    for b in domain.bytes() {
        h = (h ^ u32::from(b)).wrapping_mul(16777619);
    }
    format!("oxyn-verify={:08x}", h)
}

/// A signed-in device of someone in the workspace.
#[derive(Clone, Debug, PartialEq)]
pub struct Device {
    /// Stable id.
    pub id: u32,
    /// Who.
    pub who: &'static str,
    /// Device and app.
    pub device: &'static str,
    /// Glyph.
    pub glyph: &'static str,
    /// Place.
    pub place: &'static str,
    /// Last seen.
    pub seen: &'static str,
}

/// The sample devices.
pub fn devices(sample: bool) -> Vec<Device> {
    let me = Device {
        id: 1,
        who: PERSON,
        device: "MacBook Pro · Browser",
        glyph: "laptop",
        place: "Lisbon",
        seen: "Now",
    };
    if !sample {
        return vec![me];
    }
    vec![
        me,
        Device {
            id: 2,
            who: "Ines Duarte",
            device: "Desktop app",
            glyph: "laptop",
            place: "Porto",
            seen: "5 minutes ago",
        },
        Device {
            id: 3,
            who: "Tomas Reyna",
            device: "Phone · Mobile app",
            glyph: "phone",
            place: "Madrid",
            seen: "1 hour ago",
        },
        Device {
            id: 4,
            who: "Build server",
            device: "Command line",
            glyph: "term",
            place: "Frankfurt",
            seen: "Today",
        },
    ]
}

/// Audit filter chips: key, label.
pub const AUDIT_KINDS: [(&str, &str); 5] = [
    ("all", "All"),
    ("signin", "Sign-in"),
    ("members", "Members"),
    ("keys", "Keys"),
    ("billing", "Billing"),
];

/// An audit event: kind, who, what, when, place, tone (`ok`, `warn`, `bad`, ``).
pub type Event = (
    &'static str,
    &'static str,
    &'static str,
    &'static str,
    &'static str,
    &'static str,
);

/// The sample audit log, newest first.
pub const AUDIT: [Event; 8] = [
    (
        "signin",
        "Rowan Vale",
        "Signed in with a passkey",
        "Today 09:12",
        "Lisbon",
        "ok",
    ),
    (
        "keys",
        "Ines Duarte",
        "Rotated the key “Studio production”",
        "Today 08:40",
        "Porto",
        "",
    ),
    (
        "signin",
        "Unknown",
        "Failed sign-in for tomas@kestrel.example",
        "Yesterday 23:05",
        "Unknown place",
        "bad",
    ),
    (
        "members",
        "Rowan Vale",
        "Invited noor@kestrel.example as Member",
        "16 Nov",
        "Lisbon",
        "",
    ),
    (
        "billing",
        "Rowan Vale",
        "Paid invoice OX-2026-011",
        "1 Nov",
        "Lisbon",
        "ok",
    ),
    (
        "members",
        "Ines Duarte",
        "Changed Jules Arden to Viewer",
        "28 Oct",
        "Porto",
        "warn",
    ),
    (
        "keys",
        "Rowan Vale",
        "Added the key “Translations”",
        "3 Oct",
        "Lisbon",
        "",
    ),
    (
        "signin",
        "Mika Sato",
        "Turned on two-step sign-in",
        "1 Oct",
        "Lisbon",
        "ok",
    ),
];

/// Audit events of a kind (`all` for every one); the empty workspace has just the first.
pub fn audit(kind: &str, sample: bool) -> Vec<Event> {
    AUDIT
        .iter()
        .take(if sample { AUDIT.len() } else { 1 })
        .filter(|e| kind == "all" || e.0 == kind)
        .copied()
        .collect()
}

/// A six-digit code from an authenticator app is well formed (spaces allowed).
pub fn code_ok(c: &str) -> bool {
    let d: String = c.chars().filter(|x| !x.is_whitespace()).collect();
    d.len() == 6 && d.chars().all(|x| x.is_ascii_digit())
}

/// The setup secret shown under the QR code (fictional).
pub const TOTP_SECRET: &str = "KSTR LZ7Q M4XV 2PDN";

/// Ten recovery codes (fictional, stable per seed).
pub fn recovery_codes(seed: u32) -> Vec<String> {
    const A: &[u8] = b"abcdefghjkmnpqrstuvwxyz23456789";
    let mut h: u64 = 0x9E37_79B9_7F4A_7C15 ^ u64::from(seed);
    (0..10)
        .map(|_| {
            let mut s = String::new();
            for i in 0..10 {
                if i == 5 {
                    s.push('-');
                }
                h = h
                    .wrapping_mul(6364136223846793005)
                    .wrapping_add(1442695040888963407);
                s.push(A[(h >> 33) as usize % A.len()] as char);
            }
            s
        })
        .collect()
}

/// A fake QR module grid (21 × 21, finder squares in three corners), stable per seed.
pub fn qr_cells(seed: u32) -> Vec<bool> {
    let n = 21usize;
    let mut h: u64 = 0xD1B5_4A32_D192_ED03 ^ u64::from(seed);
    let mut v = Vec::with_capacity(n * n);
    for y in 0..n {
        for x in 0..n {
            let finder = |ox: usize, oy: usize| {
                let (dx, dy) = (x.wrapping_sub(ox), y.wrapping_sub(oy));
                if dx < 7 && dy < 7 {
                    let edge = dx == 0 || dy == 0 || dx == 6 || dy == 6;
                    let core = (2..=4).contains(&dx) && (2..=4).contains(&dy);
                    Some(edge || core)
                } else if dx < 8 && dy < 8 {
                    Some(false)
                } else {
                    None
                }
            };
            let f = finder(0, 0)
                .or_else(|| finder(n - 7, 0))
                .or_else(|| finder(0, n - 7));
            v.push(match f {
                Some(b) => b,
                None => {
                    h = h
                        .wrapping_mul(6364136223846793005)
                        .wrapping_add(1442695040888963407);
                    (h >> 40) & 1 == 1
                }
            });
        }
    }
    v
}

// ──────────────────────────────────────────────────────────── Billing ──

/// Plans: key, name, monthly price per seat in dollars, line, features.
pub const PLANS: [(&str, &str, u32, &str, [&str; 3]); 3] = [
    (
        "free",
        "Free",
        0,
        "For trying Oxyn on your own",
        [
            "1 seat and 500 credits a month",
            "Chat with the fast models",
            "7 days of run history",
        ],
    ),
    (
        "pro",
        "Pro",
        20,
        "For small teams that run agents daily",
        [
            "Every model, 5,000 credits a seat",
            "Agents, schedules and skills",
            "90 days of run history",
        ],
    ),
    (
        "team",
        "Team",
        36,
        "For companies that need control",
        [
            "Everything in Pro, 8,000 credits a seat",
            "Single sign-on and audit log",
            "Unlimited history, priority help",
        ],
    ),
];

/// Billing intervals.
pub const INTERVALS: [&str; 2] = ["Monthly", "Yearly"];
/// Monthly spend caps for extra usage.
pub const CAPS: [&str; 4] = ["No extra usage", "$100", "$250", "$500"];

/// The price per seat per month for a plan and interval (yearly is 20 % off, whole dollars).
pub fn seat_price(plan: usize, yearly: bool) -> u32 {
    let p = PLANS[plan.min(PLANS.len() - 1)].2;
    if yearly { p * 8 / 10 } else { p }
}

/// The next bill in dollars: seats × price (× 12 when yearly).
pub fn next_bill(plan: usize, yearly: bool, seats: usize) -> u32 {
    let s = u32::try_from(seats).unwrap_or(0);
    seat_price(plan, yearly) * s * if yearly { 12 } else { 1 }
}

/// "$1,234".
pub fn dollars(n: u32) -> String {
    let s = n.to_string();
    let mut out = String::new();
    for (i, ch) in s.chars().enumerate() {
        if i > 0 && (s.len() - i).is_multiple_of(3) {
            out.push(',');
        }
        out.push(ch);
    }
    format!("${out}")
}

/// A usage meter: label, used, limit, unit, glyph.
pub type Meter = (&'static str, f32, f32, &'static str, &'static str);

/// The usage meters this period.
pub fn meters(sample: bool, seats: usize) -> [Meter; 4] {
    let k = if sample { 1.0 } else { 0.0 };
    [
        ("Seats", seats as f32, SEATS as f32, "", "people"),
        ("Credits", 34_200.0 * k, 50_000.0, "", "spark"),
        ("Storage", 42.6 * k, 100.0, "GB", "db"),
        ("Agent runs", 2_310.0 * k, 2_500.0, "", "runs"),
    ]
}

/// Tone of a meter by how full it is: `bad` from 90 %, `warn` from 75 %, else `run`.
pub fn meter_tone(used: f32, limit: f32) -> &'static str {
    let p = if limit > 0.0 { used / limit } else { 0.0 };
    if p >= 0.9 {
        "bad"
    } else if p >= 0.75 {
        "warn"
    } else {
        "run"
    }
}

/// "34,200 / 50,000" or "42.6 / 100 GB".
pub fn meter_words(used: f32, limit: f32, unit: &str) -> String {
    let f = |x: f32| {
        if x.fract().abs() > f32::EPSILON {
            format!("{x:.1}")
        } else {
            dollars(x as u32).trim_start_matches('$').to_string()
        }
    };
    let u = if unit.is_empty() {
        String::new()
    } else {
        format!(" {unit}")
    };
    format!("{} / {}{u}", f(used), f(limit))
}

/// An invoice: number, date, amount in dollars, status (`Paid`, `Due`, `Refunded`).
pub type Invoice = (&'static str, &'static str, u32, &'static str);

/// The sample invoices, newest first.
pub const INVOICES: [Invoice; 5] = [
    ("OX-2026-012", "1 Dec 2026", 140, "Due"),
    ("OX-2026-011", "1 Nov 2026", 140, "Paid"),
    ("OX-2026-010", "1 Oct 2026", 120, "Paid"),
    ("OX-2026-009", "1 Sep 2026", 120, "Paid"),
    ("OX-2026-008", "1 Aug 2026", 20, "Refunded"),
];

/// Reasons offered when someone cancels.
pub const CANCEL_REASONS: [&str; 5] = [
    "It costs too much",
    "We use another tool",
    "Missing a feature we need",
    "Only needed it for a project",
    "Something else",
];

/// The cancel flow: 0 closed, 1 reason, 2 what changes, 3 done.
pub fn cancel_next(step: u8, reason: Option<usize>) -> u8 {
    match step {
        1 if reason.is_none() => 1,
        1 => 2,
        2 => 3,
        s => s,
    }
}

/// A card number passes the Luhn check (spaces allowed, 12–19 digits).
pub fn luhn(num: &str) -> bool {
    let d: Vec<u32> = num.chars().filter_map(|c| c.to_digit(10)).collect();
    if num.chars().any(|c| !c.is_ascii_digit() && c != ' ') || !(12..=19).contains(&d.len()) {
        return false;
    }
    let s: u32 = d
        .iter()
        .rev()
        .enumerate()
        .map(|(i, x)| {
            if i % 2 == 1 {
                let y = x * 2;
                if y > 9 { y - 9 } else { y }
            } else {
                *x
            }
        })
        .sum();
    s.is_multiple_of(10)
}

/// An expiry "MM/YY" is well formed and not before 11/26.
pub fn expiry_ok(v: &str) -> bool {
    let Some((mm, yy)) = v.trim().split_once('/') else {
        return false;
    };
    let (Ok(m), Ok(y)) = (mm.trim().parse::<u32>(), yy.trim().parse::<u32>()) else {
        return false;
    };
    (1..=12).contains(&m) && yy.trim().len() == 2 && (y > 26 || (y == 26 && m >= 11))
}

// ──────────────────────────────────────────────────────── Get started ──

/// Checklist steps: key, title, line, glyph, the page that does it.
pub const STEPS: [(&str, &str, &str, &str, &str); 6] = [
    (
        "profile",
        "Add your name and picture",
        "So people know who ran what",
        "user",
        "profile",
    ),
    (
        "workspace",
        "Name the workspace",
        "Its name, address and context",
        "building",
        "general",
    ),
    (
        "invite",
        "Invite your team",
        "Bring two or more people in",
        "people",
        "members",
    ),
    (
        "models",
        "Choose your models",
        "Turn on the ones you trust",
        "spark",
        "models",
    ),
    (
        "twostep",
        "Turn on two-step sign-in",
        "Protect the account with a code",
        "lock",
        "security",
    ),
    (
        "plan",
        "Pick a plan",
        "Free until you need more",
        "card",
        "billing",
    ),
];

/// (done, total, percent).
pub fn progress(done: &[bool]) -> (usize, usize, u32) {
    let n = done.iter().filter(|x| **x).count();
    let t = done.len();
    let p = u32::try_from((n * 100).checked_div(t).unwrap_or(0)).unwrap_or(100);
    (n, t, p)
}

/// The first step still open.
pub fn next_step(done: &[bool]) -> Option<usize> {
    done.iter().position(|x| !*x)
}

/// "4 of 6 done · next: Turn on two-step sign-in" / "All set".
pub fn progress_words(done: &[bool]) -> String {
    let (n, t, _) = progress(done);
    match next_step(done) {
        Some(i) => format!("{n} of {t} done · next: {}", STEPS[i].1),
        None => "All set: every step is done".to_string(),
    }
}

// ──────────────────────────────────────────────────────────── State ──

/// Everything the S7 pages hold.
#[derive(Clone, Debug, PartialEq)]
pub struct Ws {
    /// Members and invites (act at once).
    pub members: Vec<Member>,
    /// Model switches.
    pub models_on: [bool; 9],
    /// Model access (index in [`ACCESS`]).
    pub access: [usize; 9],
    /// Default chat model (None: Auto).
    pub chat_default: Option<usize>,
    /// Default image model (None: off).
    pub image_default: Option<usize>,
    /// API keys (act at once).
    pub keys: Vec<ApiKey>,
    /// Two-step sign-in on for you.
    pub two_step_me: bool,
    /// Passkeys (act at once).
    pub passkeys: Vec<Passkey>,
    /// Two-step required for everyone.
    pub two_step_all: bool,
    /// Single sign-on required.
    pub sso: bool,
    /// Join by verified domain.
    pub join_domain: bool,
    /// Session length index.
    pub session_len: usize,
    /// Email domains (act at once).
    pub domains: Vec<Domain>,
    /// Signed-in devices (act at once).
    pub devices: Vec<Device>,
    /// Plan index in [`PLANS`].
    pub plan: usize,
    /// Yearly billing.
    pub yearly: bool,
    /// Extra-usage cap index.
    pub cap: usize,
    /// Invoices by email.
    pub invoice_mail: bool,
    /// A card is on file (last four).
    pub card: Option<String>,
    /// Cancels at period end.
    pub cancelled: bool,
    /// Checklist steps done (act at once).
    pub steps: [bool; 6],
    /// Show the checklist in the sidebar.
    pub show_checklist: bool,
}

/// The sample workspace, or a new one.
pub fn ws(sample: bool) -> Ws {
    Ws {
        members: members(sample),
        models_on: if sample {
            [true, true, true, true, true, true, false, false, false]
        } else {
            [false, true, false, true, false, false, true, false, false]
        },
        access: if sample {
            [0, 0, 0, 0, 0, 2, 0, 0, 0]
        } else {
            [0; 9]
        },
        chat_default: None,
        image_default: if sample { Some(4) } else { None },
        keys: keys(sample),
        two_step_me: false,
        passkeys: passkeys(sample),
        two_step_all: sample,
        sso: false,
        join_domain: sample,
        session_len: 2,
        domains: domains(sample),
        devices: devices(sample),
        plan: usize::from(sample),
        yearly: false,
        cap: if sample { 2 } else { 0 },
        invoice_mail: true,
        card: if sample {
            Some("4242".to_string())
        } else {
            None
        },
        cancelled: false,
        steps: if sample {
            [true, true, true, true, false, true]
        } else {
            [false; 6]
        },
        show_checklist: true,
    }
}

/// How many S7 policies differ (lists act at once and are not counted).
pub fn changes(a: &Ws, b: &Ws) -> usize {
    let mut n = 0;
    let mut c = |x: bool| n += usize::from(x);
    for i in 0..9 {
        c(a.models_on[i] != b.models_on[i]);
        c(a.access[i] != b.access[i]);
    }
    c(a.chat_default != b.chat_default);
    c(a.image_default != b.image_default);
    c(a.two_step_all != b.two_step_all);
    c(a.sso != b.sso);
    c(a.join_domain != b.join_domain);
    c(a.session_len != b.session_len);
    c(a.yearly != b.yearly);
    c(a.cap != b.cap);
    c(a.invoice_mail != b.invoice_mail);
    c(a.show_checklist != b.show_checklist);
    n
}

/// What blocks saving an S7 page, if anything: SSO needs a verified domain.
pub fn ws_errors(w: &Ws) -> usize {
    usize::from(w.sso && !w.domains.iter().any(|d| d.verified))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn s7_settings() {
        assert_eq!(states_for("settings-billing").unwrap()[3].0, "cancelling");
        assert!(states_for("settings-profile").is_none());
        // Members.
        let mut l = members(true);
        assert_eq!(seats_words(&l), "7 of 10 seats · 3 free");
        assert_eq!(filter(&l, "ines", None).len(), 1);
        assert_eq!(filter(&l, "", Some(2)).len(), 4);
        assert_eq!(
            split_emails("a@x.io, b@x.io\nc@x.io"),
            vec!["a@x.io", "b@x.io", "c@x.io"]
        );
        assert_eq!(invite_error("", &l).unwrap(), "Add at least one email");
        assert!(invite_error("nope", &l).unwrap().contains("does not look"));
        assert!(
            invite_error("Ines@kestrel.example", &l)
                .unwrap()
                .contains("already")
        );
        assert!(
            invite_error("a@x.io b@x.io c@x.io d@x.io", &l)
                .unwrap()
                .starts_with("Only 3")
        );
        assert_eq!(invite_error("a@x.io b@x.io", &l), None);
        assert_eq!(invite(&mut l, "a@x.io b@x.io", 2), 2);
        assert_eq!(seats_used(&l), 9);
        assert!(l.last().unwrap().invited);
        assert_eq!(
            role_error(&l, 1, 2),
            Some("The workspace needs at least one owner")
        );
        assert_eq!(role_error(&l, 2, 0), None);
        assert!(remove_error(&l, 1).is_some());
        assert_eq!(remove_error(&l, 3), None);
        assert_eq!(team_size(&members(true), 1), 2);
        // Models.
        let mut w = ws(true);
        assert_eq!(provider_on(&w.models_on, 2), (1, 2));
        assert_eq!(models_in(1, "image", ""), vec![4]);
        assert_eq!(models_in(2, "all", "flash"), vec![6]);
        w.chat_default = Some(0);
        assert!(set_model(&mut w, 0, false).contains("Auto"));
        assert_eq!(w.chat_default, None);
        assert!(set_model(&mut w, 4, false).contains("pictures are off"));
        assert_eq!(w.image_default, None);
        assert_eq!(chat_on(&w.models_on), vec![1, 2, 3, 5]);
        assert_eq!(
            mask("sk-demo-kestrel-7Q2m9ZtL4pXv81Ha3f9a"),
            "sk-demo-••••••••3f9a"
        );
        assert_eq!(key_error("x", "short"), Some("That key looks too short"));
        assert_eq!(key_error("", "abc"), Some("Name the key"));
        assert_eq!(key_error("x", "abcdefghijklmnopqrstuv"), None);
        let r = rotated("sk-demo-kestrel-7Q2m9ZtL4pXv81Ha3f9a", 1);
        assert!(r.starts_with("sk-demo-kestrel-") && r.len() == 36);
        assert_ne!(r, rotated("sk-demo-kestrel-7Q2m9ZtL4pXv81Ha3f9a", 2));
        // Security.
        assert!(code_ok("428 913"));
        assert!(!code_ok("42891"));
        let codes = recovery_codes(7);
        assert_eq!(codes.len(), 10);
        assert_eq!(codes[0].len(), 11);
        assert_eq!(qr_cells(1).len(), 441);
        assert!(qr_cells(1)[0] && !qr_cells(1)[7]);
        let d = domains(true);
        assert_eq!(
            domain_error("kestrel.example", &d),
            Some("This domain is already listed")
        );
        assert_eq!(
            domain_error("me@x.io", &d),
            Some("Just the part after the @")
        );
        assert_eq!(domain_error("heron.example", &d), None);
        assert!(txt_record("a.io").starts_with("oxyn-verify="));
        assert_eq!(audit("keys", true).len(), 2);
        assert_eq!(audit("all", false).len(), 1);
        // Billing.
        assert_eq!(seat_price(1, true), 16);
        assert_eq!(next_bill(1, false, 7), 140);
        assert_eq!(next_bill(2, true, 10), 3360);
        assert_eq!(dollars(1234567), "$1,234,567");
        assert_eq!(meter_tone(2310.0, 2500.0), "bad");
        assert_eq!(meter_tone(80.0, 100.0), "warn");
        assert_eq!(meter_words(42.6, 100.0, "GB"), "42.6 / 100 GB");
        assert_eq!(meter_words(34200.0, 50000.0, ""), "34,200 / 50,000");
        assert_eq!(cancel_next(1, None), 1);
        assert_eq!(cancel_next(1, Some(0)), 2);
        assert!(luhn("4242 4242 4242 4242"));
        assert!(!luhn("4242 4242 4242 4241"));
        assert!(expiry_ok("08/28"));
        assert!(!expiry_ok("10/26"));
        assert!(!expiry_ok("13/28"));
        // Checklist.
        let w = ws(true);
        assert_eq!(progress(&w.steps), (5, 6, 83));
        assert_eq!(
            progress_words(&w.steps),
            "5 of 6 done · next: Turn on two-step sign-in"
        );
        assert_eq!(progress_words(&[true; 6]), "All set: every step is done");
        // Changes.
        let mut b = w.clone();
        b.members.clear();
        assert_eq!(changes(&w, &b), 0);
        b.models_on[8] = true;
        b.sso = true;
        assert_eq!(changes(&w, &b), 2);
        assert_eq!(ws_errors(&b), 0);
        b.domains.retain(|d| !d.verified);
        assert_eq!(ws_errors(&b), 1);
    }
}
