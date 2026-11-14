//! The composer (`docs/specs/composer.md`): a working input shell with attachments, model picker,
//! effort control, context meter and status tab, in the six free layouts. One derived
//! [`ComposerState`] drives every part; the parts are also exported on their own.

mod attachments;
mod context;
mod effort;
pub(crate) mod icons;
mod model;
mod parts;
mod send;

use std::{cell::RefCell, rc::Rc, time::Duration};

use dioxus::prelude::*;
use oxyn_core::{
    AgentState, ClockMs,
    composer::{
        AttachmentItem, AttachmentKind, BusyAction, ComposerLayout, ComposerPhase, ComposerState,
        ContextMeter, ContextUsage, DisabledReason, DraftNeeds, EffortControl, EffortLevel,
        EffortSetting, InputVariant, MidRunPolicy, ModelCatalog, ModelSelection, Picker, PickerKey,
        Placeholder, ReasoningSupport, SelectOutcome, SendAction, SendControl, SendGate, SendState,
        SendVariant, StopConfirm, Surface, TurnSettings,
        attachments::PendingPolicy,
        effort::{EffortFx, band_position, prompt_glyph},
        input::busy_hint,
        model::{PickerEvent, select},
    },
};

pub use attachments::{AttachmentRow, AttachmentRowProps, DropZone, DropZoneProps};
pub use context::{ContextMeterChip, ContextMeterChipProps};
pub use effort::{EffortChip, EffortChipProps, tier_color};
pub use icons::{Icon, IconProps};
pub use model::{ModelPicker, ModelPickerProps};
pub use parts::{
    AgentSteering, AgentSteeringProps, DictationButton, DictationButtonProps, MentionInput,
    MentionInputProps, ModeSwitch, ModeSwitchProps, PARTS_CSS, PermissionSelector,
    PermissionSelectorProps, QueueStack, QueueStackProps, ToolsMenu, ToolsMenuProps, VoiceRecorder,
    VoiceRecorderProps,
};
pub use send::{SendButton, SendButtonProps};

use crate::{ElapsedTimer, GlyphLoader, VerbCycler};

/// What the composer hands the host on send.
#[derive(Clone, Debug, PartialEq)]
pub struct Submission {
    /// The draft text.
    pub text: String,
    /// Attachment ids that go with it.
    pub attachments: Vec<u64>,
    /// Settings for this turn.
    pub settings: TurnSettings,
    /// What the user meant: a new turn, or queue / steer while busy.
    pub busy: Option<BusyAction>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Pop {
    Effort,
    Context,
}

/// The whole composer.
#[component]
pub fn Composer(
    /// `chat` · `ide-panel` · `desktop` · `mobile` · `terminal` · `node`.
    #[props(default = "chat".to_string())]
    layout: String,
    /// The agent state.
    agent: AgentState,
    /// Shared clock time.
    now: ClockMs,
    /// Elapsed run time (status tab).
    #[props(default)]
    elapsed: Duration,
    /// The host's model catalogue.
    catalog: ModelCatalog,
    /// Initial model.
    #[props(default)]
    model: ModelSelection,
    /// Favourite model ids.
    #[props(default)]
    favorites: Vec<String>,
    /// Recent model ids, newest first.
    #[props(default)]
    recent: Vec<String>,
    /// Initial effort.
    #[props(default)]
    effort: EffortSetting,
    /// Context usage report.
    #[props(default)]
    usage: Option<ContextUsage>,
    /// Attachments (host-owned).
    #[props(default)]
    attachments: Vec<AttachmentItem>,
    /// What Enter does while busy.
    #[props(default)]
    busy_action: BusyAction,
    /// Why sending is off.
    #[props(default)]
    disabled: Option<DisabledReason>,
    /// Initial draft text.
    #[props(default)]
    text: String,
    /// Popover open at first render (`model`, `effort`, `context`; for demos and tests).
    #[props(default)]
    open: String,
    /// Drag state driven by the host (`armed`, `valid`, `invalid`), with its label.
    #[props(default)]
    drop: Option<(String, String)>,
    /// Draw the focus ring (demos).
    #[props(default)]
    focused: bool,
    /// Repo / branch / env pills under the desktop layout.
    #[props(default)]
    tray: Vec<String>,
    /// Placeholder copy.
    #[props(default)]
    placeholder: Placeholder,
    /// Override the layout's input variant.
    #[props(default)]
    input_variant: Option<String>,
    /// Override the attachment variant.
    #[props(default)]
    attachment_variant: Option<String>,
    /// Override the model picker variant.
    #[props(default)]
    model_variant: Option<String>,
    /// Override the effort variant.
    #[props(default)]
    effort_variant: Option<String>,
    /// Override the context meter variant.
    #[props(default)]
    context_variant: Option<String>,
    /// Override the send control variant.
    #[props(default)]
    send_variant: Option<String>,
    /// The last send failed with this reason (send shows ⟲ Retry).
    #[props(default)]
    error: Option<String>,
    /// Dictation strip open (send shows ✓).
    #[props(default)]
    dictating: bool,
    /// Stop confirmation: `double-esc` or `hold` (touch, 400 ms).
    #[props(default)]
    stop_confirm: Option<String>,
    /// Time or token budget used, `0..=1` (ring-progress).
    #[props(default)]
    budget: Option<f32>,
    /// Reduced motion.
    #[props(default)]
    reduced: bool,
    /// Pause (`true`) or resume (`false`) pressed.
    #[props(default)]
    on_pause: EventHandler<bool>,
    /// Retry pressed.
    #[props(default)]
    on_retry: EventHandler<()>,
    /// Answer submitted to the pending card.
    #[props(default)]
    on_answer: EventHandler<String>,
    /// Dictated text accepted.
    #[props(default)]
    on_accept: EventHandler<()>,
    /// Split ▾ or busy "ask" pressed: the host shows its choices.
    #[props(default)]
    on_choose: EventHandler<()>,
    /// Send pressed or Enter.
    #[props(default)]
    on_submit: EventHandler<Submission>,
    /// Stop pressed while busy.
    #[props(default)]
    on_stop: EventHandler<()>,
    /// An attachment removed.
    #[props(default)]
    on_remove: EventHandler<u64>,
    /// Files dropped or picked: `(name, bytes)`.
    #[props(default)]
    on_files: EventHandler<Vec<(String, u64)>>,
    /// The attach button.
    #[props(default)]
    on_attach: EventHandler<()>,
    /// "Compact now".
    #[props(default)]
    on_compact: EventHandler<()>,
) -> Element {
    let lay = ComposerLayout::from_id(&layout).unwrap_or_default();
    let d = lay.defaults();
    let input_v = input_variant
        .as_deref()
        .and_then(InputVariant::from_id)
        .unwrap_or(d.input);
    let att_v = attachment_variant.unwrap_or_else(|| d.attachments.id().to_string());
    let model_v = model_variant.unwrap_or_else(|| d.model.id().to_string());
    let effort_v = effort_variant.unwrap_or_else(|| d.effort.id().to_string());
    let ctx_v = context_variant.unwrap_or_else(|| d.context.id().to_string());
    let send_v = send_variant
        .as_deref()
        .and_then(SendVariant::from_id)
        .unwrap_or(d.send);

    let init_open = open.clone();
    let mut draft = use_signal(|| text.clone());
    let mut model_sel = use_signal(|| model.clone());
    let mut pending_model = use_signal(|| None::<ModelSelection>);
    let mut picker = use_signal(|| {
        let mut p = Picker::default();
        if init_open == "model" {
            let rows = p.rows(&catalog, &favorites, &recent, &model);
            p.open(&rows, &model);
        }
        p
    });
    let mut pop = use_signal(|| match init_open.as_str() {
        "effort" => Some(Pop::Effort),
        "context" => Some(Pop::Context),
        _ => None,
    });
    let mut drag = use_signal(|| None::<(String, String)>);
    let mut bump = use_signal(|| 0u32);
    let effort_ctl = use_hook(|| {
        let mut c = EffortControl::new(effort.clone());
        c.reduced = reduced;
        Rc::new(RefCell::new(c))
    });
    let meter = use_hook(|| Rc::new(RefCell::new(ContextMeter::default())));
    let held = use_hook(|| Rc::new(RefCell::new(None::<oxyn_core::composer::Held<String>>)));
    let was_busy = use_hook(|| Rc::new(RefCell::new(false)));
    let mut paused = use_signal(|| false);
    let send_ctl = use_hook(|| {
        let mut c = SendControl::new(send_v);
        c.reduced = reduced;
        c.stop_confirm = match stop_confirm.as_deref() {
            Some("double-esc") => StopConfirm::DoubleEsc,
            Some("hold") => StopConfirm::Hold(Duration::from_millis(400)),
            _ => StopConfirm::None,
        };
        Rc::new(RefCell::new(c))
    });
    send_ctl.borrow_mut().variant = send_v;
    let _ = bump();

    // Model and reasoning support for the next turn.
    let sel = model_sel();
    let support = support_for(&catalog, &sel);
    let model_name = sel.label(&catalog);
    let busy_now = oxyn_core::composer::mid_run(&agent);
    {
        let mut wb = was_busy.borrow_mut();
        if *wb && !busy_now {
            effort_ctl.borrow_mut().run_ended(&support, now);
            let p = pending_model.peek().clone();
            if let Some(p) = p {
                model_sel.set(p);
                pending_model.set(None);
            }
        }
        *wb = busy_now;
    }

    // Derived composer state.
    let text_now = draft();
    let uploading = attachments.iter().filter(|a| a.is_pending()).count();
    let summary = oxyn_core::composer::DraftSummary {
        is_empty: text_now.trim().is_empty() && attachments.is_empty(),
        chars: text_now.chars().count(),
        attachments: attachments.len(),
        uploading,
        ime_composing: false,
        over_limit: false,
    };
    let effort_view = effort_ctl.borrow_mut().view(&support, now);
    let state = ComposerState::derive(
        &agent,
        summary,
        disabled.clone(),
        busy_action,
        TurnSettings {
            model: sel.clone(),
            effort: effort_view.setting.clone(),
        },
        match lay {
            ComposerLayout::IdePanel => Surface::Ide,
            ComposerLayout::Mobile => Surface::Mobile,
            ComposerLayout::Terminal => Surface::Terminal,
            ComposerLayout::Desktop => Surface::Desktop,
            _ => Surface::Chat,
        },
    );
    let send_state = SendState::derive(&state, error.as_deref(), dictating, paused());
    let send_view =
        send_ctl
            .borrow_mut()
            .view(send_state.clone(), busy_action, elapsed, budget, now);
    // The reason line already says why sending is off; the placeholder keeps its normal copy.
    let ph_now = placeholder.text(&agent, busy_action, None);
    let ph = {
        let mut h = held.borrow_mut();
        let h = h.get_or_insert_with(|| {
            oxyn_core::composer::Held::new(
                ph_now.clone(),
                oxyn_core::composer::timing::PLACEHOLDER_HOLD,
                now,
            )
        });
        h.update(ph_now, now).clone()
    };
    let ctx_view = usage.as_ref().map(|u| meter.borrow_mut().view(u, now));
    let needs = DraftNeeds {
        images: attachments
            .iter()
            .filter(|a| matches!(a.kind, AttachmentKind::Image { .. }))
            .count(),
        tools: false,
        context_tokens: usage.as_ref().map(|u| u.used()).unwrap_or(0),
    };
    let gate = SendGate::of(&attachments, PendingPolicy::Block);
    let hint = if let Some(d) = &disabled {
        Some((d.label(), "reason"))
    } else if state.phase == ComposerPhase::BusyWithDraft {
        Some((busy_hint(busy_action).to_string(), ""))
    } else {
        gate.hint().map(|h| (h, ""))
    };

    // Event closures.
    let catalog_c = catalog.clone();
    let agent_c = agent.clone();
    let support_c = support.clone();
    let effort_c = effort_ctl.clone();
    let choose = move |s: ModelSelection| match select(
        &catalog_c,
        s,
        &agent_c,
        MidRunPolicy::NextTurn,
        &needs,
        now,
    ) {
        SelectOutcome::Applied(s) | SelectOutcome::Confirm { selection: s, .. } => {
            let sup = support_for(&catalog_c, &s);
            let name = s.label(&catalog_c);
            effort_c.borrow_mut().model_switched(&sup, &name, now);
            model_sel.set(s);
            pending_model.set(None);
            picker.write().open = false;
        }
        SelectOutcome::PendingNext(s) | SelectOutcome::Restart(s) | SelectOutcome::Ask(s) => {
            pending_model.set(Some(s));
            picker.write().open = false;
        }
        SelectOutcome::NeedsKey(_) | SelectOutcome::Disabled(_) => {}
    };
    let effort_pick = {
        let effort_c = effort_ctl.clone();
        let agent_c = agent.clone();
        let support_c = support_c.clone();
        move |s: EffortSetting| {
            effort_c.borrow_mut().commit(s, &agent_c, &support_c, now);
            pop.set(None);
            bump += 1;
        }
    };
    let att_ids: Vec<u64> = attachments
        .iter()
        .filter(|a| !a.is_error())
        .map(|a| a.id)
        .collect();
    let settings = state.next_turn.clone();
    let dispatch = move |a: SendAction| {
        let emit = |busy: Option<BusyAction>| {
            on_submit.call(Submission {
                text: draft.peek().clone(),
                attachments: att_ids.clone(),
                settings: settings.clone(),
                busy,
            });
        };
        match a {
            SendAction::Send => emit(None),
            SendAction::Queue => emit(Some(BusyAction::Queue)),
            SendAction::Steer => emit(Some(BusyAction::Steer)),
            SendAction::Interrupt => emit(Some(BusyAction::Interrupt)),
            SendAction::Answer => on_answer.call(draft.peek().clone()),
            SendAction::Stop => {
                paused.set(false);
                on_stop.call(());
                return;
            }
            SendAction::Pause => {
                paused.set(true);
                on_pause.call(true);
                return;
            }
            SendAction::Resume => {
                paused.set(false);
                on_pause.call(false);
                return;
            }
            SendAction::Accept => {
                on_accept.call(());
                return;
            }
            SendAction::Retry => {
                on_retry.call(());
                return;
            }
            SendAction::Choose => {
                on_choose.call(());
                return;
            }
            SendAction::None => return,
        }
        draft.set(String::new());
    };

    // Ignition wash.
    let wash = match effort_view.fx {
        Some(EffortFx::Igniting { tier, t }) if !reduced => {
            let x = band_position(t) * 140.0 - 20.0;
            let c = tier_color(tier);
            Some(format!(
                "background:linear-gradient(100deg,transparent {:.0}%,color-mix(in oklab,{c} 28%,transparent) {x:.0}%,transparent {:.0}%);opacity:{:.2}",
                x - 30.0,
                x + 30.0,
                1.0 - t * t
            ))
        }
        _ => None,
    };
    let drop_state = drag().or(drop.clone());
    let rows = input_v.rows();
    let terminal = lay == ComposerLayout::Terminal;
    let glyph_c = tier_color(effort_view.tier);
    let shell_class = format!(
        "shell {}{}{}{}",
        input_v.id(),
        if focused { " focus" } else { "" },
        if drop_state.is_some() {
            " dragging"
        } else {
            ""
        },
        if disabled.is_some() { " disabled" } else { "" }
    );
    let send = {
        let send_c = send_ctl.clone();
        let send_c2 = send_ctl.clone();
        let mut d1 = dispatch.clone();
        let mut d2 = dispatch.clone();
        rsx! {
            SendButton {
                view: send_view.clone(),
                variant: send_v.id().to_string(),
                on_action: move |a| { let a = send_c.borrow_mut().press(a, now); d1(a); bump += 1; },
                on_hold: move |down: bool| {
                    let mut c = send_c2.borrow_mut();
                    if down { c.hold_start(now) } else { let a = c.hold_end(now); std::mem::drop(c); d2(a); }
                },
            }
        }
    };
    let lay_cls = &lay.id()[9..];
    let running = agent.is_busy() || agent.is_finished();
    let show_tab = running && !terminal;
    let model_label = model_name.clone();
    let model_pick = {
        let mut choose = choose;
        move |s: ModelSelection| choose(s)
    };
    let picker_now = picker();
    let routing = sel.routing(&agent);

    let footer_model = rsx! {
        ModelPicker {
            catalog: catalog.clone(), selected: sel.clone(), pending: pending_model(), picker: picker_now.clone(),
            favorites: favorites.clone(), recent: recent.clone(), needs, routing, variant: model_v.clone(), now,
            on_toggle: {
                let catalog = catalog.clone(); let favorites = favorites.clone(); let recent = recent.clone(); let sel = sel.clone();
                move |_| {
                    let mut p = picker.write();
                    if p.open { p.open = false } else {
                        let rows = p.rows(&catalog, &favorites, &recent, &sel);
                        p.open(&rows, &sel);
                        pop.set(None);
                    }
                }
            },
            on_key: {
                let catalog = catalog.clone(); let favorites = favorites.clone(); let recent = recent.clone(); let sel = sel.clone();
                let mut model_pick = model_pick.clone();
                move |k: PickerKey| {
                    let rows = picker.peek().rows(&catalog, &favorites, &recent, &sel);
                    let ev = picker.write().key(k, &rows);
                    if let PickerEvent::Choose(s) = ev { model_pick(s) }
                }
            },
            on_query: move |q: String| { let mut p = picker.write(); p.query = q; p.highlighted = 0; },
            on_choose: { let mut model_pick = model_pick.clone(); move |s| model_pick(s) },
            on_cancel_pending: move |_| pending_model.set(None),
        }
    };
    let footer_effort = rsx! {
        EffortChip {
            view: effort_view.clone(), support: support.clone(), variant: effort_v.clone(),
            open: pop() == Some(Pop::Effort), model: model_label.clone(),
            on_toggle: move |_| { let o = pop() == Some(Pop::Effort); pop.set(if o { None } else { Some(Pop::Effort) }); picker.write().open = false; },
            on_pick: { let mut f = effort_pick.clone(); move |s| f(s) },
        }
    };
    let footer_ctx = rsx! {
        if let (Some(v), Some(u)) = (ctx_view.clone(), usage.clone()) {
            ContextMeterChip {
                view: v, usage: u, variant: ctx_v.clone(), open: pop() == Some(Pop::Context), model: model_label.clone(),
                on_toggle: move |_| { let o = pop() == Some(Pop::Context); pop.set(if o { None } else { Some(Pop::Context) }); picker.write().open = false; },
                on_compact: move |_| { on_compact.call(()); pop.set(None); },
            }
        }
    };

    rsx! {
        form { class: "oxyn-composer {lay_cls}", "aria-label": "Message the agent",
            onsubmit: {
                let send_c = send_ctl.clone();
                let st = send_state.clone();
                let mut dispatch = dispatch.clone();
                move |e: FormEvent| {
                    e.prevent_default();
                    let a = send_c.borrow_mut().enter(&st, busy_action, false, now);
                    dispatch(a);
                }
            },
            if show_tab {
                div { class: "oxyn-tab", role: "status",
                    GlyphLoader { state: agent.clone(), now, size: 13.0 }
                    VerbCycler { state: agent.clone(), now, variant: "fade".to_string(), pack: "coder".to_string(), size: 13.0, reduced }
                    if agent.is_busy() {
                        span { class: "sep", "·" }
                        ElapsedTimer { state: agent.clone(), elapsed, variant: "clock".to_string(), size: 12.5 }
                        button { r#type: "button", class: "esc", onclick: move |_| on_stop.call(()), "esc to stop" }
                    }
                }
            }
            div { class: "{shell_class}",
                ondragover: move |e| { e.prevent_default(); if drag.peek().is_none() { drag.set(Some(("valid".into(), "Drop to attach".into()))); } },
                ondragleave: move |_| drag.set(None),
                ondrop: move |e| {
                    e.prevent_default();
                    drag.set(None);
                    let files: Vec<(String, u64)> = e.data_transfer().files().iter().map(|f| (f.name(), f.size())).collect();
                    if !files.is_empty() { on_files.call(files); }
                },
                if let Some(w) = wash { div { class: "wash", style: "{w}" } }
                AttachmentRow { items: attachments.clone(), now, variant: att_v.clone(), reduced, on_remove }
                div { class: "text-row",
                    if terminal { span { class: "prompt", style: "color:{glyph_c}", "{prompt_glyph(effort_view.tier)}" } }
                    textarea {
                        class: "input",
                        rows: "{rows.min}",
                        style: "--min-rows:{rows.min};--max-rows:{rows.max}",
                        placeholder: "{ph}",
                        value: "{text_now}",
                        "aria-label": "Message",
                        "aria-disabled": "{disabled.is_some()}",
                        oninput: move |e| draft.set(e.value()),
                        onkeydown: {
                            let effort_c = effort_ctl.clone();
                            let agent = agent.clone();
                            let support = support.clone();
                            let send_c = send_ctl.clone();
                            let st = send_state.clone();
                            let mut dispatch = dispatch.clone();
                            move |e: KeyboardEvent| {
                                let m = e.modifiers();
                                match e.key() {
                                    Key::Enter if !m.shift() && !e.is_composing() => {
                                        e.prevent_default();
                                        let a = send_c.borrow_mut().enter(&st, busy_action, m.meta() || m.ctrl(), now);
                                        dispatch(a);
                                    }
                                    Key::Escape if picker.peek().open || pop.peek().is_some() => { picker.write().open = false; pop.set(None); }
                                    Key::Escape => {
                                        let a = send_c.borrow_mut().escape(&st, now);
                                        dispatch(a);
                                        bump += 1;
                                    }
                                    Key::Character(c) if m.alt() && (c == "." || c == "≥" || c == "," || c == "≤") => {
                                        e.prevent_default();
                                        effort_c.borrow_mut().step(c == "." || c == "≥", &agent, &support, now);
                                        bump += 1;
                                    }
                                    _ => {}
                                }
                            }
                        },
                        // Initial content for server-rendered pages; the live value comes from `value`.
                        "{text_now}"
                    }
                    if terminal || input_v.inline_actions(1) && lay != ComposerLayout::Chat { {send.clone()} }
                }
                if let Some((h, cls)) = hint { div { class: "hint {cls}", "{h}" } }
                if terminal {
                    div { class: "term-foot",
                        {footer_model}
                        span { class: "sep", "·" }
                        {footer_effort}
                        span { class: "sep", "·" }
                        {footer_ctx}
                        span { class: "grow" }
                        span { class: "keys", "⏎ send · ⇧⏎ newline · ⌥. effort" }
                    }
                } else if !(input_v.inline_actions(1) && lay != ComposerLayout::Chat) || lay == ComposerLayout::Mobile {
                    div { class: "footer", role: "toolbar", "aria-label": "Composer controls",
                        button { r#type: "button", class: "tool", "aria-label": "Attach files", title: "Attach files", onclick: move |_| on_attach.call(()),
                            Icon { name: "plus", size: 16.0, stroke: 1.75 }
                        }
                        {footer_model}
                        if lay != ComposerLayout::Mobile { {footer_effort} }
                        span { class: "grow" }
                        if lay != ComposerLayout::Mobile { {footer_ctx} }
                        {send}
                    }
                } else {
                    div { class: "footer", role: "toolbar", {footer_model} {footer_effort} span { class: "grow" } {footer_ctx} }
                }
                if let Some((st, label)) = drop_state {
                    DropZone { state: st, label, variant: "inline".to_string() }
                }
            }
            if lay == ComposerLayout::Desktop && !tray.is_empty() {
                div { class: "tray",
                    for (i, t) in tray.iter().enumerate() {
                        span { class: "pill",
                            Icon { name: match i { 0 => "folder", 1 => "branch", _ => "spark" }, size: 12.0 }
                            "{t}"
                        }
                    }
                }
            }
        }
    }
}

fn support_for(catalog: &ModelCatalog, sel: &ModelSelection) -> ReasoningSupport {
    sel.model_id()
        .and_then(|id| catalog.get(id))
        .or_else(|| {
            catalog
                .models
                .iter()
                .find(|m| !matches!(m.caps.reasoning, ReasoningSupport::None))
        })
        .map(|m| m.caps.reasoning.clone())
        .unwrap_or(ReasoningSupport::Levels {
            allowed: vec![EffortLevel::Low, EffortLevel::Medium, EffortLevel::High],
            default: EffortLevel::Medium,
        })
}

/// Stylesheet for the composer and its parts.
pub const COMPOSER_CSS: &str = include_str!("composer.css");

#[cfg(test)]
mod tests {
    use super::*;
    use oxyn_core::composer::{Capabilities, ModelInfo, ModelStatus};

    fn catalog() -> ModelCatalog {
        ModelCatalog {
            providers: vec![("p".into(), "Provider".into())],
            models: vec![ModelInfo {
                id: "p/a".into(),
                provider: "p".into(),
                name: "Alpha 1".into(),
                short_name: None,
                caps: Capabilities {
                    vision: true,
                    tools: true,
                    reasoning: ReasoningSupport::Levels {
                        allowed: vec![EffortLevel::Low, EffortLevel::Medium, EffortLevel::High],
                        default: EffortLevel::Medium,
                    },
                },
                context_tokens: 200_000,
                speed: None,
                cost: None,
                status: ModelStatus::Available,
            }],
            auto: true,
        }
    }

    #[derive(Clone)]
    struct Args(AgentState, String, String, Option<DisabledReason>);

    fn app(a: Args) -> Element {
        rsx! {
            Composer {
                agent: a.0,
                now: ClockMs(5_000),
                catalog: catalog(),
                model: ModelSelection::Model("p/a".into()),
                text: a.1,
                open: a.2,
                disabled: a.3,
            }
        }
    }

    fn html(agent: AgentState, text: &str, open: &str, disabled: Option<DisabledReason>) -> String {
        let mut dom = VirtualDom::new_with_props(
            app,
            Args(agent, text.to_string(), open.to_string(), disabled),
        );
        dom.rebuild_in_place();
        dioxus_ssr::render(&dom)
    }

    #[test]
    fn idle_with_model_popover() {
        let h = html(AgentState::Idle, "hello", "model", None);
        assert!(h.contains("Search models") && h.contains("Alpha 1"));
        assert!(
            h.contains("seg primary") && h.contains("aria-label=\"Send message\""),
            "draft makes send ready"
        );
        assert!(h.contains("Ask anything"));
    }

    #[test]
    fn busy_draft_queues_and_shows_tab() {
        let h = html(AgentState::Streaming, "also this", "", None);
        assert!(h.contains("oxyn-send queue-pill") && h.contains(">Queue<"));
        assert!(h.contains("oxyn-tab") && h.contains("Enter to queue"));
        let h = html(AgentState::Streaming, "", "", None);
        assert!(h.contains("aria-label=\"Stop generating\""));
    }

    #[test]
    fn offline_keeps_draft_and_says_why() {
        let h = html(
            AgentState::Idle,
            "kept",
            "effort",
            Some(DisabledReason::Offline),
        );
        assert!(h.contains("offline · your draft is saved") && h.contains(">kept<"));
        assert!(h.contains("seg muted"));
        assert!(h.contains("Deeper reasoning, slower"), "effort menu open");
    }
}
