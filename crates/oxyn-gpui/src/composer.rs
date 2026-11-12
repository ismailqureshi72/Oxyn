//! `Composer` for GPUI (`docs/specs/composer.md`): the chat layout with the default part variants
//! (rounded-box input, tile attachments with upload rings, pill + popover model picker,
//! signal-bars effort with menu, context ring with breakdown, send / stop / queue).
//!
//! It is an entity: create it with [`Composer::new`], feed it the agent state, attachments, usage
//! and the shared clock time each frame, and listen for [`ComposerEvent`]s. Text editing is a
//! single-line-wrapped draft with Enter / Shift+Enter / Backspace and `⌥.` / `⌥,` for effort; IME
//! and selection come with the input handler in a follow-up PR.

use std::collections::HashMap;

use gpui::{
    App, Context, EventEmitter, FocusHandle, Focusable, FontWeight, Hsla, InteractiveElement,
    IntoElement, KeyDownEvent, MouseButton, ParentElement, PathBuilder, Pixels, Point, Render,
    SharedString, Styled, Window, canvas, div, point, prelude::*, px,
};
use oxyn_core::{
    AgentState, ClockMs,
    composer::{
        AttachmentItem, BusyAction, ComposerPhase, ComposerState, ContextLevel, ContextMeter,
        ContextUsage, DisabledReason, DraftNeeds, DraftSummary, EffortControl, EffortLevel,
        EffortSetting, MidRunPolicy, ModelCatalog, ModelSelection, Picker, PickerKey, PickerRow,
        Placeholder, ReasoningSupport, SelectOutcome, SubmitAction, Surface, TurnSettings,
        UploadRing,
        attachments::RingFrame,
        context::LabelMode,
        effort::{EffortFx, allowed, bars},
        input::busy_hint,
        model::{PickerEvent, format_context, select, status_note},
    },
};
use oxyn_tokens::{ColorRole, Theme};

use crate::{hsla, mix};

/// What the composer tells its host.
#[derive(Clone, Debug, PartialEq)]
pub enum ComposerEvent {
    /// Send (or queue / steer while busy).
    Submit {
        /// Draft text.
        text: String,
        /// Attachment ids that go with it.
        attachments: Vec<u64>,
        /// Settings for the turn.
        settings: TurnSettings,
        /// Queue / steer while busy; `None` for a new turn.
        busy: Option<BusyAction>,
    },
    /// Stop pressed.
    Stop,
    /// An attachment's × pressed.
    Remove(u64),
    /// The attach button.
    Attach,
    /// "Compact now".
    Compact,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Pop {
    Model,
    Effort,
    Context,
}

/// The composer entity.
pub struct Composer {
    focus: FocusHandle,
    theme: &'static Theme,
    agent: AgentState,
    now: ClockMs,
    draft: String,
    attachments: Vec<AttachmentItem>,
    rings: HashMap<u64, UploadRing>,
    catalog: ModelCatalog,
    selection: ModelSelection,
    pending_model: Option<ModelSelection>,
    favorites: Vec<String>,
    recent: Vec<String>,
    picker: Picker,
    effort: EffortControl,
    meter: ContextMeter,
    usage: Option<ContextUsage>,
    busy_action: BusyAction,
    disabled: Option<DisabledReason>,
    placeholder: Placeholder,
    pop: Option<Pop>,
    was_busy: bool,
}

impl EventEmitter<ComposerEvent> for Composer {}

impl Focusable for Composer {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus.clone()
    }
}

impl Composer {
    /// A composer over `catalog`, starting on `model`.
    pub fn new(catalog: ModelCatalog, model: ModelSelection, cx: &mut Context<Self>) -> Self {
        Self {
            focus: cx.focus_handle(),
            theme: &Theme::DARK,
            agent: AgentState::Idle,
            now: ClockMs(0),
            draft: String::new(),
            attachments: Vec::new(),
            rings: HashMap::new(),
            catalog,
            selection: model,
            pending_model: None,
            favorites: Vec::new(),
            recent: Vec::new(),
            picker: Picker::default(),
            effort: EffortControl::new(EffortSetting::default()),
            meter: ContextMeter::default(),
            usage: None,
            busy_action: BusyAction::Queue,
            disabled: None,
            placeholder: Placeholder::default(),
            pop: None,
            was_busy: false,
        }
    }

    /// Theme.
    pub fn set_theme(&mut self, theme: &'static Theme) {
        self.theme = theme;
    }

    /// Favourite and recent models.
    pub fn set_models(&mut self, favorites: Vec<String>, recent: Vec<String>) {
        self.favorites = favorites;
        self.recent = recent;
    }

    /// The agent state and shared clock time, once per frame. Pending model and effort changes
    /// apply when a run ends.
    pub fn set_agent(&mut self, agent: AgentState, now: ClockMs) {
        let busy = oxyn_core::composer::mid_run(&agent);
        if self.was_busy && !busy {
            let s = self.support();
            self.effort.run_ended(&s, now);
            if let Some(p) = self.pending_model.take() {
                self.selection = p;
            }
        }
        self.was_busy = busy;
        self.agent = agent;
        self.now = now;
    }

    /// Attachments (host-owned).
    pub fn set_attachments(&mut self, items: Vec<AttachmentItem>) {
        self.rings.retain(|id, _| items.iter().any(|i| i.id == *id));
        self.attachments = items;
    }

    /// Context usage report.
    pub fn set_usage(&mut self, usage: Option<ContextUsage>) {
        self.usage = usage;
    }

    /// Effort setting.
    pub fn set_effort(&mut self, effort: EffortSetting) {
        self.effort = EffortControl::new(effort);
    }

    /// Busy policy.
    pub fn set_busy_action(&mut self, b: BusyAction) {
        self.busy_action = b;
    }

    /// Why sending is off.
    pub fn set_disabled(&mut self, d: Option<DisabledReason>) {
        self.disabled = d;
    }

    /// The draft text.
    pub fn set_text(&mut self, text: impl Into<String>) {
        self.draft = text.into();
    }

    /// Frames are needed (an effect, a ring or a roll is moving).
    pub fn animating(&self) -> bool {
        self.effort.animating()
            || self.meter.animating(self.now)
            || self.attachments.iter().any(AttachmentItem::is_pending)
    }

    fn support(&self) -> ReasoningSupport {
        self.selection
            .model_id()
            .and_then(|id| self.catalog.get(id))
            .or_else(|| {
                self.catalog
                    .models
                    .iter()
                    .find(|m| !matches!(m.caps.reasoning, ReasoningSupport::None))
            })
            .map(|m| m.caps.reasoning.clone())
            .unwrap_or_default()
    }

    fn state(&self) -> ComposerState {
        let summary = DraftSummary {
            is_empty: self.draft.trim().is_empty() && self.attachments.is_empty(),
            chars: self.draft.chars().count(),
            attachments: self.attachments.len(),
            uploading: self.attachments.iter().filter(|a| a.is_pending()).count(),
            ime_composing: false,
            over_limit: false,
        };
        ComposerState::derive(
            &self.agent,
            summary,
            self.disabled.clone(),
            self.busy_action,
            TurnSettings {
                model: self.selection.clone(),
                effort: self.effort.setting().clone(),
            },
            Surface::Chat,
        )
    }

    fn needs(&self) -> DraftNeeds {
        DraftNeeds {
            images: self
                .attachments
                .iter()
                .filter(|a| a.file_type() == oxyn_core::composer::FileType::Image)
                .count(),
            tools: false,
            context_tokens: self.usage.as_ref().map(|u| u.used()).unwrap_or(0),
        }
    }

    fn rows(&self) -> Vec<PickerRow> {
        self.picker.rows(
            &self.catalog,
            &self.favorites,
            &self.recent,
            &self.selection,
        )
    }

    fn choose(&mut self, s: ModelSelection) {
        match select(
            &self.catalog,
            s,
            &self.agent,
            MidRunPolicy::NextTurn,
            &self.needs(),
            self.now,
        ) {
            SelectOutcome::Applied(s) | SelectOutcome::Confirm { selection: s, .. } => {
                self.selection = s;
                self.pending_model = None;
                let sup = self.support();
                let name = self.selection.label(&self.catalog);
                self.effort.model_switched(&sup, &name, self.now);
                self.picker.open = false;
                self.pop = None;
            }
            SelectOutcome::PendingNext(s) | SelectOutcome::Restart(s) | SelectOutcome::Ask(s) => {
                self.pending_model = Some(s);
                self.picker.open = false;
                self.pop = None;
            }
            SelectOutcome::NeedsKey(_) | SelectOutcome::Disabled(_) => {}
        }
    }

    fn toggle(&mut self, p: Pop) {
        if self.pop == Some(p) {
            self.pop = None;
            self.picker.open = false;
        } else {
            self.pop = Some(p);
            if p == Pop::Model {
                let rows = self.rows();
                let sel = self.selection.clone();
                self.picker.open(&rows, &sel);
            } else {
                self.picker.open = false;
            }
        }
    }

    fn submit(&mut self, cx: &mut Context<Self>) {
        let st = self.state();
        let busy = match st.submit_action() {
            SubmitAction::Send => None,
            SubmitAction::Busy(b) => Some(b),
            _ => return,
        };
        cx.emit(ComposerEvent::Submit {
            text: std::mem::take(&mut self.draft),
            attachments: self
                .attachments
                .iter()
                .filter(|a| !a.is_error())
                .map(|a| a.id)
                .collect(),
            settings: st.next_turn,
            busy,
        });
    }

    fn key(&mut self, ev: &KeyDownEvent, cx: &mut Context<Self>) {
        let k = &ev.keystroke;
        if self.picker.open {
            let pk = match k.key.as_str() {
                "up" => Some(PickerKey::Up),
                "down" => Some(PickerKey::Down),
                "enter" => Some(PickerKey::Enter),
                "escape" => Some(PickerKey::Escape),
                "backspace" => Some(PickerKey::Backspace),
                _ => k
                    .key_char
                    .as_deref()
                    .and_then(|c| c.chars().next())
                    .filter(|c| !c.is_control())
                    .map(PickerKey::Char),
            };
            if let Some(pk) = pk {
                let rows = self.rows();
                match self.picker.key(pk, &rows) {
                    PickerEvent::Choose(s) => self.choose(s),
                    PickerEvent::Close => self.pop = None,
                    PickerEvent::None => {}
                }
                cx.notify();
            }
            return;
        }
        match k.key.as_str() {
            "enter" if !k.modifiers.shift => self.submit(cx),
            "enter" => self.draft.push('\n'),
            "backspace" => {
                self.draft.pop();
            }
            "escape" => {
                if self.pop.is_some() {
                    self.pop = None;
                } else if self.agent.is_busy() {
                    cx.emit(ComposerEvent::Stop);
                }
            }
            "." | "," if k.modifiers.alt => {
                let s = self.support();
                let agent = self.agent.clone();
                self.effort.step(k.key == ".", &agent, &s, self.now);
            }
            _ => {
                if let Some(c) = k.key_char.as_deref()
                    && !k.modifiers.control
                    && !k.modifiers.platform
                {
                    self.draft.push_str(c);
                }
            }
        }
        cx.notify();
    }
}

fn c(t: &Theme, r: ColorRole) -> Hsla {
    hsla(t.color(r))
}

/// Paints an arc of a circle (`from` and `len` in turns, clockwise from 12 o'clock).
fn arc(
    window: &mut Window,
    center: Point<Pixels>,
    r: f32,
    from: f32,
    len: f32,
    w: f32,
    color: Hsla,
) {
    let len = len.clamp(0.0, 1.0);
    if len <= 0.001 {
        return;
    }
    let at = |turn: f32| {
        let a = (turn - 0.25) * std::f32::consts::TAU;
        point(center.x + px(r * a.cos()), center.y + px(r * a.sin()))
    };
    let mut b = PathBuilder::stroke(px(w));
    b.move_to(at(from));
    // Two halves so a full circle still draws.
    let mid = from + len / 2.0;
    b.arc_to(point(px(r), px(r)), px(0.), false, true, at(mid));
    b.arc_to(point(px(r), px(r)), px(0.), false, true, at(from + len));
    if let Ok(path) = b.build() {
        window.paint_path(path, color);
    }
}

fn ring(size: f32, segs: Vec<(f32, f32, Hsla)>, track: Hsla, w: f32) -> impl IntoElement {
    canvas(
        |_, _, _| {},
        move |bounds, _, window, _| {
            let center = bounds.center();
            let r = size / 2.0 - w;
            arc(window, center, r, 0.0, 0.9999, w, track);
            for (from, len, col) in &segs {
                arc(window, center, r, *from, *len, w, *col);
            }
        },
    )
    .size(px(size))
}

impl Render for Composer {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let t = self.theme;
        let now = self.now;
        let st = self.state();
        let support = self.support();
        let ev = self.effort.view(&support, now);
        let ctx = self.usage.clone().map(|u| (self.meter.view(&u, now), u));
        let focused = self.focus.is_focused(window);
        let accent = c(t, ColorRole::AccentSolid);
        let muted = c(t, ColorRole::FgMuted);
        let subtle = c(t, ColorRole::FgSubtle);

        // Attachments.
        let frames: Vec<RingFrame> = self
            .attachments
            .iter()
            .map(|a| self.rings.entry(a.id).or_default().update(&a.state, now))
            .collect();
        let tiles = self.attachments.iter().zip(frames).map(|(a, f)| {
            let id = a.id;
            let err = a.is_error();
            let ty = a.file_type();
            let word: SharedString = match ty {
                oxyn_core::composer::FileType::Code(b) => b.into(),
                _ => ty.word(&a.name).into(),
            };
            let pct = match a.state {
                oxyn_core::composer::AttachmentState::Uploading { progress: Some(p) } => {
                    Some(format!("{:.0}%", p * 100.0))
                }
                _ => None,
            };
            div()
                .relative()
                .size(px(56.))
                .rounded(px(10.))
                .bg(c(
                    t,
                    if err {
                        ColorRole::StatusDangerBg
                    } else {
                        ColorRole::SurfaceSunken
                    },
                ))
                .border_1()
                .border_color(c(
                    t,
                    if err {
                        ColorRole::StatusDangerBorder
                    } else {
                        ColorRole::BorderSubtle
                    },
                ))
                .flex()
                .flex_col()
                .items_center()
                .justify_center()
                .gap(px(2.))
                .child(
                    div()
                        .text_size(px(16.))
                        .text_color(if err {
                            c(t, ColorRole::StatusDangerFg)
                        } else {
                            muted
                        })
                        .child(ty.glyph()),
                )
                .child(
                    div()
                        .text_size(px(9.))
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(muted)
                        .child(word),
                )
                .when_some(
                    match f {
                        RingFrame::Arc { fraction, alpha } => Some(ring(
                            24.,
                            vec![(0.0, fraction, accent.opacity(alpha))],
                            c(t, ColorRole::LoaderTrack),
                            2.,
                        )),
                        RingFrame::Indeterminate { angle } => Some(ring(
                            24.,
                            vec![(angle, 0.25, accent)],
                            c(t, ColorRole::LoaderTrack),
                            2.,
                        )),
                        RingFrame::None => None,
                    },
                    |d, r| d.child(div().absolute().top(px(16.)).left(px(16.)).child(r)),
                )
                .when_some(pct, |d, p| {
                    d.child(
                        div()
                            .absolute()
                            .top(px(2.))
                            .right(px(4.))
                            .text_size(px(9.))
                            .text_color(c(t, ColorRole::FgDefault))
                            .child(p),
                    )
                })
                .child(
                    div()
                        .absolute()
                        .top(px(3.))
                        .right(px(3.))
                        .size(px(16.))
                        .rounded_full()
                        .bg(gpui::black().opacity(0.6))
                        .flex()
                        .items_center()
                        .justify_center()
                        .text_size(px(10.))
                        .text_color(gpui::white())
                        .child("×")
                        .on_mouse_down(
                            MouseButton::Left,
                            cx.listener(move |_, _, _, cx| cx.emit(ComposerEvent::Remove(id))),
                        ),
                )
        });

        // Text.
        let placeholder = self.placeholder.text(&self.agent, self.busy_action, None);
        let caret_on = focused && (now.0 / 530).is_multiple_of(2);
        let text_el = if self.draft.is_empty() {
            div().text_color(subtle).child(placeholder)
        } else {
            div().child(SharedString::from(self.draft.clone()))
        };
        let text_row = div()
            .flex()
            .items_end()
            .min_h(px(44.))
            .text_size(px(15.))
            .line_height(px(22.))
            .child(text_el)
            .when(caret_on, |d| {
                d.child(div().w(px(1.5)).h(px(18.)).mb(px(2.)).bg(accent))
            });

        let hint = if let Some(d) = &self.disabled {
            Some(d.label())
        } else if st.phase == ComposerPhase::BusyWithDraft {
            Some(busy_hint(self.busy_action).to_string())
        } else {
            None
        };

        // Model pill.
        let model_label = self.selection.label(&self.catalog);
        let pill = |label: SharedString| {
            div()
                .h(px(30.))
                .px(px(9.))
                .rounded_full()
                .flex()
                .items_center()
                .gap(px(6.))
                .text_size(px(13.))
                .text_color(muted)
                .cursor_pointer()
                .hover(|s| s.bg(c(t, ColorRole::SurfaceHover)))
                .child(label)
        };
        let model_pill = pill(model_label.clone().into())
            .child(div().text_size(px(10.)).child("⌄"))
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(|this, _, _, cx| {
                    this.toggle(Pop::Model);
                    cx.notify();
                }),
            );
        let pending = self.pending_model.as_ref().map(|p| {
            div()
                .h(px(20.))
                .px(px(6.))
                .rounded_full()
                .border_1()
                .border_dashed()
                .border_color(c(t, ColorRole::AccentBorder))
                .flex()
                .items_center()
                .text_size(px(11.))
                .text_color(c(t, ColorRole::AccentFg))
                .child(format!("→ {}", p.label(&self.catalog)))
        });

        // Effort chip.
        let levels = allowed(&support);
        let level = ev.setting.level().unwrap_or(EffortLevel::Medium);
        let lit = bars(level, &levels);
        let tier_c = match ev.tier {
            0 => muted,
            1 => c(t, ColorRole::StatusWarningFg),
            _ => c(t, ColorRole::AgentViolet),
        };
        let bars_el = div()
            .flex()
            .items_end()
            .gap(px(1.5))
            .h(px(10.))
            .children((0..4u8).map(|i| {
                div()
                    .w(px(2.5))
                    .h(px(4.0 + f32::from(i) * 2.0))
                    .rounded(px(1.))
                    .bg(if i < lit { tier_c } else { muted.opacity(0.3) })
            }));
        let effort_chip = (!levels.is_empty()).then(|| {
            pill(level.short().into()).child(bars_el).on_mouse_down(
                MouseButton::Left,
                cx.listener(|this, _, _, cx| {
                    this.toggle(Pop::Effort);
                    cx.notify();
                }),
            )
        });

        // Context ring.
        let ctx_chip = ctx.as_ref().map(|(v, _)| {
            let col = match v.level {
                ContextLevel::Warning => c(t, ColorRole::StatusWarningFg),
                ContextLevel::Critical | ContextLevel::Overflow => c(t, ColorRole::StatusDangerFg),
                _ => muted,
            };
            pill(v.label(LabelMode::Percent).into())
                .child(ring(
                    16.,
                    vec![(0.0, v.fill, col.opacity(v.alpha))],
                    c(t, ColorRole::LoaderTrack),
                    2.5,
                ))
                .on_mouse_down(
                    MouseButton::Left,
                    cx.listener(|this, _, _, cx| {
                        this.toggle(Pop::Context);
                        cx.notify();
                    }),
                )
        });

        // Send.
        let (send_bg, send_fg, glyph, label) = match (&st.phase, st.submit_action()) {
            (ComposerPhase::Busy | ComposerPhase::Submitted, _) => (
                c(t, ColorRole::FgDefault),
                c(t, ColorRole::BgBase),
                "■",
                None,
            ),
            (ComposerPhase::BusyWithDraft, _) => (
                c(t, ColorRole::AccentBg),
                c(t, ColorRole::AccentFg),
                "↑",
                Some(if self.busy_action == BusyAction::Steer {
                    "Steer"
                } else {
                    "Queue"
                }),
            ),
            (_, SubmitAction::Send) => (accent, c(t, ColorRole::FgOnAccent), "↑", None),
            _ => (c(t, ColorRole::SurfaceHover), subtle, "↑", None),
        };
        let stop = matches!(st.phase, ComposerPhase::Busy | ComposerPhase::Submitted);
        let send = div()
            .h(px(32.))
            .min_w(px(32.))
            .px(px(if label.is_some() { 12. } else { 0. }))
            .rounded_full()
            .bg(send_bg)
            .text_color(send_fg)
            .flex()
            .items_center()
            .justify_center()
            .gap(px(6.))
            .text_size(px(14.))
            .font_weight(FontWeight::SEMIBOLD)
            .cursor_pointer()
            .child(glyph)
            .when_some(label, |d, l| d.child(div().text_size(px(13.)).child(l)))
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(move |this, _, _, cx| {
                    if stop {
                        cx.emit(ComposerEvent::Stop)
                    } else {
                        this.submit(cx)
                    }
                    cx.notify();
                }),
            );

        // Popovers.
        let popover = |w: f32, right: bool| {
            div()
                .absolute()
                .bottom(px(40.))
                .when(right, |d| d.right(px(0.)))
                .when(!right, |d| d.left(px(0.)))
                .w(px(w))
                .p(px(6.))
                .rounded(px(14.))
                .bg(c(t, ColorRole::SurfaceOverlay))
                .border_1()
                .border_color(c(t, ColorRole::BorderDefault))
                .shadow_lg()
                .flex()
                .flex_col()
        };
        let model_pop = (self.picker.open).then(|| {
            let rows = self.rows();
            let hl = self.picker.highlighted_row(&rows).cloned();
            let needs = self.needs();
            let query = self.picker.query.clone();
            popover(320., false)
                .child(
                    div()
                        .px(px(8.))
                        .py(px(6.))
                        .text_size(px(13.))
                        .text_color(if query.is_empty() {
                            subtle
                        } else {
                            c(t, ColorRole::FgDefault)
                        })
                        .border_b_1()
                        .border_color(c(t, ColorRole::BorderSubtle))
                        .mb(px(4.))
                        .child(if query.is_empty() {
                            "Search models".to_string()
                        } else {
                            query
                        }),
                )
                .children(rows.into_iter().map(|row| {
                    let is_hl = hl.as_ref() == Some(&row);
                    match row {
                        PickerRow::Header(h) => div()
                            .px(px(8.))
                            .pt(px(8.))
                            .pb(px(4.))
                            .text_size(px(11.))
                            .text_color(subtle)
                            .child(h.to_uppercase()),
                        PickerRow::NoResults(m) => div()
                            .p(px(8.))
                            .text_size(px(13.))
                            .text_color(muted)
                            .child(m),
                        PickerRow::Auto => div()
                            .px(px(8.))
                            .py(px(6.))
                            .rounded(px(8.))
                            .text_size(px(13.5))
                            .flex()
                            .gap(px(8.))
                            .when(is_hl, |d| d.bg(c(t, ColorRole::SurfaceSelected)))
                            .child("Auto")
                            .child(
                                div()
                                    .text_color(muted)
                                    .text_size(px(11.))
                                    .child("Picks per turn"),
                            )
                            .on_mouse_down(
                                MouseButton::Left,
                                cx.listener(|this, _, _, cx| {
                                    this.choose(ModelSelection::Auto { resolved: None });
                                    cx.notify();
                                }),
                            ),
                        PickerRow::Model { id, matched } => {
                            let Some(m) = self.catalog.get(&id) else {
                                return div();
                            };
                            let note = status_note(m, &needs, now);
                            let name = m.name.clone();
                            let highlights: Vec<_> = name
                                .char_indices()
                                .filter(|(i, _)| matched.contains(&name[..*i].chars().count()))
                                .map(|(i, ch)| {
                                    (
                                        i..i + ch.len_utf8(),
                                        gpui::HighlightStyle {
                                            color: Some(c(t, ColorRole::AccentFg)),
                                            font_weight: Some(FontWeight::SEMIBOLD),
                                            ..Default::default()
                                        },
                                    )
                                })
                                .collect();
                            let sel = self.selection.model_id() == Some(id.as_str());
                            let choice = ModelSelection::Model(id.clone());
                            div()
                                .px(px(8.))
                                .py(px(6.))
                                .rounded(px(8.))
                                .text_size(px(13.5))
                                .flex()
                                .items_center()
                                .gap(px(8.))
                                .when(is_hl, |d| d.bg(c(t, ColorRole::SurfaceSelected)))
                                .child(
                                    div().flex_1().child(
                                        gpui::StyledText::new(name).with_highlights(highlights),
                                    ),
                                )
                                .child(
                                    div()
                                        .text_size(px(11.5))
                                        .text_color(muted)
                                        .child(format_context(m.context_tokens)),
                                )
                                .when_some(note, |d, n| {
                                    d.child(
                                        div()
                                            .text_size(px(11.))
                                            .text_color(c(t, ColorRole::StatusWarningFg))
                                            .child(n),
                                    )
                                })
                                .when(sel, |d| {
                                    d.child(div().text_color(c(t, ColorRole::AccentFg)).child("✓"))
                                })
                                .on_mouse_down(
                                    MouseButton::Left,
                                    cx.listener(move |this, _, _, cx| {
                                        this.choose(choice.clone());
                                        cx.notify();
                                    }),
                                )
                        }
                    }
                }))
        });
        let effort_pop = (self.pop == Some(Pop::Effort)).then(|| {
            popover(260., false).children(levels.iter().rev().map(|l| {
                let l = *l;
                let on = l == level;
                div()
                    .px(px(8.))
                    .py(px(7.))
                    .rounded(px(8.))
                    .flex()
                    .gap(px(8.))
                    .text_size(px(13.5))
                    .when(on, |d| d.bg(c(t, ColorRole::SurfaceSelected)))
                    .child(l.label())
                    .child(div().text_size(px(12.)).text_color(subtle).child(l.hint()))
                    .on_mouse_down(
                        MouseButton::Left,
                        cx.listener(move |this, _, _, cx| {
                            let s = this.support();
                            let agent = this.agent.clone();
                            let now = this.now;
                            this.effort.commit(EffortSetting::Level(l), &agent, &s, now);
                            this.pop = None;
                            cx.notify();
                        }),
                    )
            }))
        });
        let ctx_pop = (self.pop == Some(Pop::Context))
            .then(|| ctx.clone())
            .flatten()
            .map(|(v, u)| {
                popover(300., true)
                    .p(px(14.))
                    .gap(px(5.))
                    .child(
                        div()
                            .text_size(px(13.))
                            .font_weight(FontWeight::SEMIBOLD)
                            .mb(px(6.))
                            .child(format!(
                                "{} tokens · {}",
                                v.label(LabelMode::Tokens),
                                v.label(LabelMode::Percent)
                            )),
                    )
                    .child(
                        div()
                            .flex()
                            .h(px(8.))
                            .rounded(px(4.))
                            .overflow_hidden()
                            .mb(px(8.))
                            .children(u.rows().into_iter().map(|(cat, _, p)| {
                                div().h_full().w(px(272. * p)).bg(c(t, cat.color()))
                            })),
                    )
                    .children(u.rows().into_iter().map(|(cat, n, p)| {
                        div()
                            .flex()
                            .items_center()
                            .gap(px(8.))
                            .text_size(px(12.5))
                            .child(div().size(px(8.)).rounded(px(2.)).bg(c(t, cat.color())))
                            .child(div().flex_1().child(cat.label()))
                            .child(
                                div()
                                    .text_color(muted)
                                    .child(oxyn_core::composer::context::k(n)),
                            )
                            .child(
                                div()
                                    .w(px(44.))
                                    .text_color(muted)
                                    .child(format!("{:.1}%", p * 100.0)),
                            )
                    }))
                    .when_some(v.prompt.clone(), |d, pr| {
                        d.child(
                            div()
                                .mt(px(8.))
                                .text_size(px(12.))
                                .text_color(c(t, ColorRole::StatusWarningFg))
                                .child(pr),
                        )
                    })
                    .child(
                        div()
                            .mt(px(8.))
                            .h(px(28.))
                            .px(px(10.))
                            .rounded(px(8.))
                            .bg(c(t, ColorRole::FgDefault))
                            .text_color(c(t, ColorRole::BgBase))
                            .text_size(px(12.5))
                            .flex()
                            .items_center()
                            .child("Compact now")
                            .on_mouse_down(
                                MouseButton::Left,
                                cx.listener(|this, _, _, cx| {
                                    this.pop = None;
                                    cx.emit(ComposerEvent::Compact);
                                    cx.notify();
                                }),
                            ),
                    )
            });

        // Ignition wash.
        let wash = match ev.fx {
            Some(EffortFx::Igniting { tier, t: k }) => Some((tier, k)),
            _ => None,
        };
        let border = if focused {
            c(t, ColorRole::AccentBorder)
        } else {
            c(t, ColorRole::BorderDefault)
        };

        div()
            .track_focus(&self.focus)
            .key_context("OxynComposer")
            .on_key_down(cx.listener(|this, ev: &KeyDownEvent, _, cx| this.key(ev, cx)))
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(|this, _, window, cx| {
                    window.focus(&this.focus, cx);
                }),
            )
            .relative()
            .w_full()
            .max_w(px(760.))
            .font_family("Inter")
            .text_color(c(t, ColorRole::FgDefault))
            .flex()
            .flex_col()
            .p(px(10.))
            .pl(px(14.))
            .gap(px(8.))
            .rounded(px(16.))
            .bg(c(t, ColorRole::SurfaceRaised))
            .border_1()
            .border_color(border)
            .when_some(wash, |d, (tier, k)| {
                let col = if tier >= 2 {
                    ColorRole::AgentViolet
                } else {
                    ColorRole::StatusWarningFg
                };
                d.bg(hsla(mix(
                    t.color(ColorRole::SurfaceRaised),
                    t.color(col),
                    0.18 * (1.0 - k),
                )))
            })
            .when(!self.attachments.is_empty(), |d| {
                d.child(div().flex().flex_wrap().gap(px(8.)).children(tiles))
            })
            .child(text_row)
            .when_some(hint, |d, h| {
                d.child(div().text_size(px(12.)).text_color(muted).child(h))
            })
            .child(
                div()
                    .relative()
                    .flex()
                    .items_center()
                    .gap(px(4.))
                    .ml(px(-6.))
                    .min_h(px(32.))
                    .child(
                        div()
                            .size(px(30.))
                            .rounded_full()
                            .border_1()
                            .border_color(c(t, ColorRole::BorderSubtle))
                            .flex()
                            .items_center()
                            .justify_center()
                            .text_color(muted)
                            .cursor_pointer()
                            .child("+")
                            .on_mouse_down(
                                MouseButton::Left,
                                cx.listener(|_, _, _, cx| cx.emit(ComposerEvent::Attach)),
                            ),
                    )
                    .child(
                        div()
                            .relative()
                            .flex()
                            .items_center()
                            .gap(px(4.))
                            .child(model_pill)
                            .children(pending)
                            .children(model_pop),
                    )
                    .child(div().relative().children(effort_chip).children(effort_pop))
                    .when_some(ev.note.clone(), |d, n| {
                        d.child(
                            div()
                                .text_size(px(11.5))
                                .text_color(c(t, ColorRole::StatusWarningFg))
                                .child(n),
                        )
                    })
                    .child(div().flex_1())
                    .child(div().relative().children(ctx_chip).children(ctx_pop))
                    .child(send),
            )
    }
}
