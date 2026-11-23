//! GPUI components for Oxyn (primary renderer).
//!
//! Components take `&AgentState` plus props and read time from the window's shared
//! [`MotionClock`]; the host keeps the clock advanced and requests frames while
//! [`MotionClock::next_frame_in`] says so.

#[cfg(oxyn_pro)]
#[rustfmt::skip]
mod aikit;
#[cfg(oxyn_pro)]
#[rustfmt::skip]
mod blocks;
mod brand;
mod chat;
#[cfg(oxyn_pro)]
#[rustfmt::skip]
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
#[cfg(oxyn_pro)]
#[rustfmt::skip]
mod status_indicator;
mod status_text;
mod surfaces;
#[cfg(oxyn_pro)]
#[rustfmt::skip]
mod xpost;

#[cfg(oxyn_pro)]
pub use aikit::{AiKitState, AiKitView, AiVariant};
#[cfg(oxyn_pro)]
pub use blocks::{BlockVariant, BlocksState, BlocksView};
#[cfg(oxyn_pro)]
pub use brand::tone_color;
pub use brand::{BrandAssets, brand_icon, brand_mark, brand_path, ink};
#[cfg(oxyn_pro)]
pub use chat::AnswerEl;
pub use chat::{ChatThreadEl, FeedbackEl, MarkdownEl, MessageEl, ReasoningEl};
#[cfg(oxyn_pro)]
pub use chat::{SourceCardsEl, StartersEl, TraceEl};
#[cfg(oxyn_pro)]
pub use checkpoints::{CheckpointsView, CpState, CpVariant};
pub use composer::{Composer, ComposerEvent};
#[cfg(oxyn_pro)]
pub use composer_parts::{
    DictationEl, MentionListEl, ModeSwitchEl, PermissionEl, SteeringEl, ToolsMenuEl,
};
pub use composer_parts::{QueueStackEl, VoiceRecorderEl};
#[cfg(oxyn_pro)]
pub use forms::{
    ApprovalQuestionsEl, DatePickerEl, DecisionCardEl, FilterMenuEl, PasswordFieldEl,
    RangePickerEl, SmartFillEl, StepperEl, TimeDialEl, TimePickerEl,
};
pub use forms::{
    CalendarEl, ComboboxEl, DateWheelEl, InlineEditEl, OtpEl, PasswordStrengthEl, PhoneInputEl,
    TagInputEl,
};
pub use glyph_loader::GlyphLoaderView;
pub use hitl::{ApprovalCardEl, CheckpointLanesEl};
#[cfg(oxyn_pro)]
pub use kit::AccentDensityEl;
#[cfg(oxyn_pro)]
pub use kit::AccountMenuEl;
#[cfg(oxyn_pro)]
pub use kit::ActivityPillEl;
#[cfg(oxyn_pro)]
pub use kit::AgentThreadEl;
#[cfg(oxyn_pro)]
pub use kit::AnalyticsEl;
#[cfg(oxyn_pro)]
pub use kit::AppIconPickerEl;
#[cfg(oxyn_pro)]
pub use kit::AttachmentChipsEl;
#[cfg(oxyn_pro)]
pub use kit::AttemptRaceEl;
#[cfg(oxyn_pro)]
pub use kit::BackgroundPickerEl;
#[cfg(oxyn_pro)]
pub use kit::ChatAgentsEl;
#[cfg(oxyn_pro)]
pub use kit::ChatOverviewEl;
#[cfg(oxyn_pro)]
pub use kit::ChatProgressEl;
#[cfg(oxyn_pro)]
pub use kit::ChatRailEl;
#[cfg(oxyn_pro)]
pub use kit::ChatRunEl;
#[cfg(oxyn_pro)]
pub use kit::ChatStartEl;
#[cfg(oxyn_pro)]
pub use kit::ContextBarEl;
#[cfg(oxyn_pro)]
pub use kit::ContextBreakdownEl;
#[cfg(oxyn_pro)]
pub use kit::ContextEffortMenuEl;
#[cfg(oxyn_pro)]
pub use kit::ContextStackEl;
#[cfg(oxyn_pro)]
pub use kit::ContextTideEl;
#[cfg(oxyn_pro)]
pub use kit::CrmEl;
#[cfg(oxyn_pro)]
pub use kit::DiffTimelineRunEl;
#[cfg(oxyn_pro)]
pub use kit::DisplayComfortEl;
#[cfg(oxyn_pro)]
pub use kit::DropLanesEl;
#[cfg(oxyn_pro)]
pub use kit::EffortListEl;
#[cfg(oxyn_pro)]
pub use kit::EffortPillEl;
#[cfg(oxyn_pro)]
pub use kit::EffortRulerEl;
#[cfg(oxyn_pro)]
pub use kit::EffortStairsEl;
#[cfg(oxyn_pro)]
pub use kit::EvalsEl;
#[cfg(oxyn_pro)]
pub use kit::FolderDetailEl;
#[cfg(oxyn_pro)]
pub use kit::FoldersEl;
#[cfg(oxyn_pro)]
pub use kit::GooeyAttachMenuEl;
#[cfg(oxyn_pro)]
pub use kit::GooeyComposerEl;
#[cfg(oxyn_pro)]
pub use kit::GooeyModelMenuEl;
#[cfg(oxyn_pro)]
pub use kit::IntegrationsEl;
#[cfg(oxyn_pro)]
pub use kit::KanbanEl;
#[cfg(oxyn_pro)]
pub use kit::LightPlaygroundEl;
#[cfg(oxyn_pro)]
pub use kit::LiquidDropsEl;
#[cfg(oxyn_pro)]
pub use kit::LiquidTabsEl;
#[cfg(oxyn_pro)]
pub use kit::LoadPlannerEl;
#[cfg(oxyn_pro)]
pub use kit::MemoryPanelEl;
#[cfg(oxyn_pro)]
pub use kit::ModelEffortMenuEl;
pub use kit::ModelKnockoutEl;
#[cfg(oxyn_pro)]
pub use kit::NozzleSliderEl;
#[cfg(oxyn_pro)]
pub use kit::OrchestrationEl;
#[cfg(oxyn_pro)]
pub use kit::PlanVsActualEl;
#[cfg(oxyn_pro)]
pub use kit::PolaroidStackEl;
#[cfg(oxyn_pro)]
pub use kit::ReasoningPanelEl;
#[cfg(oxyn_pro)]
pub use kit::RootCauseTreeEl;
pub use kit::ScheduledRunsEl;
#[cfg(oxyn_pro)]
pub use kit::ScreenRailEl;
#[cfg(oxyn_pro)]
pub use kit::ScreenShellEl;
#[cfg(oxyn_pro)]
pub use kit::ScreenStageEl;
#[cfg(oxyn_pro)]
pub use kit::SettingsEl;
#[cfg(oxyn_pro)]
pub use kit::StorageCleanupEl;
#[cfg(oxyn_pro)]
pub use kit::StorageForecastEl;
#[cfg(oxyn_pro)]
pub use kit::StorageMeterEl;
#[cfg(oxyn_pro)]
pub use kit::StorageWaffleEl;
pub use kit::TaskGridEl;
#[cfg(oxyn_pro)]
pub use kit::ThemeGridEl;
#[cfg(oxyn_pro)]
pub use kit::ThemeScheduleEl;
#[cfg(oxyn_pro)]
pub use kit::ThemeSplitEl;
#[cfg(oxyn_pro)]
pub use kit::ThinkingLoaderEl;
pub use kit::ToolReachEl;
pub use kit::TraceWaterfallEl;
#[cfg(oxyn_pro)]
pub use kit::TypeScaleEl;
#[cfg(oxyn_pro)]
pub use kit::UploadDropzoneEl;
#[cfg(oxyn_pro)]
pub use kit::WorkflowEl;
pub use kit::{Act, AgentTrailEl, ApprovalPagerEl, ConfirmPopoverEl, FlowStepperEl};
#[cfg(oxyn_pro)]
pub use kit::{ActivityEl, InboxBoardEl, InboxScreenEl, MemoryEl, TasksEl};
#[cfg(oxyn_pro)]
pub use kit::{
    ActivityFeedEl, AgentApprovalsEl, AgentPlanEl, AgentRunEl, AgentStatusEl, AgentTreeEl,
    ApprovalGatesEl, BreakdownEl, ChecklistEl, CountdownEl, CounterEl, FleetEl, HoldConfirmEl,
    InlineConfirmEl, OrchestratorEl, PermissionRequestEl, ReadinessEl, TaskRowsEl, TreeTableEl,
};
#[cfg(oxyn_pro)]
pub use kit::{
    ActivityHeatmapEl, ActivityHomeEl, AttachmentStripEl, DocGridEl, ExperimentGridEl,
    PollingCostEl, RunModeMenuEl, ThreadListEl, TokenUsageChartEl,
};
#[cfg(oxyn_pro)]
pub use kit::{
    AdminOverviewEl, AgentToolCallsEl, ApiKeysWebhooksEl, AuditLogEl, BillingUsageEl,
    MembersRolesEl, StatusBoardEl,
};
#[cfg(oxyn_pro)]
pub use kit::{AgentApp, AgentAppView, Live, Own, ScreenView, screen_element};
#[cfg(oxyn_pro)]
pub use kit::{AgentCommentThreadEl, CommentsPanelEl, ThreadMsg};
pub use kit::{AgentRunPanelEl, ModelPickerMenuEl, file_badge};
#[cfg(oxyn_pro)]
pub use kit::{AgentsEl, PermissionsEl, RunDetailEl, SchedulesEl, SkillsEl, UsageEl};
#[cfg(oxyn_pro)]
pub use kit::{
    AiComposerEl, AvatarUploadEl, ContextFieldEl, ContextGridEl, GaugeEl, ModelPillEl,
    NumberFieldEl, RichTextEl, RubberSliderEl, ScrubInputEl, SplitBarEl, UnfoldSelectEl,
    UsageMeterEl,
};
#[cfg(oxyn_pro)]
pub use kit::{
    AiInlineEditEl, ChangeSummaryEl, CommentThreadEl, DiffTableEl, ImageCompareEl, ReviewCardEl,
    ReviewCenterEl,
};
#[cfg(oxyn_pro)]
pub use kit::{
    AnswerCardEl, CitedAnswerEl, MessageActionsEl, ThinkingPanelEl, ThinkingPillEl,
    WritingAssistantEl,
};
#[cfg(oxyn_pro)]
pub use kit::{
    ArcDrag, GanttTimelineEl, KanbanBoardEl, MetricsDashboardEl, ProjectBoardEl, TeamDirectoryEl,
};
#[cfg(oxyn_pro)]
pub use kit::{
    AssignBoardEl, ChangeLedgerEl, FindingsFeedEl, JobQueueEl, LiveLanesEl, OvernightRecapEl,
    RelayTrailEl, ReturnBriefEl, RunMonitorEl, SequenceDiagramEl, TaskInboxEl, ToolNetworkEl,
};
pub use kit::{
    AutonomyLevelsEl, ContextTreemapEl, CrewChatEl, CrewTreeEl, DecisionTreeEl, FallbackChainEl,
    PresenceCursorsEl, SpendBreakdownEl,
};
#[cfg(oxyn_pro)]
pub use kit::{
    BillingSettingsEl, GeneralSettingsEl, MembersSettingsEl, ModelsSettingsEl,
    NotificationsSettingsEl, OnboardingSettingsEl, PreferencesSettingsEl, PrivacySettingsEl,
    ProfileSettingsEl, SecuritySettingsEl,
};
pub use kit::{ChatComposerEl, FileDropzoneEl, ModelMenuEl, Pick, RunControlsEl};
pub use kit::{CodeSuggestionEl, DiffTimelineEl, DiffViewerEl, HunkAct, IdAct, InlineDiffEl};
#[cfg(oxyn_pro)]
pub use kit::{CommandBoardEl, FleetWindowEl, WorkspaceCoderEl};
#[cfg(oxyn_pro)]
pub use kit::{CommandCenterEl, SessionStudioEl};
#[cfg(oxyn_pro)]
pub use kit::{ConflictRadarEl, MergeAct, MergeConflictEl, RadarAct};
#[cfg(oxyn_pro)]
pub use kit::{CourierEl, HaloChatEl, HangarEl, PilotPanelEl};
#[cfg(oxyn_pro)]
pub use kit::{
    ExpandableCardEl, JsonViewerEl, LinkUnfurlEl, ToolCallCardEl, ToolCallEl, ToolChipsEl,
};
#[cfg(oxyn_pro)]
pub use kit::{
    HelpCenterEl, InboxTriageEl, MonthCalendarEl, SupportConversationEl, WeekCalendarEl,
};
pub use kit::{ImeEvent, ImeInput, bind_ime_keys};
#[cfg(oxyn_pro)]
pub use kit::{InvoiceDeskEl, ProductDetailEl, ProductListEl, SignInSplitEl, TradingDeskEl};
#[cfg(oxyn_pro)]
pub use kit::{LoadingStateEl, ShimmerEl, SlotTextEl};
#[cfg(oxyn_pro)]
pub use kit::{OnboardingBrandEl, OnboardingReadyEl, OnboardingToneEl, OnboardingWebsiteEl};
#[cfg(oxyn_pro)]
pub use kit::{PreviewDeskEl, RepoDeskEl, TabstackEl, ThreadbarEl};
#[cfg(oxyn_pro)]
pub use kit::{
    TaskdeckBoardEl, TaskdeckPluginsEl, TaskdeckScheduledEl, TaskdeckSitesEl, TaskdeckStartEl,
    TaskdeckTaskEl,
};
#[cfg(oxyn_pro)]
pub use navigation::{
    AccordionEl, BreadcrumbEl, ChipGroupEl, DockEl, ExpandableTextEl, HoverCardEl,
    NotificationCenterEl, PageHeaderEl, RadioCardsEl, RatingEl, ReactionsEl, ReorderListEl,
    ResizableEl, TeamSwitcherEl, TocProgressEl, ToggleGroupEl, TreeViewEl, UserMenuEl,
};
pub use navigation::{CodeBlockEl, CopyButtonEl, TabsEl};
#[cfg(oxyn_pro)]
pub use primitives::{
    ActionButtonEl, EmptyStateEl, EntityChipEl, ExpandingGroupEl, IconButtonEl, KbdEl, MonogramEl,
    NumberTickerEl, PopoverEl, RadioGroupEl, ReactionBarEl, SeparatorEl, SplitButtonEl, TooltipEl,
    ValuePillEl,
};
pub use primitives::{
    AvatarEl, BadgeEl, BadgeSize, ButtonEl, ButtonGroupEl, CheckboxEl, ChipEl, MenuEl, SegmentedEl,
    SelectEl, SliderEl, SpinnerEl, SwitchEl, TextFieldEl,
};
pub use run::{SubagentTreeEl, TodoChecklist, ToolCallRowEl};
pub use shell::WorkspaceSidebarEl;
#[cfg(oxyn_pro)]
pub use shell::{
    AiPanelEl, AppShellEl, ChatShellEl, HarnessEl, InboxEl, RailShellEl, SidebarEl, SidebarNavEl,
    TermTabsEl,
};
#[cfg(oxyn_pro)]
pub use status_indicator::{IndicatorSize, IndicatorVariant, StatusIndicator};
pub use status_text::{RevealText, ShimmerText, TimerText, VerbCyclerView};
#[cfg(oxyn_pro)]
pub use surfaces::{
    AiCommandEl, AiSearchBarEl, AlertEl, BottomSheetEl, ControlCenterEl, ExpandingSearchEl,
    NotificationStackEl, PaginationEl, ScrollAreaEl, SearchFieldEl, SheetEl, SheetStackEl,
    ShortcutRecorderEl, SkeletonRevealEl, ToasterEl,
};
pub use surfaces::{CardEl, CommandPaletteEl, DialogEl, SkeletonEl, dialog_size};
#[cfg(oxyn_pro)]
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
#[cfg(oxyn_pro)]
/// Renamed: use [`FindingsFeedEl`].
#[deprecated(note = "renamed to `FindingsFeedEl`")]
pub type AgentFindingsEl = FindingsFeedEl;
#[cfg(oxyn_pro)]
/// Renamed: use [`ToolNetworkEl`].
#[deprecated(note = "renamed to `ToolNetworkEl`")]
pub type AgentGraphEl = ToolNetworkEl;
#[cfg(oxyn_pro)]
/// Renamed: use [`RelayTrailEl`].
#[deprecated(note = "renamed to `RelayTrailEl`")]
pub type AgentHandoffEl = RelayTrailEl;
#[cfg(oxyn_pro)]
/// Renamed: use [`TaskInboxEl`].
#[deprecated(note = "renamed to `TaskInboxEl`")]
pub type AgentInboxEl = TaskInboxEl;
#[cfg(oxyn_pro)]
/// Renamed: use [`LiveLanesEl`].
#[deprecated(note = "renamed to `LiveLanesEl`")]
pub type AgentLanesEl = LiveLanesEl;
#[cfg(oxyn_pro)]
/// Renamed: use [`ChangeLedgerEl`].
#[deprecated(note = "renamed to `ChangeLedgerEl`")]
pub type AgentLedgerEl = ChangeLedgerEl;
#[cfg(oxyn_pro)]
/// Renamed: use [`RunMonitorEl`].
#[deprecated(note = "renamed to `RunMonitorEl`")]
pub type AgentMonitorEl = RunMonitorEl;
#[cfg(oxyn_pro)]
/// Renamed: use [`JobQueueEl`].
#[deprecated(note = "renamed to `JobQueueEl`")]
pub type AgentQueueEl = JobQueueEl;
#[cfg(oxyn_pro)]
/// Renamed: use [`OvernightRecapEl`].
#[deprecated(note = "renamed to `OvernightRecapEl`")]
pub type AgentRecapEl = OvernightRecapEl;
#[cfg(oxyn_pro)]
/// Renamed: use [`SequenceDiagramEl`].
#[deprecated(note = "renamed to `SequenceDiagramEl`")]
pub type AgentSequenceEl = SequenceDiagramEl;
/// Renamed: use [`AutonomyLevelsEl`].
#[deprecated(note = "renamed to `AutonomyLevelsEl`")]
pub type AutonomyDialEl = AutonomyLevelsEl;
#[cfg(oxyn_pro)]
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
#[cfg(oxyn_pro)]
/// Renamed: use [`AttemptRaceEl`].
#[deprecated(note = "renamed to `AttemptRaceEl`")]
pub type ParallelAttemptsEl = AttemptRaceEl;
/// Renamed: use [`ModelKnockoutEl`].
#[deprecated(note = "renamed to `ModelKnockoutEl`")]
pub type JudgeBracketEl = ModelKnockoutEl;
#[cfg(oxyn_pro)]
/// Renamed: use [`ActivityPillEl`].
#[deprecated(note = "renamed to `ActivityPillEl`")]
pub type RunningNowEl = ActivityPillEl;
/// Renamed: use [`ToolReachEl`].
#[deprecated(note = "renamed to `ToolReachEl`")]
pub type ToolAccessMapEl = ToolReachEl;
#[cfg(oxyn_pro)]
/// Renamed: use [`LoadPlannerEl`].
#[deprecated(note = "renamed to `LoadPlannerEl`")]
pub type RateLimitPlannerEl = LoadPlannerEl;
/// Renamed: use [`TaskGridEl`].
#[deprecated(note = "renamed to `TaskGridEl`")]
pub type FanoutGridEl = TaskGridEl;
#[cfg(oxyn_pro)]
/// Renamed: use [`AssignBoardEl`].
#[deprecated(note = "renamed to `AssignBoardEl`")]
pub type DelegateBoardEl = AssignBoardEl;
#[cfg(oxyn_pro)]
/// Renamed: use [`RubberSliderEl`].
#[deprecated(note = "renamed to `RubberSliderEl`")]
pub type ElasticSliderEl = RubberSliderEl;
#[cfg(oxyn_pro)]
/// Renamed: use [`UnfoldSelectEl`].
#[deprecated(note = "renamed to `UnfoldSelectEl`")]
pub type MorphSelectEl = UnfoldSelectEl;
#[cfg(oxyn_pro)]
/// Renamed: use [`InlineConfirmEl`].
#[deprecated(note = "renamed to `InlineConfirmEl`")]
pub type ConfirmMorphEl = InlineConfirmEl;
#[cfg(oxyn_pro)]
/// Renamed: use [`SkeletonRevealEl`].
#[deprecated(note = "renamed to `SkeletonRevealEl`")]
pub type SkeletonMorphEl = SkeletonRevealEl;
/// Renamed: use [`DateWheelEl`].
#[deprecated(note = "renamed to `DateWheelEl`")]
pub type DateReelEl = DateWheelEl;
#[cfg(oxyn_pro)]
/// Renamed: use [`TradingDeskEl`].
#[deprecated(note = "renamed to `TradingDeskEl`")]
pub type MarketTerminalEl = TradingDeskEl;
#[cfg(oxyn_pro)]
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
