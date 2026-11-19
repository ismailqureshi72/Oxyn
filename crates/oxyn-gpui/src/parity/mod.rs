//! GPUI elements for the plan #28 parity components, drawn from the same
//! `oxyn_core::parity` models as the Dioxus views. Elements are `RenderOnce`; buttons report
//! through one [`Act`] callback with a short action id (`"confirm"`, `"cancel"`, …) so a host can
//! route them to the model without one closure per button.

use std::rc::Rc;

use gpui::{
    App, ElementId, FontWeight, Hsla, IntoElement, ParentElement, SharedString, Styled, Window,
    div, prelude::*, px,
};
use oxyn_tokens::{ColorRole, Theme};

use crate::hsla;

mod agentscr;
mod agentsfid;
mod answers;
mod approvals;
mod apps;
mod arcscr;
mod arcscr2;
mod attachfid;
mod autonomyfid;
mod awayfid;
mod buildscr;
mod chatfid;
mod chatscr;
mod cmdboard;
mod cmdstudio;
mod collabfid;
mod costfid;
mod crewfid;
mod ctxefffid;
mod ctxmapfid;
mod ctxvisfid;
mod cursorsfid;
mod delegatefid;
mod difftlfid;
mod dtreefid;
mod efflistfid;
mod effpillfid;
mod fallbackfid;
mod fanfid;
mod fidelity;
mod fields;
mod fleetwin;
mod gooeyfid;
mod graphfid;
mod handofffid;
mod home;
mod ime;
mod judgefid;
mod lanesfid;
mod ledgerfid;
mod memoryfid;
mod merge;
mod modeleffid;
mod monitorfid;
mod nozzlefid;
mod onbscr;
mod opsscr;
mod parallelfid;
mod planfid;
mod queuefid;
mod ratefid;
mod reasonfid;
mod recapfid;
mod review;
mod rootfid;
mod rulerfid;
mod run;
mod runningfid;
mod schedfid;
mod screens2fid;
mod screensfid;
mod screenview;
mod seqfid;
mod setscr;
mod setscr2;
mod stairsfid;
mod storagefid;
mod taskdeck;
mod text;
mod themefid;
mod toolmapfid;
mod tools;
mod tracefid;
mod workscr;
mod wscoder;

pub use agentscr::{ActivityEl, AgentThreadEl, InboxBoardEl, InboxScreenEl, MemoryEl, TasksEl};
pub use agentsfid::{AgentFindingsEl, AgentInboxEl};
pub use answers::{
    AnswerCardEl, CitedAnswerEl, MessageActionsEl, ThinkingPanelEl, ThinkingPillEl,
    WritingAssistantEl,
};
pub use approvals::{
    AgentApprovalsEl, ApprovalGatesEl, ApprovalPagerEl, ConfirmMorphEl, ConfirmPopoverEl,
    HoldConfirmEl, PermissionRequestEl,
};
pub use apps::{AgentApp, AgentAppView, Live, Own};
pub use arcscr::{
    ArcDrag, GanttTimelineEl, KanbanBoardEl, MetricsDashboardEl, ProjectBoardEl, TeamDirectoryEl,
};
pub use arcscr2::{
    HelpCenterEl, InboxTriageEl, MonthCalendarEl, SupportConversationEl, WeekCalendarEl,
};
pub use attachfid::{AttachmentChipsEl, DropLanesEl, PolaroidStackEl, UploadDropzoneEl};
pub use autonomyfid::AutonomyDialEl;
pub use awayfid::AwaySummaryEl;
pub use buildscr::EvalsEl;
pub use chatfid::{AgentRunPanelEl, ModelPickerMenuEl};
pub use chatscr::{
    ChatAgentsEl, ChatOverviewEl, ChatProgressEl, ChatRailEl, ChatRunEl, ChatStartEl,
};
pub use cmdboard::CommandBoardEl;
pub use cmdstudio::{CommandCenterEl, SessionStudioEl};
pub use collabfid::CollabStreamEl;
pub use costfid::CostWaterfallEl;
pub use crewfid::AgentCrewEl;
pub use ctxefffid::ContextEffortMenuEl;
pub use ctxmapfid::ContextMapEl;
pub use ctxvisfid::{ContextBarEl, ContextBreakdownEl, ContextStackEl, ContextTideEl};
pub use cursorsfid::AgentCursorsEl;
pub use delegatefid::DelegateBoardEl;
pub use difftlfid::DiffTimelineRunEl;
pub use dtreefid::DecisionTreeEl;
pub use efflistfid::EffortListEl;
pub use effpillfid::EffortPillEl;
pub use fallbackfid::FallbackChainEl;
pub use fanfid::FanoutGridEl;
pub use fidelity::{AgentCommentThreadEl, ChatComposerEl, CommentsPanelEl, ThreadMsg, file_badge};
pub use fields::{
    AiComposerEl, AvatarUploadEl, ContextFieldEl, ContextGridEl, ElasticSliderEl, FileDropzoneEl,
    GaugeEl, ModelMenuEl, ModelPillEl, MorphSelectEl, NumberFieldEl, Pick, RichTextEl,
    RunControlsEl, ScrubInputEl, SplitBarEl, UsageMeterEl,
};
pub use fleetwin::FleetWindowEl;
pub use gooeyfid::{
    GooeyAttachMenuEl, GooeyComposerEl, GooeyModelMenuEl, LiquidDropsEl, LiquidTabsEl,
};
pub use graphfid::AgentGraphEl;
pub use handofffid::AgentHandoffEl;
pub use home::{
    ActivityHeatmapEl, ActivityHomeEl, AttachmentStripEl, DocGridEl, ExperimentGridEl,
    PollingCostEl, RunModeMenuEl, ThreadListEl, TokenUsageChartEl,
};
pub use ime::{ImeEvent, ImeInput, bind_ime_keys};
pub use judgefid::JudgeBracketEl;
pub use lanesfid::AgentLanesEl;
pub use ledgerfid::AgentLedgerEl;
pub use memoryfid::MemoryPanelEl;
pub use merge::{ConflictRadarEl, MergeAct, MergeConflictEl, RadarAct};
pub use modeleffid::ModelEffortMenuEl;
pub use monitorfid::AgentMonitorEl;
pub use nozzlefid::NozzleSliderEl;
pub use onbscr::{OnboardingBrandEl, OnboardingReadyEl, OnboardingToneEl, OnboardingWebsiteEl};
pub use opsscr::{AgentsEl, PermissionsEl, RunDetailEl, SchedulesEl, SkillsEl, UsageEl};
pub use parallelfid::ParallelAttemptsEl;
pub use planfid::PlanVsActualEl;
pub use queuefid::AgentQueueEl;
pub use ratefid::RateLimitPlannerEl;
pub use reasonfid::ReasoningPanelEl;
pub use recapfid::AgentRecapEl;
pub use review::{
    AiInlineEditEl, ChangeSummaryEl, CodeSuggestionEl, CommentThreadEl, DiffTableEl,
    DiffTimelineEl, DiffViewerEl, HunkAct, IdAct, ImageCompareEl, InlineDiffEl, ReviewCardEl,
    ReviewCenterEl,
};
pub use rootfid::RootCauseTreeEl;
pub use rulerfid::EffortRulerEl;
pub use run::{
    ActivityFeedEl, AgentPlanEl, AgentRunEl, AgentStatusEl, AgentTrailEl, AgentTreeEl, BreakdownEl,
    ChecklistEl, CountdownEl, CounterEl, FleetEl, FlowStepperEl, OrchestratorEl, ReadinessEl,
    TaskRowsEl, TreeTableEl,
};
pub use runningfid::RunningNowEl;
pub use schedfid::ScheduledRunsEl;
pub use screens2fid::OrchestrationEl;
pub use screensfid::{
    AccountMenuEl, ScreenRailEl, ScreenShellEl, ScreenStageEl, SettingsEl, WorkflowEl,
};
pub use screenview::{ScreenView, screen_element};
pub use seqfid::AgentSequenceEl;
pub use setscr::{
    BillingSettingsEl, GeneralSettingsEl, MembersSettingsEl, ModelsSettingsEl,
    NotificationsSettingsEl, OnboardingSettingsEl, PreferencesSettingsEl, PrivacySettingsEl,
    ProfileSettingsEl, SecuritySettingsEl,
};
pub use stairsfid::EffortStairsEl;
pub use storagefid::{StorageCleanupEl, StorageForecastEl, StorageMeterEl, StorageWaffleEl};
pub use taskdeck::{
    TaskdeckBoardEl, TaskdeckPluginsEl, TaskdeckScheduledEl, TaskdeckSitesEl, TaskdeckStartEl,
    TaskdeckTaskEl,
};
pub use text::{LoadingStateEl, ShimmerEl, SlotTextEl, ThinkingLoaderEl};
pub use themefid::{
    AccentDensityEl, AppIconPickerEl, BackgroundPickerEl, DisplayComfortEl, LightPlaygroundEl,
    ThemeGridEl, ThemeScheduleEl, ThemeSplitEl, TypeScaleEl,
};
pub use toolmapfid::ToolAccessMapEl;
pub use tools::{
    ExpandableCardEl, JsonViewerEl, LinkUnfurlEl, ToolCallCardEl, ToolCallEl, ToolChipsEl,
};
pub use tracefid::TraceWaterfallEl;
pub use workscr::{AnalyticsEl, CrmEl, FolderDetailEl, FoldersEl, IntegrationsEl, KanbanEl};
pub use wscoder::WorkspaceCoderEl;

/// Button callback: the action id and the window.
pub type Act = Rc<dyn Fn(&'static str, &mut Window, &mut App)>;

pub(crate) fn c(t: &Theme, r: ColorRole) -> Hsla {
    hsla(t.color(r))
}

pub(crate) fn alpha(mut h: Hsla, a: f32) -> Hsla {
    h.a *= a;
    h
}

/// Visual weight of a kit button.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum Weight {
    Quiet,
    Plain,
    Primary,
    Danger,
}

/// A 30 px button that reports `action` through `act`.
pub(crate) fn button(
    t: &'static Theme,
    id: impl Into<ElementId>,
    label: impl Into<SharedString>,
    w: Weight,
    action: &'static str,
    act: Option<Act>,
) -> impl IntoElement {
    let (bg, fg, border) = match w {
        Weight::Quiet => (
            alpha(c(t, ColorRole::BgBase), 0.0),
            c(t, ColorRole::FgMuted),
            None,
        ),
        Weight::Plain => (
            c(t, ColorRole::SurfaceRaised),
            c(t, ColorRole::FgDefault),
            Some(c(t, ColorRole::BorderDefault)),
        ),
        Weight::Primary => (c(t, ColorRole::FgDefault), c(t, ColorRole::BgBase), None),
        Weight::Danger => (
            c(t, ColorRole::StatusDangerFg),
            c(t, ColorRole::FgOnAccent),
            None,
        ),
    };
    div()
        .id(id.into())
        .flex()
        .flex_none()
        .items_center()
        .gap(px(6.))
        .h(px(30.))
        .px(px(12.))
        .rounded(px(8.))
        .bg(bg)
        .text_color(fg)
        .text_size(px(12.5))
        .font_weight(FontWeight::MEDIUM)
        .cursor_pointer()
        .when_some(border, |d, b| d.border_1().border_color(b))
        .hover(move |s| s.opacity(0.9))
        .child(label.into())
        .when_some(act, move |d, f| {
            d.on_click(move |_, win, cx| f(action, win, cx))
        })
}

/// A small pill tag.
pub(crate) fn tag(
    t: &'static Theme,
    text: impl Into<SharedString>,
    role: Option<ColorRole>,
) -> impl IntoElement {
    let fg = c(t, role.unwrap_or(ColorRole::FgMuted));
    div()
        .flex_none()
        .h(px(18.))
        .px(px(6.))
        .rounded(px(999.))
        .border_1()
        .border_color(alpha(fg, 0.45))
        .text_color(fg)
        .text_size(px(10.5))
        .flex()
        .items_center()
        .child(text.into())
}

/// A card surface.
pub(crate) fn card(t: &'static Theme) -> gpui::Div {
    div()
        .flex()
        .flex_col()
        .gap(px(9.))
        .p(px(14.))
        .rounded(px(12.))
        .border_1()
        .border_color(c(t, ColorRole::BorderDefault))
        .bg(c(t, ColorRole::SurfaceRaised))
        .text_color(c(t, ColorRole::FgDefault))
        .text_size(px(13.))
}

/// Muted text.
pub(crate) fn muted(t: &'static Theme, s: impl Into<SharedString>) -> impl IntoElement {
    div()
        .text_color(c(t, ColorRole::FgMuted))
        .text_size(px(12.5))
        .child(s.into())
}

/// A coloured dot.
pub(crate) fn dot(col: Hsla) -> impl IntoElement {
    div().flex_none().size(px(7.)).rounded_full().bg(col)
}
