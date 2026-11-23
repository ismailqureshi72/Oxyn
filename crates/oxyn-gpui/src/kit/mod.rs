//! GPUI elements for the plan #28 parity components, drawn from the same
//! `oxyn_core::parity` models as the Dioxus views. Elements are `RenderOnce`; buttons report
//! through one [`Act`] callback with a short action id (`"confirm"`, `"cancel"`, …) so a host can
//! route them to the model without one closure per button.
#![cfg_attr(not(oxyn_pro), allow(dead_code, unused_imports))]

use std::rc::Rc;

use gpui::{
    App, ElementId, FontWeight, Hsla, IntoElement, ParentElement, SharedString, Styled, Window,
    div, prelude::*, px,
};
use oxyn_tokens::{ColorRole, Theme};

use crate::hsla;

#[cfg(oxyn_pro)]
#[rustfmt::skip]
mod agentscr;
#[cfg(oxyn_pro)]
#[rustfmt::skip]
mod agentsfid;
#[cfg(oxyn_pro)]
#[rustfmt::skip]
mod answers;
mod approvals;
#[cfg(oxyn_pro)]
#[rustfmt::skip]
mod apps;
#[cfg(oxyn_pro)]
#[rustfmt::skip]
mod arcscr;
#[cfg(oxyn_pro)]
#[rustfmt::skip]
mod arcscr2;
#[cfg(oxyn_pro)]
#[rustfmt::skip]
mod arcscr3;
#[cfg(oxyn_pro)]
#[rustfmt::skip]
mod arcscr4;
#[cfg(oxyn_pro)]
#[rustfmt::skip]
mod attachfid;
mod autonomyfid;
#[cfg(oxyn_pro)]
#[rustfmt::skip]
mod awayfid;
#[cfg(oxyn_pro)]
#[rustfmt::skip]
mod buildscr;
mod chatfid;
#[cfg(oxyn_pro)]
#[rustfmt::skip]
mod chatscr;
#[cfg(oxyn_pro)]
#[rustfmt::skip]
mod cmdboard;
#[cfg(oxyn_pro)]
#[rustfmt::skip]
mod cmdstudio;
mod collabfid;
mod costfid;
#[cfg(oxyn_pro)]
#[rustfmt::skip]
mod courier;
mod crewfid;
#[cfg(oxyn_pro)]
#[rustfmt::skip]
mod ctxefffid;
mod ctxmapfid;
#[cfg(oxyn_pro)]
#[rustfmt::skip]
mod ctxvisfid;
mod cursorsfid;
#[cfg(oxyn_pro)]
#[rustfmt::skip]
mod delegatefid;
#[cfg(oxyn_pro)]
#[rustfmt::skip]
mod difftlfid;
mod dtreefid;
#[cfg(oxyn_pro)]
#[rustfmt::skip]
mod efflistfid;
#[cfg(oxyn_pro)]
#[rustfmt::skip]
mod effpillfid;
mod fallbackfid;
mod fanfid;
mod fidelity;
mod fields;
#[cfg(oxyn_pro)]
#[rustfmt::skip]
mod fleetwin;
#[cfg(oxyn_pro)]
#[rustfmt::skip]
mod gooeyfid;
#[cfg(oxyn_pro)]
#[rustfmt::skip]
mod graphfid;
#[cfg(oxyn_pro)]
#[rustfmt::skip]
mod halochat;
#[cfg(oxyn_pro)]
#[rustfmt::skip]
mod handofffid;
#[cfg(oxyn_pro)]
#[rustfmt::skip]
mod hangar;
#[cfg(oxyn_pro)]
#[rustfmt::skip]
mod home;
mod ime;
mod judgefid;
#[cfg(oxyn_pro)]
#[rustfmt::skip]
mod lanesfid;
#[cfg(oxyn_pro)]
#[rustfmt::skip]
mod ledgerfid;
#[cfg(oxyn_pro)]
#[rustfmt::skip]
mod memoryfid;
#[cfg(oxyn_pro)]
#[rustfmt::skip]
mod merge;
#[cfg(oxyn_pro)]
#[rustfmt::skip]
mod modeleffid;
#[cfg(oxyn_pro)]
#[rustfmt::skip]
mod monitorfid;
#[cfg(oxyn_pro)]
#[rustfmt::skip]
mod nozzlefid;
#[cfg(oxyn_pro)]
#[rustfmt::skip]
mod onbscr;
#[cfg(oxyn_pro)]
#[rustfmt::skip]
mod opsscr;
#[cfg(oxyn_pro)]
#[rustfmt::skip]
mod parallelfid;
#[cfg(oxyn_pro)]
#[rustfmt::skip]
mod pilotpanel;
#[cfg(oxyn_pro)]
#[rustfmt::skip]
mod planfid;
#[cfg(oxyn_pro)]
#[rustfmt::skip]
mod previewdesk;
#[cfg(oxyn_pro)]
#[rustfmt::skip]
mod queuefid;
#[cfg(oxyn_pro)]
#[rustfmt::skip]
mod ratefid;
#[cfg(oxyn_pro)]
#[rustfmt::skip]
mod reasonfid;
#[cfg(oxyn_pro)]
#[rustfmt::skip]
mod recapfid;
#[cfg(oxyn_pro)]
#[rustfmt::skip]
mod repodesk;
mod review;
#[cfg(oxyn_pro)]
#[rustfmt::skip]
mod rootfid;
#[cfg(oxyn_pro)]
#[rustfmt::skip]
mod rulerfid;
mod run;
#[cfg(oxyn_pro)]
#[rustfmt::skip]
mod runningfid;
mod schedfid;
#[cfg(oxyn_pro)]
#[rustfmt::skip]
mod screens2fid;
#[cfg(oxyn_pro)]
#[rustfmt::skip]
mod screensfid;
#[cfg(oxyn_pro)]
#[rustfmt::skip]
mod screenview;
#[cfg(oxyn_pro)]
#[rustfmt::skip]
mod seqfid;
#[cfg(oxyn_pro)]
#[rustfmt::skip]
mod setscr;
#[cfg(oxyn_pro)]
#[rustfmt::skip]
mod setscr2;
#[cfg(oxyn_pro)]
#[rustfmt::skip]
mod stairsfid;
#[cfg(oxyn_pro)]
#[rustfmt::skip]
mod storagefid;
#[cfg(oxyn_pro)]
#[rustfmt::skip]
mod tabstack;
#[cfg(oxyn_pro)]
#[rustfmt::skip]
mod taskdeck;
mod text;
#[cfg(oxyn_pro)]
#[rustfmt::skip]
mod themefid;
#[cfg(oxyn_pro)]
#[rustfmt::skip]
mod threadbar;
mod toolmapfid;
#[cfg(oxyn_pro)]
#[rustfmt::skip]
mod tools;
mod tracefid;
#[cfg(oxyn_pro)]
#[rustfmt::skip]
mod workscr;
#[cfg(oxyn_pro)]
#[rustfmt::skip]
mod wscoder;

#[cfg(oxyn_pro)]
pub use agentscr::{ActivityEl, AgentThreadEl, InboxBoardEl, InboxScreenEl, MemoryEl, TasksEl};
#[cfg(oxyn_pro)]
pub use agentsfid::{FindingsFeedEl, TaskInboxEl};
#[cfg(oxyn_pro)]
pub use answers::{
    AnswerCardEl, CitedAnswerEl, MessageActionsEl, ThinkingPanelEl, ThinkingPillEl,
    WritingAssistantEl,
};
#[cfg(oxyn_pro)]
pub use approvals::{
    AgentApprovalsEl, ApprovalGatesEl, HoldConfirmEl, InlineConfirmEl, PermissionRequestEl,
};
pub use approvals::{ApprovalPagerEl, ConfirmPopoverEl};
#[cfg(oxyn_pro)]
pub use apps::{AgentApp, AgentAppView, Live, Own};
#[cfg(oxyn_pro)]
pub use arcscr::{
    ArcDrag, GanttTimelineEl, KanbanBoardEl, MetricsDashboardEl, ProjectBoardEl, TeamDirectoryEl,
};
#[cfg(oxyn_pro)]
pub use arcscr2::{
    HelpCenterEl, InboxTriageEl, MonthCalendarEl, SupportConversationEl, WeekCalendarEl,
};
#[cfg(oxyn_pro)]
pub use arcscr3::{InvoiceDeskEl, ProductDetailEl, ProductListEl, SignInSplitEl, TradingDeskEl};
#[cfg(oxyn_pro)]
pub use arcscr4::{
    AdminOverviewEl, AgentToolCallsEl, ApiKeysWebhooksEl, AuditLogEl, BillingUsageEl,
    MembersRolesEl, StatusBoardEl,
};
#[cfg(oxyn_pro)]
pub use attachfid::{AttachmentChipsEl, DropLanesEl, PolaroidStackEl, UploadDropzoneEl};
pub use autonomyfid::AutonomyLevelsEl;
#[cfg(oxyn_pro)]
pub use awayfid::ReturnBriefEl;
#[cfg(oxyn_pro)]
pub use buildscr::EvalsEl;
pub use chatfid::{AgentRunPanelEl, ModelPickerMenuEl};
#[cfg(oxyn_pro)]
pub use chatscr::{
    ChatAgentsEl, ChatOverviewEl, ChatProgressEl, ChatRailEl, ChatRunEl, ChatStartEl,
};
#[cfg(oxyn_pro)]
pub use cmdboard::CommandBoardEl;
#[cfg(oxyn_pro)]
pub use cmdstudio::{CommandCenterEl, SessionStudioEl};
pub use collabfid::CrewChatEl;
pub use costfid::SpendBreakdownEl;
#[cfg(oxyn_pro)]
pub use courier::CourierEl;
pub use crewfid::CrewTreeEl;
#[cfg(oxyn_pro)]
pub use ctxefffid::ContextEffortMenuEl;
pub use ctxmapfid::ContextTreemapEl;
#[cfg(oxyn_pro)]
pub use ctxvisfid::{ContextBarEl, ContextBreakdownEl, ContextStackEl, ContextTideEl};
pub use cursorsfid::PresenceCursorsEl;
#[cfg(oxyn_pro)]
pub use delegatefid::AssignBoardEl;
#[cfg(oxyn_pro)]
pub use difftlfid::DiffTimelineRunEl;
pub use dtreefid::DecisionTreeEl;
#[cfg(oxyn_pro)]
pub use efflistfid::EffortListEl;
#[cfg(oxyn_pro)]
pub use effpillfid::EffortPillEl;
pub use fallbackfid::FallbackChainEl;
pub use fanfid::TaskGridEl;
#[cfg(oxyn_pro)]
pub use fidelity::{AgentCommentThreadEl, CommentsPanelEl, ThreadMsg};
pub use fidelity::{ChatComposerEl, file_badge};
#[cfg(oxyn_pro)]
pub use fields::{
    AiComposerEl, AvatarUploadEl, ContextFieldEl, ContextGridEl, GaugeEl, ModelPillEl,
    NumberFieldEl, RichTextEl, RubberSliderEl, ScrubInputEl, SplitBarEl, UnfoldSelectEl,
    UsageMeterEl,
};
pub use fields::{FileDropzoneEl, ModelMenuEl, Pick, RunControlsEl};
#[cfg(oxyn_pro)]
pub use fleetwin::FleetWindowEl;
#[cfg(oxyn_pro)]
pub use gooeyfid::{
    GooeyAttachMenuEl, GooeyComposerEl, GooeyModelMenuEl, LiquidDropsEl, LiquidTabsEl,
};
#[cfg(oxyn_pro)]
pub use graphfid::ToolNetworkEl;
#[cfg(oxyn_pro)]
pub use halochat::HaloChatEl;
#[cfg(oxyn_pro)]
pub use handofffid::RelayTrailEl;
#[cfg(oxyn_pro)]
pub use hangar::HangarEl;
#[cfg(oxyn_pro)]
pub use home::{
    ActivityHeatmapEl, ActivityHomeEl, AttachmentStripEl, DocGridEl, ExperimentGridEl,
    PollingCostEl, RunModeMenuEl, ThreadListEl, TokenUsageChartEl,
};
pub use ime::{ImeEvent, ImeInput, bind_ime_keys};
pub use judgefid::ModelKnockoutEl;
#[cfg(oxyn_pro)]
pub use lanesfid::LiveLanesEl;
#[cfg(oxyn_pro)]
pub use ledgerfid::ChangeLedgerEl;
#[cfg(oxyn_pro)]
pub use memoryfid::MemoryPanelEl;
#[cfg(oxyn_pro)]
pub use merge::{ConflictRadarEl, MergeAct, MergeConflictEl, RadarAct};
#[cfg(oxyn_pro)]
pub use modeleffid::ModelEffortMenuEl;
#[cfg(oxyn_pro)]
pub use monitorfid::RunMonitorEl;
#[cfg(oxyn_pro)]
pub use nozzlefid::NozzleSliderEl;
#[cfg(oxyn_pro)]
pub use onbscr::{OnboardingBrandEl, OnboardingReadyEl, OnboardingToneEl, OnboardingWebsiteEl};
#[cfg(oxyn_pro)]
pub use opsscr::{AgentsEl, PermissionsEl, RunDetailEl, SchedulesEl, SkillsEl, UsageEl};
#[cfg(oxyn_pro)]
pub use parallelfid::AttemptRaceEl;
#[cfg(oxyn_pro)]
pub use pilotpanel::PilotPanelEl;
#[cfg(oxyn_pro)]
pub use planfid::PlanVsActualEl;
#[cfg(oxyn_pro)]
pub use previewdesk::PreviewDeskEl;
#[cfg(oxyn_pro)]
pub use queuefid::JobQueueEl;
#[cfg(oxyn_pro)]
pub use ratefid::LoadPlannerEl;
#[cfg(oxyn_pro)]
pub use reasonfid::ReasoningPanelEl;
#[cfg(oxyn_pro)]
pub use recapfid::OvernightRecapEl;
#[cfg(oxyn_pro)]
pub use repodesk::RepoDeskEl;
#[cfg(oxyn_pro)]
pub use review::{
    AiInlineEditEl, ChangeSummaryEl, CommentThreadEl, DiffTableEl, ImageCompareEl, ReviewCardEl,
    ReviewCenterEl,
};
pub use review::{CodeSuggestionEl, DiffTimelineEl, DiffViewerEl, HunkAct, IdAct, InlineDiffEl};
#[cfg(oxyn_pro)]
pub use rootfid::RootCauseTreeEl;
#[cfg(oxyn_pro)]
pub use rulerfid::EffortRulerEl;
#[cfg(oxyn_pro)]
pub use run::{
    ActivityFeedEl, AgentPlanEl, AgentRunEl, AgentStatusEl, AgentTreeEl, BreakdownEl, ChecklistEl,
    CountdownEl, CounterEl, FleetEl, OrchestratorEl, ReadinessEl, TaskRowsEl, TreeTableEl,
};
pub use run::{AgentTrailEl, FlowStepperEl};
#[cfg(oxyn_pro)]
pub use runningfid::ActivityPillEl;
pub use schedfid::ScheduledRunsEl;
#[cfg(oxyn_pro)]
pub use screens2fid::OrchestrationEl;
#[cfg(oxyn_pro)]
pub use screensfid::{
    AccountMenuEl, ScreenRailEl, ScreenShellEl, ScreenStageEl, SettingsEl, WorkflowEl,
};
#[cfg(oxyn_pro)]
pub use screenview::{ScreenView, screen_element};
#[cfg(oxyn_pro)]
pub use seqfid::SequenceDiagramEl;
#[cfg(oxyn_pro)]
pub use setscr::{
    BillingSettingsEl, GeneralSettingsEl, MembersSettingsEl, ModelsSettingsEl,
    NotificationsSettingsEl, OnboardingSettingsEl, PreferencesSettingsEl, PrivacySettingsEl,
    ProfileSettingsEl, SecuritySettingsEl,
};
#[cfg(oxyn_pro)]
pub use stairsfid::EffortStairsEl;
#[cfg(oxyn_pro)]
pub use storagefid::{StorageCleanupEl, StorageForecastEl, StorageMeterEl, StorageWaffleEl};
#[cfg(oxyn_pro)]
pub use tabstack::TabstackEl;
#[cfg(oxyn_pro)]
pub use taskdeck::{
    TaskdeckBoardEl, TaskdeckPluginsEl, TaskdeckScheduledEl, TaskdeckSitesEl, TaskdeckStartEl,
    TaskdeckTaskEl,
};
#[cfg(oxyn_pro)]
pub use text::ThinkingLoaderEl;
#[cfg(oxyn_pro)]
pub use text::{LoadingStateEl, ShimmerEl, SlotTextEl};
#[cfg(oxyn_pro)]
pub use themefid::{
    AccentDensityEl, AppIconPickerEl, BackgroundPickerEl, DisplayComfortEl, LightPlaygroundEl,
    ThemeGridEl, ThemeScheduleEl, ThemeSplitEl, TypeScaleEl,
};
#[cfg(oxyn_pro)]
pub use threadbar::ThreadbarEl;
pub use toolmapfid::ToolReachEl;
#[cfg(oxyn_pro)]
pub use tools::{
    ExpandableCardEl, JsonViewerEl, LinkUnfurlEl, ToolCallCardEl, ToolCallEl, ToolChipsEl,
};
pub use tracefid::TraceWaterfallEl;
#[cfg(oxyn_pro)]
pub use workscr::{AnalyticsEl, CrmEl, FolderDetailEl, FoldersEl, IntegrationsEl, KanbanEl};
#[cfg(oxyn_pro)]
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

// Oxyn Pro part of this module (private repo ismailqureshi72/oxyn-pro; overlaid by CI).
#[cfg(oxyn_pro)]
#[rustfmt::skip]
mod pro;
#[cfg(oxyn_pro)]
#[allow(unused_imports)]
pub use pro::*;
