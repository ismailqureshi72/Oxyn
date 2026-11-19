//! `ImeInput`: a real single-line text field for GPUI with IME composition, selection, clipboard
//! and the platform input handler, so composers, plan titles, type-to-confirm phrases and search
//! fields can be edited natively (closes the "GPUI text entry / IME" gap).
//!
//! The element and input-handler plumbing follow the `input.rs` example that ships with GPUI
//! (zed-industries/zed, Apache-2.0); the editing model is `oxyn_core::parity::edit::TextBuffer`,
//! so the same buffer is tested without a window.

use std::ops::Range;

use gpui::{
    App, Bounds, ClipboardItem, Context, CursorStyle, Element, ElementId, ElementInputHandler,
    Entity, EntityInputHandler, FocusHandle, Focusable, GlobalElementId, KeyBinding, LayoutId,
    MouseButton, MouseDownEvent, MouseMoveEvent, MouseUpEvent, PaintQuad, Pixels, Point, Render,
    ShapedLine, SharedString, Style, TextRun, UTF16Selection, UnderlineStyle, Window, actions, div,
    fill, point, prelude::*, px, relative, size,
};
use oxyn_core::parity::edit::TextBuffer;
use oxyn_tokens::{ColorRole, Theme};

use super::{alpha, c};

actions!(
    oxyn_ime_input,
    [
        Backspace,
        Delete,
        Left,
        Right,
        WordLeft,
        WordRight,
        SelectLeft,
        SelectRight,
        SelectAll,
        Home,
        End,
        Paste,
        Cut,
        Copy,
        Submit,
        Newline,
        AcceptGhost,
        DismissGhost,
    ]
);

/// Key bindings for [`ImeInput`] (call once at app start).
pub fn bind_ime_keys(cx: &mut App) {
    let ctx = Some("OxynImeInput");
    cx.bind_keys([
        KeyBinding::new("backspace", Backspace, ctx),
        KeyBinding::new("delete", Delete, ctx),
        KeyBinding::new("left", Left, ctx),
        KeyBinding::new("right", Right, ctx),
        KeyBinding::new("alt-left", WordLeft, ctx),
        KeyBinding::new("alt-right", WordRight, ctx),
        KeyBinding::new("shift-left", SelectLeft, ctx),
        KeyBinding::new("shift-right", SelectRight, ctx),
        KeyBinding::new("cmd-a", SelectAll, ctx),
        KeyBinding::new("ctrl-a", SelectAll, ctx),
        KeyBinding::new("home", Home, ctx),
        KeyBinding::new("end", End, ctx),
        KeyBinding::new("cmd-v", Paste, ctx),
        KeyBinding::new("ctrl-v", Paste, ctx),
        KeyBinding::new("cmd-c", Copy, ctx),
        KeyBinding::new("ctrl-c", Copy, ctx),
        KeyBinding::new("cmd-x", Cut, ctx),
        KeyBinding::new("ctrl-x", Cut, ctx),
        KeyBinding::new("enter", Submit, ctx),
        KeyBinding::new("shift-enter", Newline, ctx),
        KeyBinding::new("tab", AcceptGhost, ctx),
        KeyBinding::new("escape", DismissGhost, ctx),
        KeyBinding::new("cmd-right", WordRight, ctx),
        KeyBinding::new("ctrl-right", WordRight, ctx),
    ]);
}

type SubmitFn = Box<dyn Fn(&str, &mut Window, &mut App)>;

/// What an [`ImeInput`] tells its host.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ImeEvent {
    /// The text changed (typing, IME commit, paste, cut, delete).
    Changed,
    /// Enter outside a composition; `modk` is Cmd/Ctrl held.
    Enter {
        /// Cmd or Ctrl was held.
        modk: bool,
    },
    /// Ghost text taken: all of it (Tab) or one word (Alt/Cmd+Right).
    Accepted {
        /// The text appended.
        text: String,
        /// Tab took the whole suggestion.
        all: bool,
    },
    /// The ghost was dismissed with Escape.
    Dismissed(String),
}

impl gpui::EventEmitter<ImeEvent> for ImeInput {}

/// A focusable single-line input with IME support. Read [`ImeInput::text`]; listen for Enter
/// with [`ImeInput::on_submit`].
pub struct ImeInput {
    focus: FocusHandle,
    buf: TextBuffer,
    placeholder: SharedString,
    layout: Option<ShapedLine>,
    bounds: Option<Bounds<Pixels>>,
    selecting: bool,
    submit: Option<SubmitFn>,
    multiline: bool,
    bare: bool,
    ghost: Option<SharedString>,
    theme: &'static Theme,
}

impl ImeInput {
    /// A new input (create with `cx.new(|cx| ImeInput::new(cx, "Ask anything"))`).
    pub fn new(cx: &mut Context<Self>, placeholder: impl Into<SharedString>) -> Self {
        Self {
            focus: cx.focus_handle(),
            buf: TextBuffer::default(),
            placeholder: placeholder.into(),
            layout: None,
            bounds: None,
            selecting: false,
            submit: None,
            multiline: false,
            bare: false,
            ghost: None,
            theme: &Theme::DARK,
        }
    }
    /// Current text.
    pub fn text(&self) -> &str {
        &self.buf.text
    }
    /// Replaces the text (caret at the end).
    pub fn set_text(&mut self, t: &str, cx: &mut Context<Self>) {
        self.buf = TextBuffer::new(t);
        cx.notify();
    }
    /// Called with the text on Enter.
    pub fn on_submit(&mut self, f: impl Fn(&str, &mut Window, &mut App) + 'static) {
        self.submit = Some(Box::new(f));
    }
    /// Shift+Enter inserts a line break and pasted line breaks are kept (the one-line layout
    /// draws a break as a space until wrapped layout lands).
    pub fn set_multiline(&mut self, on: bool) {
        self.multiline = on;
    }
    /// No border, fill or fixed height: the host draws the field (composer shells).
    pub fn set_bare(&mut self, on: bool) {
        self.bare = on;
    }
    /// Ghost text drawn muted after the text while the caret sits at the end (the host owns the
    /// `oxyn_core::parity::input::Ghost` model and sets what it shows).
    pub fn set_ghost(&mut self, g: Option<String>, cx: &mut Context<Self>) {
        self.ghost = g.filter(|g| !g.is_empty()).map(Into::into);
        cx.notify();
    }
    fn ghost_showing(&self) -> bool {
        self.ghost.is_some()
            && self.buf.marked.is_none()
            && self.buf.selected.is_empty()
            && self.buf.cursor() == self.buf.text.len()
    }
    fn take_ghost(&mut self, all: bool, w: &mut Window, cx: &mut Context<Self>) {
        let Some(g) = self.ghost.take() else { return };
        let n = if all {
            g.len()
        } else {
            oxyn_core::parity::input::next_word_end(&g)
        };
        let (took, rest) = g.split_at(n);
        let took = took.to_string();
        self.ghost = (!rest.trim().is_empty()).then(|| rest.to_string().into());
        self.buf.replace(None, &took);
        self.changed(w, cx);
        cx.emit(ImeEvent::Accepted { text: took, all });
    }
    fn accept_ghost(&mut self, _: &AcceptGhost, w: &mut Window, cx: &mut Context<Self>) {
        if self.ghost_showing() {
            self.take_ghost(true, w, cx);
        } else {
            cx.propagate();
        }
    }
    fn dismiss_ghost(&mut self, _: &DismissGhost, _: &mut Window, cx: &mut Context<Self>) {
        match self
            .ghost
            .take()
            .filter(|_| self.buf.cursor() == self.buf.text.len())
        {
            Some(g) => {
                cx.emit(ImeEvent::Dismissed(g.to_string()));
                cx.notify();
            }
            None => cx.propagate(),
        }
    }
    /// Light or dark text and caret.
    pub fn set_theme(&mut self, t: &'static Theme, cx: &mut Context<Self>) {
        self.theme = t;
        cx.notify();
    }

    /// Placeholder text.
    pub fn set_placeholder(&mut self, p: impl Into<SharedString>) {
        self.placeholder = p.into();
    }
    /// Focus handle, for hosts that move focus in and out.
    pub fn focus(&self) -> &FocusHandle {
        &self.focus
    }
    /// Is an IME composition in progress.
    pub fn composing(&self) -> bool {
        self.buf.marked.is_some()
    }

    fn changed(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        window.notify_text_input_changed(&self.focus);
        cx.emit(ImeEvent::Changed);
        cx.notify();
    }
    fn moved(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.buf.cursor() != self.buf.text.len() {
            self.ghost = None;
        }
        window.notify_text_selection_changed(&self.focus);
        cx.notify();
    }
    fn backspace(&mut self, _: &Backspace, w: &mut Window, cx: &mut Context<Self>) {
        if self.buf.backspace() {
            self.changed(w, cx);
        }
    }
    fn delete(&mut self, _: &Delete, w: &mut Window, cx: &mut Context<Self>) {
        if self.buf.delete() {
            self.changed(w, cx);
        }
    }
    fn left(&mut self, _: &Left, w: &mut Window, cx: &mut Context<Self>) {
        self.buf.left();
        self.moved(w, cx);
    }
    fn right(&mut self, _: &Right, w: &mut Window, cx: &mut Context<Self>) {
        self.buf.right();
        self.moved(w, cx);
    }
    fn word_left(&mut self, _: &WordLeft, w: &mut Window, cx: &mut Context<Self>) {
        let o = self.buf.prev_word(self.buf.cursor());
        self.buf.move_to(o);
        self.moved(w, cx);
    }
    fn word_right(&mut self, _: &WordRight, w: &mut Window, cx: &mut Context<Self>) {
        if self.ghost_showing() {
            self.take_ghost(false, w, cx);
            return;
        }
        let o = self.buf.next_word(self.buf.cursor());
        self.buf.move_to(o);
        self.moved(w, cx);
    }
    fn select_left(&mut self, _: &SelectLeft, w: &mut Window, cx: &mut Context<Self>) {
        let o = self.buf.prev_boundary(self.buf.cursor());
        self.buf.select_to(o);
        self.moved(w, cx);
    }
    fn select_right(&mut self, _: &SelectRight, w: &mut Window, cx: &mut Context<Self>) {
        let o = self.buf.next_boundary(self.buf.cursor());
        self.buf.select_to(o);
        self.moved(w, cx);
    }
    fn select_all(&mut self, _: &SelectAll, w: &mut Window, cx: &mut Context<Self>) {
        self.buf.select_all();
        self.moved(w, cx);
    }
    fn home(&mut self, _: &Home, w: &mut Window, cx: &mut Context<Self>) {
        self.buf.move_to(0);
        self.moved(w, cx);
    }
    fn end(&mut self, _: &End, w: &mut Window, cx: &mut Context<Self>) {
        self.buf.move_to(self.buf.text.len());
        self.moved(w, cx);
    }
    fn paste(&mut self, _: &Paste, w: &mut Window, cx: &mut Context<Self>) {
        if let Some(t) = cx.read_from_clipboard().and_then(|i| i.text()) {
            let t = if self.multiline {
                t
            } else {
                t.replace('\n', " ")
            };
            self.buf.replace(None, &t);
            self.changed(w, cx);
        }
    }
    fn copy(&mut self, _: &Copy, _: &mut Window, cx: &mut Context<Self>) {
        if !self.buf.selected.is_empty() {
            cx.write_to_clipboard(ClipboardItem::new_string(self.buf.selection_text().into()));
        }
    }
    fn cut(&mut self, _: &Cut, w: &mut Window, cx: &mut Context<Self>) {
        if !self.buf.selected.is_empty() {
            cx.write_to_clipboard(ClipboardItem::new_string(self.buf.selection_text().into()));
            self.buf.replace(None, "");
            self.changed(w, cx);
        }
    }
    fn submit(&mut self, _: &Submit, w: &mut Window, cx: &mut Context<Self>) {
        // Enter during composition commits the IME text instead of submitting.
        if self.composing() {
            return;
        }
        let m = w.modifiers();
        cx.emit(ImeEvent::Enter {
            modk: m.platform || m.control,
        });
        if let Some(f) = &self.submit {
            f(&self.buf.text, w, cx);
        }
    }
    fn newline(&mut self, _: &Newline, w: &mut Window, cx: &mut Context<Self>) {
        if self.multiline && !self.composing() {
            self.buf.replace(None, "\n");
            self.changed(w, cx);
        }
    }
    fn index_at(&self, p: Point<Pixels>) -> usize {
        let (Some(b), Some(l)) = (self.bounds.as_ref(), self.layout.as_ref()) else {
            return 0;
        };
        if self.buf.text.is_empty() || p.y < b.top() {
            return 0;
        }
        if p.y > b.bottom() {
            return self.buf.text.len();
        }
        l.closest_index_for_x(p.x - b.left())
            .min(self.buf.text.len())
    }
    fn mouse_down(&mut self, e: &MouseDownEvent, w: &mut Window, cx: &mut Context<Self>) {
        self.selecting = true;
        let i = self.index_at(e.position);
        if e.modifiers.shift {
            self.buf.select_to(i);
        } else {
            self.buf.move_to(i);
        }
        self.moved(w, cx);
    }
    fn mouse_up(&mut self, _: &MouseUpEvent, _: &mut Window, _: &mut Context<Self>) {
        self.selecting = false;
    }
    fn mouse_move(&mut self, e: &MouseMoveEvent, w: &mut Window, cx: &mut Context<Self>) {
        if self.selecting {
            let i = self.index_at(e.position);
            self.buf.select_to(i);
            self.moved(w, cx);
        }
    }
}

impl EntityInputHandler for ImeInput {
    fn text_for_range(
        &mut self,
        r16: Range<usize>,
        actual: &mut Option<Range<usize>>,
        _: &mut Window,
        _: &mut Context<Self>,
    ) -> Option<String> {
        let r = self.buf.range_from_utf16(&r16);
        actual.replace(self.buf.range_to_utf16(&r));
        Some(self.buf.text[r].to_string())
    }
    fn selected_text_range(
        &mut self,
        _: bool,
        _: &mut Window,
        _: &mut Context<Self>,
    ) -> Option<UTF16Selection> {
        Some(UTF16Selection {
            range: self.buf.range_to_utf16(&self.buf.selected),
            reversed: self.buf.reversed,
        })
    }
    fn set_selected_text_range(
        &mut self,
        r16: Range<usize>,
        w: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.buf.selected = self.buf.range_from_utf16(&r16);
        self.buf.reversed = false;
        self.moved(w, cx);
    }
    fn text_length_utf16(&mut self, _: &mut Window, _: &mut Context<Self>) -> Option<usize> {
        Some(self.buf.len_utf16())
    }
    fn marked_text_range(&self, _: &mut Window, _: &mut Context<Self>) -> Option<Range<usize>> {
        self.buf.marked.as_ref().map(|r| self.buf.range_to_utf16(r))
    }
    fn unmark_text(&mut self, w: &mut Window, cx: &mut Context<Self>) {
        self.buf.unmark();
        self.changed(w, cx);
    }
    fn replace_text_in_range(
        &mut self,
        r16: Option<Range<usize>>,
        new: &str,
        w: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let r = r16.map(|r| self.buf.range_from_utf16(&r));
        self.buf.replace(r, new);
        self.changed(w, cx);
    }
    fn replace_and_mark_text_in_range(
        &mut self,
        r16: Option<Range<usize>>,
        new: &str,
        sel16: Option<Range<usize>>,
        w: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let r = r16.map(|r| self.buf.range_from_utf16(&r));
        self.buf.replace_and_mark(r, new, sel16);
        self.changed(w, cx);
    }
    fn bounds_for_range(
        &mut self,
        r16: Range<usize>,
        bounds: Bounds<Pixels>,
        _: &mut Window,
        _: &mut Context<Self>,
    ) -> Option<Bounds<Pixels>> {
        let l = self.layout.as_ref()?;
        if l.text != self.buf.text {
            return None;
        }
        let r = self.buf.range_from_utf16(&r16);
        Some(Bounds::from_corners(
            point(bounds.left() + l.x_for_index(r.start), bounds.top()),
            point(bounds.left() + l.x_for_index(r.end), bounds.bottom()),
        ))
    }
    fn selection_bounds_for_range(
        &mut self,
        r16: Range<usize>,
        bounds: Bounds<Pixels>,
        w: &mut Window,
        cx: &mut Context<Self>,
    ) -> Vec<Bounds<Pixels>> {
        self.bounds_for_range(r16, bounds, w, cx)
            .into_iter()
            .collect()
    }
    fn character_index_for_point(
        &mut self,
        p: Point<Pixels>,
        _: &mut Window,
        _: &mut Context<Self>,
    ) -> Option<usize> {
        let lp = self.bounds?.localize(&p)?;
        let l = self.layout.as_ref()?;
        if l.text != self.buf.text {
            return None;
        }
        Some(self.buf.to_utf16(l.closest_index_for_x(lp.x)))
    }
}

struct LineElement {
    input: Entity<ImeInput>,
}

struct Prepaint {
    line: Option<ShapedLine>,
    cursor: Option<PaintQuad>,
    selection: Option<PaintQuad>,
}

impl Element for LineElement {
    type RequestLayoutState = ();
    type PrepaintState = Prepaint;

    fn id(&self) -> Option<ElementId> {
        None
    }
    fn source_location(&self) -> Option<&'static core::panic::Location<'static>> {
        None
    }
    fn request_layout(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&gpui::InspectorElementId>,
        window: &mut Window,
        cx: &mut App,
    ) -> (LayoutId, ()) {
        let mut style = Style::default();
        style.size.width = relative(1.).into();
        style.size.height = window.line_height().into();
        (window.request_layout(style, [], cx), ())
    }
    fn prepaint(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&gpui::InspectorElementId>,
        bounds: Bounds<Pixels>,
        _: &mut (),
        window: &mut Window,
        cx: &mut App,
    ) -> Prepaint {
        let input = self.input.read(cx);
        let t = input.theme;
        let buf = &input.buf;
        let style = window.text_style();
        let ghost = input.ghost_showing().then(|| input.ghost.clone()).flatten();
        let empty = buf.text.is_empty() && ghost.is_none();
        let text_len = buf.text.len();
        let shown: SharedString = if empty {
            input.placeholder.clone()
        } else {
            // A line break is one byte, as is a space: offsets stay valid in the one-line layout.
            format!(
                "{}{}",
                buf.text.replace('\n', " "),
                ghost.as_deref().unwrap_or("")
            )
            .into()
        };
        let run = TextRun {
            len: shown.len(),
            font: style.font(),
            color: if empty {
                c(t, ColorRole::FgSubtle)
            } else {
                style.color
            },
            background_color: None,
            underline: None,
            strikethrough: None,
        };
        let mut runs: Vec<TextRun> = match buf.marked.as_ref().filter(|_| !empty) {
            Some(m) => [
                TextRun {
                    len: m.start,
                    ..run.clone()
                },
                TextRun {
                    len: m.end - m.start,
                    underline: Some(UnderlineStyle {
                        color: Some(run.color),
                        thickness: px(1.),
                        wavy: false,
                    }),
                    ..run.clone()
                },
                TextRun {
                    len: text_len - m.end,
                    ..run
                },
            ]
            .into_iter()
            .filter(|r| r.len > 0)
            .collect(),
            None if empty => vec![run],
            None => vec![TextRun {
                len: text_len,
                ..run
            }]
            .into_iter()
            .filter(|r| r.len > 0)
            .collect(),
        };
        if let Some(g) = ghost.as_ref().filter(|_| !empty) {
            runs.push(TextRun {
                len: g.len(),
                font: style.font(),
                color: c(t, ColorRole::FgSubtle),
                background_color: None,
                underline: None,
                strikethrough: None,
            });
        }
        let fs = style.font_size.to_pixels(window.rem_size());
        let line = window.text_system().shape_line(shown, fs, &runs, None);
        let (selection, cursor) = if buf.selected.is_empty() {
            let x = if empty {
                px(0.)
            } else {
                line.x_for_index(buf.cursor())
            };
            (
                None,
                Some(fill(
                    Bounds::new(
                        point(bounds.left() + x, bounds.top()),
                        size(px(1.5), bounds.bottom() - bounds.top()),
                    ),
                    c(t, ColorRole::FgDefault),
                )),
            )
        } else {
            (
                Some(fill(
                    Bounds::from_corners(
                        point(
                            bounds.left() + line.x_for_index(buf.selected.start),
                            bounds.top(),
                        ),
                        point(
                            bounds.left() + line.x_for_index(buf.selected.end),
                            bounds.bottom(),
                        ),
                    ),
                    alpha(c(t, ColorRole::Selection), 1.0),
                )),
                None,
            )
        };
        Prepaint {
            line: Some(line),
            cursor,
            selection,
        }
    }
    fn paint(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&gpui::InspectorElementId>,
        bounds: Bounds<Pixels>,
        _: &mut (),
        pre: &mut Prepaint,
        window: &mut Window,
        cx: &mut App,
    ) {
        let focus = self.input.read(cx).focus.clone();
        window.handle_input(
            &focus,
            ElementInputHandler::new(bounds, self.input.clone()),
            cx,
        );
        if let Some(s) = pre.selection.take() {
            window.paint_quad(s);
        }
        let Some(line) = pre.line.take() else { return };
        let _ = line.paint(
            bounds.origin,
            window.line_height(),
            gpui::TextAlign::Left,
            None,
            window,
            cx,
        );
        if focus.is_focused(window)
            && let Some(cur) = pre.cursor.take()
        {
            window.paint_quad(cur);
        }
        self.input.update(cx, |i, _| {
            i.layout = Some(line);
            i.bounds = Some(bounds);
        });
    }
}

impl Render for ImeInput {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let t = self.theme;
        let focused = self.focus.is_focused(window);
        div()
            .key_context("OxynImeInput")
            .track_focus(&self.focus)
            .cursor(CursorStyle::IBeam)
            .on_action(cx.listener(Self::backspace))
            .on_action(cx.listener(Self::delete))
            .on_action(cx.listener(Self::left))
            .on_action(cx.listener(Self::right))
            .on_action(cx.listener(Self::word_left))
            .on_action(cx.listener(Self::word_right))
            .on_action(cx.listener(Self::select_left))
            .on_action(cx.listener(Self::select_right))
            .on_action(cx.listener(Self::select_all))
            .on_action(cx.listener(Self::home))
            .on_action(cx.listener(Self::end))
            .on_action(cx.listener(Self::paste))
            .on_action(cx.listener(Self::cut))
            .on_action(cx.listener(Self::copy))
            .on_action(cx.listener(Self::submit))
            .on_action(cx.listener(Self::newline))
            .on_action(cx.listener(Self::accept_ghost))
            .on_action(cx.listener(Self::dismiss_ghost))
            .on_mouse_down(MouseButton::Left, cx.listener(Self::mouse_down))
            .on_mouse_up(MouseButton::Left, cx.listener(Self::mouse_up))
            .on_mouse_up_out(MouseButton::Left, cx.listener(Self::mouse_up))
            .on_mouse_move(cx.listener(Self::mouse_move))
            .w_full()
            .flex()
            .items_center()
            .text_color(c(t, ColorRole::FgDefault))
            .when(!self.bare, |d| {
                d.h(px(32.))
                    .px(px(10.))
                    .rounded(px(8.))
                    .border_1()
                    .border_color(if focused {
                        c(t, ColorRole::Focus)
                    } else {
                        c(t, ColorRole::BorderDefault)
                    })
                    .bg(c(t, ColorRole::SurfaceBase))
                    .text_size(px(13.))
            })
            .child(LineElement { input: cx.entity() })
    }
}

impl Focusable for ImeInput {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus.clone()
    }
}
