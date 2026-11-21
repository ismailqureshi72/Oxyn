//! Screens S7: Settings II – Members, Models, Security, Billing and the Get started checklist – as
//! pages of the S6 settings shell (`setscr.rs`). Policies (model switches and access, defaults,
//! sign-in rules, billing interval and cap, the checklist's visibility) go through the shell's save
//! bar (or save at once with `autosave`); lists (members, keys, passkeys, domains, devices, steps)
//! act at once and announce what they did. Rules live in `oxyn_core::kit::setscr2`. Styles in
//! `setscr2.css` (`oxs7-`), on top of the settings rows (`oxsst-`), S6 parts (`oxs6-`) and shell
//! parts (`oxscn-`). Chrome stays monochrome; colour is for official marks, identity hues and status.

use dioxus::prelude::*;
use oxyn_core::kit::setscr2::{
    ACCESS, AUDIT_KINDS, CANCEL_REASONS, CAPS, INTERVALS, INVOICES, MODELS, NETWORKS, PLANS,
    PROVIDERS, ROLES, SEATS, SESSION_LEN, STEPS, TEAMS, TOTP_SECRET, Ws, audit, cancel_next,
    card_network, card_number_ok, chat_on, code_ok, cvc_ok, demo_qr, dollars, domain_error,
    expiry_ok, filter, invite, invite_error, key_error, mask, meter_tone, meter_words, meters,
    models_in, next_bill, next_step, owners, progress, progress_words, provider_on, recovery_codes,
    remove_error, role_error, rotated, seat_price, seats_used, seats_words, set_model, team_size,
    txt_record,
};

use super::screensfid::{MenuButton, MenuItem, ico, nico};
use super::setscr::{Ctx, choice, focus_on, row, sec, switch};
use crate::brand::BrandIcon;

const HUES: [&str; 6] = ["blue", "purple", "teal", "orange", "pink", "green"];

/// Apply a list change to both the edited and the saved copy (lists act at once).
fn now(ctx: Ctx, mut f: impl FnMut(&mut Ws)) {
    let (mut c, mut s) = (ctx.cur, ctx.saved);
    f(&mut c.write().ws);
    f(&mut s.write().ws);
}

fn say(ctx: Ctx, m: impl Into<String>) {
    let mut s = ctx.said;
    s.set(m.into());
}

fn ini(name: &str) -> String {
    oxyn_core::kit::setscr::initials(name)
}

/// A labelled meter bar with its tone.
fn bar(pct: f32, tone: &str) -> Element {
    let w = (pct * 100.0).clamp(0.0, 100.0);
    rsx! { span { class: "oxs7-bar", "data-tone": "{tone}", span { style: "width:{w:.0}%" } } }
}

fn empty(glyph: &str, title: &str, line: &str, act: Element) -> Element {
    rsx! {
        div { class: "oxs7-empty",
            span { class: "oxs6-ic", {nico(glyph)} }
            b { "{title}" }
            p { class: "oxsst-hint", "{line}" }
            {act}
        }
    }
}

// ───────────────────────────────────────────────────────────── Members ──

#[component]
pub(super) fn MembersPage(ctx: Ctx, inviting: bool) -> Element {
    let p = ctx.cur.read().ws.clone();
    let mut tab = use_signal(|| "members");
    let mut q = use_signal(String::new);
    let mut role_f = use_signal(|| None::<usize>);
    let mut open = use_signal(move || inviting);
    let mut typed = use_signal(move || {
        if inviting {
            "lena@kestrel.example, arlo@kestrel.example".to_string()
        } else {
            String::new()
        }
    });
    let mut new_role = use_signal(|| 2usize);
    let mut ask_rm = use_signal(|| None::<u32>);
    let list = p.members.clone();
    let joined = list.iter().filter(|m| !m.invited).count();
    let invited = list.len() - joined;
    let admins = list.iter().filter(|m| m.role <= 1 && !m.invited).count();
    let used = seats_used(&list);
    let shown = filter(&list, &q(), role_f());
    let err = invite_error(&typed(), &list);
    let only_me = list.len() == 1;
    let role_items: Vec<MenuItem> = std::iter::once(("all", "All roles", ""))
        .chain(ROLES.iter().map(|r| (r.0, r.0, "")))
        .collect();
    let rf_label = role_f().map(|i| ROLES[i].0).unwrap_or("All roles");
    let t = tab();
    rsx! {
        div { class: "oxs6-tabs oxs7-tabs", role: "tablist", "aria-label": "Members and teams",
            for (k, l) in [("members", "Members"), ("teams", "Teams")] {
                button { key: "{k}", r#type: "button", class: "oxs6-tab", role: "tab", "aria-selected": "{t == k}", onclick: move |_| tab.set(k), "{l}" }
            }
        }
        div { class: "oxs7-stats",
            div { class: "oxs7-stat", span { class: "oxs7-k", "Members" } b { "{joined}" } span { class: "oxs6-q", "{joined} joined · {invited} invited" } }
            div { class: "oxs7-stat", "data-hit": "{ctx.hit(\"Seats\")}", span { class: "oxs7-k", "Seats" } b { "{used} " span { class: "oxs6-q", "of {SEATS}" } } {bar(used as f32 / SEATS as f32, meter_tone(used as f32, SEATS as f32))} }
            div { class: "oxs7-stat", span { class: "oxs7-k", "Admins" } b { "{admins}" } span { class: "oxs6-q", "Can invite, remove and bill" } }
        }
        if t == "members" {
            div { class: "oxs7-bar-row",
                label { class: "oxscn-search oxs7-search",
                    {ico("search")}
                    input { r#type: "search", placeholder: "Search name or email", "aria-label": "Search members", value: "{q}", oninput: move |e| q.set(e.value()) }
                }
                MenuButton { label: rf_label.to_string(), aria: "Role filter: {rf_label}", items: role_items, checked: rf_label.to_string(), caret: true, icon: "filter".to_string(), class: "oxsst-mb".to_string(),
                    on_pick: move |k: String| role_f.set(ROLES.iter().position(|r| r.0 == k)) }
                span { class: "oxscn-grow" }
                button { r#type: "button", class: "oxscn-btn", onclick: move |_| say(ctx, "Exported 7 members as CSV (sample)"), {nico("download")} span { "Export" } }
                button { r#type: "button", class: "oxscn-primary", "aria-expanded": "{open}", "data-hit": "{ctx.hit(\"Invite members\")}", onclick: move |_| open.toggle(), {nico("plus")} span { "Invite members" } }
            }
            if open() {
                div { class: "oxsst-ask oxs7-invite", role: "group", "aria-label": "Invite members",
                    onkeydown: move |e: KeyboardEvent| if e.key() == Key::Escape { open.set(false); },
                    label { class: "oxsst-fl", "Email addresses"
                        textarea { class: "oxsst-in oxsst-ta", rows: "2", value: "{typed}", placeholder: "name@company.com, another@company.com", "aria-invalid": "{err.is_some() && !typed().is_empty()}", onmounted: focus_on, oninput: move |e| typed.set(e.value()) }
                    }
                    div { class: "oxs7-inv-r",
                        span { class: "oxs6-q", "Role" }
                        {choice("Role for new members", &["Owner", "Admin", "Member", "Viewer"], new_role(), move |i| new_role.set(i))}
                        span { class: "oxs6-q oxs7-grow", "{ROLES[new_role()].1}" }
                    }
                    div { class: "oxsst-tfm",
                        if let Some(m) = err.as_ref().filter(|_| !typed().is_empty()) { span { class: "oxsst-err", "{m}" } }
                        else { span { class: "oxs6-q", "{seats_words(&list)}" } }
                    }
                    div { class: "oxsin-acts",
                        button { r#type: "button", class: "oxscn-btn", onclick: move |_| open.set(false), "Cancel" }
                        button { r#type: "button", class: "oxscn-primary", disabled: err.is_some(),
                            onclick: move |_| {
                                let tx = typed.peek().clone();
                                let r = *new_role.peek();
                                let mut n = 0;
                                now(ctx, |w| n = invite(&mut w.members, &tx, r));
                                say(ctx, if n == 1 { "1 invite sent".to_string() } else { format!("{n} invites sent") });
                                typed.set(String::new());
                                open.set(false);
                            }, "Send invites" }
                    }
                }
            }
            if only_me {
                {empty("people", "Just you so far", "Invite people by email; each one takes a seat on the plan.", rsx! {
                    button { r#type: "button", class: "oxscn-btn", onclick: move |_| open.set(true), {nico("plus")} span { "Invite members" } }
                })}
            }
            div { class: "oxsst-card oxs7-table", role: "table", "aria-label": "Members", "data-hit": "{ctx.hit(\"Member roles\")}",
                div { class: "oxs7-tr oxs7-th", role: "row",
                    span { role: "columnheader", "Name" } span { role: "columnheader", "Role" } span { role: "columnheader", class: "oxs7-c-team", "Team" } span { role: "columnheader", class: "oxs7-c-use", "Usage" } span { role: "columnheader", span { class: "oxscn-sr", "Actions" } }
                }
                if shown.is_empty() {
                    p { class: "oxa-mute oxs7-none", "No one matches. Clear the search or the role filter." }
                }
                for mb in shown.iter().map(|x| (*x).clone()) {
                    {
                        let id = mb.id;
                        let me = mb.is_me();
                        let label = mb.label().to_string();
                        let hue = HUES[(id as usize) % HUES.len()];
                        let rl = ROLES[mb.role].0;
                        let items: Vec<MenuItem> = ROLES.iter().map(|r| (r.0, r.0, "")).collect();
                        let more: Vec<MenuItem> = if mb.invited { vec![("resend", "Resend invite", ""), ("remove", "Revoke invite", "")] } else if me { vec![("profile", "Edit your profile", "")] } else { vec![("remove", "Remove from workspace", "")] };
                        let tone = meter_tone(f32::from(mb.usage), 100.0);
                        let team = mb.team.map(|t| TEAMS[t].0).unwrap_or("—");
                        let lb2 = label.clone();
                        rsx! {
                            div { key: "{id}", class: "oxs7-tr", role: "row", "data-invited": "{mb.invited}",
                                span { class: "oxs7-who", role: "cell",
                                    if mb.invited { span { class: "oxs7-av", "data-inv": "true", {nico("mail")} } }
                                    else { span { class: "oxs7-av", "data-hue": "{hue}", "{ini(&mb.name)}" } }
                                    span { class: "oxs6-tt",
                                        b { class: "oxv-ell", "{label}" if me { span { class: "oxs6-q", " (you)" } } }
                                        if mb.invited { span { class: "oxv-ell oxs6-q", "{mb.seen}" } } else { span { class: "oxv-ell oxs6-q", "{mb.email}" } }
                                    }
                                    if mb.invited { span { class: "oxa-pill", "Invited" } }
                                }
                                span { role: "cell",
                                    MenuButton { label: rl.to_string(), aria: "Role of {label}: {rl}", items, checked: rl.to_string(), caret: true, class: "oxsst-mb oxs7-role".to_string(),
                                        on_pick: move |k: String| {
                                            let Some(r) = ROLES.iter().position(|x| x.0 == k) else { return };
                                            let e = role_error(&ctx.cur.peek().ws.members, id, r);
                                            match e {
                                                Some(m) => say(ctx, m),
                                                None => { now(ctx, |w| if let Some(x) = w.members.iter_mut().find(|x| x.id == id) { x.role = r; }); say(ctx, format!("{lb2} is now {k}")); }
                                            }
                                        } }
                                }
                                span { class: "oxs7-c-team oxs6-q oxv-ell", role: "cell", "{team}" }
                                span { class: "oxs7-c-use oxs7-use", role: "cell",
                                    if mb.invited { span { class: "oxs6-q", "—" } } else { {bar(f32::from(mb.usage) / 100.0, tone)} span { class: "oxs6-q", "{mb.usage}%" } }
                                }
                                span { role: "cell", class: "oxs7-more",
                                    MenuButton { label: String::new(), aria: "More for {label}", items: more, checked: String::new(), right: true, icon: "more".to_string(), class: "oxsst-mb oxs7-dots".to_string(),
                                        on_pick: move |k: String| match k.as_str() {
                                            "resend" => say(ctx, "Invite sent again"),
                                            "remove" => {
                                                if let Some(m) = remove_error(&ctx.cur.peek().ws.members, id) { say(ctx, m); } else { ask_rm.set(Some(id)); }
                                            }
                                            _ => say(ctx, "Profile is under Personal"),
                                        } }
                                }
                                if ask_rm() == Some(id) {
                                    div { class: "oxsst-ask oxs7-rm", role: "group", "aria-label": "Confirm remove",
                                        onkeydown: move |e: KeyboardEvent| if e.key() == Key::Escape { ask_rm.set(None); },
                                        span { class: "oxs6-q", if mb.invited { "Revoke the invite for {label}? The link stops working." } else { "Remove {label}? Their chats stay in the workspace; their seat frees up." } }
                                        div { class: "oxsin-acts",
                                            button { r#type: "button", class: "oxscn-btn", onmounted: focus_on, onclick: move |_| ask_rm.set(None), "Keep" }
                                            button { r#type: "button", class: "oxscn-btn oxsst-danger oxsst-go",
                                                onclick: move |_| { now(ctx, |w| w.members.retain(|x| x.id != id)); ask_rm.set(None); say(ctx, "Removed; 1 seat is free"); },
                                                if mb.invited { "Revoke" } else { "Remove" } }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            p { class: "oxs6-note", "Owners: {owners(&list)} · roles: Owner, Admin, Member, Viewer. Seats come from the plan on Billing." }
        } else {
            div { class: "oxs7-teams", "data-hit": "{ctx.hit(\"Teams\")}",
                for (ti, (tn, th)) in TEAMS.iter().copied().enumerate() {
                    {
                        let inn: Vec<_> = list.iter().filter(|m| m.team == Some(ti)).cloned().collect();
                        let out: Vec<MenuItem> = list.iter().filter(|m| m.team != Some(ti) && !m.invited).map(|m| (if m.is_me() { "me" } else { m.name }, if m.is_me() { "You" } else { m.name }, "")).collect();
                        let n = team_size(&list, ti);
                        rsx! {
                            div { key: "{tn}", class: "oxs7-team", "data-hue": "{th}",
                                div { class: "oxs7-team-h",
                                    span { class: "oxs7-th-dot" }
                                    b { "{tn}" }
                                    span { class: "oxs6-q", if n == 1 { "1 person" } else { "{n} people" } }
                                    span { class: "oxscn-grow" }
                                    if !out.is_empty() {
                                        MenuButton { label: "Add".to_string(), aria: "Add someone to {tn}", items: out, checked: String::new(), right: true, icon: "plus".to_string(), class: "oxsst-mb".to_string(),
                                            on_pick: move |k: String| {
                                                now(ctx, |w| if let Some(x) = w.members.iter_mut().find(|x| (k == "me" && x.is_me()) || x.name == k) { x.team = Some(ti); });
                                                say(ctx, format!("Added to {tn}"));
                                            } }
                                    }
                                }
                                div { class: "oxs7-team-b",
                                    if inn.is_empty() { span { class: "oxs6-q", "No one yet" } }
                                    for x in inn.iter() {
                                        span { key: "{x.id}", class: "oxs7-chip",
                                            span { class: "oxs7-av oxs7-av-s", "data-hue": "{HUES[(x.id as usize) % HUES.len()]}", "{ini(x.label())}" }
                                            span { class: "oxv-ell", "{x.label()}" }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

// ────────────────────────────────────────────────────────────── Models ──

#[component]
pub(super) fn ModelsPage(ctx: Ctx, newkey: bool) -> Element {
    let p = ctx.cur.read().ws.clone();
    let mut kt = use_signal(|| "all");
    let mut q = use_signal(String::new);
    let mut shut = use_signal(Vec::<usize>::new);
    let mut reveal = use_signal(move || if newkey { vec![1u32] } else { Vec::new() });
    let mut fresh = use_signal(move || if newkey { Some(1u32) } else { None });
    let mut ask_rot = use_signal(|| None::<u32>);
    let mut adding = use_signal(|| false);
    let mut k_prov = use_signal(|| 0usize);
    let mut k_name = use_signal(String::new);
    let mut k_sec = use_signal(String::new);
    let mut rot_n = use_signal(|| 1u32);
    let chat: Vec<usize> = chat_on(&p.models_on);
    let mut chat_items: Vec<MenuItem> = vec![("Auto", "Auto", "")];
    chat_items.extend(
        chat.iter()
            .map(|i| (MODELS[*i].0, MODELS[*i].0, MODELS[*i].3)),
    );
    let cl = p.chat_default.map(|i| MODELS[i].0).unwrap_or("Auto");
    let cb = p.chat_default.map(|i| MODELS[i].3).unwrap_or("");
    let imgs: Vec<usize> = (0..MODELS.len())
        .filter(|i| MODELS[*i].2 == "image" && p.models_on[*i])
        .collect();
    let mut img_items: Vec<MenuItem> = vec![("Off", "Off", "")];
    img_items.extend(
        imgs.iter()
            .map(|i| (MODELS[*i].0, MODELS[*i].0, MODELS[*i].3)),
    );
    let il = p.image_default.map(|i| MODELS[i].0).unwrap_or("Off");
    let ib = p.image_default.map(|i| MODELS[i].3).unwrap_or("");
    let ke = key_error(&k_name(), &k_sec());
    let t = kt();
    let mut cur = ctx.cur;
    rsx! {
        {sec("Defaults", rsx! {}, rsx! {
            div { class: "oxsst-row oxs6-row", "data-dirty": "{ctx.dirty(|x| x.ws.chat_default)}", "data-hit": "{ctx.hit(\"Chat model\")}",
                div { class: "oxsst-rl", span { class: "oxsst-lb", "Chat model" } p { class: "oxsst-hint", "Every new chat starts with it; people can pick their own." } }
                div { class: "oxsst-rc",
                    MenuButton { label: cl.to_string(), aria: "Chat model: {cl}", items: chat_items, checked: cl.to_string(), right: true, caret: true, class: "oxsst-mb".to_string(), brand: cb.to_string(),
                        icon: if cb.is_empty() { "spark".to_string() } else { String::new() },
                        on_pick: move |k: String| cur.write().ws.chat_default = MODELS.iter().position(|m| m.0 == k) }
                }
            }
            div { class: "oxsst-row oxs6-row", "data-dirty": "{ctx.dirty(|x| x.ws.image_default)}", "data-hit": "{ctx.hit(\"Image model\")}",
                div { class: "oxsst-rl", span { class: "oxsst-lb", "Image model" } p { class: "oxsst-hint", "Used when someone asks for a picture." } }
                div { class: "oxsst-rc",
                    MenuButton { label: il.to_string(), aria: "Image model: {il}", items: img_items, checked: il.to_string(), right: true, caret: true, class: "oxsst-mb".to_string(), brand: ib.to_string(),
                        icon: if ib.is_empty() { "image".to_string() } else { String::new() },
                        on_pick: move |k: String| cur.write().ws.image_default = MODELS.iter().position(|m| m.0 == k) }
                }
            }
        })}
        div { class: "oxs7-bar-row oxs7-mt",
            div { class: "oxscn-seg oxs7-seg", role: "radiogroup", "aria-label": "Kind",
                for (k, l) in [("all", "All"), ("chat", "Chat"), ("image", "Image")] {
                    button { key: "{k}", r#type: "button", class: "oxscn-seg-b", role: "radio", "aria-checked": "{t == k}", "aria-selected": "{t == k}", onclick: move |_| kt.set(k), "{l}" }
                }
            }
            span { class: "oxscn-grow" }
            label { class: "oxscn-search oxs7-search",
                {ico("search")}
                input { r#type: "search", placeholder: "Search models", "aria-label": "Search models", value: "{q}", oninput: move |e| q.set(e.value()) }
            }
        }
        for (pi, (pb, pn, ph)) in PROVIDERS.iter().copied().enumerate() {
            {
                let ms = models_in(pi, t, &q());
                let (on, all) = provider_on(&p.models_on, pi);
                let closed = shut().contains(&pi);
                let tone = if on == 0 { "" } else { "ok" };
                rsx! {
                    if !ms.is_empty() {
                        section { key: "{pb}", class: "oxsst-sec oxs7-prov", "data-hit": "{ctx.hit(\"Providers\")}",
                            div { class: "oxsst-card",
                                div { class: "oxsst-row oxs6-row oxs7-ph",
                                    span { class: "oxs6-logo", BrandIcon { name: pb.to_string(), size: 16.0, coloured: true } }
                                    div { class: "oxsst-rl", span { class: "oxsst-lb", "{pn}" } p { class: "oxsst-hint", "{ph}" } }
                                    span { class: "oxa-pill", "data-tone": "{tone}", "{on}/{all} on" }
                                    button { r#type: "button", class: "oxa-x oxs7-chev", "data-open": "{!closed}", "aria-expanded": "{!closed}", "aria-label": "Show {pn} models",
                                        onclick: move |_| { let mut s = shut.write(); if let Some(j) = s.iter().position(|x| *x == pi) { s.remove(j); } else { s.push(pi); } }, {nico("caret")} }
                                }
                                if !closed {
                                    for i in ms.iter().copied() {
                                        {
                                            let (mn, _, kind, mk, ml) = MODELS[i];
                                            let mon = p.models_on[i];
                                            let acc = p.access[i];
                                            let is_def = p.chat_default == Some(i) || p.image_default == Some(i);
                                            rsx! {
                                                div { key: "{mn}", class: "oxsst-row oxs6-row oxs7-model", "data-off": "{!mon}", "data-dirty": "{ctx.dirty(move |x| (x.ws.models_on[i], x.ws.access[i]))}",
                                                    span { class: "oxs7-mk", BrandIcon { name: mk.to_string(), size: 15.0, coloured: true } }
                                                    div { class: "oxsst-rl", span { class: "oxsst-lb", "{mn}" if kind == "image" { span { class: "oxa-pill oxs7-kind", "Image" } } if is_def { span { class: "oxa-pill oxs7-kind", "data-tone": "run", "Default" } } } p { class: "oxsst-hint", "{ml}" } }
                                                    div { class: "oxsst-rc oxs6-gap",
                                                        {choice(&format!("Who can use {mn}"), &ACCESS, acc, move |j| cur.write().ws.access[i] = j)}
                                                        {switch(&format!("oxs7-m{i}"), &format!("{mn} on"), mon, move |v| { let m = set_model(&mut cur.write().ws, i, v); say(ctx, m); })}
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        {sec("API keys", rsx! {
            button { r#type: "button", class: "oxscn-btn", "aria-expanded": "{adding}", onclick: move |_| { k_name.set(String::new()); k_sec.set(String::new()); adding.toggle(); }, {nico("plus")} span { "Add key" } }
        }, rsx! {
            if adding() {
                div { class: "oxsst-row oxs6-row",
                    div { class: "oxsst-ask", role: "group", "aria-label": "Add an API key",
                        onkeydown: move |e: KeyboardEvent| if e.key() == Key::Escape { adding.set(false); },
                        div { class: "oxs7-inv-r",
                            span { class: "oxs6-q", "Provider" }
                            MenuButton { label: PROVIDERS[k_prov()].1.to_string(), aria: "Provider: {PROVIDERS[k_prov()].1}", items: PROVIDERS.iter().map(|x| (x.1, x.1, x.0)).collect::<Vec<MenuItem>>(), checked: PROVIDERS[k_prov()].1.to_string(), caret: true, brand: PROVIDERS[k_prov()].0.to_string(), class: "oxsst-mb".to_string(),
                                on_pick: move |k: String| if let Some(j) = PROVIDERS.iter().position(|x| x.1 == k) { k_prov.set(j); } }
                        }
                        label { class: "oxsst-fl", "Name"
                            input { class: "oxsst-in", value: "{k_name}", placeholder: "What it is for", onmounted: focus_on, oninput: move |e| k_name.set(e.value()) } }
                        label { class: "oxsst-fl", "Key"
                            input { class: "oxsst-in", r#type: "password", value: "{k_sec}", placeholder: "Paste the key", autocomplete: "off", "aria-invalid": "{!k_sec().is_empty() && ke.is_some()}", oninput: move |e| k_sec.set(e.value()) } }
                        div { class: "oxsst-tfm",
                            if let Some(m) = ke.filter(|_| !k_sec().is_empty() || !k_name().is_empty()) { span { class: "oxsst-err", "{m}" } }
                            else { span { class: "oxs6-q", "Stored encrypted. Usage on this key is billed by the provider." } }
                        }
                        div { class: "oxsin-acts",
                            button { r#type: "button", class: "oxscn-btn", onclick: move |_| adding.set(false), "Cancel" }
                            button { r#type: "button", class: "oxscn-primary", disabled: ke.is_some(),
                                onclick: move |_| {
                                    let (nm, sc, pv) = (k_name.peek().trim().to_string(), k_sec.peek().trim().to_string(), *k_prov.peek());
                                    let mut id = 0;
                                    now(ctx, |w| { id = w.keys.iter().map(|k| k.id).max().unwrap_or(0) + 1; w.keys.push(oxyn_core::kit::setscr2::ApiKey { id, prov: pv, name: nm.clone(), secret: sc.clone(), added: "Today", used: "Not used yet" }); });
                                    adding.set(false);
                                    say(ctx, format!("Key “{nm}” added"));
                                }, "Add key" }
                        }
                    }
                }
            }
            if p.keys.is_empty() && !adding() {
                div { class: "oxsst-row oxs6-row", "data-hit": "{ctx.hit(\"API keys\")}",
                    {empty("key", "No API keys", "Usage runs on your plan until you add a provider's key.", rsx! {
                        button { r#type: "button", class: "oxscn-btn", onclick: move |_| adding.set(true), {nico("plus")} span { "Add key" } }
                    })}
                }
            }
            for k in p.keys.iter().cloned() {
                {
                    let id = k.id;
                    let shown = reveal().contains(&id);
                    let txt = if shown { k.secret.clone() } else { mask(&k.secret) };
                    let nm = k.name.clone();
                    let nm2 = nm.clone();
                    let nm3 = nm.clone();
                    rsx! {
                        div { key: "{id}", class: "oxsst-row oxs6-row oxs7-key", "data-hit": "{ctx.hit(\"API keys\")}",
                            span { class: "oxs6-logo", BrandIcon { name: PROVIDERS[k.prov].0.to_string(), size: 16.0, coloured: true } }
                            div { class: "oxsst-rl",
                                span { class: "oxsst-lb", "{nm}" }
                                code { class: "oxs7-secret oxv-ell", "data-shown": "{shown}", "aria-label": if shown { "Key" } else { "Key, hidden" }, "{txt}" }
                                p { class: "oxsst-hint", "{PROVIDERS[k.prov].1} · added {k.added} · used {k.used}" }
                            }
                            div { class: "oxsst-rc oxs7-icons",
                                button { r#type: "button", class: "oxa-x", "aria-pressed": "{shown}", "aria-label": if shown { "Hide {nm}" } else { "Reveal {nm}" }, title: if shown { "Hide" } else { "Reveal" },
                                    onclick: move |_| { let mut r = reveal.write(); if let Some(j) = r.iter().position(|x| *x == id) { r.remove(j); } else { r.push(id); } },
                                    if shown { {nico("eyeoff")} } else { {nico("eye")} } }
                                button { r#type: "button", class: "oxa-x", "aria-label": "Copy {nm}", title: "Copy", onclick: move |_| say(ctx, format!("Copied “{nm2}”")), {nico("copy")} }
                                button { r#type: "button", class: "oxa-x", "aria-label": "Rotate {nm}", title: "Rotate", "aria-expanded": "{ask_rot() == Some(id)}", onclick: move |_| ask_rot.set(Some(id)), {nico("refresh")} }
                                button { r#type: "button", class: "oxa-x oxs7-del", "aria-label": "Delete {nm}", title: "Delete",
                                    onclick: move |_| { now(ctx, |w| w.keys.retain(|x| x.id != id)); say(ctx, format!("Key “{nm3}” deleted; requests with it now fail")); }, {nico("trash")} }
                            }
                            if fresh() == Some(id) {
                                div { class: "oxs7-fresh", role: "status",
                                    span { class: "oxs7-fresh-i", {nico("key")} }
                                    span { class: "oxs6-q", "New key: copy it now. Once you leave this page it shows masked." }
                                    button { r#type: "button", class: "oxscn-btn", onclick: move |_| { fresh.set(None); reveal.write().retain(|x| *x != id); say(ctx, "Copied the new key"); }, "Copy and hide" }
                                }
                            }
                            if ask_rot() == Some(id) {
                                div { class: "oxsst-ask", role: "group", "aria-label": "Confirm rotate",
                                    onkeydown: move |e: KeyboardEvent| if e.key() == Key::Escape { ask_rot.set(None); },
                                    span { class: "oxs6-q", "Rotate “{k.name}”? The old key stops working now; update anything that uses it." }
                                    div { class: "oxsin-acts",
                                        button { r#type: "button", class: "oxscn-btn", onmounted: focus_on, onclick: move |_| ask_rot.set(None), "Cancel" }
                                        button { r#type: "button", class: "oxscn-primary",
                                            onclick: move |_| {
                                                let n = *rot_n.peek() + 1;
                                                rot_n.set(n);
                                                now(ctx, |w| if let Some(x) = w.keys.iter_mut().find(|x| x.id == id) { x.secret = rotated(&x.secret, n); x.used = "Not used yet"; x.added = "Today"; });
                                                ask_rot.set(None);
                                                if !reveal.peek().contains(&id) { reveal.write().push(id); }
                                                fresh.set(Some(id));
                                                say(ctx, "Key rotated: the new one is shown once");
                                            }, "Rotate key" }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        })}
    }
}

// ──────────────────────────────────────────────────────────── Security ──

#[component]
pub(super) fn SecurityPage(ctx: Ctx, setup: bool) -> Element {
    let p = ctx.cur.read().ws.clone();
    let mut step = use_signal(move || if setup { 1u8 } else { 0 });
    let mut code = use_signal(String::new);
    let mut show_codes = use_signal(|| false);
    let mut seed = use_signal(|| 7u32);
    let mut dom = use_signal(String::new);
    let mut add_dom = use_signal(|| false);
    let mut ask_all = use_signal(|| false);
    let mut af = use_signal(|| "all");
    let mut pk_n = use_signal(|| 0u32);
    let mut cur = ctx.cur;
    let sample = !p.domains.is_empty() || p.passkeys.len() > 1;
    let has_verified = p.domains.iter().any(|d| d.verified);
    let de = domain_error(&dom(), &p.domains);
    let codes = recovery_codes(seed());
    let qr = demo_qr();
    let qn = qr.size + 4;
    let qd = qr.svg_path(2);
    let ev = audit(af(), sample);
    let others = p.devices.len().saturating_sub(1);
    let st = step();
    rsx! {
        {sec("Your sign-in", rsx! {}, rsx! {
            div { class: "oxsst-row oxs6-row", "data-hit": "{ctx.hit(\"Two-step sign-in\")}",
                span { class: "oxs6-logo", {nico("lock")} }
                div { class: "oxsst-rl", span { class: "oxsst-lb", "Two-step sign-in" } p { class: "oxsst-hint", if p.two_step_me { "On: a code from your authenticator app after your password." } else { "Off: add a code from an authenticator app." } } }
                if p.two_step_me { span { class: "oxa-pill", "data-tone": "ok", "On" } }
                div { class: "oxsst-rc",
                    if p.two_step_me {
                        button { r#type: "button", class: "oxscn-btn", onclick: move |_| { now(ctx, |w| w.two_step_me = false); say(ctx, "Two-step sign-in turned off"); }, "Turn off" }
                    } else if st == 0 {
                        button { r#type: "button", class: "oxscn-primary", onclick: move |_| { code.set(String::new()); step.set(1); }, "Set up" }
                    }
                }
                if st > 0 {
                    div { class: "oxsst-ask oxs7-2fa", role: "group", "aria-label": "Set up two-step sign-in",
                        onkeydown: move |e: KeyboardEvent| if e.key() == Key::Escape { step.set(0); },
                        ol { class: "oxs7-steps", "aria-label": "Steps",
                            for (i, l) in ["Scan", "Enter code", "Save codes"].iter().enumerate() {
                                li { key: "{i}", "data-on": "{st as usize == i + 1}", "data-done": "{(st as usize) > i + 1}", span { "{i + 1}" } "{l}" }
                            }
                        }
                        if st == 1 {
                            div { class: "oxs7-qr-w",
                                svg { class: "oxs7-qr", view_box: "0 0 {qn} {qn}", role: "img", "aria-label": "QR code for your authenticator app (sample key)",
                                    rect { x: "0", y: "0", width: "{qn}", height: "{qn}", fill: "#fff" }
                                    path { d: "{qd}", fill: "#111" }
                                }
                                div { class: "oxs7-qr-t",
                                    b { "Scan with your authenticator app" }
                                    p { class: "oxsst-hint", "Any app that shows six-digit codes works. Can't scan? Type this key:" }
                                    code { class: "oxs7-secret", "{TOTP_SECRET}" }
                                    div { class: "oxsin-acts",
                                        button { r#type: "button", class: "oxscn-btn", onclick: move |_| say(ctx, "Setup key copied"), {nico("copy")} span { "Copy key" } }
                                        button { r#type: "button", class: "oxscn-primary", onclick: move |_| step.set(2), "Next" }
                                    }
                                }
                            }
                        }
                        if st == 2 {
                            label { class: "oxsst-fl", "The six-digit code your app shows"
                                input { class: "oxsst-in oxs7-code", inputmode: "numeric", autocomplete: "one-time-code", maxlength: "7", value: "{code}", placeholder: "123 456", onmounted: focus_on,
                                    "aria-invalid": "{!code().is_empty() && !code_ok(&code())}", oninput: move |e| code.set(e.value()) } }
                            div { class: "oxsin-acts",
                                button { r#type: "button", class: "oxscn-btn", onclick: move |_| step.set(1), "Back" }
                                button { r#type: "button", class: "oxscn-primary", disabled: !code_ok(&code()), onclick: move |_| { step.set(3); say(ctx, "Code accepted"); }, "Verify" }
                            }
                        }
                        if st == 3 {
                            p { class: "oxsst-hint", "Keep these somewhere safe. Each one signs you in once if you lose your phone." }
                            div { class: "oxs7-codes", "data-hit": "{ctx.hit(\"Recovery codes\")}",
                                for c in codes.iter() { code { key: "{c}", "{c}" } }
                            }
                            div { class: "oxsin-acts",
                                button { r#type: "button", class: "oxscn-btn", onclick: move |_| say(ctx, "10 recovery codes copied"), {nico("copy")} span { "Copy" } }
                                button { r#type: "button", class: "oxscn-btn", onclick: move |_| say(ctx, "Downloaded oxyn-recovery-codes.txt"), {nico("download")} span { "Download" } }
                                span { class: "oxscn-grow" }
                                button { r#type: "button", class: "oxscn-primary", onclick: move |_| { now(ctx, |w| { w.two_step_me = true; w.steps[4] = true; }); step.set(0); say(ctx, "Two-step sign-in is on"); }, "I saved them" }
                            }
                        }
                    }
                }
            }
            if p.two_step_me {
                div { class: "oxsst-row oxs6-row", "data-hit": "{ctx.hit(\"Recovery codes\")}",
                    span { class: "oxs6-logo", {nico("doc")} }
                    div { class: "oxsst-rl", span { class: "oxsst-lb", "Recovery codes" } p { class: "oxsst-hint", "10 left. New codes replace the old ones." } }
                    div { class: "oxsst-rc oxs6-gap",
                        button { r#type: "button", class: "oxscn-btn", "aria-expanded": "{show_codes}", onclick: move |_| show_codes.toggle(), if show_codes() { "Hide" } else { "Show" } }
                        button { r#type: "button", class: "oxscn-btn", onclick: move |_| { seed.with_mut(|s| *s += 1); show_codes.set(true); say(ctx, "New recovery codes made; the old ones stop working"); }, "New codes" }
                    }
                    if show_codes() {
                        div { class: "oxs7-codes", for c in codes.iter() { code { key: "{c}", "{c}" } } }
                    }
                }
            }
            div { class: "oxsst-row oxs6-row", "data-hit": "{ctx.hit(\"Passkeys\")}",
                span { class: "oxs6-logo", {nico("finger")} }
                div { class: "oxsst-rl", span { class: "oxsst-lb", "Passkeys" } p { class: "oxsst-hint", if p.passkeys.is_empty() { "None yet: sign in with your fingerprint, face or screen lock." } else { "Sign in with your fingerprint, face or screen lock." } } }
                div { class: "oxsst-rc",
                    button { r#type: "button", class: "oxscn-btn",
                        onclick: move |_| { let n = *pk_n.peek() + 1; pk_n.set(n); now(ctx, |w| { let id = w.passkeys.iter().map(|x| x.id).max().unwrap_or(0) + 1; w.passkeys.push(oxyn_core::kit::setscr2::Passkey { id, name: if n == 1 { "This browser".to_string() } else { format!("This browser ({n})") }, glyph: "laptop", added: "Today", used: "Just now" }); }); say(ctx, "Passkey added for this browser"); },
                        {nico("plus")} span { "Add passkey" } }
                }
            }
            for k in p.passkeys.iter().cloned() {
                {
                    let id = k.id;
                    let nm = k.name.clone();
                    rsx! {
                        div { key: "pk{id}", class: "oxsst-row oxs6-row", "data-hit": "{ctx.hit(\"Passkeys\")}",
                            span { class: "oxs6-logo", {nico("finger")} }
                            div { class: "oxsst-rl", span { class: "oxsst-lb oxv-ell", "{k.name}" } p { class: "oxsst-hint", "Passkey · added {k.added} · used {k.used}" } }
                            div { class: "oxsst-rc",
                                button { r#type: "button", class: "oxscn-btn", "aria-label": "Remove {nm}", onclick: move |_| { now(ctx, |w| w.passkeys.retain(|x| x.id != id)); say(ctx, "Passkey removed"); }, "Remove" }
                            }
                        }
                    }
                }
            }
        })}
        {sec("Workspace sign-in", rsx! {}, rsx! {
            {row("oxs7-2all", "Require two-step for everyone", "People without it are asked to set it up at their next sign-in", switch("oxs7-2all", "Require two-step for everyone", p.two_step_all, move |v| cur.write().ws.two_step_all = v), ctx.dirty(|x| x.ws.two_step_all), ctx.hit("Two-step sign-in"))}
            div { class: "oxsst-row oxs6-row", "data-dirty": "{ctx.dirty(|x| x.ws.sso)}", "data-hit": "{ctx.hit(\"Single sign-on\")}",
                div { class: "oxsst-rl",
                    label { class: "oxsst-lb", r#for: "oxs7-sso", "Require single sign-on" }
                    p { class: "oxsst-hint", "People with a verified domain sign in through your identity provider" }
                    if p.sso && !has_verified { span { class: "oxsst-err", "Verify a domain first" } }
                }
                div { class: "oxsst-rc", {switch("oxs7-sso", "Require single sign-on", p.sso, move |v| cur.write().ws.sso = v)} }
            }
            if p.sso {
                div { class: "oxsst-row oxs6-row oxs7-idp",
                    span { class: "oxs6-logo", BrandIcon { name: "google".to_string(), size: 16.0, coloured: true } }
                    div { class: "oxsst-rl", span { class: "oxsst-lb", "Google Workspace" } p { class: "oxsst-hint", "Or any SAML 2.0 provider: copy these values into it" } }
                    for (l, v) in [("Entity ID", "https://sso.oxyn.app/saml/kestrel-studio"), ("Reply URL", "https://sso.oxyn.app/saml/kestrel-studio/acs")] {
                        div { key: "{l}", class: "oxs7-copy",
                            span { class: "oxs6-q", "{l}" }
                            code { class: "oxs7-secret oxv-ell", title: "{v}", "{v}" }
                            button { r#type: "button", class: "oxscn-btn", "aria-label": "Copy {l}", onclick: move |_| say(ctx, format!("{l} copied")), {nico("copy")} span { "Copy" } }
                        }
                    }
                }
            }
            {row("oxs7-len", "Session length", "How long people stay signed in without activity", choice("Session length", &SESSION_LEN, p.session_len, move |i| cur.write().ws.session_len = i), ctx.dirty(|x| x.ws.session_len), false)}
            {row("oxs7-join", "Join by domain", "Anyone with a verified address joins as a Member without an invite", switch("oxs7-join", "Join by domain", p.join_domain, move |v| cur.write().ws.join_domain = v), ctx.dirty(|x| x.ws.join_domain), ctx.hit("Email domains"))}
        })}
        {sec("Email domains", rsx! {
            button { r#type: "button", class: "oxscn-btn", "aria-expanded": "{add_dom}", onclick: move |_| { dom.set(String::new()); add_dom.toggle(); }, {nico("plus")} span { "Add domain" } }
        }, rsx! {
            if add_dom() {
                div { class: "oxsst-row oxs6-row",
                    div { class: "oxsst-ask", role: "group", "aria-label": "Add a domain",
                        onkeydown: move |e: KeyboardEvent| if e.key() == Key::Escape { add_dom.set(false); },
                        label { class: "oxsst-fl", "Domain"
                            input { class: "oxsst-in", value: "{dom}", placeholder: "company.com", onmounted: focus_on, "aria-invalid": "{!dom().is_empty() && de.is_some()}", oninput: move |e| dom.set(e.value()) } }
                        if let Some(m) = de.filter(|_| !dom().is_empty()) { span { class: "oxsst-err", "{m}" } }
                        div { class: "oxsin-acts",
                            button { r#type: "button", class: "oxscn-btn", onclick: move |_| add_dom.set(false), "Cancel" }
                            button { r#type: "button", class: "oxscn-primary", disabled: de.is_some(),
                                onclick: move |_| { let d = dom.peek().trim().to_lowercase(); now(ctx, |w| w.domains.push(oxyn_core::kit::setscr2::Domain { name: d.clone(), verified: false })); add_dom.set(false); say(ctx, format!("{d} added: add the TXT record, then Verify")); }, "Add" }
                        }
                    }
                }
            }
            if p.domains.is_empty() && !add_dom() {
                div { class: "oxsst-row oxs6-row", "data-hit": "{ctx.hit(\"Email domains\")}",
                    {empty("globe", "No domains yet", "Verify the domain your team signs in with to use single sign-on and join by domain.", rsx! {
                        button { r#type: "button", class: "oxscn-btn", onclick: move |_| add_dom.set(true), {nico("plus")} span { "Add domain" } }
                    })}
                }
            }
            for d in p.domains.iter().cloned() {
                {
                    let n = d.name.clone();
                    let n2 = n.clone();
                    let n3 = n.clone();
                    rsx! {
                        div { key: "{n}", class: "oxsst-row oxs6-row", "data-hit": "{ctx.hit(\"Email domains\")}",
                            span { class: "oxs6-logo", {nico("globe")} }
                            div { class: "oxsst-rl", span { class: "oxsst-lb oxv-ell", "{n}" }
                                if !d.verified { p { class: "oxsst-hint", "Add a TXT record " code { class: "oxs7-inl", "{txt_record(&n)}" } " to its DNS, then Verify." } }
                            }
                            span { class: "oxa-pill", "data-tone": if d.verified { "ok" } else { "warn" }, if d.verified { "Verified" } else { "Pending" } }
                            div { class: "oxsst-rc oxs6-gap",
                                if !d.verified { button { r#type: "button", class: "oxscn-btn", onclick: move |_| { now(ctx, |w| if let Some(x) = w.domains.iter_mut().find(|x| x.name == n2) { x.verified = true; }); say(ctx, format!("{n2} verified")); }, "Verify" } }
                                button { r#type: "button", class: "oxa-x", "aria-label": "Remove {n}", onclick: move |_| { now(ctx, |w| w.domains.retain(|x| x.name != n3)); say(ctx, "Domain removed"); }, {nico("close")} }
                            }
                        }
                    }
                }
            }
        })}
        {sec("Signed-in devices", rsx! {
            if others > 0 { button { r#type: "button", class: "oxscn-btn oxsst-danger", "aria-expanded": "{ask_all}", onclick: move |_| ask_all.set(true), "Sign out everyone else" } }
        }, rsx! {
            if ask_all() {
                div { class: "oxsst-row oxs6-row",
                    div { class: "oxsst-ask", role: "group", "aria-label": "Confirm sign out everyone",
                        onkeydown: move |e: KeyboardEvent| if e.key() == Key::Escape { ask_all.set(false); },
                        span { class: "oxs6-q", "Sign out {others} other devices across the workspace? People sign in again; running agents keep going." }
                        div { class: "oxsin-acts",
                            button { r#type: "button", class: "oxscn-btn", onmounted: focus_on, onclick: move |_| ask_all.set(false), "Cancel" }
                            button { r#type: "button", class: "oxscn-btn oxsst-danger oxsst-go", onclick: move |_| { now(ctx, |w| w.devices.truncate(1)); ask_all.set(false); say(ctx, format!("Signed out {others} devices")); }, "Sign out everyone" }
                        }
                    }
                }
            }
            for d in p.devices.iter().cloned() {
                {
                    let id = d.id;
                    let me = id == 1;
                    rsx! {
                        div { key: "{id}", class: "oxsst-row oxs6-row", "data-hit": "{ctx.hit(\"Signed-in devices\")}",
                            span { class: "oxs6-logo", {nico(d.glyph)} }
                            div { class: "oxsst-rl", span { class: "oxsst-lb oxv-ell", "{d.who}" } p { class: "oxsst-hint oxv-ell", "{d.device} · {d.place} · {d.seen}" } }
                            if me { span { class: "oxa-pill", "data-tone": "ok", "This device" } }
                            else { div { class: "oxsst-rc", button { r#type: "button", class: "oxscn-btn", "aria-label": "Sign out {d.who}", onclick: move |_| { now(ctx, |w| w.devices.retain(|x| x.id != id)); say(ctx, "Signed out"); }, "Sign out" } } }
                        }
                    }
                }
            }
        })}
        {sec("Audit log", rsx! {
            button { r#type: "button", class: "oxscn-btn", onclick: move |_| say(ctx, "Audit log exported as CSV (sample)"), {nico("download")} span { "Export" } }
        }, rsx! {
            div { class: "oxs7-chips", role: "radiogroup", "aria-label": "Filter events", "data-hit": "{ctx.hit(\"Audit log\")}",
                for (k, l) in AUDIT_KINDS.iter().copied() {
                    button { key: "{k}", r#type: "button", class: "oxs7-chipb", role: "radio", "aria-checked": "{af() == k}", onclick: move |_| af.set(k), "{l}" }
                }
            }
            if ev.is_empty() { p { class: "oxa-mute oxs7-none", "No events of this kind yet." } }
            for (i, (_, who, what, when, place, tone)) in ev.iter().copied().enumerate() {
                div { key: "{i}", class: "oxs7-ev",
                    span { class: "oxa-dot", "data-tone": "{tone}" }
                    span { class: "oxs6-tt", b { class: "oxs7-wrap", "{what}" } span { class: "oxv-ell oxs6-q", "{who} · {place}" } }
                    span { class: "oxs6-q oxs7-when", "{when}" }
                }
            }
        })}
    }
}

// ───────────────────────────────────────────────────────────── Billing ──

#[component]
pub(super) fn BillingPage(ctx: Ctx, cancelling: bool) -> Element {
    let p = ctx.cur.read().ws.clone();
    let mut cstep = use_signal(move || if cancelling { 1u8 } else { 0 });
    let mut reason = use_signal(|| None::<usize>);
    let mut switch_to = use_signal(|| None::<usize>);
    let mut card_open = use_signal(|| false);
    let mut num = use_signal(String::new);
    let mut exp = use_signal(String::new);
    let mut cvc = use_signal(String::new);
    let mut cur = ctx.cur;
    let seats = seats_used(&p.members);
    let plan = PLANS[p.plan];
    let price = seat_price(p.plan, p.yearly);
    let bill = next_bill(p.plan, p.yearly, seats);
    let per = if p.yearly { "year" } else { "month" };
    let ms = meters(p.card.is_some(), seats);
    let net = card_network(&num());
    let num_ok = card_number_ok(&num());
    let exp_ok = expiry_ok(&exp());
    let cvc_good = cvc_ok(&num(), &cvc());
    let card_ok = num_ok && exp_ok && cvc_good;
    let saved_net = p.card_net.map(|i| NETWORKS[i]);
    let saved_name = saved_net.map_or("Card", |n| n.1);
    let has_inv = p.card.is_some();
    let cs = cstep();
    rsx! {
        div { class: "oxs7-plan", "data-hit": "{ctx.hit(\"Plan\")}",
            div { class: "oxs7-plan-a",
                span { class: "oxs7-plan-i", {nico("spark")} }
                span { class: "oxs6-tt", b { "{plan.1} plan" } span { class: "oxs6-q", if p.plan == 0 { "Free for one seat" } else { "{dollars(price)} per seat / month, billed {INTERVALS[usize::from(p.yearly)].to_lowercase()}" } } }
            }
            div { class: "oxs7-plan-b",
                span { class: "oxs6-tt", b { "Upcoming bill" } span { class: "oxs6-q", if p.cancelled { "Plan ends on 1 Dec 2026" } else if p.plan == 0 { "Nothing to pay" } else { "Renews on 1 Dec 2026 · {seats} seats" } } }
                b { class: "oxs7-amt", if p.cancelled || p.plan == 0 { "$0" } else { "{dollars(bill)}" } }
            }
            if p.cancelled {
                div { class: "oxs7-plan-c", role: "status",
                    span { class: "oxa-pill", "data-tone": "warn", "Cancels 1 Dec" }
                    span { class: "oxs6-q", "You keep {plan.1} until then." }
                    button { r#type: "button", class: "oxscn-btn", onclick: move |_| { now(ctx, |w| w.cancelled = false); cstep.set(0); say(ctx, "Plan resumed: it renews on 1 December"); }, "Resume plan" }
                }
            }
        }
        div { class: "oxs7-meters", "data-hit": "{ctx.hit(\"Usage\")}",
            for (l, u, lim, unit, g) in ms.iter().copied() {
                div { key: "{l}", class: "oxs7-meter",
                    span { class: "oxs7-k", {nico(g)} "{l}" }
                    b { "{meter_words(u, lim, unit)}" }
                    {bar(u / lim, meter_tone(u, lim))}
                }
            }
        }
        {sec("Plans", rsx! {
            div { class: "oxscn-seg oxs7-seg", role: "radiogroup", "aria-label": "Billing interval",
                for (i, l) in INTERVALS.iter().enumerate() {
                    button { key: "{l}", r#type: "button", class: "oxscn-seg-b", role: "radio", "aria-checked": "{usize::from(p.yearly) == i}", "aria-selected": "{usize::from(p.yearly) == i}", onclick: move |_| cur.write().ws.yearly = i == 1,
                        "{l}" if i == 1 { span { class: "oxs7-save", "−20%" } } }
                }
            }
        }, rsx! {
            div { class: "oxs7-plans", "data-dirty": "{ctx.dirty(|x| x.ws.yearly)}",
                for (i, (k, n, _, line, feats)) in PLANS.iter().copied().enumerate() {
                    {
                        let is = i == p.plan;
                        let pr = seat_price(i, p.yearly);
                        let up = i > p.plan;
                        let total = next_bill(i, p.yearly, if i == 0 { 1 } else { seats });
                        rsx! {
                            div { key: "{k}", class: "oxs7-pc", "data-cur": "{is}",
                                div { class: "oxs7-pc-h", b { "{n}" } if is { span { class: "oxa-pill", "Current plan" } } }
                                p { class: "oxsst-hint", "{line}" }
                                div { class: "oxs7-price", b { "{dollars(pr)}" } span { class: "oxs6-q", "per seat / month" } }
                                ul { class: "oxs7-feats",
                                    for f in feats.iter() { li { key: "{f}", span { class: "oxs7-tick", {nico("check")} } span { "{f}" } } }
                                }
                                if switch_to() == Some(i) {
                                    div { class: "oxsst-ask", role: "group", "aria-label": "Confirm plan change",
                                        span { class: "oxs6-q", if i == 0 { "Move to Free at the end of the period? Extra seats and models switch off." } else { "From the next bill: {dollars(total)} a {per}, prorated today." } }
                                        div { class: "oxsin-acts",
                                            button { r#type: "button", class: "oxscn-btn", onmounted: focus_on, onclick: move |_| switch_to.set(None), "Cancel" }
                                            button { r#type: "button", class: "oxscn-primary", onclick: move |_| { now(ctx, |w| { w.plan = i; w.cancelled = false; w.steps[5] = true; }); switch_to.set(None); say(ctx, format!("Plan changed to {n}")); }, "Confirm" }
                                        }
                                    }
                                } else if !is {
                                    button { r#type: "button", class: if up { "oxscn-primary oxs7-pc-b" } else { "oxscn-btn oxs7-pc-b" }, onclick: move |_| switch_to.set(Some(i)), if up { "Upgrade to {n}" } else { "Switch to {n}" } }
                                }
                            }
                        }
                    }
                }
            }
        })}
        {sec("Extra usage", rsx! {}, rsx! {
            {row("oxs7-cap", "Monthly cap", "When credits run out, agents keep running up to this amount", choice("Monthly cap", &CAPS, p.cap, move |i| cur.write().ws.cap = i), ctx.dirty(|x| x.ws.cap), ctx.hit("Usage"))}
            {row("oxs7-mail", "Email invoices", &format!("To {} and other owners", oxyn_core::kit::setscr::PERSON_EMAIL), switch("oxs7-mail", "Email invoices", p.invoice_mail, move |v| cur.write().ws.invoice_mail = v), ctx.dirty(|x| x.ws.invoice_mail), ctx.hit("Invoices"))}
        })}
        {sec("Payment method", rsx! {}, rsx! {
            div { class: "oxsst-row oxs6-row", "data-hit": "{ctx.hit(\"Payment method\")}",
                span { class: "oxs7-cardi", "data-net": "{saved_net.is_some() && p.card.is_some()}",
                    if let (Some(n), true) = (saved_net, p.card.is_some()) { BrandIcon { name: n.0.to_string(), size: 22.0, coloured: true } } else { {nico("card")} } }
                div { class: "oxsst-rl",
                    span { class: "oxsst-lb", if let Some(l) = p.card.as_ref() { "{saved_name} ending {l}" } else { "No card yet" } }
                    p { class: "oxsst-hint", if p.card.is_some() { "Expires 08/28 · charged on the 1st" } else { "Add one before you leave the Free plan." } }
                }
                div { class: "oxsst-rc", button { r#type: "button", class: "oxscn-btn", "aria-expanded": "{card_open}", onclick: move |_| { num.set(String::new()); exp.set(String::new()); cvc.set(String::new()); card_open.toggle(); }, if p.card.is_some() { "Update" } else { "Add card" } } }
                if card_open() {
                    div { class: "oxsst-ask oxs7-cardf", role: "group", "aria-label": "Card details",
                        onkeydown: move |e: KeyboardEvent| if e.key() == Key::Escape { card_open.set(false); },
                        label { class: "oxsst-fl oxs7-cn", "Card number"
                            span { class: "oxs7-cnw",
                                input { class: "oxsst-in", inputmode: "numeric", autocomplete: "cc-number", value: "{num}", placeholder: "4242 4242 4242 4242", onmounted: focus_on, "aria-invalid": "{!num().is_empty() && !num_ok}", "aria-describedby": "oxs7-nets", oninput: move |e| num.set(e.value()) }
                                span { class: "oxs7-nets", id: "oxs7-nets", "aria-live": "polite",
                                    span { class: "oxscn-sr", if let Some(i) = net { "{NETWORKS[i].1} card" } else { "Visa, Mastercard and American Express accepted" } }
                                    for (i, (id, name, _, _)) in NETWORKS.iter().enumerate() {
                                        span { key: "{id}", class: "oxs7-net", title: "{name}", "aria-hidden": "true", "data-on": "{net == Some(i)}", "data-off": "{net.is_some() && net != Some(i)}",
                                            BrandIcon { name: id.to_string(), size: 18.0, coloured: true } }
                                    }
                                }
                            }
                        }
                        label { class: "oxsst-fl", "Expiry"
                            input { class: "oxsst-in", autocomplete: "cc-exp", value: "{exp}", placeholder: "MM/YY", "aria-invalid": "{!exp().is_empty() && !exp_ok}", oninput: move |e| exp.set(e.value()) } }
                        label { class: "oxsst-fl", "Security code"
                            input { class: "oxsst-in", inputmode: "numeric", autocomplete: "cc-csc", r#type: "password", value: "{cvc}", placeholder: if net == Some(2) { "1234" } else { "123" }, "aria-invalid": "{!cvc().is_empty() && !cvc_good}", oninput: move |e| cvc.set(e.value()) } }
                        div { class: "oxsst-tfm oxs7-span",
                            if !num().is_empty() && !num_ok { span { class: "oxsst-err", "Check the card number" } }
                            else if cvc().len() >= 4 && !cvc_good { span { class: "oxsst-err", if net == Some(2) { "American Express codes have 4 digits" } else { "Use the 3 digits on the back" } } }
                            else if !exp().is_empty() && !exp_ok { span { class: "oxsst-err", "Use MM/YY, not in the past" } }
                            else { span { class: "oxs6-q", {nico("lock")} " Sample form: nothing is sent." } }
                        }
                        div { class: "oxsin-acts oxs7-span",
                            button { r#type: "button", class: "oxscn-btn", onclick: move |_| card_open.set(false), "Cancel" }
                            button { r#type: "button", class: "oxscn-primary", disabled: !card_ok,
                                onclick: move |_| { let d: String = num.peek().chars().filter(|c| c.is_ascii_digit()).collect(); let l = d[d.len() - 4..].to_string(); let n = card_network(&d); now(ctx, |w| { w.card = Some(l.clone()); w.card_net = n; }); card_open.set(false); say(ctx, format!("Card ending {l} saved")); }, "Save card" }
                        }
                    }
                }
            }
        })}
        {sec("Invoices", rsx! {}, rsx! {
            if !has_inv {
                div { class: "oxsst-row oxs6-row", {empty("doc", "No invoices yet", "They appear here after your first paid month.", rsx! {})} }
            } else {
                div { class: "oxs7-table oxs7-inv", role: "table", "aria-label": "Invoices", "data-hit": "{ctx.hit(\"Invoices\")}",
                    div { class: "oxs7-tr oxs7-th", role: "row", span { role: "columnheader", "Invoice" } span { role: "columnheader", "Date" } span { role: "columnheader", "Amount" } span { role: "columnheader", "Status" } span { role: "columnheader", span { class: "oxscn-sr", "Download" } } }
                    for (no, date, amt, stt) in INVOICES.iter().copied() {
                        div { key: "{no}", class: "oxs7-tr", role: "row",
                            span { role: "cell", class: "oxv-ell oxs7-mono", "{no}" }
                            span { role: "cell", class: "oxv-ell oxs6-q", "{date}" }
                            span { role: "cell", class: "oxs7-num", "{dollars(amt)}" }
                            span { role: "cell", span { class: "oxa-pill", "data-tone": match stt { "Paid" => "ok", "Due" => "warn", _ => "" }, "{stt}" } }
                            span { role: "cell", class: "oxs7-more", button { r#type: "button", class: "oxa-x", "aria-label": "Download {no}", title: "Download PDF", onclick: move |_| say(ctx, format!("Downloaded {no}.pdf")), {nico("download")} } }
                        }
                    }
                }
            }
        })}
        if p.plan > 0 {
            {sec("Danger zone", rsx! {}, rsx! {
                div { class: "oxsst-row oxs6-row", "data-hit": "{ctx.hit(\"Cancel plan\")}",
                    div { class: "oxsst-rl", span { class: "oxsst-lb", "Cancel plan" } p { class: "oxsst-hint", if p.cancelled { "Cancelled: the workspace moves to Free on 1 December." } else { "You keep {plan.1} until this billing period ends, then move to Free." } } }
                    div { class: "oxsst-rc", button { r#type: "button", class: "oxscn-btn oxsst-danger", disabled: p.cancelled, "aria-expanded": "{cs > 0}", onclick: move |_| { reason.set(None); cstep.set(1); }, "Cancel plan" } }
                    if cs > 0 && cs < 3 {
                        div { class: "oxsst-ask oxs7-cancel", role: "group", "aria-label": "Cancel plan",
                            onkeydown: move |e: KeyboardEvent| if e.key() == Key::Escape { cstep.set(0); },
                            ol { class: "oxs7-steps", "aria-label": "Steps",
                                for (i, l) in ["Why", "What changes"].iter().enumerate() {
                                    li { key: "{i}", "data-on": "{cs as usize == i + 1}", "data-done": "{(cs as usize) > i + 1}", span { "{i + 1}" } "{l}" }
                                }
                            }
                            if cs == 1 {
                                div { class: "oxs7-reasons", role: "radiogroup", "aria-label": "Why are you cancelling?",
                                    for (i, r) in CANCEL_REASONS.iter().enumerate() {
                                        button { key: "{i}", r#type: "button", class: "oxs7-reason", role: "radio", "aria-checked": "{reason() == Some(i)}", onclick: move |_| reason.set(Some(i)), span { class: "oxs7-radio" } span { "{r}" } }
                                    }
                                }
                                div { class: "oxsin-acts",
                                    button { r#type: "button", class: "oxscn-btn", onclick: move |_| cstep.set(0), "Keep my plan" }
                                    button { r#type: "button", class: "oxscn-primary", disabled: reason().is_none(), onclick: move |_| cstep.set(cancel_next(1, *reason.peek())), "Continue" }
                                }
                            }
                            if cs == 2 {
                                ul { class: "oxs7-feats oxs7-lose",
                                    li { span { class: "oxs7-tick", {nico("close")} } span { "Seats beyond one stop on 1 December; {seats - 1} people move to view only" } }
                                    li { span { class: "oxs7-tick", {nico("close")} } span { "Schedules pause and run history keeps 7 days" } }
                                    li { span { class: "oxs7-tick", {nico("check")} } span { "Chats, files and agents stay; resume any time" } }
                                }
                                if reason() == Some(0) { p { class: "oxs7-offer", "Yearly billing is 20 % less: {dollars(next_bill(p.plan, true, seats))} a year instead of {dollars(next_bill(p.plan, false, seats) * 12)}." } }
                                div { class: "oxsin-acts",
                                    button { r#type: "button", class: "oxscn-btn", onclick: move |_| cstep.set(1), "Back" }
                                    span { class: "oxscn-grow" }
                                    button { r#type: "button", class: "oxscn-btn", onclick: move |_| { cstep.set(0); say(ctx, "Glad you are staying"); }, "Keep my plan" }
                                    button { r#type: "button", class: "oxscn-btn oxsst-danger oxsst-go", onclick: move |_| { now(ctx, |w| w.cancelled = true); cstep.set(3); say(ctx, "Plan cancelled: it ends on 1 December"); }, "Cancel at period end" }
                                }
                            }
                        }
                    }
                }
            })}
        }
    }
}

// ───────────────────────────────────────────────────────── Get started ──

#[component]
pub(super) fn StartPage(ctx: Ctx, on_open: EventHandler<&'static str>) -> Element {
    let p = ctx.cur.read().ws.clone();
    let mut cur = ctx.cur;
    let (n, t, pct) = progress(&p.steps);
    let nx = next_step(&p.steps);
    let r = 26.0f32;
    let c = 2.0 * std::f32::consts::PI * r;
    let dash = c * pct as f32 / 100.0;
    let all = nx.is_none();
    rsx! {
        div { class: "oxs7-prog", "data-all": "{all}", "data-hit": "{ctx.hit(\"Get started\")}",
            svg { class: "oxs7-ring", width: "64", height: "64", view_box: "0 0 64 64", role: "img", "aria-label": "{pct}% done",
                circle { cx: "32", cy: "32", r: "{r}", fill: "none", stroke: "var(--oxth-line)", stroke_width: "6" }
                circle { cx: "32", cy: "32", r: "{r}", fill: "none", stroke: "currentColor", stroke_width: "6", stroke_linecap: "round", stroke_dasharray: "{dash:.1} {c:.1}", transform: "rotate(-90 32 32)" }
                text { x: "32", y: "36", text_anchor: "middle", font_size: "13", font_weight: "600", fill: "var(--oxth-fg)", "{pct}%" }
            }
            div { class: "oxs6-tt oxs7-grow",
                b { if all { "You are all set" } else { "Set up {oxyn_core::kit::setscr::WORKSPACE}" } }
                span { class: "oxs6-q", "{progress_words(&p.steps)}" }
                span { class: "oxs7-pbar", role: "progressbar", "aria-valuenow": "{n}", "aria-valuemin": "0", "aria-valuemax": "{t}", "aria-label": "Steps done", span { style: "width:{pct}%" } }
            }
            if let Some(i) = nx {
                button { r#type: "button", class: "oxscn-primary", onclick: move |_| on_open.call(STEPS[i].4), "Continue" }
            }
        }
        if all {
            div { class: "oxs6-banner oxs7-done", role: "status",
                span { class: "oxs7-done-i", {nico("check")} }
                span { class: "oxs6-tt oxs7-grow", b { "Every step is done" } span { class: "oxs6-q", "Hide the checklist; it stays here under Get started." } }
            }
        }
        div { class: "oxsst-card oxs7-list", role: "list", "aria-label": "Steps",
            for (i, (k, title, line, g, page)) in STEPS.iter().copied().enumerate() {
                {
                    let done = p.steps[i];
                    let is_next = nx == Some(i);
                    let pl = oxyn_core::kit::setscr::NAV.iter().find(|x| x.0 == page).map(|x| x.1).unwrap_or("");
                    rsx! {
                        div { key: "{k}", class: "oxsst-row oxs6-row oxs7-step", role: "listitem", "data-done": "{done}", "data-next": "{is_next}",
                            span { class: "oxs7-check", "aria-hidden": "true", if done { {nico("check")} } else { "{i + 1}" } }
                            span { class: "oxs7-sg", {ico(g)} }
                            div { class: "oxsst-rl", span { class: "oxsst-lb", "{title}" } p { class: "oxsst-hint", "{line}" } }
                            div { class: "oxsst-rc oxs6-gap",
                                if !done { button { r#type: "button", class: if is_next { "oxscn-btn oxs7-go" } else { "oxscn-btn" }, onclick: move |_| on_open.call(page), "Open {pl}" } }
                                button { r#type: "button", class: "oxscn-btn", "aria-pressed": "{done}", "aria-label": if done { "Undo {title}" } else { "Mark {title} done" },
                                    onclick: move |_| { now(ctx, |w| w.steps[i] = !done); say(ctx, if done { format!("“{title}” open again") } else { format!("“{title}” done") }); },
                                    if done { "Undo" } else { "Mark done" } }
                            }
                        }
                    }
                }
            }
        }
        {sec("Checklist", rsx! {}, rsx! {
            {row("oxs7-show", "Show in the sidebar", "A small progress card above your account until every step is done", switch("oxs7-show", "Show the checklist in the sidebar", p.show_checklist, move |v| cur.write().ws.show_checklist = v), ctx.dirty(|x| x.ws.show_checklist), ctx.hit("Hide the checklist"))}
        })}
    }
}
