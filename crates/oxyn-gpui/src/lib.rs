//! GPUI components for Oxyn (primary renderer).
//!
//! Components take `&AgentState` plus props and read time from the window's shared
//! [`MotionClock`]; the host keeps the clock advanced and requests frames while
//! [`MotionClock::next_frame_in`] says so.

mod aikit;
mod blocks;
mod brand;
mod chat;
mod checkpoints;
mod composer;
mod composer_parts;
mod forms;
mod glyph_loader;
mod hitl;
mod kit;
mod navigation;
mod primitives;
mod run;
mod shell;
mod status_indicator;
mod status_text;
mod surfaces;
mod xpost;

pub use aikit::{AiKitState, AiKitView, AiVariant};
pub use blocks::{BlockVariant, BlocksState, BlocksView};
pub use brand::{BrandAssets, brand_icon, brand_mark, brand_path, ink, tone_color};
pub use chat::{
    AnswerEl, ChatThreadEl, FeedbackEl, MarkdownEl, MessageEl, ReasoningEl, SourceCardsEl,
    StartersEl, TraceEl,
};
pub use checkpoints::{CheckpointsView, CpState, CpVariant};
pub use composer::{Composer, ComposerEvent};
pub use composer_parts::{
    DictationEl, MentionListEl, ModeSwitchEl, PermissionEl, QueueStackEl, SteeringEl, ToolsMenuEl,
    VoiceRecorderEl,
};
pub use forms::{
    ApprovalQuestionsEl, CalendarEl, ComboboxEl, DatePickerEl, DateWheelEl, DecisionCardEl,
    FilterMenuEl, InlineEditEl, OtpEl, PasswordFieldEl, PasswordStrengthEl, PhoneInputEl,
    RangePickerEl, SmartFillEl, StepperEl, TagInputEl, TimeDialEl, TimePickerEl,
};
pub use glyph_loader::GlyphLoaderView;
pub use hitl::{ApprovalCardEl, CheckpointLanesEl};
pub use kit::AccentDensityEl;
pub use kit::AccountMenuEl;
pub use kit::ActivityPillEl;
pub use kit::AgentThreadEl;
pub use kit::AnalyticsEl;
pub use kit::AppIconPickerEl;
pub use kit::AttachmentChipsEl;
pub use kit::AttemptRaceEl;
pub use kit::BackgroundPickerEl;
pub use kit::ChatAgentsEl;
pub use kit::ChatOverviewEl;
pub use kit::ChatProgressEl;
pub use kit::ChatRailEl;
pub use kit::ChatRunEl;
pub use kit::ChatStartEl;
pub use kit::ContextBarEl;
pub use kit::ContextBreakdownEl;
pub use kit::ContextEffortMenuEl;
pub use kit::ContextStackEl;
pub use kit::ContextTideEl;
pub use kit::CrmEl;
pub use kit::DiffTimelineRunEl;
pub use kit::DisplayComfortEl;
pub use kit::DropLanesEl;
pub use kit::EffortListEl;
pub use kit::EffortPillEl;
pub use kit::EffortRulerEl;
pub use kit::EffortStairsEl;
pub use kit::EvalsEl;
pub use kit::FolderDetailEl;
pub use kit::FoldersEl;
pub use kit::GooeyAttachMenuEl;
pub use kit::GooeyComposerEl;
pub use kit::GooeyModelMenuEl;
pub use kit::IntegrationsEl;
pub use kit::KanbanEl;
pub use kit::LightPlaygroundEl;
pub use kit::LiquidDropsEl;
pub use kit::LiquidTabsEl;
pub use kit::LoadPlannerEl;
pub use kit::MemoryPanelEl;
pub use kit::ModelEffortMenuEl;
pub use kit::ModelKnockoutEl;
pub use kit::NozzleSliderEl;
pub use kit::OrchestrationEl;
pub use kit::PlanVsActualEl;
pub use kit::PolaroidStackEl;
pub use kit::ReasoningPanelEl;
pub use kit::RootCauseTreeEl;
pub use kit::ScheduledRunsEl;
pub use kit::ScreenRailEl;
pub use kit::ScreenShellEl;
pub use kit::ScreenStageEl;
pub use kit::SettingsEl;
pub use kit::StorageCleanupEl;
pub use kit::StorageForecastEl;
pub use kit::StorageMeterEl;
pub use kit::StorageWaffleEl;
pub use kit::TaskGridEl;
pub use kit::ThemeGridEl;
pub use kit::ThemeScheduleEl;
pub use kit::ThemeSplitEl;
pub use kit::ToolReachEl;
pub use kit::TraceWaterfallEl;
pub use kit::TypeScaleEl;
pub use kit::UploadDropzoneEl;
pub use kit::WorkflowEl;
pub use kit::{
    Act, ActivityFeedEl, AgentApprovalsEl, AgentPlanEl, AgentRunEl, AgentStatusEl, AgentTrailEl,
    AgentTreeEl, ApprovalGatesEl, ApprovalPagerEl, BreakdownEl, ChecklistEl, ConfirmPopoverEl,
    CountdownEl, CounterEl, FleetEl, FlowStepperEl, HoldConfirmEl, InlineConfirmEl, OrchestratorEl,
    PermissionRequestEl, ReadinessEl, TaskRowsEl, TreeTableEl,
};
pub use kit::{ActivityEl, InboxBoardEl, InboxScreenEl, MemoryEl, TasksEl};
pub use kit::{
    ActivityHeatmapEl, ActivityHomeEl, AttachmentStripEl, DocGridEl, ExperimentGridEl,
    PollingCostEl, RunModeMenuEl, ThreadListEl, TokenUsageChartEl,
};
pub use kit::{
    AdminOverviewEl, AgentToolCallsEl, ApiKeysWebhooksEl, AuditLogEl, BillingUsageEl,
    MembersRolesEl, StatusBoardEl,
};
pub use kit::{AgentApp, AgentAppView, Live, Own, ScreenView, screen_element};
pub use kit::{
    AgentCommentThreadEl, AgentRunPanelEl, CommentsPanelEl, ModelPickerMenuEl, ThreadMsg,
    file_badge,
};
pub use kit::{AgentsEl, PermissionsEl, RunDetailEl, SchedulesEl, SkillsEl, UsageEl};
pub use kit::{
    AiComposerEl, AvatarUploadEl, ChatComposerEl, ContextFieldEl, ContextGridEl, FileDropzoneEl,
    GaugeEl, ModelMenuEl, ModelPillEl, NumberFieldEl, Pick, RichTextEl, RubberSliderEl,
    RunControlsEl, ScrubInputEl, SplitBarEl, UnfoldSelectEl, UsageMeterEl,
};
pub use kit::{
    AiInlineEditEl, ChangeSummaryEl, CodeSuggestionEl, CommentThreadEl, DiffTableEl,
    DiffTimelineEl, DiffViewerEl, HunkAct, IdAct, ImageCompareEl, InlineDiffEl, ReviewCardEl,
    ReviewCenterEl,
};
pub use kit::{
    AnswerCardEl, CitedAnswerEl, MessageActionsEl, ThinkingPanelEl, ThinkingPillEl,
    WritingAssistantEl,
};
pub use kit::{
    ArcDrag, GanttTimelineEl, KanbanBoardEl, MetricsDashboardEl, ProjectBoardEl, TeamDirectoryEl,
};
pub use kit::{
    AssignBoardEl, AutonomyLevelsEl, ChangeLedgerEl, ContextTreemapEl, CrewChatEl, CrewTreeEl,
    DecisionTreeEl, FallbackChainEl, FindingsFeedEl, JobQueueEl, LiveLanesEl, OvernightRecapEl,
    PresenceCursorsEl, RelayTrailEl, ReturnBriefEl, RunMonitorEl, SequenceDiagramEl,
    SpendBreakdownEl, TaskInboxEl, ToolNetworkEl,
};
pub use kit::{
    BillingSettingsEl, GeneralSettingsEl, MembersSettingsEl, ModelsSettingsEl,
    NotificationsSettingsEl, OnboardingSettingsEl, PreferencesSettingsEl, PrivacySettingsEl,
    ProfileSettingsEl, SecuritySettingsEl,
};
pub use kit::{CommandBoardEl, FleetWindowEl, WorkspaceCoderEl};
pub use kit::{CommandCenterEl, SessionStudioEl};
pub use kit::{ConflictRadarEl, MergeAct, MergeConflictEl, RadarAct};
pub use kit::{CourierEl, HaloChatEl, HangarEl, PilotPanelEl};
pub use kit::{
    ExpandableCardEl, JsonViewerEl, LinkUnfurlEl, ToolCallCardEl, ToolCallEl, ToolChipsEl,
};
pub use kit::{
    HelpCenterEl, InboxTriageEl, MonthCalendarEl, SupportConversationEl, WeekCalendarEl,
};
pub use kit::{ImeEvent, ImeInput, bind_ime_keys};
pub use kit::{InvoiceDeskEl, ProductDetailEl, ProductListEl, SignInSplitEl, TradingDeskEl};
pub use kit::{LoadingStateEl, ShimmerEl, SlotTextEl, ThinkingLoaderEl};
pub use kit::{OnboardingBrandEl, OnboardingReadyEl, OnboardingToneEl, OnboardingWebsiteEl};
pub use kit::{PreviewDeskEl, RepoDeskEl, TabstackEl, ThreadbarEl};
pub use kit::{
    TaskdeckBoardEl, TaskdeckPluginsEl, TaskdeckScheduledEl, TaskdeckSitesEl, TaskdeckStartEl,
    TaskdeckTaskEl,
};
pub use navigation::{
    AccordionEl, BreadcrumbEl, ChipGroupEl, CodeBlockEl, CopyButtonEl, DockEl, ExpandableTextEl,
    HoverCardEl, NotificationCenterEl, PageHeaderEl, RadioCardsEl, RatingEl, ReactionsEl,
    ReorderListEl, ResizableEl, TabsEl, TeamSwitcherEl, TocProgressEl, ToggleGroupEl, TreeViewEl,
    UserMenuEl,
};
pub use primitives::{
    ActionButtonEl, AvatarEl, BadgeEl, BadgeSize, ButtonEl, ButtonGroupEl, CheckboxEl, ChipEl,
    EmptyStateEl, EntityChipEl, ExpandingGroupEl, IconButtonEl, KbdEl, MenuEl, MonogramEl,
    NumberTickerEl, PopoverEl, RadioGroupEl, ReactionBarEl, SegmentedEl, SelectEl, SeparatorEl,
    SliderEl, SpinnerEl, SplitButtonEl, SwitchEl, TextFieldEl, TooltipEl, ValuePillEl,
};
pub use run::{SubagentTreeEl, TodoChecklist, ToolCallRowEl};
pub use shell::{
    AiPanelEl, AppShellEl, ChatShellEl, HarnessEl, InboxEl, RailShellEl, SidebarEl, SidebarNavEl,
    TermTabsEl, WorkspaceSidebarEl,
};
pub use status_indicator::{IndicatorSize, IndicatorVariant, StatusIndicator};
pub use status_text::{RevealText, ShimmerText, TimerText, VerbCyclerView};
pub use surfaces::{
    AiCommandEl, AiSearchBarEl, AlertEl, BottomSheetEl, CardEl, CommandPaletteEl, ControlCenterEl,
    DialogEl, ExpandingSearchEl, NotificationStackEl, PaginationEl, ScrollAreaEl, SearchFieldEl,
    SheetEl, SheetStackEl, ShortcutRecorderEl, SkeletonEl, SkeletonRevealEl, ToasterEl,
    dialog_size,
};
pub use xpost::{XPostState, XPostVariant, XPostView};

use gpui::{Hsla, Rgba};
use oxyn_tokens::Color;

#[doc(no_inline)]
pub use oxyn_core::MotionClock;

/// Converts a token colour to GPUI's colour type.
pub fn hsla(c: Color) -> Hsla {
    Rgba {
        r: f32::from(c.r) / 255.0,
        g: f32::from(c.g) / 255.0,
        b: f32::from(c.b) / 255.0,
        a: c.a,
    }
    .into()
}

/// Linear mix of two token colours in sRGB (`t` in `[0, 1]`).
pub fn mix(a: Color, b: Color, t: f32) -> Color {
    let t = t.clamp(0.0, 1.0);
    let ch = |x: u8, y: u8| (f32::from(x) + (f32::from(y) - f32::from(x)) * t).round() as u8;
    Color::rgba(
        ch(a.r, b.r),
        ch(a.g, b.g),
        ch(a.b, b.b),
        a.a + (b.a - a.a) * t,
    )
}

// Old names of elements renamed in the naming pass (docs/renames.md), kept so existing code
// keeps building.
/// Renamed: use [`CrewTreeEl`].
#[deprecated(note = "renamed to `CrewTreeEl`")]
pub type AgentCrewEl = CrewTreeEl;
/// Renamed: use [`PresenceCursorsEl`].
#[deprecated(note = "renamed to `PresenceCursorsEl`")]
pub type AgentCursorsEl = PresenceCursorsEl;
/// Renamed: use [`FindingsFeedEl`].
#[deprecated(note = "renamed to `FindingsFeedEl`")]
pub type AgentFindingsEl = FindingsFeedEl;
/// Renamed: use [`ToolNetworkEl`].
#[deprecated(note = "renamed to `ToolNetworkEl`")]
pub type AgentGraphEl = ToolNetworkEl;
/// Renamed: use [`RelayTrailEl`].
#[deprecated(note = "renamed to `RelayTrailEl`")]
pub type AgentHandoffEl = RelayTrailEl;
/// Renamed: use [`TaskInboxEl`].
#[deprecated(note = "renamed to `TaskInboxEl`")]
pub type AgentInboxEl = TaskInboxEl;
/// Renamed: use [`LiveLanesEl`].
#[deprecated(note = "renamed to `LiveLanesEl`")]
pub type AgentLanesEl = LiveLanesEl;
/// Renamed: use [`ChangeLedgerEl`].
#[deprecated(note = "renamed to `ChangeLedgerEl`")]
pub type AgentLedgerEl = ChangeLedgerEl;
/// Renamed: use [`RunMonitorEl`].
#[deprecated(note = "renamed to `RunMonitorEl`")]
pub type AgentMonitorEl = RunMonitorEl;
/// Renamed: use [`JobQueueEl`].
#[deprecated(note = "renamed to `JobQueueEl`")]
pub type AgentQueueEl = JobQueueEl;
/// Renamed: use [`OvernightRecapEl`].
#[deprecated(note = "renamed to `OvernightRecapEl`")]
pub type AgentRecapEl = OvernightRecapEl;
/// Renamed: use [`SequenceDiagramEl`].
#[deprecated(note = "renamed to `SequenceDiagramEl`")]
pub type AgentSequenceEl = SequenceDiagramEl;
/// Renamed: use [`AutonomyLevelsEl`].
#[deprecated(note = "renamed to `AutonomyLevelsEl`")]
pub type AutonomyDialEl = AutonomyLevelsEl;
/// Renamed: use [`ReturnBriefEl`].
#[deprecated(note = "renamed to `ReturnBriefEl`")]
pub type AwaySummaryEl = ReturnBriefEl;
/// Renamed: use [`CrewChatEl`].
#[deprecated(note = "renamed to `CrewChatEl`")]
pub type CollabStreamEl = CrewChatEl;
/// Renamed: use [`ContextTreemapEl`].
#[deprecated(note = "renamed to `ContextTreemapEl`")]
pub type ContextMapEl = ContextTreemapEl;
/// Renamed: use [`SpendBreakdownEl`].
#[deprecated(note = "renamed to `SpendBreakdownEl`")]
pub type CostWaterfallEl = SpendBreakdownEl;
/// Renamed: use [`AttemptRaceEl`].
#[deprecated(note = "renamed to `AttemptRaceEl`")]
pub type ParallelAttemptsEl = AttemptRaceEl;
/// Renamed: use [`ModelKnockoutEl`].
#[deprecated(note = "renamed to `ModelKnockoutEl`")]
pub type JudgeBracketEl = ModelKnockoutEl;
/// Renamed: use [`ActivityPillEl`].
#[deprecated(note = "renamed to `ActivityPillEl`")]
pub type RunningNowEl = ActivityPillEl;
/// Renamed: use [`ToolReachEl`].
#[deprecated(note = "renamed to `ToolReachEl`")]
pub type ToolAccessMapEl = ToolReachEl;
/// Renamed: use [`LoadPlannerEl`].
#[deprecated(note = "renamed to `LoadPlannerEl`")]
pub type RateLimitPlannerEl = LoadPlannerEl;
/// Renamed: use [`TaskGridEl`].
#[deprecated(note = "renamed to `TaskGridEl`")]
pub type FanoutGridEl = TaskGridEl;
/// Renamed: use [`AssignBoardEl`].
#[deprecated(note = "renamed to `AssignBoardEl`")]
pub type DelegateBoardEl = AssignBoardEl;
/// Renamed: use [`RubberSliderEl`].
#[deprecated(note = "renamed to `RubberSliderEl`")]
pub type ElasticSliderEl = RubberSliderEl;
/// Renamed: use [`UnfoldSelectEl`].
#[deprecated(note = "renamed to `UnfoldSelectEl`")]
pub type MorphSelectEl = UnfoldSelectEl;
/// Renamed: use [`InlineConfirmEl`].
#[deprecated(note = "renamed to `InlineConfirmEl`")]
pub type ConfirmMorphEl = InlineConfirmEl;
/// Renamed: use [`SkeletonRevealEl`].
#[deprecated(note = "renamed to `SkeletonRevealEl`")]
pub type SkeletonMorphEl = SkeletonRevealEl;
/// Renamed: use [`DateWheelEl`].
#[deprecated(note = "renamed to `DateWheelEl`")]
pub type DateReelEl = DateWheelEl;
/// Renamed: use [`TradingDeskEl`].
#[deprecated(note = "renamed to `TradingDeskEl`")]
pub type MarketTerminalEl = TradingDeskEl;
/// Renamed: use [`InvoiceDeskEl`].
#[deprecated(note = "renamed to `InvoiceDeskEl`")]
pub type InvoiceStudioEl = InvoiceDeskEl;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn colour_conversion() {
        let h = hsla(Color::rgb(255, 90, 31));
        let back: Rgba = h.into();
        assert!((back.r - 1.0).abs() < 1e-3 && (back.g - 90.0 / 255.0).abs() < 1e-3);
        assert_eq!(
            mix(Color::rgb(0, 0, 0), Color::rgb(200, 100, 50), 0.5),
            Color::rgb(100, 50, 25)
        );
    }
}
