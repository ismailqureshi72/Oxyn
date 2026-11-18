//! Screens S6: Settings I – Profile, Preferences, Notifications, Privacy and General – in the
//! settings shell: the settings sidebar (Back to app, a search over every setting, Personal and
//! Workspace pages, the account row and its menu) in the shared Inset or Flat frame, and a centred
//! 696 px column per page. Every page is real: fields check themselves, edits collect in a save bar
//! (or save at once with `autosave`), destructive actions ask first, and the screen's own state
//! (`oxyn_core::parity::setscr::states_for`) is one press away. Styles in `setscr.css` (`oxs6-`);
//! the settings-shell rows (`oxsst-`) and shell parts (`oxscn-`) are reused. Chrome stays
//! monochrome; colour is for official marks, identity hues and status.

use dioxus::prelude::*;
use oxyn_core::parity::screensfid::{PERSON_PLAN, account_menu};
use oxyn_core::parity::setscr::{
    CHANNELS, COMPANY_CAP, CONTEXT_CAP, COOKIES, DAYS, DEFAULT_MODELS, DIGEST_AT, GROUPS, HOURS,
    INSTR_CAP, KINDS, LANGS, NAV, PERSON, PERSON_EMAIL, PERSON_INI, Prefs, ROLES, SHARING, STYLES,
    THEMES, TOOLS, WEEK_START, WORKSPACE, WS_HUES, ZONES, changes, changes_words, col_all,
    confirm_ok, digest_words, email_error, export_step, find, initials, lead, link_url, links_in,
    name_error, page_errors, page_of, password_ok, prefs, quiet_words, reach_words,
    sign_out_others, slug_error, slug_from, split, states_for, strength, toggle_col, ws_name_error,
};

use oxyn_core::parity::setscr::ScreenState;

use super::screensfid::{AppFrame, MenuButton, MenuItem, SidePeek, ico, nico, skel, use_threads};
use crate::brand::BrandIcon;

/// The state bar entries of each S6 screen (for a host's `ScreenStage { states }`).
pub fn s6_states(slug: &str) -> Vec<(&'static str, &'static str)> {
    states_for(slug).to_vec()
}

fn focus_on(e: MountedEvent) {
    let d = e.data();
    spawn(async move {
        let _ = d.set_focus(true).await;
    });
}

/// A settings row: label (with its hint) on the left, the control on the right.
fn row(id: &str, label: &str, hint: &str, ctl: Element, dirty: bool, hit: bool) -> Element {
    rsx! {
        div { class: "oxsst-row oxs6-row", "data-dirty": "{dirty}", "data-hit": "{hit}",
            div { class: "oxsst-rl",
                label { class: "oxsst-lb", r#for: "{id}", "{label}" }
                if !hint.is_empty() { p { class: "oxsst-hint", "{hint}" } }
            }
            div { class: "oxsst-rc", {ctl} }
        }
    }
}

/// A section: heading, an optional right part, then a card of rows.
fn sec(title: &str, right: Element, body: Element) -> Element {
    let danger = title == "Danger zone";
    rsx! {
        section { class: "oxsst-sec", "data-danger": "{danger}",
            div { class: "oxs6-sh", h2 { class: "oxsst-h2", "{title}" } span { class: "oxscn-grow" } {right} }
            div { class: "oxsst-card", {body} }
        }
    }
}

fn switch(id: &str, label: &str, on: bool, mut set: impl FnMut(bool) + 'static) -> Element {
    rsx! {
        button { r#type: "button", class: "oxscn-switch", role: "switch", id: "{id}", "aria-checked": "{on}", "aria-label": "{label}",
            onclick: move |_| set(!on), span {} }
    }
}

fn choice(
    label: &str,
    opts: &'static [&'static str],
    i: usize,
    mut set: impl FnMut(usize) + 'static,
) -> Element {
    let items: Vec<MenuItem> = opts.iter().map(|o| (*o, *o, "")).collect();
    let cur = opts.get(i).copied().unwrap_or("");
    rsx! {
        MenuButton { label: cur.to_string(), aria: "{label}: {cur}", items, checked: cur.to_string(), right: true, caret: true, class: "oxsst-mb".to_string(),
            on_pick: move |k: String| if let Some(j) = opts.iter().position(|o| *o == k) { set(j) } }
    }
}

/// A text field with its check and counter.
#[allow(clippy::too_many_arguments)]
fn field(
    id: &str,
    aria: &str,
    v: &str,
    cap: usize,
    long: bool,
    err: Option<&'static str>,
    ph: &str,
    mut set: impl FnMut(String) + 'static,
) -> Element {
    let n = v.chars().count();
    let over = cap > 0 && n > cap;
    let bad = err.is_some() || over;
    rsx! {
        div { class: "oxsst-tf", "data-long": "{long}",
            if long {
                textarea { class: "oxsst-in oxsst-ta", id: "{id}", rows: "3", value: "{v}", "aria-label": "{aria}", "aria-invalid": "{bad}", placeholder: "{ph}",
                    oninput: move |e| set(e.value()) }
            } else {
                input { class: "oxsst-in", id: "{id}", value: "{v}", "aria-label": "{aria}", "aria-invalid": "{bad}", placeholder: "{ph}",
                    oninput: move |e| set(e.value()) }
            }
            div { class: "oxsst-tfm",
                if let Some(m) = err { span { class: "oxsst-err", "{m}" } }
                if cap > 0 && (long || over) { span { class: "oxsst-cnt", "data-over": "{over}", "{n}/{cap}" } }
            }
        }
    }
}

/// A model mark (official colours), or the Auto spark.
fn model_mark(i: usize) -> Element {
    let (_, b, _) = DEFAULT_MODELS[i.min(DEFAULT_MODELS.len() - 1)];
    if b.is_empty() {
        rsx! { span { class: "oxs6-auto", {ico("spark")} } }
    } else {
        rsx! { BrandIcon { name: b.to_string(), size: 14.0, coloured: true } }
    }
}

// ─────────────────────────────────────────────────────────────── Shell ──

/// Field errors the edits introduced (a new account's blank fields do not block other changes).
fn new_errors(saved: &Prefs, cur: &Prefs) -> usize {
    ["profile", "preferences", "general"]
        .iter()
        .map(|pg| page_errors(cur, pg).saturating_sub(page_errors(saved, pg)))
        .sum()
}

#[derive(Clone, Copy, PartialEq)]
struct Ctx {
    cur: Signal<Prefs>,
    saved: Signal<Prefs>,
    said: Signal<String>,
    hits: Signal<Vec<&'static str>>,
}

impl Ctx {
    fn dirty<T: PartialEq>(&self, f: impl Fn(&Prefs) -> T) -> bool {
        f(&self.cur.read()) != f(&self.saved.read())
    }
    fn hit(&self, l: &str) -> bool {
        self.hits.read().contains(&l)
    }
}

/// The settings shell around one S6 page. Nav presses on the other S6 pages switch the page in place;
/// the S7 pages (Members, Models, Security, Billing) are listed and open a short summary.
#[component]
fn SetShell(
    layout: String,
    state: String,
    height: u32,
    slug: String,
    compact: bool,
    autosave: bool,
    #[props(default)] preview: bool,
    #[props(default)] on_back: EventHandler<()>,
) -> Element {
    let (st, own) = split(&state);
    let thr = use_threads(st);
    let active = use_signal(|| 0u32);
    let sample = st != ScreenState::Empty;
    let loading = st == ScreenState::Loading;
    let start = page_of(&slug);
    let mut saved = use_signal(move || {
        let mut p = prefs(sample);
        if own == "memoff" {
            p.memory = false;
        }
        p
    });
    let mut cur = use_signal(move || {
        let mut p = saved.peek().clone();
        if own == "unsaved" {
            p.name = "Rowan A. Vale".to_string();
            p.role = 0;
            p.zone = 2;
        }
        p
    });
    let mut page = use_signal(move || start);
    let mut q = use_signal(String::new);
    let mut said = use_signal(String::new);
    let mut acct = use_signal(|| false);
    let mut hits = use_signal(Vec::<&'static str>::new);
    let ctx = Ctx {
        cur,
        saved,
        said,
        hits,
    };
    let qs = q();
    let found = find(&qs);
    let searching = !qs.trim().is_empty();
    let pg = page();
    let label = NAV
        .iter()
        .find(|n| n.0 == pg)
        .map(|n| n.1)
        .unwrap_or("Profile");
    let s6 = NAV.iter().find(|n| n.0 == pg).is_some_and(|n| n.4);
    let n_changes = changes(&saved(), &cur());
    let errs = new_errors(&saved(), &cur());
    // Autosave: a valid change is kept as soon as it is made, and announced.
    use_effect(move || {
        let c = cur();
        if !autosave {
            return;
        }
        let n = changes(&saved.peek(), &c);
        if n > 0 && new_errors(&saved.peek(), &c) == 0 {
            saved.set(c);
            said.set("Saved".to_string());
        }
    });
    let mut peek = try_use_context::<SidePeek>().map(|s| s.0);
    let ac = acct();
    let side = rsx! {
        div { class: "oxsst-top",
            button { r#type: "button", class: "oxscn-btn oxsst-back", onclick: move |_| { said.set("Back to the app".to_string()); on_back.call(()); }, {ico("back")} span { "Back to app" } }
        }
        p { class: "oxsst-st", "Settings" }
        label { class: "oxscn-search",
            {ico("search")}
            input { r#type: "search", placeholder: "Search settings", "aria-label": "Search settings", value: "{q}",
                oninput: move |e| { let v = e.value(); hits.set(find(&v).iter().map(|h| h.1).collect()); q.set(v); },
                onkeydown: move |e: KeyboardEvent| {
                    if e.key() == Key::Escape { q.set(String::new()); hits.set(Vec::new()); }
                    if e.key() == Key::Enter && let Some(f) = find(&q.peek()).first() { page.set(f.0); if let Some(p) = peek.as_mut() { p.set(false); } }
                } }
        }
        div { class: "oxscn-lists oxsst-nav", role: "list",
            if loading { {skel(8)} } else {
                for (g, gl) in GROUPS {
                    p { key: "{g}", class: "oxscn-gh", "{gl}" }
                    for (k, l, _, ic, _) in NAV.iter().copied().filter(|n| n.2 == g) {
                        {
                            let n = found.iter().filter(|h| h.0 == k).count();
                            let on = pg == k;
                            let dim = searching && n == 0;
                            rsx! {
                                button { key: "{k}", r#type: "button", class: "oxscn-nav-b oxsst-nb", "data-on": "{on}", "data-dim": "{dim}", "aria-current": if on { "page" } else { "false" },
                                    onclick: move |_| { page.set(k); if let Some(p) = peek.as_mut() { p.set(false); } },
                                    {ico(ic)} span { class: "oxscn-lbl", "{l}" }
                                    if n > 0 { span { class: "oxscn-badge", "aria-label": "{n} matches", "{n}" } }
                                }
                            }
                        }
                    }
                }
            }
        }
        div { class: "oxscn-foot",
            div { class: "oxscn-acct",
                button { r#type: "button", class: "oxscn-acct-b", "aria-haspopup": "menu", "aria-expanded": "{ac}", "aria-label": "Account, {PERSON}, {PERSON_PLAN}", onclick: move |_| acct.toggle(),
                    span { class: "oxscn-av oxscn-av-id", "{PERSON_INI}" }
                    span { class: "oxscn-acct-txt", span { class: "oxscn-acct-n", "{PERSON}" } span { class: "oxscn-acct-p", "{PERSON_PLAN}" } }
                    span { class: "oxscn-acct-ud", {ico("updown")} }
                }
                if ac {
                    div { class: "oxscn-menu oxscn-acct-menu", role: "menu", "aria-label": "Account",
                        onkeydown: move |e: KeyboardEvent| if e.key() == Key::Escape { acct.set(false); },
                        div { class: "oxscn-acct-hd",
                            span { class: "oxscn-av oxscn-av-id oxscn-av-l", "{PERSON_INI}" }
                            span { class: "oxscn-acct-txt", span { class: "oxscn-acct-n", "{PERSON}" } span { class: "oxscn-acct-p", title: "{PERSON_EMAIL}", "{PERSON_EMAIL}" } }
                        }
                        for (i, (k, l, g, h)) in account_menu().iter().copied().enumerate() {
                            if k.is_empty() {
                                div { key: "{i}", class: "oxscn-msep", role: "separator" }
                            } else {
                                button { key: "{i}", r#type: "button", class: "oxscn-mi", role: "menuitem", "data-k": "{k}",
                                    onclick: move |_| { acct.set(false); if k == "settings" { page.set("profile"); } said.set(l.to_string()); },
                                    {ico(g)} span { class: "oxscn-mi-l", "{l}" }
                                    if h == "›" { span { class: "oxscn-mi-k oxscn-mi-sub", {ico("next")} } } else if !h.is_empty() { span { class: "oxscn-mi-k", "{h}" } }
                                }
                            }
                        }
                    }
                }
            }
        }
    };
    let title = format!("Settings / {label}");
    let body = match pg {
        "profile" => rsx! { ProfilePage { ctx, preview } },
        "preferences" => rsx! { PrefsPage { ctx } },
        "notifications" => rsx! { NotifPage { ctx, quiet_now: own == "quiet" } },
        "privacy" => rsx! { PrivacyPage { ctx, exporting: own == "exporting" } },
        "general" => rsx! { GeneralPage { ctx, deleting: own == "deleting" } },
        _ => {
            rsx! { OtherPage { key: "{pg}", page: pg, on_open: move |p: &'static str| page.set(p) } }
        }
    };
    rsx! {
        AppFrame { layout, state: st.key().to_string(), height, threads: thr, active, title, side_el: side,
            div { class: "oxsst oxs6", "data-compact": "{compact}",
                if loading {
                    div { class: "oxsst-col oxs6-col", {skel(3)} div { class: "oxsst-card", {skel(6)} } div { class: "oxsst-card", {skel(4)} } }
                } else {
                    div { class: "oxsst-col oxs6-col",
                        header { class: "oxsst-head",
                            h1 { class: "oxscn-h1", "{label}" }
                            p { class: "oxscn-lead", "{lead(pg)}" }
                            if searching {
                                p { class: "oxsst-found", role: "status",
                                    if found.is_empty() { "No setting matches “{qs}”." } else if found.len() == 1 { "1 setting matches “{qs}”." } else { "{found.len()} settings match “{qs}”." }
                                    if let Some(f) = found.first().map(|h| h.0) { if f != pg { button { r#type: "button", class: "oxscn-link", onclick: move |_| page.set(f), "Go to the first" } } }
                                }
                            }
                        }
                        if st == ScreenState::Empty && s6 {
                            p { class: "oxsst-new", "A new account: nothing is filled in yet. Add what you need; the defaults cover the rest." }
                        }
                        {body}
                    }
                    if n_changes > 0 && !autosave {
                        div { class: "oxsst-save", role: "region", "aria-label": "Unsaved changes",
                            span { class: "oxsst-save-t", "{changes_words(n_changes)}" }
                            if errs > 0 { span { class: "oxsst-save-e", "Fix the marked field first" } }
                            span { class: "oxscn-grow" }
                            button { r#type: "button", class: "oxscn-btn", onclick: move |_| { let s = saved.peek().clone(); cur.set(s); said.set("Changes discarded".to_string()); }, "Discard" }
                            button { r#type: "button", class: "oxscn-primary", disabled: errs > 0,
                                onclick: move |_| { let c = cur.peek().clone(); saved.set(c); said.set(if n_changes == 1 { "1 change saved".to_string() } else { format!("{n_changes} changes saved") }); }, "Save changes" }
                        }
                    }
                }
                p { class: "oxscn-sr", role: "status", "aria-live": "polite", "{said}" }
            }
        }
    }
}

/// Members, Models, Security and Billing: listed in the nav; their full pages are the Settings II
/// screens. Here a short card says what lives there.
#[component]
fn OtherPage(page: &'static str, on_open: EventHandler<&'static str>) -> Element {
    let rows: &[(&str, &str, &str)] = match page {
        "members" => &[
            (
                "people",
                "12 members, 3 guests",
                "Invite people and set their roles",
            ),
            ("mail", "2 invites waiting", "Resend or revoke"),
        ],
        "models" => &[
            ("spark", "Auto plus 4 models on", "Turn models on per team"),
            (
                "lock",
                "No API keys",
                "Usage runs on the plan until you add one",
            ),
        ],
        "security" => &[
            (
                "shield",
                "Two-step sign-in for everyone",
                "Required since September",
            ),
            (
                "globe",
                "1 email domain",
                "kestrel.example joins as members",
            ),
        ],
        _ => &[
            ("card", "Pro plan, monthly", "Renews on 1 December"),
            ("doc", "11 invoices", "Download any of them"),
        ],
    };
    rsx! {
        section { class: "oxsst-sec",
            div { class: "oxsst-card",
                for (i, (g, t, l)) in rows.iter().copied().enumerate() {
                    div { key: "{i}", class: "oxsst-row oxs6-row",
                        span { class: "oxs6-ic", {nico(g)} }
                        div { class: "oxsst-rl", span { class: "oxsst-lb", "{t}" } p { class: "oxsst-hint", "{l}" } }
                    }
                }
            }
            p { class: "oxs6-note", "This page is part of Settings II. ",
                button { r#type: "button", class: "oxscn-link", onclick: move |_| on_open.call("general"), "Open General" } }
        }
    }
}

// ───────────────────────────────────────────────────────────── Profile ──

#[component]
fn ProfilePage(ctx: Ctx, preview: bool) -> Element {
    let mut cur = ctx.cur;
    let mut said = ctx.said;
    let p = cur();
    let mut pw_open = use_signal(|| false);
    let mut pw = use_signal(String::new);
    let mut pw2 = use_signal(String::new);
    let mut mail_edit = use_signal(|| false);
    let (score, word) = strength(&pw());
    let ok = password_ok(&pw(), &pw2());
    let ini = initials(&p.name);
    let local = if p.clock24 { "14:20" } else { "2:20 pm" };
    rsx! {
        if preview {
            div { class: "oxs6-card-prev", "aria-label": "How others see you",
                span { class: "oxs6-av oxs6-av-l", "data-pic": "{p.picture}", "{ini}" }
                span { class: "oxs6-tt", b { class: "oxv-ell", if p.name.trim().is_empty() { "Your name" } else { "{p.name}" } } span { class: "oxv-ell oxs6-q", "{ROLES[p.role]} · {local} local time" } }
                span { class: "oxs6-tag", "How others see you" }
            }
        }
        {sec("About you", rsx! {}, rsx! {
            div { class: "oxsst-row oxs6-row", "data-dirty": "{ctx.dirty(|x| x.picture)}", "data-hit": "{ctx.hit(\"Picture\")}",
                div { class: "oxsst-rl", span { class: "oxsst-lb", "Picture" } p { class: "oxsst-hint", "Square, at least 256 px" } }
                div { class: "oxsst-rc oxs6-gap",
                    span { class: "oxs6-av", "data-pic": "{p.picture}", "aria-hidden": "true", "{ini}" }
                    button { r#type: "button", class: "oxscn-btn", onclick: move |_| { cur.write().picture = true; said.set("Picture uploaded".to_string()); }, {nico("upload")} span { if p.picture { "Change" } else { "Upload" } } }
                    if p.picture { button { r#type: "button", class: "oxscn-btn", onclick: move |_| { cur.write().picture = false; said.set("Picture removed".to_string()); }, "Remove" } }
                }
            }
            {row("oxs6-name", "Full name", "Shown on your messages and runs",
                field("oxs6-name", "Full name", &p.name, 60, false, name_error(&p.name), "Your name", move |v| cur.write().name = v),
                ctx.dirty(|x| x.name.clone()), ctx.hit("Full name"))}
            {row("oxs6-role", "Role", "Helps agents pitch their answers", choice("Role", &ROLES, p.role, move |i| cur.write().role = i), ctx.dirty(|x| x.role), ctx.hit("Role"))}
            div { class: "oxsst-row oxs6-row", "data-dirty": "{ctx.dirty(|x| x.email.clone())}", "data-hit": "{ctx.hit(\"Email\")}",
                div { class: "oxsst-rl", label { class: "oxsst-lb", r#for: "oxs6-mail", "Email" } p { class: "oxsst-hint", "Sign-in and receipts go here" } }
                div { class: "oxsst-rc",
                    if mail_edit() {
                        div { class: "oxs6-inl",
                            {field("oxs6-mail", "Email", &p.email, 0, false, email_error(&p.email), "name@company.com", move |v| cur.write().email = v)}
                            button { r#type: "button", class: "oxscn-btn", disabled: email_error(&p.email).is_some(),
                                onclick: move |_| { mail_edit.set(false); said.set("We sent a link to confirm the new address".to_string()); }, "Done" }
                        }
                    } else {
                        span { class: "oxs6-val oxv-ell", id: "oxs6-mail", "{p.email}" }
                        button { r#type: "button", class: "oxscn-btn", onclick: move |_| mail_edit.set(true), "Change" }
                    }
                }
            }
        })}
        {sec("Time", rsx! {}, rsx! {
            {row("oxs6-zone", "Time zone", "Digests and quiet hours follow it", choice("Time zone", &ZONES, p.zone, move |i| cur.write().zone = i), ctx.dirty(|x| x.zone), ctx.hit("Time zone"))}
            div { class: "oxsst-row oxs6-row", "data-dirty": "{ctx.dirty(|x| x.clock24)}", "data-hit": "{ctx.hit(\"Time format\")}",
                div { class: "oxsst-rl", span { class: "oxsst-lb", "Time format" } p { class: "oxsst-hint", "Now it reads {local}" } }
                div { class: "oxsst-rc",
                    div { class: "oxscn-seg oxs6-seg", role: "radiogroup", "aria-label": "Time format",
                        for (v, l) in [(false, "12-hour"), (true, "24-hour")] {
                            button { key: "{l}", r#type: "button", class: "oxscn-seg-b", role: "radio", "aria-checked": "{p.clock24 == v}", "aria-selected": "{p.clock24 == v}", onclick: move |_| cur.write().clock24 = v, "{l}" }
                        }
                    }
                }
            }
            {row("oxs6-week", "First day of the week", "", choice("First day of the week", &WEEK_START, p.week, move |i| cur.write().week = i), ctx.dirty(|x| x.week), ctx.hit("First day of the week"))}
        })}
        {sec("Personal context", rsx! {}, rsx! {
            div { class: "oxsst-row oxs6-row", "data-dirty": "{ctx.dirty(|x| x.context.clone())}", "data-hit": "{ctx.hit(\"Personal context\")}",
                div { class: "oxsst-rl", label { class: "oxsst-lb", r#for: "oxs6-ctx", "What should agents know about you?" } p { class: "oxsst-hint", "Used in every chat. Never shared outside the workspace." } }
                div { class: "oxsst-rc", {field("oxs6-ctx", "Personal context", &p.context, CONTEXT_CAP, true, None, "Your work, how you like answers, words to avoid", move |v| cur.write().context = v)} }
            }
        })}
        {sec("Sign-in", rsx! {}, rsx! {
            div { class: "oxsst-row oxs6-row",
                span { class: "oxs6-logo", BrandIcon { name: "google".to_string(), size: 16.0, coloured: true } }
                div { class: "oxsst-rl", span { class: "oxsst-lb", "Google" } p { class: "oxsst-hint oxv-ell", "Signed in as {PERSON_EMAIL}" } }
                span { class: "oxa-pill", "data-tone": "ok", "Connected" }
            }
            div { class: "oxsst-row oxs6-row", "data-dirty": "{ctx.dirty(|x| x.password)}", "data-hit": "{ctx.hit(\"Password\")}",
                span { class: "oxs6-logo", {nico("lock")} }
                div { class: "oxsst-rl", span { class: "oxsst-lb", "Password" } p { class: "oxsst-hint", if p.password { "Set. Change it any time." } else { "None yet: you sign in with Google only." } } }
                div { class: "oxsst-rc",
                    button { r#type: "button", class: "oxscn-btn", "aria-expanded": "{pw_open}", onclick: move |_| { pw.set(String::new()); pw2.set(String::new()); pw_open.toggle(); },
                        if p.password { "Change password" } else { "Add password" } }
                }
                if pw_open() {
                    div { class: "oxsst-ask oxs6-pw", role: "group", "aria-label": "New password",
                        onkeydown: move |e: KeyboardEvent| if e.key() == Key::Escape { pw_open.set(false); },
                        label { class: "oxsst-fl", "New password"
                            input { class: "oxsst-in", r#type: "password", value: "{pw}", "aria-describedby": "oxs6-meter", onmounted: focus_on, oninput: move |e| pw.set(e.value()) } }
                        div { class: "oxs6-meter", id: "oxs6-meter", "data-s": "{score}", "aria-live": "polite",
                            span { class: "oxs6-meter-b", for i in 0..4u8 { span { key: "{i}", "data-on": "{i < score}" } } }
                            span { class: "oxs6-q", if word.is_empty() { "8 characters or more; mix cases, digits and a symbol" } else { "{word}" } }
                        }
                        label { class: "oxsst-fl", "Type it again"
                            input { class: "oxsst-in", r#type: "password", value: "{pw2}", "aria-invalid": "{!pw2().is_empty() && pw2() != pw()}", oninput: move |e| pw2.set(e.value()) } }
                        div { class: "oxsin-acts",
                            button { r#type: "button", class: "oxscn-btn", onclick: move |_| pw_open.set(false), "Cancel" }
                            button { r#type: "button", class: "oxscn-primary", disabled: !ok,
                                onclick: move |_| { cur.write().password = true; pw_open.set(false); said.set("Password set: you can sign in with it or with Google".to_string()); }, "Save password" }
                        }
                    }
                }
            }
        })}
    }
}

// ────────────────────────────────────────────────────────── Preferences ──

#[component]
fn PrefsPage(ctx: Ctx) -> Element {
    let mut cur = ctx.cur;
    let mut said = ctx.said;
    let p = cur();
    let mut ask_clear = use_signal(|| false);
    let model_items: Vec<MenuItem> = DEFAULT_MODELS.iter().map(|m| (m.0, m.0, m.2)).collect();
    let (ml, mb, mh) = DEFAULT_MODELS[p.model];
    rsx! {
        {sec("Models", rsx! {}, rsx! {
            div { class: "oxsst-row oxs6-row", "data-dirty": "{ctx.dirty(|x| x.model)}", "data-hit": "{ctx.hit(\"Default model\")}",
                div { class: "oxsst-rl", span { class: "oxsst-lb", "Default model" } p { class: "oxsst-hint", "{mh}" } }
                div { class: "oxsst-rc",
                    MenuButton { label: ml.to_string(), aria: "Default model: {ml}", items: model_items, checked: ml.to_string(), right: true, caret: true, class: "oxsst-mb".to_string(), brand: mb.to_string(),
                        icon: if mb.is_empty() { "spark".to_string() } else { String::new() },
                        on_pick: move |k: String| if let Some(j) = DEFAULT_MODELS.iter().position(|m| m.0 == k) { cur.write().model = j; } }
                }
            }
            if p.model == 0 {
                div { class: "oxs6-auto-x", "aria-label": "How Auto picks",
                    for (i, (l, m)) in [("Quick question", 2usize), ("Long document", 3), ("Careful task", 1)].iter().copied().enumerate() {
                        div { key: "{i}", class: "oxs6-auto-r", style: "--i:{i}",
                            span { class: "oxv-ell", "{l}" } span { class: "oxs6-arrow", "aria-hidden": "true", "→" }
                            span { class: "oxs6-auto-m", {model_mark(m)} span { class: "oxv-ell", "{DEFAULT_MODELS[m].0}" } }
                        }
                    }
                }
            }
        })}
        {sec("Language and look", rsx! {}, rsx! {
            {row("oxs6-lang", "Language", "Menus and answers", choice("Language", &LANGS, p.lang, move |i| cur.write().lang = i), ctx.dirty(|x| x.lang), ctx.hit("Language"))}
            div { class: "oxsst-row oxs6-row", "data-dirty": "{ctx.dirty(|x| x.theme)}", "data-hit": "{ctx.hit(\"Appearance\")}",
                div { class: "oxsst-rl", span { class: "oxsst-lb", "Appearance" } p { class: "oxsst-hint", "System follows your device" } }
                div { class: "oxsst-rc",
                    div { class: "oxs6-themes", role: "radiogroup", "aria-label": "Appearance",
                        for (i, t) in THEMES.iter().copied().enumerate() {
                            button { key: "{t}", r#type: "button", class: "oxs6-theme", role: "radio", "aria-checked": "{p.theme == i}", "data-t": "{t}", onclick: move |_| cur.write().theme = i,
                                span { class: "oxs6-theme-p", "aria-hidden": "true", span {} span {} }
                                span { class: "oxs6-theme-l", "{t}" }
                            }
                        }
                    }
                }
            }
        })}
        {sec("Chat tools", rsx! { span { class: "oxs6-q", "{p.tools.iter().filter(|t| **t).count()} of 4 on" } }, rsx! {
            div { class: "oxs6-tiles", "data-hit": "{ctx.hit(\"Chat tools\")}",
                for (i, (k, l, w, g)) in TOOLS.iter().copied().enumerate() {
                    div { key: "{k}", class: "oxs6-tile", "data-on": "{p.tools[i]}", "data-dirty": "{ctx.dirty(move |x| x.tools[i])}",
                        span { class: "oxs6-tile-i", "data-k": "{k}", {ico(g)} }
                        span { class: "oxs6-tt", b { class: "oxv-ell", "{l}" } span { class: "oxs6-q oxv-ell", "{w}" } }
                        {switch(&format!("oxs6-tool-{k}"), l, p.tools[i], move |v| cur.write().tools[i] = v)}
                    }
                }
            }
        })}
        {sec("Custom instructions", rsx! {}, rsx! {
            div { class: "oxsst-row oxs6-row", "data-dirty": "{ctx.dirty(|x| x.instructions.clone())}", "data-hit": "{ctx.hit(\"Custom instructions\")}",
                div { class: "oxsst-rl", label { class: "oxsst-lb", r#for: "oxs6-ins", "How should it answer?" } p { class: "oxsst-hint", "Added to every chat you start" } }
                div { class: "oxsst-rc", {field("oxs6-ins", "Custom instructions", &p.instructions, INSTR_CAP, true, None, "Tone, length, what to always or never do", move |v| cur.write().instructions = v)} }
            }
            {row("oxs6-style", "Answer length", "", choice("Answer length", &STYLES, p.style, move |i| cur.write().style = i), ctx.dirty(|x| x.style), false)}
        })}
        {sec("Memory", rsx! {}, rsx! {
            div { class: "oxsst-row oxs6-row", "data-dirty": "{ctx.dirty(|x| x.memory)}", "data-hit": "{ctx.hit(\"Memory\")}",
                div { class: "oxsst-rl", span { class: "oxsst-lb", "Remember things across chats" } p { class: "oxsst-hint", "It saves short facts it learns and uses them later" } }
                div { class: "oxsst-rc", {switch("oxs6-mem", "Remember things across chats", p.memory, move |v| cur.write().memory = v)} }
            }
            if !p.memory {
                p { class: "oxsst-row oxs6-row oxs6-q", "Memory is off: nothing new is saved, and kept memories are not used." }
            } else if p.memories.is_empty() {
                p { class: "oxsst-row oxs6-row oxs6-q", "Nothing kept yet. Facts appear here as the assistant learns them." }
            } else {
                for m in p.memories.clone() {
                    div { key: "{m}", class: "oxsst-row oxs6-row oxs6-mem",
                        span { class: "oxs6-ic", {ico("brain")} }
                        span { class: "oxs6-grow oxv-ell", "{m}" }
                        button { r#type: "button", class: "oxscn-ib", "aria-label": "Forget “{m}”", onclick: move |_| { cur.write().memories.retain(|x| *x != m); said.set(format!("Forgot “{m}”")); }, {nico("close")} }
                    }
                }
                div { class: "oxsst-row oxs6-row",
                    span { class: "oxs6-q oxs6-grow", "{p.memories.len()} kept" }
                    button { r#type: "button", class: "oxscn-btn oxsst-danger", "aria-expanded": "{ask_clear}", onclick: move |_| ask_clear.set(true), "Clear all" }
                    if ask_clear() {
                        div { class: "oxsst-ask", role: "group", "aria-label": "Confirm clear all",
                            onkeydown: move |e: KeyboardEvent| if e.key() == Key::Escape { ask_clear.set(false); },
                            p { class: "oxsst-fl", "Forget all {p.memories.len()} memories? Chats stay; only these facts go." }
                            div { class: "oxsin-acts",
                                button { r#type: "button", class: "oxscn-btn", onclick: move |_| ask_clear.set(false), "Cancel" }
                                button { r#type: "button", class: "oxscn-btn oxsst-danger oxsst-go", onmounted: focus_on,
                                    onclick: move |_| { cur.write().memories.clear(); ask_clear.set(false); said.set("All memories cleared".to_string()); }, "Clear all" }
                            }
                        }
                    }
                }
            }
        })}
    }
}

// ──────────────────────────────────────────────────────── Notifications ──

#[component]
fn NotifPage(ctx: Ctx, quiet_now: bool) -> Element {
    let mut cur = ctx.cur;
    let mut said = ctx.said;
    let p = cur();
    let mut paused = use_signal(move || quiet_now);
    let mut tested = use_signal(|| false);
    rsx! {
        if paused() {
            div { class: "oxs6-banner", role: "status",
                span { class: "oxs6-moon", {ico("clock")} }
                span { class: "oxs6-tt", b { "Quiet hours are on" } span { class: "oxs6-q", "Until 07:00. Approvals still come through." } }
                span { class: "oxscn-grow" }
                button { r#type: "button", class: "oxscn-btn", onclick: move |_| { paused.set(false); said.set("Quiet hours ended early".to_string()); }, "End now" }
            }
        }
        {sec("Daily digest", rsx! { span { class: "oxs6-q", "{digest_words(&p)}" } }, rsx! {
            div { class: "oxsst-row oxs6-row", "data-dirty": "{ctx.dirty(|x| x.digest)}", "data-hit": "{ctx.hit(\"Daily digest\")}",
                div { class: "oxsst-rl", span { class: "oxsst-lb", "Send a digest" } p { class: "oxsst-hint", "What ran, what waits for you, what it cost" } }
                div { class: "oxsst-rc", {switch("oxs6-dig", "Send a digest", p.digest, move |v| cur.write().digest = v)} }
            }
            if p.digest {
                {row("oxs6-digat", "Send at", "Your local time", choice("Send at", &DIGEST_AT, p.digest_at, move |i| cur.write().digest_at = i), ctx.dirty(|x| x.digest_at), false)}
                div { class: "oxsst-row oxs6-row", "data-dirty": "{ctx.dirty(|x| x.digest_days)}",
                    div { class: "oxsst-rl", span { class: "oxsst-lb", "On" } }
                    div { class: "oxsst-rc",
                        div { class: "oxs6-days", role: "group", "aria-label": "Digest days",
                            for (i, d) in DAYS.iter().copied().enumerate() {
                                button { key: "{d}", r#type: "button", class: "oxs6-day", "aria-pressed": "{p.digest_days[i]}", "aria-label": "{d}", onclick: move |_| { let v = !cur.peek().digest_days[i]; cur.write().digest_days[i] = v; }, "{&d[..1]}" }
                            }
                        }
                    }
                }
            }
        })}
        {sec("What to tell you", rsx! {}, rsx! {
            div { class: "oxs6-grid", role: "table", "aria-label": "Notifications by kind and channel",
                div { class: "oxs6-gr oxs6-gh", role: "row",
                    span { role: "columnheader", class: "oxs6-gk", "Kind" }
                    for (c, l) in CHANNELS.iter().copied().enumerate() {
                        span { key: "{l}", role: "columnheader", class: "oxs6-gc",
                            button { r#type: "button", class: "oxs6-colb", "aria-pressed": "{col_all(&p.grid, c)}", title: "All on or off",
                                onclick: move |_| { toggle_col(&mut cur.write().grid, c); said.set(format!("{l}: {}", if col_all(&cur.peek().grid, c) { "all on" } else { "all off" })); }, "{l}" }
                        }
                    }
                }
                for (r, (l, w, g, tone)) in KINDS.iter().copied().enumerate() {
                    div { key: "{l}", class: "oxs6-gr", role: "row", "data-hit": "{ctx.hit(l)}", "data-dirty": "{ctx.dirty(move |x| x.grid[r])}",
                        span { role: "rowheader", class: "oxs6-gk",
                            span { class: "oxs6-kind-i", "data-tone": "{tone}", {nico(g)} }
                            span { class: "oxs6-tt", b { class: "oxv-ell", "{l}" } span { class: "oxs6-q oxv-ell", "{w} · {reach_words(&p.grid[r])}" } }
                        }
                        for (c, cl) in CHANNELS.iter().copied().enumerate() {
                            span { key: "{cl}", role: "cell", class: "oxs6-gc",
                                input { r#type: "checkbox", class: "oxs6-cb", checked: p.grid[r][c], "aria-label": "{l} by {cl}",
                                    onchange: move |e| cur.write().grid[r][c] = e.checked() }
                            }
                        }
                    }
                }
            }
        })}
        {sec("Quiet hours", rsx! {}, rsx! {
            div { class: "oxsst-row oxs6-row", "data-dirty": "{ctx.dirty(|x| x.quiet)}", "data-hit": "{ctx.hit(\"Quiet hours\")}",
                div { class: "oxsst-rl", span { class: "oxsst-lb", "Quiet hours" } p { class: "oxsst-hint", "{quiet_words(&p)}" } }
                div { class: "oxsst-rc", {switch("oxs6-quiet", "Quiet hours", p.quiet, move |v| cur.write().quiet = v)} }
            }
            if p.quiet {
                div { class: "oxsst-row oxs6-row", "data-dirty": "{ctx.dirty(|x| (x.quiet_from, x.quiet_to))}",
                    div { class: "oxsst-rl", span { class: "oxsst-lb", "From – to" } }
                    div { class: "oxsst-rc oxs6-gap",
                        {choice("Quiet from", &HOURS, p.quiet_from, move |i| cur.write().quiet_from = i)}
                        span { class: "oxs6-q", "–" }
                        {choice("Quiet to", &HOURS, p.quiet_to, move |i| cur.write().quiet_to = i)}
                    }
                }
                div { class: "oxsst-row oxs6-row", "data-dirty": "{ctx.dirty(|x| x.quiet_urgent)}",
                    div { class: "oxsst-rl", span { class: "oxsst-lb", "Let approvals through" } p { class: "oxsst-hint", "An agent waiting on you can still reach your phone" } }
                    div { class: "oxsst-rc", {switch("oxs6-urg", "Let approvals through", p.quiet_urgent, move |v| cur.write().quiet_urgent = v)} }
                }
            }
        })}
        div { class: "oxs6-foot",
            button { r#type: "button", class: "oxscn-btn", onclick: move |_| { tested.set(true); said.set("Test notification sent to your email, phone and inbox".to_string()); }, {nico("send")} span { if tested() { "Sent – check your phone" } else { "Send a test notification" } } }
        }
    }
}

// ────────────────────────────────────────────────────────────── Privacy ──

#[component]
fn PrivacyPage(ctx: Ctx, exporting: bool) -> Element {
    let mut cur = ctx.cur;
    let mut said = ctx.said;
    let p = cur();
    let mut tab = use_signal(|| "workspace");
    let mut copied = use_signal(|| 0u32);
    let mut export = use_signal(move || if exporting { Some(20u32) } else { None });
    let mut ask_del = use_signal(|| false);
    let mut typed = use_signal(String::new);
    let mut wiped = use_signal(|| false);
    let ws_n = links_in(&p.links, "workspace").len();
    let pub_n = links_in(&p.links, "public").len();
    let shown: Vec<_> = links_in(&p.links, tab()).into_iter().cloned().collect();
    let others = p.sessions.iter().filter(|s| !s.current).count();
    rsx! {
        {sec("How your data is used", rsx! {}, rsx! {
            for (k, l, h, v, d) in [
                ("train", "Improve the models", "Let chats from this account train future models", p.train, ctx.dirty(|x| x.train)),
                ("analytics", "Product analytics", "Anonymous usage that helps us fix what is slow or confusing", p.analytics, ctx.dirty(|x| x.analytics)),
                ("ads", "Ad measurement", "Whether an ad brought you here", p.ads, ctx.dirty(|x| x.ads)),
            ] {
                div { key: "{k}", class: "oxsst-row oxs6-row", "data-dirty": "{d}", "data-hit": "{ctx.hit(l)}",
                    div { class: "oxsst-rl", span { class: "oxsst-lb", "{l}" } p { class: "oxsst-hint", "{h}" } }
                    div { class: "oxsst-rc", {switch(&format!("oxs6-{k}"), l, v, move |b| { let mut w = cur.write(); match k { "train" => w.train = b, "analytics" => w.analytics = b, _ => w.ads = b } })} }
                }
            }
            {row("oxs6-cook", "Cookies", "Essential cookies keep you signed in", choice("Cookies", &COOKIES, p.cookies, move |i| cur.write().cookies = i), ctx.dirty(|x| x.cookies), ctx.hit("Cookies"))}
            div { class: "oxsst-row oxs6-row", "data-dirty": "{ctx.dirty(|x| x.product_mail)}",
                div { class: "oxsst-rl", span { class: "oxsst-lb", "Product emails" } p { class: "oxsst-hint", "Tips and new features, at most twice a month" } }
                div { class: "oxsst-rc", {switch("oxs6-pm", "Product emails", p.product_mail, move |v| cur.write().product_mail = v)} }
            }
        })}
        {sec("Active sessions", rsx! {
            if others > 0 { button { r#type: "button", class: "oxscn-link", onclick: move |_| { let n = sign_out_others(&mut cur.write().sessions); said.set(format!("Signed out of {n} other sessions")); }, "Sign out of all others" } }
        }, rsx! {
            for s in p.sessions.clone() {
                div { key: "{s.id}", class: "oxsst-row oxs6-row", "data-hit": "{ctx.hit(\"Active sessions\")}",
                    span { class: "oxs6-ic", {nico(s.glyph)} }
                    span { class: "oxs6-tt oxs6-grow", b { class: "oxv-ell", "{s.device}" } span { class: "oxs6-q oxv-ell", "{s.app} · {s.place} · {s.seen}" } }
                    if s.current { span { class: "oxa-pill", "data-tone": "ok", "This device" } }
                    else { button { r#type: "button", class: "oxscn-btn", onclick: move |_| { cur.write().sessions.retain(|x| x.id != s.id); said.set(format!("Signed out of {}", s.device)); }, "Sign out" } }
                }
            }
        })}
        {sec("Shared links", rsx! {}, rsx! {
            div { class: "oxs6-tabs", role: "tablist", "aria-label": "Shared links",
                for (k, l, n) in [("workspace", "In the workspace", ws_n), ("public", "Public links", pub_n)] {
                    button { key: "{k}", r#type: "button", role: "tab", class: "oxs6-tab", "aria-selected": "{tab() == k}", onclick: move |_| tab.set(k), "{l}" span { class: "oxscn-badge", "{n}" } }
                }
            }
            if shown.is_empty() {
                p { class: "oxsst-row oxs6-row oxs6-q", if tab() == "public" { "No public links. Anyone with a public link can read the chat." } else { "Nothing shared in the workspace yet." } }
            }
            for l in shown {
                {
                let (lid, ltitle, url) = (l.id, l.title, link_url(&l));
                rsx! {
                div { key: "{lid}", class: "oxsst-row oxs6-row", "data-hit": "{ctx.hit(\"Shared links\")}",
                    span { class: "oxs6-ic", {nico(if l.scope == "public" { "globe" } else { "people" })} }
                    span { class: "oxs6-tt oxs6-grow", b { class: "oxv-ell", "{l.title}" } span { class: "oxs6-q oxv-ell", title: "{link_url(&l)}", "{l.views} views · shared {l.when}" } }
                    button { r#type: "button", class: "oxscn-btn", onclick: move |_| { copied.set(lid); said.set(format!("Link copied: {url}")); }, {nico("copy")} span { if copied() == lid { "Copied" } else { "Copy link" } } }
                    button { r#type: "button", class: "oxscn-ib", "aria-label": "Stop sharing {ltitle}", title: "Stop sharing", onclick: move |_| { cur.write().links.retain(|x| x.id != lid); said.set(format!("Stopped sharing {ltitle}")); }, {nico("close")} }
                }
                }
                }
            }
        })}
        {sec("Your data", rsx! {}, rsx! {
            div { class: "oxsst-row oxs6-row", "data-hit": "{ctx.hit(\"Export your data\")}",
                div { class: "oxsst-rl", span { class: "oxsst-lb", "Export your data" } p { class: "oxsst-hint", "Chats, files and memories as one archive, by email" } }
                div { class: "oxsst-rc",
                    button { r#type: "button", class: "oxscn-btn", disabled: export().is_some_and(|x| x < 100), onclick: move |_| { export.set(Some(0)); said.set("Export started".to_string()); }, {nico("download")} span { "Export" } }
                }
                if let Some(x) = export() {
                    {
                        let (pc, step, done) = export_step(x);
                        rsx! {
                            div { class: "oxs6-prog", role: "progressbar", "aria-label": "Export", "aria-valuemin": "0", "aria-valuemax": "100", "aria-valuenow": "{pc}",
                                span { class: "oxs6-prog-t", "data-done": "{done}", if done { {ico("check")} } "{step}" span { class: "oxs6-q", " {pc}%" } }
                                span { class: "oxs6-prog-bar", span { style: "width:{pc}%" } }
                                if !done {
                                    for k in [x] {
                                        span { key: "{k}", class: "oxs6-tick", onanimationend: move |_| { let n = (k + 20).min(100); export.set(Some(n)); if n >= 100 { said.set("Export ready: the link is on its way".to_string()); } } }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            div { class: "oxsst-row oxs6-row",
                div { class: "oxsst-rl", span { class: "oxsst-lb", "Delete all chats" } p { class: "oxsst-hint", if wiped() { "Done: every chat is gone. Shared links stopped working." } else { "Removes every chat and its shared links. Memories stay." } } }
                div { class: "oxsst-rc",
                    button { r#type: "button", class: "oxscn-btn oxsst-danger", disabled: wiped(), "aria-expanded": "{ask_del}", onclick: move |_| { typed.set(String::new()); ask_del.set(true); }, "Delete all chats" }
                }
                if ask_del() {
                    div { class: "oxsst-ask", role: "group", "aria-label": "Confirm delete all chats",
                        onkeydown: move |e: KeyboardEvent| if e.key() == Key::Escape { ask_del.set(false); },
                        label { class: "oxsst-fl", "Type delete to confirm"
                            input { class: "oxsst-in", value: "{typed}", "aria-label": "Type delete", onmounted: focus_on, oninput: move |e| typed.set(e.value()) } }
                        div { class: "oxsin-acts",
                            button { r#type: "button", class: "oxscn-btn", onclick: move |_| ask_del.set(false), "Cancel" }
                            button { r#type: "button", class: "oxscn-btn oxsst-danger oxsst-go", disabled: !confirm_ok(&typed(), "delete"),
                                onclick: move |_| { ask_del.set(false); wiped.set(true); cur.write().links.clear(); said.set("All chats deleted (sample, nothing was removed)".to_string()); }, "Delete all chats" }
                        }
                    }
                }
            }
        })}
    }
}

// ───────────────────────────────────────────────────────────── General ──

#[component]
fn GeneralPage(ctx: Ctx, deleting: bool) -> Element {
    let mut cur = ctx.cur;
    let mut said = ctx.said;
    let p = cur();
    let mut ask = use_signal(move || deleting);
    let mut typed = use_signal(move || {
        if deleting {
            "Kestrel".to_string()
        } else {
            String::new()
        }
    });
    let mut gone = use_signal(|| false);
    let mut follow = use_signal(|| true);
    let ws_ini = initials(&p.ws_name);
    let name_now = if p.ws_name.trim().is_empty() {
        WORKSPACE.to_string()
    } else {
        p.ws_name.trim().to_string()
    };
    let suggest = slug_from(&p.ws_name);
    rsx! {
        {sec("Workspace", rsx! {}, rsx! {
            div { class: "oxsst-row oxs6-row", "data-dirty": "{ctx.dirty(|x| x.ws_hue)}",
                div { class: "oxsst-rl", span { class: "oxsst-lb", "Icon" } p { class: "oxsst-hint", "Its initials on a colour of your choice" } }
                div { class: "oxsst-rc oxs6-gap",
                    span { class: "oxs6-wsi", "data-hue": "{WS_HUES[p.ws_hue]}", "aria-hidden": "true", "{ws_ini}" }
                    div { class: "oxs6-hues", role: "radiogroup", "aria-label": "Icon colour",
                        for (i, h) in WS_HUES.iter().copied().enumerate() {
                            button { key: "{h}", r#type: "button", class: "oxs6-hue", role: "radio", "data-hue": "{h}", "aria-checked": "{p.ws_hue == i}", "aria-label": "{h}", onclick: move |_| cur.write().ws_hue = i }
                        }
                    }
                }
            }
            {row("oxs6-wsn", "Workspace name", "Shown to everyone in it and on invites",
                field("oxs6-wsn", "Workspace name", &p.ws_name, 48, false, ws_name_error(&p.ws_name), "Your company or team", move |v| {
                    let s = slug_from(&v);
                    let mut w = cur.write();
                    if *follow.peek() && !s.is_empty() { w.ws_slug = s; }
                    w.ws_name = v;
                }),
                ctx.dirty(|x| x.ws_name.clone()), ctx.hit("Workspace name"))}
            div { class: "oxsst-row oxs6-row", "data-dirty": "{ctx.dirty(|x| x.ws_slug.clone())}", "data-hit": "{ctx.hit(\"Workspace address\")}",
                div { class: "oxsst-rl", label { class: "oxsst-lb", r#for: "oxs6-slug", "Workspace address" } p { class: "oxsst-hint", "Changing it breaks old links" } }
                div { class: "oxsst-rc",
                    div { class: "oxsst-tf",
                        div { class: "oxs6-url", "aria-invalid": "{slug_error(&p.ws_slug).is_some()}",
                            input { class: "oxs6-url-in", id: "oxs6-slug", value: "{p.ws_slug}", "aria-label": "Workspace address", spellcheck: "false",
                                oninput: move |e| { follow.set(false); cur.write().ws_slug = e.value(); } }
                            span { class: "oxs6-url-s", ".oxyn.app" }
                        }
                        div { class: "oxsst-tfm",
                            if let Some(m) = slug_error(&p.ws_slug) {
                                span { class: "oxsst-err", "{m}" }
                                if !suggest.is_empty() && suggest != p.ws_slug && slug_error(&suggest).is_none() {
                                    button { r#type: "button", class: "oxscn-link", onclick: move |_| { let s = slug_from(&cur.peek().ws_name); cur.write().ws_slug = s; }, "Use {suggest}" }
                                }
                            } else if !p.ws_slug.is_empty() {
                                span { class: "oxs6-ok", {ico("check")} "Available" }
                            }
                        }
                    }
                }
            }
        })}
        {sec("Company context", rsx! {}, rsx! {
            div { class: "oxsst-row oxs6-row", "data-dirty": "{ctx.dirty(|x| x.company.clone())}",
                div { class: "oxsst-rl", label { class: "oxsst-lb", r#for: "oxs6-co", "What every agent here should know" } p { class: "oxsst-hint", "Products, customers, words you use. Members can read it." } }
                div { class: "oxsst-rc", {field("oxs6-co", "Company context", &p.company, COMPANY_CAP, true, None, "What the company does, who it serves, how it talks", move |v| cur.write().company = v)} }
            }
            {row("oxs6-share", "New chats are visible to", "People can still share one chat at a time", choice("New chats are visible to", &SHARING, p.sharing, move |i| cur.write().sharing = i), ctx.dirty(|x| x.sharing), false)}
        })}
        {sec("Danger zone", rsx! {}, rsx! {
            div { class: "oxsst-row oxs6-row", "data-hit": "{ctx.hit(\"Delete workspace\")}",
                div { class: "oxsst-rl", span { class: "oxsst-lb", "Delete workspace" } p { class: "oxsst-hint", if gone() { "Scheduled: it goes in 14 days unless an owner restores it." } else { "Chats, files, agents and members go with it, after 14 days." } } }
                div { class: "oxsst-rc",
                    button { r#type: "button", class: "oxscn-btn oxsst-danger", disabled: gone(), "aria-expanded": "{ask}", onclick: move |_| { typed.set(String::new()); ask.set(true); }, "Delete workspace" }
                }
                if ask() {
                    div { class: "oxsst-ask", role: "group", "aria-label": "Confirm delete workspace",
                        onkeydown: move |e: KeyboardEvent| if e.key() == Key::Escape { ask.set(false); },
                        label { class: "oxsst-fl", "Type {name_now} to confirm"
                            input { class: "oxsst-in", value: "{typed}", "aria-label": "Workspace name", onmounted: focus_on, oninput: move |e| typed.set(e.value()) } }
                        div { class: "oxsin-acts",
                            button { r#type: "button", class: "oxscn-btn", onclick: move |_| ask.set(false), "Cancel" }
                            button { r#type: "button", class: "oxscn-btn oxsst-danger oxsst-go", disabled: !confirm_ok(&typed(), &name_now),
                                onclick: move |_| { ask.set(false); gone.set(true); said.set("Workspace scheduled for deletion (sample, nothing was removed)".to_string()); }, "Delete workspace" }
                        }
                    }
                }
            }
        })}
    }
}

// ───────────────────────────────────────────────────────────── Screens ──

macro_rules! s6_screen {
    ($(#[$m:meta])* $name:ident, $slug:literal) => {
        $(#[$m])*
        #[component]
        pub fn $name(
            /// `inset` or `flat`.
            #[props(default = "inset".to_string())]
            layout: String,
            /// A state key (`sample`, `loading`, `empty` or the screen's own state).
            #[props(default = "sample".to_string())]
            state: String,
            /// Height px.
            #[props(default = 640)]
            height: u32,
            /// Oxyn: tighter rows.
            #[props(default)]
            compact: bool,
            /// Oxyn: every valid change is kept at once (no save bar).
            #[props(default)]
            autosave: bool,
            /// Back to app pressed.
            #[props(default)]
            on_back: EventHandler<()>,
        ) -> Element {
            rsx! { SetShell { layout, state, height, slug: $slug.to_string(), compact, autosave, preview: $slug == "settings-profile" && compact, on_back } }
        }
    };
}

s6_screen!(
    /// Settings / Profile: picture, name, role and email rows (email changes are confirmed by
    /// link), time zone and format, personal context, and sign-in (Google connected, Add or
    /// Change password with a strength meter). State `unsaved` opens with pending edits.
    ProfileSettingsScreen,
    "settings-profile"
);
s6_screen!(
    /// Settings / Preferences: default model (Auto explains how it picks; official model marks),
    /// language, appearance tiles, chat tools as tiles with switches, custom instructions and
    /// answer length, memory with kept facts, Forget and Clear all. State `memoff`.
    PreferencesSettingsScreen,
    "settings-preferences"
);
s6_screen!(
    /// Settings / Notifications: the daily digest (time, days), a kinds × channels grid (a column
    /// header toggles the column), quiet hours with what still comes through, and a test send.
    /// State `quiet` shows quiet hours in force with End now.
    NotificationsSettingsScreen,
    "settings-notifications"
);
s6_screen!(
    /// Settings / Privacy: data-use switches, cookies and product emails, active sessions with
    /// sign-out (one or all others), shared links by scope with Copy link and Stop sharing, Export
    /// (a stepped progress) and Delete all chats (type delete). State `exporting`.
    PrivacySettingsScreen,
    "settings-privacy"
);
s6_screen!(
    /// Settings / General: workspace icon (initials on an identity hue), name, address with live
    /// checks and a suggestion, company context, default sharing, and Delete workspace (type the
    /// name). State `deleting` opens the confirmation.
    GeneralSettingsScreen,
    "settings-general"
);
