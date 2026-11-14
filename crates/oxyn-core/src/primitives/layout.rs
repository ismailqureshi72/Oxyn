//! Navigation and layout primitives (plan #27): tabs, breadcrumb, accordion, resizable panels
//! (with the sidebar snap-to-rail rule), hover-card intent, tree view, reorderable list, toggle
//! group, chip group, radio cards, rating, copy state, expandable text, dock, table-of-contents
//! progress, page-header fold and empty-state presets.
//!
//! Behaviour follows the Arc UI docs (rebuilt from the public descriptions; no source used).
//! Everything here is framework-free: renderers hold these models, map their events onto the
//! methods and draw from the getters.

use std::collections::BTreeSet;
use std::time::Duration;

use super::display::{EmptyLayout, EmptyState};
use super::nav::{NavKey, step};
use crate::ClockMs;

// ───────────────────────────────────────────── tabs ──

/// How a focused tab is activated.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Activation {
    /// Arrow keys move focus and select (Arc default).
    #[default]
    Auto,
    /// Arrow keys only move focus; Enter / Space selects.
    Manual,
}

/// One tab.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Tab {
    /// Value (matches its panel).
    pub value: String,
    /// Label.
    pub label: String,
    /// Oxyn plus: a count that rolls (Page header tabs).
    pub count: Option<u32>,
    /// Oxyn plus: icon name.
    pub icon: Option<String>,
    /// Disabled.
    pub disabled: bool,
    /// Oxyn plus: a close button (terminal / editor tabs).
    pub closable: bool,
    /// Oxyn plus: busy dot (a terminal still running).
    pub busy: bool,
}

impl Tab {
    /// A tab.
    pub fn new(value: &str, label: &str) -> Self {
        Self {
            value: value.into(),
            label: label.into(),
            count: None,
            icon: None,
            disabled: false,
            closable: false,
            busy: false,
        }
    }
    /// With a count.
    pub fn count(mut self, n: u32) -> Self {
        self.count = Some(n);
        self
    }
    /// With an icon.
    pub fn icon(mut self, i: &str) -> Self {
        self.icon = Some(i.into());
        self
    }
    /// Closable.
    pub fn closable(mut self) -> Self {
        self.closable = true;
        self
    }
    /// Busy.
    pub fn busy(mut self, b: bool) -> Self {
        self.busy = b;
        self
    }
    /// Disabled.
    pub fn disabled(mut self) -> Self {
        self.disabled = true;
        self
    }
}

/// Which way the selection moved (panels slide this way).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Travel {
    /// No movement (first render).
    #[default]
    None,
    /// To a later tab.
    Forward,
    /// To an earlier tab.
    Back,
}

impl Travel {
    /// CSS id.
    pub fn id(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Forward => "fwd",
            Self::Back => "back",
        }
    }
}

/// Tabs state: selected value, focus, travel direction and closing.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Tabs {
    /// Tabs in order.
    pub tabs: Vec<Tab>,
    /// Selected value.
    pub value: String,
    /// Focused index (roving tab stop).
    pub focus: Option<usize>,
    /// Direction of the last change.
    pub travel: Travel,
    /// Activation mode.
    pub activation: Activation,
    /// Bumps on every change (panels re-key on it).
    pub version: u32,
}

impl Tabs {
    /// Tabs selecting `value` (or the first enabled tab).
    pub fn new(tabs: Vec<Tab>, value: Option<&str>) -> Self {
        let value = value
            .filter(|v| tabs.iter().any(|t| t.value == *v && !t.disabled))
            .map(str::to_string)
            .or_else(|| tabs.iter().find(|t| !t.disabled).map(|t| t.value.clone()))
            .unwrap_or_default();
        Self {
            tabs,
            value,
            focus: None,
            travel: Travel::None,
            activation: Activation::Auto,
            version: 0,
        }
    }
    /// Selected index.
    pub fn selected(&self) -> Option<usize> {
        self.tabs.iter().position(|t| t.value == self.value)
    }
    /// The tab stop: focus, else the selected tab.
    pub fn tab_stop(&self) -> Option<usize> {
        self.focus.or_else(|| self.selected())
    }
    /// Selects index `i`; returns the new value when it changed.
    pub fn select(&mut self, i: usize) -> Option<String> {
        let t = self.tabs.get(i)?;
        if t.disabled || t.value == self.value {
            self.focus = Some(i);
            return None;
        }
        let from = self.selected().unwrap_or(0);
        self.travel = if i >= from {
            Travel::Forward
        } else {
            Travel::Back
        };
        self.value = t.value.clone();
        self.focus = Some(i);
        self.version += 1;
        Some(self.value.clone())
    }
    /// Selects by value.
    pub fn select_value(&mut self, v: &str) -> Option<String> {
        let i = self.tabs.iter().position(|t| t.value == v)?;
        self.select(i)
    }
    /// Arrow / Home / End. Auto activation selects as focus moves.
    pub fn key(&mut self, k: NavKey) -> Option<String> {
        let tabs = &self.tabs;
        let to = step(self.tab_stop(), k, tabs.len(), true, |i| !tabs[i].disabled)?;
        self.focus = Some(to);
        if self.activation == Activation::Auto {
            self.select(to)
        } else {
            None
        }
    }
    /// Enter / Space on the focused tab (manual activation).
    pub fn activate(&mut self) -> Option<String> {
        let f = self.tab_stop()?;
        self.select(f)
    }
    /// Closes tab `i`. Closing the selected tab selects its right neighbour (else left), like
    /// editors and terminals. Returns the closed tab.
    pub fn close(&mut self, i: usize) -> Option<Tab> {
        if i >= self.tabs.len() {
            return None;
        }
        let was_sel = self.selected() == Some(i);
        let t = self.tabs.remove(i);
        if self.tabs.is_empty() {
            self.value.clear();
            self.focus = None;
        } else if was_sel {
            let n = i.min(self.tabs.len() - 1);
            self.value = self.tabs[n].value.clone();
            self.travel = Travel::None;
            self.focus = Some(n);
        } else if let Some(f) = self.focus {
            self.focus = Some(if f > i {
                f - 1
            } else {
                f.min(self.tabs.len() - 1)
            });
        }
        self.version += 1;
        Some(t)
    }
    /// Adds a tab at the end and selects it (terminal "+").
    pub fn push(&mut self, t: Tab) -> String {
        self.tabs.push(t);
        let i = self.tabs.len() - 1;
        self.select(i);
        self.value.clone()
    }
    /// Moves tab `from` to `to` (drag to reorder), keeping the selection.
    pub fn reorder(&mut self, from: usize, to: usize) {
        if from >= self.tabs.len() || to >= self.tabs.len() || from == to {
            return;
        }
        let t = self.tabs.remove(from);
        self.tabs.insert(to, t);
        self.focus = Some(to);
        self.version += 1;
    }
}

/// Overflow edge buttons for a scrolling strip: (show left, show right).
pub fn overflow_edges(scroll_left: f32, client: f32, scroll_width: f32) -> (bool, bool) {
    (scroll_left > 1.0, scroll_left + client < scroll_width - 1.0)
}

/// The scroll offset that brings `[x, x + w]` into a strip of width `client` with `pad` air.
pub fn scroll_into_view(scroll_left: f32, client: f32, x: f32, w: f32, pad: f32) -> f32 {
    if x - pad < scroll_left {
        (x - pad).max(0.0)
    } else if x + w + pad > scroll_left + client {
        x + w + pad - client
    } else {
        scroll_left
    }
}

// ─────────────────────────────────────── breadcrumb ──

/// One crumb.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Crumb {
    /// Label.
    pub label: String,
    /// Link (None renders a button or plain text).
    pub href: Option<String>,
    /// Oxyn plus: icon name (workspace / project marks).
    pub icon: Option<String>,
}

impl Crumb {
    /// A crumb.
    pub fn new(label: &str) -> Self {
        Self {
            label: label.into(),
            href: None,
            icon: None,
        }
    }
    /// With a link.
    pub fn href(mut self, h: &str) -> Self {
        self.href = Some(h.into());
        self
    }
    /// With an icon.
    pub fn icon(mut self, i: &str) -> Self {
        self.icon = Some(i.into());
        self
    }
}

/// A slot in a collapsed path.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CrumbSlot {
    /// A crumb by index.
    Item(usize),
    /// Oxyn plus: hidden middle crumbs behind a "…" menu.
    More(Vec<usize>),
}

/// Lays out `len` crumbs within `max` slots: the root, a "…" holding the middle, and the last
/// `max - 2` crumbs. `max == 0` never collapses (Arc's behaviour: wrap instead).
pub fn crumb_slots(len: usize, max: usize) -> Vec<CrumbSlot> {
    if max == 0 || len <= max || max < 3 {
        return (0..len).map(CrumbSlot::Item).collect();
    }
    let tail = max - 2;
    let mut v = vec![
        CrumbSlot::Item(0),
        CrumbSlot::More((1..len - tail).collect()),
    ];
    v.extend((len - tail..len).map(CrumbSlot::Item));
    v
}

// ──────────────────────────────────────── accordion ──

/// Accordion state.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Accordion {
    /// Number of rows.
    pub len: usize,
    /// Open rows.
    pub open: BTreeSet<usize>,
    /// Several rows may be open (Oxyn plus; Arc is single-open).
    pub multiple: bool,
    /// The open row can be closed (single mode).
    pub collapsible: bool,
    /// Focused trigger.
    pub focus: Option<usize>,
    /// Disabled rows.
    pub disabled: BTreeSet<usize>,
}

impl Accordion {
    /// `len` rows with `default_open` open (`None` = all closed).
    pub fn new(len: usize, default_open: Option<usize>) -> Self {
        Self {
            len,
            open: default_open.filter(|&i| i < len).into_iter().collect(),
            multiple: false,
            collapsible: true,
            focus: None,
            disabled: BTreeSet::new(),
        }
    }
    /// Is row `i` open.
    pub fn is_open(&self, i: usize) -> bool {
        self.open.contains(&i)
    }
    /// Toggles row `i`.
    pub fn toggle(&mut self, i: usize) {
        if i >= self.len || self.disabled.contains(&i) {
            return;
        }
        self.focus = Some(i);
        if self.open.contains(&i) {
            if self.multiple || self.collapsible {
                self.open.remove(&i);
            }
        } else {
            if !self.multiple {
                self.open.clear();
            }
            self.open.insert(i);
        }
    }
    /// ↑ ↓ Home End between triggers.
    pub fn key(&mut self, k: NavKey) -> Option<usize> {
        let dis = &self.disabled;
        let to = step(self.focus, k, self.len, true, |i| !dis.contains(&i))?;
        self.focus = Some(to);
        Some(to)
    }
    /// Expands every row (multiple mode).
    pub fn expand_all(&mut self) {
        if self.multiple {
            self.open = (0..self.len)
                .filter(|i| !self.disabled.contains(i))
                .collect();
        }
    }
    /// Collapses every row.
    pub fn collapse_all(&mut self) {
        self.open.clear();
    }
}

// ─────────────────────────────────── resizable panels ──

/// One pane of a resizable group.
#[derive(Clone, Debug, PartialEq)]
pub struct Pane {
    /// Stable id.
    pub id: String,
    /// Names the pane for its divider.
    pub label: String,
    /// Starting share (relative).
    pub default_size: f32,
    /// Width px where it starts to resist.
    pub min: f32,
    /// Max width px.
    pub max: Option<f32>,
    /// Can snap closed.
    pub collapsible: bool,
}

impl Pane {
    /// A pane.
    pub fn new(id: &str, label: &str, default_size: f32) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            default_size,
            min: 80.0,
            max: None,
            collapsible: false,
        }
    }
    /// Min px.
    pub fn min(mut self, px: f32) -> Self {
        self.min = px;
        self
    }
    /// Max px.
    pub fn max(mut self, px: f32) -> Self {
        self.max = Some(px);
        self
    }
    /// Collapsible.
    pub fn collapsible(mut self) -> Self {
        self.collapsible = true;
        self
    }
}

/// Keyboard step on a divider.
pub const DIVIDER_STEP: f32 = 16.0;
/// Shift + arrow step.
pub const DIVIDER_STEP_BIG: f32 = 64.0;
/// How far past a minimum a drag must go to collapse.
pub const COLLAPSE_OVERSHOOT: f32 = 48.0;

/// A horizontal (or vertical) split with draggable dividers. Widths are px; the container width
/// comes from the host.
#[derive(Clone, Debug, PartialEq)]
pub struct Resizable {
    /// Panes in order.
    pub panes: Vec<Pane>,
    /// Current px sizes.
    pub sizes: Vec<f32>,
    /// Collapsed flags.
    pub collapsed: Vec<bool>,
    /// Size before collapse (restore).
    pub restore: Vec<f32>,
    /// Container size px.
    pub total: f32,
    /// Divider being dragged and its start sizes (Escape restores).
    pub drag: Option<(usize, Vec<f32>)>,
    /// Vertical stack instead of a row.
    pub vertical: bool,
}

impl Resizable {
    /// A group of `panes` filling `total` px.
    pub fn new(panes: Vec<Pane>, total: f32) -> Self {
        let n = panes.len();
        let mut r = Self {
            sizes: vec![0.0; n],
            collapsed: vec![false; n],
            restore: vec![0.0; n],
            panes,
            total,
            drag: None,
            vertical: false,
        };
        r.reset();
        r
    }
    /// Back to the default shares (double-click a divider).
    pub fn reset(&mut self) {
        let sum: f32 = self.panes.iter().map(|p| p.default_size.max(0.0)).sum();
        let sum = if sum <= 0.0 { 1.0 } else { sum };
        for (i, p) in self.panes.iter().enumerate() {
            self.sizes[i] = self.total * p.default_size.max(0.0) / sum;
            self.collapsed[i] = false;
        }
    }
    /// New container size: scales the open panes.
    pub fn set_total(&mut self, total: f32) {
        if total <= 0.0 || (total - self.total).abs() < 0.5 {
            return;
        }
        let k = total / self.total.max(1.0);
        for s in &mut self.sizes {
            *s *= k;
        }
        self.total = total;
    }
    /// Shares in percent (what `on_layout_change` reports).
    pub fn shares(&self) -> Vec<f32> {
        self.sizes
            .iter()
            .map(|s| (s / self.total.max(1.0) * 1000.0).round() / 10.0)
            .collect()
    }
    fn max_of(&self, i: usize) -> f32 {
        self.panes[i].max.unwrap_or(f32::MAX)
    }
    /// Moves divider `d` (between pane `d` and `d + 1`) by `dx` px. Rubber-bands past limits;
    /// dragging well past a collapsible pane's minimum snaps it closed. Returns true on change.
    pub fn move_divider(&mut self, d: usize, dx: f32) -> bool {
        if d + 1 >= self.panes.len() || dx == 0.0 {
            return false;
        }
        let (a, b) = (d, d + 1);
        let pair = self.sizes[a] + self.sizes[b];
        let mut na = self.sizes[a] + dx;
        // Collapse when pushed well past the minimum.
        if self.panes[a].collapsible && na < self.panes[a].min - COLLAPSE_OVERSHOOT {
            return self.collapse(a);
        }
        if self.panes[b].collapsible && pair - na < self.panes[b].min - COLLAPSE_OVERSHOOT {
            return self.collapse(b);
        }
        // Reopening a collapsed pane by dragging out of it.
        if self.collapsed[a] && dx > 0.0 {
            self.collapsed[a] = false;
        }
        if self.collapsed[b] && dx < 0.0 {
            self.collapsed[b] = false;
        }
        let lo = if self.collapsed[a] {
            0.0
        } else {
            self.panes[a].min
        };
        let hi = (pair
            - if self.collapsed[b] {
                0.0
            } else {
                self.panes[b].min
            })
        .min(self.max_of(a));
        let lo2 = (pair - self.max_of(b)).max(lo);
        na = na.clamp(lo2, hi.max(lo2));
        let changed = (na - self.sizes[a]).abs() > 0.01;
        self.sizes[a] = na;
        self.sizes[b] = pair - na;
        changed
    }
    /// Collapses pane `i` (its space goes to the neighbour). Returns true on change.
    pub fn collapse(&mut self, i: usize) -> bool {
        if !self.panes.get(i).is_some_and(|p| p.collapsible) || self.collapsed[i] {
            return false;
        }
        let n = if i + 1 < self.panes.len() {
            i + 1
        } else if i > 0 {
            i - 1
        } else {
            return false;
        };
        self.restore[i] = self.sizes[i].max(self.panes[i].min);
        self.sizes[n] += self.sizes[i];
        self.sizes[i] = 0.0;
        self.collapsed[i] = true;
        true
    }
    /// Restores a collapsed pane to its last size.
    pub fn expand(&mut self, i: usize) -> bool {
        if !self.collapsed.get(i).copied().unwrap_or(false) {
            return false;
        }
        let n = if i + 1 < self.panes.len() {
            i + 1
        } else {
            i - 1
        };
        let want = self.restore[i]
            .min(self.sizes[n] - self.panes[n].min)
            .max(0.0);
        self.sizes[i] = want;
        self.sizes[n] -= want;
        self.collapsed[i] = false;
        true
    }
    /// Enter on divider `d`: collapses or restores the collapsible neighbour.
    pub fn toggle(&mut self, d: usize) -> bool {
        for i in [d, d + 1] {
            if self.collapsed.get(i).copied().unwrap_or(false) {
                return self.expand(i);
            }
        }
        for i in [d, d + 1] {
            if self.panes.get(i).is_some_and(|p| p.collapsible) {
                return self.collapse(i);
            }
        }
        false
    }
    /// Starts a drag on divider `d`.
    pub fn begin(&mut self, d: usize) {
        self.drag = Some((d, self.sizes.clone()));
    }
    /// Ends a drag (keeps the sizes).
    pub fn end(&mut self) -> bool {
        self.drag.take().is_some()
    }
    /// Escape during a drag: back to the start.
    pub fn cancel(&mut self) -> bool {
        if let Some((_, s)) = self.drag.take() {
            self.sizes = s;
            for (i, c) in self.collapsed.iter_mut().enumerate() {
                *c = self.sizes[i] == 0.0 && self.panes[i].collapsible;
            }
            return true;
        }
        false
    }
    /// Home / End: moves the divider to the left pane's min or max.
    pub fn to_limit(&mut self, d: usize, max: bool) -> bool {
        let dx = if max { self.total } else { -self.total };
        // Clamp without collapsing.
        let c = self.panes[d].collapsible;
        let c2 = self
            .panes
            .get(d + 1)
            .map(|p| p.collapsible)
            .unwrap_or(false);
        self.panes[d].collapsible = false;
        if let Some(p) = self.panes.get_mut(d + 1) {
            p.collapsible = false;
        }
        let ch = self.move_divider(d, dx);
        self.panes[d].collapsible = c;
        if let Some(p) = self.panes.get_mut(d + 1) {
            p.collapsible = c2;
        }
        ch
    }
    /// `aria-valuenow` for divider `d` (left pane share, percent).
    pub fn value_now(&self, d: usize) -> u32 {
        (self.sizes[d] / self.total.max(1.0) * 100.0).round() as u32
    }
    /// `aria-valuetext` for divider `d`.
    pub fn value_text(&self, d: usize) -> String {
        for i in [d, d + 1] {
            if self.collapsed.get(i).copied().unwrap_or(false) {
                return format!("{} hidden, press Enter to show", self.panes[i].label);
            }
        }
        format!("{} {}%", self.panes[d].label, self.value_now(d))
    }
    /// Pane opacity while squeezed below its minimum (content fades).
    pub fn squeeze(&self, i: usize) -> f32 {
        let m = self.panes[i].min.max(1.0);
        (self.sizes[i] / m).clamp(0.0, 1.0)
    }
}

/// Where a sidebar lands for a dragged width (the app shell's resizable sidebar that snaps to
/// icons).
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum SidebarSnap {
    /// Hidden (off-canvas).
    Hidden,
    /// Icon rail.
    Rail,
    /// Full sidebar at this width.
    Open(f32),
}

impl SidebarSnap {
    /// CSS id.
    pub fn id(self) -> &'static str {
        match self {
            Self::Hidden => "hidden",
            Self::Rail => "rail",
            Self::Open(_) => "open",
        }
    }
    /// Rendered width.
    pub fn width(self, rail: f32) -> f32 {
        match self {
            Self::Hidden => 0.0,
            Self::Rail => rail,
            Self::Open(w) => w,
        }
    }
}

/// Snap rule for a resizable sidebar: below `min * 0.6` it snaps to the icon rail, below a third
/// of the rail it hides (when `hideable`), otherwise it clamps to `[min, max]`.
pub fn sidebar_snap(width: f32, rail: f32, min: f32, max: f32, hideable: bool) -> SidebarSnap {
    if hideable && width < rail * 0.4 {
        SidebarSnap::Hidden
    } else if width < min * 0.6 {
        SidebarSnap::Rail
    } else {
        SidebarSnap::Open(width.clamp(min, max))
    }
}

// ─────────────────────────────────────── hover card ──

/// Hover-card intent: open delay, close grace, warm re-open, Escape suppression.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HoverIntent {
    /// First open delay.
    pub open_delay: Duration,
    /// Grace after the pointer leaves.
    pub close_delay: Duration,
    /// Pending open at.
    pub opening: Option<ClockMs>,
    /// Pending close at.
    pub closing: Option<ClockMs>,
    /// Open.
    pub open: bool,
    /// Last close (warm window).
    pub last_close: Option<ClockMs>,
    /// Escape pressed: stays closed while the pointer rests on the trigger.
    pub suppressed: bool,
    /// The last open was warm (fade only).
    pub warm: bool,
}

/// Window after a close in which the next card opens fast.
pub const WARM_WINDOW: Duration = Duration::from_millis(300);
/// Warm open delay.
pub const WARM_DELAY: Duration = Duration::from_millis(80);

impl Default for HoverIntent {
    fn default() -> Self {
        Self {
            open_delay: Duration::from_millis(500),
            close_delay: Duration::from_millis(140),
            opening: None,
            closing: None,
            open: false,
            last_close: None,
            suppressed: false,
            warm: false,
        }
    }
}

impl HoverIntent {
    /// With delays (ms).
    pub fn with_delays(open: u64, close: u64) -> Self {
        Self {
            open_delay: Duration::from_millis(open),
            close_delay: Duration::from_millis(close),
            ..Self::default()
        }
    }
    /// Pointer entered the trigger or the card.
    pub fn enter(&mut self, now: ClockMs) {
        self.closing = None;
        if self.open || self.suppressed {
            return;
        }
        let warm = self.last_close.is_some_and(|t| now.since(t) <= WARM_WINDOW);
        self.warm = warm;
        let d = if warm { WARM_DELAY } else { self.open_delay };
        self.opening.get_or_insert(now.plus(d));
    }
    /// Keyboard focus or Enter: opens at once.
    pub fn focus(&mut self) {
        self.open = true;
        self.opening = None;
        self.closing = None;
        self.suppressed = false;
    }
    /// Pointer left (the card stays while the pointer travels into it).
    pub fn leave(&mut self, now: ClockMs) {
        self.opening = None;
        self.suppressed = false;
        if self.open {
            self.closing = Some(now.plus(self.close_delay));
        }
    }
    /// Escape.
    pub fn escape(&mut self, now: ClockMs) -> bool {
        let was = self.open;
        self.open = false;
        self.opening = None;
        self.closing = None;
        self.suppressed = true;
        if was {
            self.last_close = Some(now);
        }
        was
    }
    /// Blur / tap away.
    pub fn close(&mut self, now: ClockMs) {
        if self.open {
            self.last_close = Some(now);
        }
        self.open = false;
        self.opening = None;
        self.closing = None;
    }
    /// Touch: a tap toggles.
    pub fn tap(&mut self, now: ClockMs) {
        if self.open {
            self.close(now);
        } else {
            self.focus();
        }
    }
    /// Advances timers; returns true when `open` changed.
    pub fn tick(&mut self, now: ClockMs) -> bool {
        if let Some(t) = self.opening
            && now >= t
        {
            self.opening = None;
            self.open = true;
            return true;
        }
        if let Some(t) = self.closing
            && now >= t
        {
            self.closing = None;
            self.open = false;
            self.last_close = Some(now);
            return true;
        }
        false
    }
    /// Next timer deadline.
    pub fn deadline(&self) -> Option<ClockMs> {
        self.opening.or(self.closing)
    }
}

/// Initials for a name ("Ada Lovelace" → "AL").
pub fn initials(name: &str) -> String {
    let mut it = name.split_whitespace().filter_map(|w| w.chars().next());
    let a = it.next();
    let b = name
        .split_whitespace()
        .nth(1)
        .and_then(|w| w.chars().next());
    a.into_iter()
        .chain(b)
        .flat_map(char::to_uppercase)
        .collect()
}

// ──────────────────────────────────────── tree view ──

/// A tree node.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TreeNode {
    /// Id.
    pub id: String,
    /// Label.
    pub label: String,
    /// Icon name.
    pub icon: Option<String>,
    /// Children (a folder when non-empty or `folder`).
    pub children: Vec<TreeNode>,
    /// Oxyn plus: a git / agent status letter (M, A, D, ●).
    pub badge: Option<String>,
    /// Force folder even when empty.
    pub folder: bool,
}

impl TreeNode {
    /// A leaf.
    pub fn leaf(id: &str, label: &str) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            icon: None,
            children: vec![],
            badge: None,
            folder: false,
        }
    }
    /// A folder.
    pub fn folder(id: &str, label: &str, children: Vec<TreeNode>) -> Self {
        Self {
            folder: true,
            children,
            ..Self::leaf(id, label)
        }
    }
    /// Icon.
    pub fn icon(mut self, i: &str) -> Self {
        self.icon = Some(i.into());
        self
    }
    /// Badge.
    pub fn badge(mut self, b: &str) -> Self {
        self.badge = Some(b.into());
        self
    }
    /// Is a folder.
    pub fn is_folder(&self) -> bool {
        self.folder || !self.children.is_empty()
    }
}

/// A visible row of the flattened tree.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TreeRow {
    /// Node id.
    pub id: String,
    /// Label.
    pub label: String,
    /// Icon.
    pub icon: Option<String>,
    /// Badge.
    pub badge: Option<String>,
    /// 1-based level.
    pub level: usize,
    /// 1-based position among siblings.
    pub pos: usize,
    /// Sibling count.
    pub size: usize,
    /// Folder.
    pub folder: bool,
    /// Expanded (folders).
    pub expanded: bool,
    /// Parent id.
    pub parent: Option<String>,
    /// For guide lines: whether each ancestor level has more siblings below.
    pub guides: Vec<bool>,
}

/// Tree key.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TreeKey {
    /// ↑.
    Up,
    /// ↓.
    Down,
    /// →.
    Right,
    /// ←.
    Left,
    /// Home.
    Home,
    /// End.
    End,
    /// Enter / Space.
    Activate,
    /// `*`: expand all siblings.
    ExpandSiblings,
}

/// Tree state.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Tree {
    /// Roots.
    pub nodes: Vec<TreeNode>,
    /// Open folders.
    pub expanded: BTreeSet<String>,
    /// Selected id.
    pub selected: Option<String>,
    /// Focused id.
    pub focus: Option<String>,
}

impl Tree {
    /// A tree with `expanded` folders open.
    pub fn new(nodes: Vec<TreeNode>, expanded: &[&str]) -> Self {
        Self {
            nodes,
            expanded: expanded.iter().map(|s| s.to_string()).collect(),
            selected: None,
            focus: None,
        }
    }
    /// Visible rows in order.
    pub fn rows(&self) -> Vec<TreeRow> {
        fn walk(
            ns: &[TreeNode],
            lvl: usize,
            parent: Option<&str>,
            guides: &[bool],
            ex: &BTreeSet<String>,
            out: &mut Vec<TreeRow>,
        ) {
            for (i, n) in ns.iter().enumerate() {
                let open = n.is_folder() && ex.contains(&n.id);
                out.push(TreeRow {
                    id: n.id.clone(),
                    label: n.label.clone(),
                    icon: n.icon.clone(),
                    badge: n.badge.clone(),
                    level: lvl,
                    pos: i + 1,
                    size: ns.len(),
                    folder: n.is_folder(),
                    expanded: open,
                    parent: parent.map(str::to_string),
                    guides: guides.to_vec(),
                });
                if open {
                    let mut g = guides.to_vec();
                    g.push(i + 1 < ns.len());
                    walk(&n.children, lvl + 1, Some(&n.id), &g, ex, out);
                }
            }
        }
        let mut out = Vec::new();
        walk(&self.nodes, 1, None, &[], &self.expanded, &mut out);
        out
    }
    fn find<'a>(ns: &'a [TreeNode], id: &str) -> Option<&'a TreeNode> {
        for n in ns {
            if n.id == id {
                return Some(n);
            }
            if let Some(x) = Self::find(&n.children, id) {
                return Some(x);
            }
        }
        None
    }
    /// A node by id.
    pub fn node(&self, id: &str) -> Option<&TreeNode> {
        Self::find(&self.nodes, id)
    }
    /// Toggles a folder.
    pub fn toggle(&mut self, id: &str) {
        if self.node(id).is_some_and(TreeNode::is_folder) && !self.expanded.remove(id) {
            self.expanded.insert(id.to_string());
        }
    }
    /// Click / activate a row: selects it, toggles a folder. Returns the selected id.
    pub fn activate(&mut self, id: &str) -> Option<String> {
        self.node(id)?;
        self.focus = Some(id.to_string());
        self.selected = Some(id.to_string());
        self.toggle(id);
        self.selected.clone()
    }
    /// Keyboard. Returns the newly selected id on Activate.
    pub fn key(&mut self, k: TreeKey) -> Option<String> {
        let rows = self.rows();
        if rows.is_empty() {
            return None;
        }
        let cur = self
            .focus
            .as_ref()
            .and_then(|f| rows.iter().position(|r| &r.id == f));
        let set = |t: &mut Self, i: usize| t.focus = Some(rows[i].id.clone());
        match k {
            TreeKey::Down => set(self, cur.map_or(0, |c| (c + 1).min(rows.len() - 1))),
            TreeKey::Up => set(self, cur.map_or(0, |c| c.saturating_sub(1))),
            TreeKey::Home => set(self, 0),
            TreeKey::End => set(self, rows.len() - 1),
            TreeKey::Right => {
                let c = cur.unwrap_or(0);
                let r = &rows[c];
                if r.folder && !r.expanded {
                    self.expanded.insert(r.id.clone());
                } else if r.folder && c + 1 < rows.len() && rows[c + 1].level > r.level {
                    set(self, c + 1);
                }
            }
            TreeKey::Left => {
                let c = cur.unwrap_or(0);
                let r = &rows[c];
                if r.folder && r.expanded {
                    self.expanded.remove(&r.id);
                } else if let Some(p) = &r.parent {
                    self.focus = Some(p.clone());
                }
            }
            TreeKey::Activate => {
                let id = rows[cur.unwrap_or(0)].id.clone();
                return self.activate(&id);
            }
            TreeKey::ExpandSiblings => {
                let c = cur.unwrap_or(0);
                let p = rows[c].parent.clone();
                for r in rows.iter().filter(|r| r.parent == p && r.folder) {
                    self.expanded.insert(r.id.clone());
                }
            }
        }
        None
    }
}

// ─────────────────────────────────── reorderable list ──

/// Moves item `i` by `delta` places in `v`; returns the new index.
pub fn move_item<T>(v: &mut Vec<T>, i: usize, delta: isize) -> usize {
    if i >= v.len() {
        return i;
    }
    let to = (i as isize + delta).clamp(0, v.len() as isize - 1) as usize;
    if to != i {
        let x = v.remove(i);
        v.insert(to, x);
    }
    to
}

/// Target index while dragging row `from` by `dy` px over rows of `row_h`.
pub fn drag_target(from: usize, dy: f32, row_h: f32, len: usize) -> usize {
    if len == 0 || row_h <= 0.0 {
        return from;
    }
    let shift = (dy / row_h).round() as isize;
    (from as isize + shift).clamp(0, len as isize - 1) as usize
}

/// Offset a non-dragged row takes while `from` is dragged over `to` (neighbours slide).
pub fn drag_shift(i: usize, from: usize, to: usize, row_h: f32) -> f32 {
    if from < to && i > from && i <= to {
        -row_h
    } else if from > to && i >= to && i < from {
        row_h
    } else {
        0.0
    }
}

/// Handle label: "Reorder Ship pricing page. Position 1 of 3. Use arrow keys."
pub fn handle_label(title: &str, pos: usize, len: usize) -> String {
    format!("Reorder {title}. Position {pos} of {len}. Use arrow keys.")
}

// ───────────────────────────── toggle / chip / radio ──

/// A toggle-group item.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ToggleItem {
    /// Value.
    pub value: String,
    /// Accessible name + tooltip (required for icon-only).
    pub label: String,
    /// Icon name.
    pub icon: Option<String>,
    /// Show the text.
    pub text: bool,
    /// Disabled.
    pub disabled: bool,
}

impl ToggleItem {
    /// An icon item.
    pub fn icon(value: &str, label: &str, icon: &str) -> Self {
        Self {
            value: value.into(),
            label: label.into(),
            icon: Some(icon.into()),
            text: false,
            disabled: false,
        }
    }
    /// A text item.
    pub fn text(value: &str, label: &str) -> Self {
        Self {
            value: value.into(),
            label: label.into(),
            icon: None,
            text: true,
            disabled: false,
        }
    }
}

/// Toggle group: single (radio) or multiple (pressed buttons whose neighbours join).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ToggleGroup {
    /// Items.
    pub items: Vec<ToggleItem>,
    /// Multiple mode.
    pub multiple: bool,
    /// Pressed values (in item order).
    pub value: Vec<String>,
    /// Focus.
    pub focus: Option<usize>,
}

impl ToggleGroup {
    /// A group.
    pub fn new(items: Vec<ToggleItem>, multiple: bool, value: &[&str]) -> Self {
        let mut g = Self {
            items,
            multiple,
            value: vec![],
            focus: None,
        };
        for v in value {
            if let Some(i) = g.items.iter().position(|x| x.value == *v) {
                g.press(i);
            }
        }
        g
    }
    /// Is item `i` on.
    pub fn on(&self, i: usize) -> bool {
        self.items
            .get(i)
            .is_some_and(|x| self.value.contains(&x.value))
    }
    /// Presses item `i`. Single mode: choosing the pressed option keeps it.
    pub fn press(&mut self, i: usize) -> bool {
        let Some(it) = self.items.get(i) else {
            return false;
        };
        if it.disabled {
            return false;
        }
        self.focus = Some(i);
        let v = it.value.clone();
        if self.multiple {
            if let Some(p) = self.value.iter().position(|x| *x == v) {
                self.value.remove(p);
            } else {
                self.value.push(v);
                let order: Vec<String> = self.items.iter().map(|x| x.value.clone()).collect();
                self.value
                    .sort_by_key(|x| order.iter().position(|o| o == x).unwrap_or(0));
            }
            true
        } else if self.value.first() == Some(&v) {
            false
        } else {
            self.value = vec![v];
            true
        }
    }
    /// Arrow keys move focus only (toolbar behaviour).
    pub fn key(&mut self, k: NavKey) -> Option<usize> {
        let it = &self.items;
        let start = self
            .focus
            .or_else(|| (0..it.len()).find(|&i| self.value.contains(&it[i].value)));
        let to = step(start, k, it.len(), true, |i| !it[i].disabled)?;
        self.focus = Some(to);
        Some(to)
    }
    /// Multiple mode: (joins left, joins right) for each item (neighbouring fills merge).
    pub fn joins(&self) -> Vec<(bool, bool)> {
        (0..self.items.len())
            .map(|i| {
                let on = self.multiple && self.on(i);
                (
                    on && i > 0 && self.on(i - 1),
                    on && i + 1 < self.items.len() && self.on(i + 1),
                )
            })
            .collect()
    }
}

/// Chip group: filter chips that fold behind "+N more".
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ChipGroup {
    /// (value, label).
    pub options: Vec<(String, String)>,
    /// Selected values in option order.
    pub value: Vec<String>,
    /// Several at once.
    pub multiple: bool,
    /// Visible before folding (0 = all).
    pub max_visible: usize,
    /// Overflow revealed.
    pub expanded: bool,
}

impl ChipGroup {
    /// A group.
    pub fn new(options: &[(&str, &str)], value: &[&str], multiple: bool) -> Self {
        Self {
            options: options
                .iter()
                .map(|(v, l)| (v.to_string(), l.to_string()))
                .collect(),
            value: value.iter().map(|s| s.to_string()).collect(),
            multiple,
            max_visible: 0,
            expanded: false,
        }
    }
    /// Toggles option `i`. Single mode still lets the chosen chip clear.
    pub fn toggle(&mut self, i: usize) -> Vec<String> {
        let Some((v, _)) = self.options.get(i).cloned() else {
            return self.value.clone();
        };
        if self.value.contains(&v) {
            self.value.retain(|x| *x != v);
        } else if self.multiple {
            self.value.push(v);
            let order: Vec<String> = self.options.iter().map(|o| o.0.clone()).collect();
            self.value
                .sort_by_key(|x| order.iter().position(|o| o == x).unwrap_or(0));
        } else {
            self.value = vec![v];
        }
        self.value.clone()
    }
    /// Indices shown and the folded count. Selected chips always stay visible.
    pub fn visible(&self) -> (Vec<usize>, usize) {
        let n = self.options.len();
        if self.max_visible == 0 || self.expanded || n <= self.max_visible {
            return ((0..n).collect(), 0);
        }
        let mut v: Vec<usize> = (0..self.max_visible).collect();
        for (i, (val, _)) in self.options.iter().enumerate().skip(self.max_visible) {
            if self.value.contains(val) {
                v.push(i);
            }
        }
        let hidden = n - v.len();
        (v, hidden)
    }
    /// Is option `i` selected.
    pub fn on(&self, i: usize) -> bool {
        self.options
            .get(i)
            .is_some_and(|(v, _)| self.value.contains(v))
    }
}

/// A radio card.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RadioCard {
    /// Value.
    pub value: String,
    /// Label.
    pub label: String,
    /// Description.
    pub description: Option<String>,
    /// Meta (price) at the end.
    pub meta: Option<String>,
    /// Icon.
    pub icon: Option<String>,
    /// Disabled.
    pub disabled: bool,
    /// Why disabled (describes the card).
    pub disabled_reason: Option<String>,
    /// Oxyn plus: a risk / status tone (`low` · `medium` · `high`).
    pub tone: Option<String>,
}

impl RadioCard {
    /// A card.
    pub fn new(value: &str, label: &str) -> Self {
        Self {
            value: value.into(),
            label: label.into(),
            description: None,
            meta: None,
            icon: None,
            disabled: false,
            disabled_reason: None,
            tone: None,
        }
    }
    /// Description.
    pub fn desc(mut self, d: &str) -> Self {
        self.description = Some(d.into());
        self
    }
    /// Meta.
    pub fn meta(mut self, m: &str) -> Self {
        self.meta = Some(m.into());
        self
    }
    /// Icon.
    pub fn icon(mut self, i: &str) -> Self {
        self.icon = Some(i.into());
        self
    }
    /// Disabled with a reason.
    pub fn disabled(mut self, why: &str) -> Self {
        self.disabled = true;
        self.disabled_reason = Some(why.into());
        self
    }
    /// Tone.
    pub fn tone(mut self, t: &str) -> Self {
        self.tone = Some(t.into());
        self
    }
}

/// Radio-card selection: arrows select and wrap, skipping disabled cards.
pub fn radio_step(cards: &[RadioCard], value: Option<&str>, k: NavKey) -> Option<usize> {
    let from = value.and_then(|v| cards.iter().position(|c| c.value == v));
    step(from, k, cards.len(), true, |i| !cards[i].disabled)
}

// ─────────────────────────────────────────── rating ──

/// Default words for five whole stars.
pub const RATING_WORDS: [&str; 5] = ["Poor", "Fair", "Good", "Great", "Excellent"];

/// Star rating.
#[derive(Clone, Debug, PartialEq)]
pub struct Rating {
    /// Stars.
    pub max: u32,
    /// Committed value (0 = not rated).
    pub value: f32,
    /// Pointer preview.
    pub hover: Option<f32>,
    /// Half steps.
    pub allow_half: bool,
    /// Choosing the value again clears it.
    pub allow_clear: bool,
    /// Direction of the last change (+1 / -1) for the sweep and the word roll.
    pub dir: i8,
    /// Bumps on commit (pop wave).
    pub version: u32,
}

impl Rating {
    /// A rating.
    pub fn new(max: u32, value: f32) -> Self {
        Self {
            max: max.max(1),
            value: value.clamp(0.0, max as f32),
            hover: None,
            allow_half: false,
            allow_clear: true,
            dir: 0,
            version: 0,
        }
    }
    fn stepv(&self) -> f32 {
        if self.allow_half { 0.5 } else { 1.0 }
    }
    /// Value under the pointer: star `i` (0-based) at `frac` across it.
    pub fn value_at(&self, i: u32, frac: f32) -> f32 {
        let base = i as f32;
        if self.allow_half && frac < 0.5 {
            base + 0.5
        } else {
            base + 1.0
        }
        .min(self.max as f32)
    }
    /// Commits `v`. Same value clears when allowed. Returns the committed value.
    pub fn commit(&mut self, v: f32) -> f32 {
        let v = v.clamp(0.0, self.max as f32);
        let nv = if self.allow_clear && (v - self.value).abs() < 0.01 {
            0.0
        } else {
            v
        };
        self.dir = if nv >= self.value { 1 } else { -1 };
        self.value = nv;
        self.version += 1;
        nv
    }
    /// Keys: ±step, Home / End, digits, clear.
    pub fn key(&mut self, k: RatingKey) -> f32 {
        let s = self.stepv();
        let v = match k {
            RatingKey::Up => (self.value + s).min(self.max as f32),
            RatingKey::Down => (self.value - s).max(if self.allow_clear { 0.0 } else { s }),
            RatingKey::Home => s,
            RatingKey::End => self.max as f32,
            RatingKey::Digit(n) => (n as f32).min(self.max as f32),
            RatingKey::Clear => {
                if self.allow_clear {
                    0.0
                } else {
                    self.value
                }
            }
        };
        if (v - self.value).abs() > 0.001 {
            self.dir = if v > self.value { 1 } else { -1 };
            self.value = v;
            self.version += 1;
        }
        self.value
    }
    /// Shown value (preview or committed).
    pub fn shown(&self) -> f32 {
        self.hover.unwrap_or(self.value)
    }
    /// Fill of star `i`: 0, 0.5 or 1.
    pub fn fill(&self, i: u32) -> f32 {
        (self.shown() - i as f32).clamp(0.0, 1.0)
    }
    /// Word for a value.
    pub fn word(&self, v: f32, labels: &[String], empty: &str) -> String {
        if v <= 0.0 {
            return empty.to_string();
        }
        let whole = v.ceil() as usize;
        if let Some(l) = labels.get(whole - 1) {
            return l.clone();
        }
        if self.max == 5 && labels.is_empty() {
            return RATING_WORDS[whole - 1].to_string();
        }
        fmt_num(v)
    }
    /// Radio label: "Great, 4 of 5 stars".
    pub fn radio_label(&self, v: f32, labels: &[String]) -> String {
        format!(
            "{}, {} of {} stars",
            self.word(v, labels, ""),
            fmt_num(v),
            self.max
        )
    }
}

fn fmt_num(v: f32) -> String {
    if v.fract() == 0.0 {
        format!("{}", v as u32)
    } else {
        format!("{v:.1}")
    }
}

/// Rating key.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RatingKey {
    /// → ↑.
    Up,
    /// ← ↓.
    Down,
    /// Home.
    Home,
    /// End.
    End,
    /// 1–9.
    Digit(u32),
    /// Backspace / Delete.
    Clear,
}

// ───────────────────────────────────── copy / expand ──

/// Copy-button state.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum CopyState {
    /// Ready.
    #[default]
    Idle,
    /// Copied at.
    Copied(ClockMs),
    /// Failed at.
    Failed(ClockMs),
}

/// How long Copied / Failed stays.
pub const COPY_HOLD: Duration = Duration::from_millis(1500);

impl CopyState {
    /// State at `now` (resets after [`COPY_HOLD`]).
    pub fn at(self, now: ClockMs) -> Self {
        match self {
            Self::Copied(t) | Self::Failed(t) if now.since(t) >= COPY_HOLD => Self::Idle,
            s => s,
        }
    }
    /// Id.
    pub fn id(self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::Copied(_) => "copied",
            Self::Failed(_) => "failed",
        }
    }
    /// Label for the state.
    pub fn label(self, idle: &str) -> &str {
        match self {
            Self::Idle => idle,
            Self::Copied(_) => "Copied",
            Self::Failed(_) => "Failed",
        }
    }
    /// Live-region text.
    pub fn announce(self, idle: &str) -> String {
        match self {
            Self::Idle => String::new(),
            Self::Copied(_) => format!("{idle}: Copied"),
            Self::Failed(_) => format!("{idle}: Could not copy"),
        }
    }
}

/// Expandable text: whether the toggle is needed (text taller than `lines` lines).
pub fn needs_clamp(full_height: f32, line_height: f32, lines: u32) -> bool {
    full_height > line_height * lines as f32 + 1.0
}

/// Open / close duration for an expandable block: grows gently with distance (Arc: 500–600 ms
/// open, 400–450 ms close).
pub fn expand_duration(distance: f32, opening: bool) -> Duration {
    let (lo, hi) = if opening {
        (500.0, 600.0)
    } else {
        (400.0, 450.0)
    };
    let k = (distance / 800.0).clamp(0.0, 1.0);
    Duration::from_millis((lo + (hi - lo) * k) as u64)
}

// ───────────────────────────────────────────── dock ──

/// A dock item; with `items` it is a group whose tray grows out of the dock.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DockItem {
    /// Id.
    pub id: String,
    /// Label.
    pub label: String,
    /// Icon.
    pub icon: String,
    /// Display-only shortcut.
    pub shortcut: Option<String>,
    /// Badge count.
    pub badge: Option<u32>,
    /// Group members.
    pub items: Vec<DockItem>,
}

impl DockItem {
    /// An item.
    pub fn new(id: &str, label: &str, icon: &str) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            icon: icon.into(),
            shortcut: None,
            badge: None,
            items: vec![],
        }
    }
    /// Shortcut.
    pub fn key(mut self, k: &str) -> Self {
        self.shortcut = Some(k.into());
        self
    }
    /// Badge.
    pub fn badge(mut self, n: u32) -> Self {
        self.badge = Some(n);
        self
    }
    /// Group members.
    pub fn group(mut self, items: Vec<DockItem>) -> Self {
        self.items = items;
        self
    }
}

/// Dock state.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Dock {
    /// Items.
    pub items: Vec<DockItem>,
    /// Selected id (item or member).
    pub value: Option<String>,
    /// Shown member per group (group id → member id).
    pub shown: Vec<(String, String)>,
    /// Open tray (group index).
    pub tray: Option<usize>,
    /// Focus (dock index).
    pub focus: Option<usize>,
    /// Focus inside the tray.
    pub tray_focus: Option<usize>,
}

impl Dock {
    /// A dock.
    pub fn new(items: Vec<DockItem>, value: Option<&str>) -> Self {
        let shown = items
            .iter()
            .filter(|g| !g.items.is_empty())
            .map(|g| {
                let m = g
                    .items
                    .iter()
                    .find(|m| Some(m.id.as_str()) == value)
                    .unwrap_or(&g.items[0]);
                (g.id.clone(), m.id.clone())
            })
            .collect();
        Self {
            items,
            value: value.map(str::to_string),
            shown,
            tray: None,
            focus: None,
            tray_focus: None,
        }
    }
    /// The member a group shows.
    pub fn shown_member(&self, i: usize) -> Option<&DockItem> {
        let g = self.items.get(i)?;
        let id = self.shown.iter().find(|(gid, _)| *gid == g.id)?.1.clone();
        g.items.iter().find(|m| m.id == id)
    }
    /// Is dock item `i` selected (itself or its shown member).
    pub fn is_selected(&self, i: usize) -> bool {
        let Some(v) = &self.value else { return false };
        let g = &self.items[i];
        g.id == *v || g.items.iter().any(|m| m.id == *v)
    }
    /// Press item `i`: selects; a group selects its shown member and toggles its tray.
    pub fn press(&mut self, i: usize) -> Option<String> {
        let g = self.items.get(i)?.clone();
        self.focus = Some(i);
        if g.items.is_empty() {
            self.tray = None;
            self.value = Some(g.id.clone());
            return self.value.clone();
        }
        let m = self.shown_member(i)?.id.clone();
        self.value = Some(m);
        self.tray = if self.tray == Some(i) { None } else { Some(i) };
        self.tray_focus = None;
        self.value.clone()
    }
    /// Chooses member `j` of group `i` (closes the tray).
    pub fn choose(&mut self, i: usize, j: usize) -> Option<String> {
        let g = self.items.get(i)?;
        let m = g.items.get(j)?.id.clone();
        if let Some(s) = self.shown.iter_mut().find(|(gid, _)| *gid == g.id) {
            s.1 = m.clone();
        }
        self.value = Some(m);
        self.tray = None;
        self.tray_focus = None;
        self.focus = Some(i);
        self.value.clone()
    }
    /// Escape / ArrowDown in a tray / pointer down outside.
    pub fn close_tray(&mut self) -> bool {
        self.tray_focus = None;
        self.tray.take().is_some()
    }
    /// ArrowUp on a group: opens its tray and focuses the chosen member.
    pub fn open_tray(&mut self) -> bool {
        let Some(i) = self.focus else { return false };
        if self.items[i].items.is_empty() {
            return false;
        }
        self.tray = Some(i);
        let shown = self.shown_member(i).map(|m| m.id.clone());
        self.tray_focus = self.items[i]
            .items
            .iter()
            .position(|m| Some(&m.id) == shown.as_ref());
        true
    }
    /// ← → Home End (in the tray when one has focus).
    pub fn key(&mut self, k: NavKey) {
        if let (Some(t), Some(_)) = (self.tray, self.tray_focus) {
            let n = self.items[t].items.len();
            self.tray_focus = step(self.tray_focus, k, n, true, |_| true);
            return;
        }
        self.focus = step(self.focus, k, self.items.len(), true, |_| true);
    }
    /// Alt + ← →: moves the focused item. Returns (id, new index).
    pub fn move_focused(&mut self, delta: isize) -> Option<(String, usize)> {
        let i = self.focus?;
        let id = self.items[i].id.clone();
        let to = move_item(&mut self.items, i, delta);
        self.focus = Some(to);
        if let Some(t) = self.tray {
            self.tray = Some(if t == i { to } else { t });
        }
        Some((id, to))
    }
    /// Accessible name ("Pen, P"; "Shapes: Ellipse, O, selected"; "Inbox, 3 new").
    pub fn aria(&self, i: usize) -> String {
        let g = &self.items[i];
        let mut s = if let Some(m) = self.shown_member(i) {
            let mut x = format!("{}: {}", g.label, m.label);
            if let Some(k) = &m.shortcut {
                x.push_str(&format!(", {k}"));
            }
            x
        } else {
            let mut x = g.label.clone();
            if let Some(k) = &g.shortcut {
                x.push_str(&format!(", {k}"));
            }
            x
        };
        if let Some(b) = g.badge {
            s.push_str(&format!(", {b} new"));
        }
        if !g.items.is_empty() && self.is_selected(i) {
            s.push_str(", selected");
        }
        s
    }
}

// ─────────────────────────────────────── toc / header ──

/// A heading in the table of contents.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TocItem {
    /// Heading element id.
    pub id: String,
    /// Label.
    pub label: String,
    /// 1–3.
    pub level: u8,
}

impl TocItem {
    /// An item.
    pub fn new(id: &str, label: &str, level: u8) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            level: level.clamp(1, 3),
        }
    }
}

/// Reading position: (section index, progress through it 0–1, overall 0–1).
///
/// `tops` are heading offsets from the top of the scrolled content. A heading is reached when it
/// crosses `offset` px below the viewport top; near the end the reading line eases down to the
/// bottom edge so short last sections are still reached.
pub fn toc_progress(
    tops: &[f32],
    scroll_top: f32,
    viewport: f32,
    content: f32,
    offset: f32,
) -> (usize, f32, f32) {
    if tops.is_empty() {
        return (0, 0.0, 0.0);
    }
    let max_scroll = (content - viewport).max(1.0);
    let near_end = ((scroll_top - (max_scroll - viewport)) / viewport.max(1.0)).clamp(0.0, 1.0);
    let line = scroll_top + offset + (viewport - offset) * near_end;
    let mut idx = 0;
    for (i, &t) in tops.iter().enumerate() {
        if t <= line {
            idx = i;
        }
    }
    let start = tops[idx];
    let end = tops.get(idx + 1).copied().unwrap_or(content);
    let frac = if line < tops[0] {
        0.0
    } else {
        ((line - start) / (end - start).max(1.0)).clamp(0.0, 1.0)
    };
    let overall = ((idx as f32 + frac) / tops.len() as f32).clamp(0.0, 1.0);
    (idx, frac, overall)
}

/// Minutes left at `overall` progress of a `total` minute read ("Finished" at the end).
pub fn time_left(total_min: f32, overall: f32) -> String {
    let left = (total_min * (1.0 - overall)).ceil() as u32;
    if overall >= 0.995 || left == 0 {
        "Finished".into()
    } else {
        format!("{left} min left")
    }
}

/// Reading time at 230 words a minute.
pub fn reading_minutes(words: usize) -> f32 {
    (words as f32 / 230.0).max(1.0)
}

/// Page header: folded into the compact bar past `threshold` px (with 8 px hysteresis).
pub fn header_folded(scroll_top: f32, threshold: f32, was: bool) -> bool {
    if was {
        scroll_top > threshold - 8.0
    } else {
        scroll_top > threshold
    }
}

// ─────────────────────────────────────── empty states ──

/// Empty-state presets (Arc Empty states block + agent apps).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum EmptyPreset {
    /// Nothing created yet.
    #[default]
    FirstRun,
    /// A search matched nothing.
    NoResults,
    /// No runs.
    NoRuns,
    /// Inbox zero.
    InboxZero,
    /// Offline.
    Offline,
    /// Failed to load.
    Error,
    /// No access.
    NoAccess,
}

impl EmptyPreset {
    /// From an id.
    pub fn from_id(s: &str) -> Self {
        match s {
            "no-results" => Self::NoResults,
            "no-runs" => Self::NoRuns,
            "inbox-zero" => Self::InboxZero,
            "offline" => Self::Offline,
            "error" => Self::Error,
            "no-access" => Self::NoAccess,
            _ => Self::FirstRun,
        }
    }
    /// Id.
    pub fn id(self) -> &'static str {
        match self {
            Self::FirstRun => "first-run",
            Self::NoResults => "no-results",
            Self::NoRuns => "no-runs",
            Self::InboxZero => "inbox-zero",
            Self::Offline => "offline",
            Self::Error => "error",
            Self::NoAccess => "no-access",
        }
    }
    /// The empty state for the preset.
    pub fn state(self, query: &str) -> EmptyState {
        let (title, text, action, secondary, icon): (
            String,
            &str,
            Option<&str>,
            Option<&str>,
            &str,
        ) = match self {
            Self::FirstRun => (
                "No projects yet".into(),
                "Create a project to give your agents a repository, tools and rules.",
                Some("New project"),
                Some("Import from GitHub"),
                "folder",
            ),
            Self::NoResults => (
                if query.is_empty() {
                    "No results".into()
                } else {
                    format!("No results for “{query}”")
                },
                "Check the spelling or try a broader term. Filters may be hiding matches.",
                Some("Clear filters"),
                None,
                "search",
            ),
            Self::NoRuns => (
                "No runs yet".into(),
                "Start a run from the composer. Runs show up here with their cost and result.",
                Some("Start a run"),
                Some("Read the docs"),
                "play",
            ),
            Self::InboxZero => (
                "You're all caught up".into(),
                "No approvals or mentions waiting. New ones appear here as agents work.",
                None,
                None,
                "inbox",
            ),
            Self::Offline => (
                "You're offline".into(),
                "Changes are kept on this device and sync when the connection comes back.",
                Some("Retry"),
                None,
                "globe",
            ),
            Self::Error => (
                "Couldn't load runs".into(),
                "The server returned an error. Your work is safe; try again in a moment.",
                Some("Try again"),
                Some("Status page"),
                "alert",
            ),
            Self::NoAccess => (
                "You don't have access".into(),
                "Ask a workspace admin to add you to this project.",
                Some("Request access"),
                None,
                "lock",
            ),
        };
        EmptyState {
            title,
            text: text.into(),
            action: action.map(str::to_string),
            secondary: secondary.map(str::to_string),
            icon: icon.into(),
            layout: EmptyLayout::default(),
            tall: false,
            prompts: vec![],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tabs() -> Tabs {
        Tabs::new(
            vec![
                Tab::new("a", "Overview"),
                Tab::new("b", "Activity").disabled(),
                Tab::new("c", "Settings").closable(),
            ],
            Some("a"),
        )
    }

    #[test]
    fn tabs_skip_disabled_and_track_travel() {
        let mut t = tabs();
        assert_eq!(t.key(NavKey::Next), Some("c".into()));
        assert_eq!(t.travel, Travel::Forward);
        assert_eq!(t.key(NavKey::Next), Some("a".into()));
        assert_eq!(t.travel, Travel::Back);
        t.activation = Activation::Manual;
        assert_eq!(t.key(NavKey::Last), None);
        assert_eq!(t.activate(), Some("c".into()));
    }

    #[test]
    fn closing_selected_tab_picks_neighbour() {
        let mut t = tabs();
        t.select(2);
        t.close(2);
        assert_eq!(t.value, "b");
        t.close(0);
        assert_eq!(t.tabs.len(), 1);
        let v = t.push(Tab::new("d", "zsh"));
        assert_eq!(v, "d");
    }

    #[test]
    fn overflow_and_scroll() {
        assert_eq!(overflow_edges(0.0, 200.0, 400.0), (false, true));
        assert_eq!(overflow_edges(200.0, 200.0, 400.0), (true, false));
        assert_eq!(scroll_into_view(0.0, 200.0, 250.0, 60.0, 8.0), 118.0);
    }

    #[test]
    fn crumbs_collapse_middle() {
        assert_eq!(crumb_slots(3, 4).len(), 3);
        let s = crumb_slots(6, 4);
        assert_eq!(
            s,
            vec![
                CrumbSlot::Item(0),
                CrumbSlot::More(vec![1, 2, 3]),
                CrumbSlot::Item(4),
                CrumbSlot::Item(5)
            ]
        );
    }

    #[test]
    fn accordion_single_and_multiple() {
        let mut a = Accordion::new(3, Some(0));
        a.toggle(1);
        assert!(a.is_open(1) && !a.is_open(0));
        a.toggle(1);
        assert!(a.open.is_empty());
        a.multiple = true;
        a.toggle(0);
        a.toggle(2);
        assert_eq!(a.open.len(), 2);
        assert_eq!(a.key(NavKey::Last), Some(2));
    }

    #[test]
    fn resizable_clamps_collapses_and_restores() {
        let mut r = Resizable::new(
            vec![
                Pane::new("nav", "Navigation", 25.0)
                    .min(160.0)
                    .collapsible(),
                Pane::new("main", "Editor", 75.0).min(200.0),
            ],
            800.0,
        );
        assert_eq!(r.sizes, vec![200.0, 600.0]);
        r.move_divider(0, -30.0);
        assert_eq!(r.sizes[0], 170.0);
        r.move_divider(0, -20.0);
        assert_eq!(r.sizes[0], 160.0);
        assert!(r.move_divider(0, -100.0));
        assert!(r.collapsed[0]);
        assert!(r.value_text(0).contains("hidden"));
        assert!(r.toggle(0));
        assert!(!r.collapsed[0]);
        r.begin(0);
        r.move_divider(0, 100.0);
        r.cancel();
        assert_eq!(r.sizes[0], 160.0);
        r.to_limit(0, true);
        assert_eq!(r.sizes[0], 600.0);
        assert_eq!(r.shares(), vec![75.0, 25.0]);
    }

    #[test]
    fn sidebar_snaps_to_rail() {
        assert_eq!(
            sidebar_snap(300.0, 56.0, 200.0, 360.0, true),
            SidebarSnap::Open(300.0)
        );
        assert_eq!(
            sidebar_snap(150.0, 56.0, 200.0, 360.0, true),
            SidebarSnap::Open(200.0)
        );
        assert_eq!(
            sidebar_snap(100.0, 56.0, 200.0, 360.0, true),
            SidebarSnap::Rail
        );
        assert_eq!(
            sidebar_snap(10.0, 56.0, 200.0, 360.0, true),
            SidebarSnap::Hidden
        );
        assert_eq!(
            sidebar_snap(10.0, 56.0, 200.0, 360.0, false),
            SidebarSnap::Rail
        );
    }

    #[test]
    fn hover_intent_delays_and_warm() {
        let mut h = HoverIntent::default();
        h.enter(ClockMs(0));
        assert!(!h.tick(ClockMs(400)));
        assert!(h.tick(ClockMs(500)) && h.open);
        h.leave(ClockMs(600));
        h.enter(ClockMs(700));
        assert!(!h.tick(ClockMs(800)) && h.open);
        h.leave(ClockMs(900));
        assert!(h.tick(ClockMs(1040)) && !h.open);
        h.enter(ClockMs(1100));
        assert!(h.warm);
        assert!(h.tick(ClockMs(1180)));
        assert!(h.escape(ClockMs(1200)));
        h.enter(ClockMs(1210));
        assert!(h.opening.is_none());
        assert_eq!(initials("Ada Lovelace"), "AL");
    }

    fn tree() -> Tree {
        Tree::new(
            vec![
                TreeNode::folder(
                    "src",
                    "src",
                    vec![
                        TreeNode::leaf("main", "main.rs"),
                        TreeNode::leaf("lib", "lib.rs"),
                    ],
                ),
                TreeNode::leaf("readme", "README.md"),
            ],
            &[],
        )
    }

    #[test]
    fn tree_keyboard() {
        let mut t = tree();
        assert_eq!(t.rows().len(), 2);
        t.key(TreeKey::Home);
        t.key(TreeKey::Right);
        assert_eq!(t.rows().len(), 4);
        t.key(TreeKey::Right);
        assert_eq!(t.focus.as_deref(), Some("main"));
        let r = &t.rows()[1];
        assert_eq!((r.level, r.pos, r.size), (2, 1, 2));
        t.key(TreeKey::Left);
        assert_eq!(t.focus.as_deref(), Some("src"));
        t.key(TreeKey::Left);
        assert_eq!(t.rows().len(), 2);
        t.key(TreeKey::End);
        assert_eq!(t.key(TreeKey::Activate), Some("readme".into()));
    }

    #[test]
    fn reorder_helpers() {
        let mut v = vec!["a", "b", "c"];
        assert_eq!(move_item(&mut v, 0, 1), 1);
        assert_eq!(v, vec!["b", "a", "c"]);
        assert_eq!(move_item(&mut v, 2, 5), 2);
        assert_eq!(drag_target(0, 70.0, 56.0, 3), 1);
        assert_eq!(drag_shift(1, 0, 2, 56.0), -56.0);
        assert_eq!(drag_shift(0, 2, 0, 56.0), 56.0);
        assert!(handle_label("Ship", 1, 3).starts_with("Reorder Ship. Position 1 of 3"));
    }

    #[test]
    fn toggle_group_modes() {
        let items = vec![
            ToggleItem::icon("b", "Bold", "type"),
            ToggleItem::icon("i", "Italic", "type"),
            ToggleItem::icon("u", "Underline", "type"),
        ];
        let mut g = ToggleGroup::new(items.clone(), true, &["u", "b"]);
        assert_eq!(g.value, vec!["b", "u"]);
        g.press(1);
        assert_eq!(g.joins()[1], (true, true));
        let mut s = ToggleGroup::new(items, false, &["b"]);
        assert!(!s.press(0));
        assert!(s.press(2));
        assert_eq!(s.value, vec!["u"]);
        assert_eq!(s.key(NavKey::Next), Some(0));
    }

    #[test]
    fn chip_group_keeps_selected_visible() {
        let mut c = ChipGroup::new(
            &[("a", "A"), ("b", "B"), ("c", "C"), ("d", "D"), ("e", "E")],
            &["e"],
            true,
        );
        c.max_visible = 2;
        assert_eq!(c.visible(), (vec![0, 1, 4], 2));
        c.toggle(2);
        assert_eq!(c.value, vec!["c", "e"]);
        c.multiple = false;
        c.toggle(0);
        assert_eq!(c.value, vec!["a"]);
        c.toggle(0);
        assert!(c.value.is_empty());
    }

    #[test]
    fn radio_cards_skip_disabled() {
        let cards = vec![
            RadioCard::new("a", "A"),
            RadioCard::new("b", "B").disabled("Sold out"),
            RadioCard::new("c", "C"),
        ];
        assert_eq!(radio_step(&cards, Some("a"), NavKey::Next), Some(2));
        assert_eq!(radio_step(&cards, Some("c"), NavKey::Next), Some(0));
    }

    #[test]
    fn rating_half_clear_and_words() {
        let mut r = Rating::new(5, 0.0);
        r.allow_half = true;
        assert_eq!(r.value_at(2, 0.2), 2.5);
        assert_eq!(r.commit(4.0), 4.0);
        assert_eq!(r.commit(4.0), 0.0);
        assert_eq!(r.key(RatingKey::Up), 0.5);
        assert_eq!(r.key(RatingKey::Digit(4)), 4.0);
        assert_eq!(r.fill(3), 1.0);
        assert_eq!(r.word(4.0, &[], "Not rated"), "Great");
        assert_eq!(r.radio_label(4.0, &[]), "Great, 4 of 5 stars");
        assert_eq!(r.key(RatingKey::Clear), 0.0);
    }

    #[test]
    fn copy_state_resets() {
        let s = CopyState::Copied(ClockMs(0));
        assert_eq!(s.at(ClockMs(1000)).id(), "copied");
        assert_eq!(s.at(ClockMs(1500)), CopyState::Idle);
        assert_eq!(s.announce("Copy"), "Copy: Copied");
        assert!(needs_clamp(80.0, 20.0, 3));
        assert_eq!(expand_duration(0.0, true), Duration::from_millis(500));
    }

    #[test]
    fn dock_groups_and_trays() {
        let mut d = Dock::new(
            vec![
                DockItem::new("pen", "Pen", "pencil").key("P"),
                DockItem::new("shapes", "Shapes", "symbol").group(vec![
                    DockItem::new("rect", "Rectangle", "symbol").key("R"),
                    DockItem::new("ell", "Ellipse", "symbol").key("O"),
                ]),
                DockItem::new("inbox", "Inbox", "inbox").badge(3),
            ],
            Some("pen"),
        );
        assert_eq!(d.aria(0), "Pen, P");
        d.press(1);
        assert_eq!(d.value.as_deref(), Some("rect"));
        assert_eq!(d.tray, Some(1));
        d.choose(1, 1);
        assert_eq!(d.aria(1), "Shapes: Ellipse, O, selected");
        assert_eq!(d.aria(2), "Inbox, 3 new");
        d.focus = Some(1);
        assert!(d.open_tray());
        assert_eq!(d.tray_focus, Some(1));
        assert!(d.close_tray());
        assert_eq!(d.move_focused(1), Some(("shapes".into(), 2)));
    }

    #[test]
    fn toc_progress_follows_scroll() {
        let tops = [0.0, 400.0, 800.0];
        let (i, f, _) = toc_progress(&tops, 0.0, 300.0, 1200.0, 100.0);
        assert_eq!(i, 0);
        assert!((f - 0.25).abs() < 0.01);
        let (i, _, _) = toc_progress(&tops, 350.0, 300.0, 1200.0, 100.0);
        assert_eq!(i, 1);
        let (i, _, o) = toc_progress(&tops, 900.0, 300.0, 1200.0, 100.0);
        assert_eq!(i, 2);
        assert!(o > 0.9);
        assert_eq!(time_left(6.0, 0.5), "3 min left");
        assert_eq!(time_left(6.0, 1.0), "Finished");
        assert!(header_folded(70.0, 64.0, false));
        assert!(header_folded(60.0, 64.0, true));
    }

    #[test]
    fn empty_presets() {
        let s = EmptyPreset::NoResults.state("auth");
        assert!(s.title.contains("auth"));
        assert_eq!(EmptyPreset::from_id("offline").id(), "offline");
    }
}
