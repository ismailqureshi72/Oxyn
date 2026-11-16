//! GPUI components for Oxyn (primary renderer).
//!
//! Components take `&AgentState` plus props and read time from the window's shared
//! [`MotionClock`]; the host keeps the clock advanced and requests frames while
//! [`MotionClock::next_frame_in`] says so.

mod brand;
mod chat;
mod composer;
mod composer_parts;
mod forms;
mod glyph_loader;
mod hitl;
mod navigation;
mod parity;
mod primitives;
mod run;
mod shell;
mod status_indicator;
mod status_text;
mod surfaces;

pub use brand::{BrandAssets, brand_icon, brand_mark, brand_path};
pub use chat::{
    AnswerEl, ChatThreadEl, FeedbackEl, MarkdownEl, MessageEl, ReasoningEl, SourceCardsEl,
    StartersEl, TraceEl,
};
pub use composer::{Composer, ComposerEvent};
pub use composer_parts::{
    DictationEl, MentionListEl, ModeSwitchEl, PermissionEl, QueueStackEl, SteeringEl, ToolsMenuEl,
    VoiceRecorderEl,
};
pub use forms::{
    ApprovalQuestionsEl, CalendarEl, ComboboxEl, DatePickerEl, DateReelEl, DecisionCardEl,
    FilterMenuEl, InlineEditEl, OtpEl, PasswordFieldEl, PasswordStrengthEl, PhoneInputEl,
    RangePickerEl, SmartFillEl, StepperEl, TagInputEl, TimeDialEl, TimePickerEl,
};
pub use glyph_loader::GlyphLoaderView;
pub use hitl::{ApprovalCardEl, CheckpointLanesEl};
pub use navigation::{
    AccordionEl, BreadcrumbEl, ChipGroupEl, CodeBlockEl, CopyButtonEl, DockEl, ExpandableTextEl,
    HoverCardEl, NotificationCenterEl, PageHeaderEl, RadioCardsEl, RatingEl, ReactionsEl,
    ReorderListEl, ResizableEl, TabsEl, TeamSwitcherEl, TocProgressEl, ToggleGroupEl, TreeViewEl,
    UserMenuEl,
};
pub use parity::AccentDensityEl;
pub use parity::AppIconPickerEl;
pub use parity::AttachmentChipsEl;
pub use parity::BackgroundPickerEl;
pub use parity::ChatAgentsEl;
pub use parity::ChatOverviewEl;
pub use parity::ChatProgressEl;
pub use parity::ChatRailEl;
pub use parity::ChatRunEl;
pub use parity::ChatStartEl;
pub use parity::ContextBarEl;
pub use parity::ContextBreakdownEl;
pub use parity::ContextEffortMenuEl;
pub use parity::ContextStackEl;
pub use parity::ContextTideEl;
pub use parity::DiffTimelineRunEl;
pub use parity::DisplayComfortEl;
pub use parity::DropLanesEl;
pub use parity::EffortListEl;
pub use parity::EffortPillEl;
pub use parity::EffortRulerEl;
pub use parity::EffortStairsEl;
pub use parity::FanoutGridEl;
pub use parity::GooeyAttachMenuEl;
pub use parity::GooeyComposerEl;
pub use parity::GooeyModelMenuEl;
pub use parity::JudgeBracketEl;
pub use parity::LightPlaygroundEl;
pub use parity::LiquidDropsEl;
pub use parity::LiquidTabsEl;
pub use parity::MemoryPanelEl;
pub use parity::ModelEffortMenuEl;
pub use parity::NozzleSliderEl;
pub use parity::ParallelAttemptsEl;
pub use parity::PlanVsActualEl;
pub use parity::PolaroidStackEl;
pub use parity::RateLimitPlannerEl;
pub use parity::ReasoningPanelEl;
pub use parity::RootCauseTreeEl;
pub use parity::RunningNowEl;
pub use parity::ScheduledRunsEl;
pub use parity::ScreenRailEl;
pub use parity::ScreenShellEl;
pub use parity::StorageCleanupEl;
pub use parity::StorageForecastEl;
pub use parity::StorageMeterEl;
pub use parity::StorageWaffleEl;
pub use parity::ThemeGridEl;
pub use parity::ThemeScheduleEl;
pub use parity::ThemeSplitEl;
pub use parity::ToolAccessMapEl;
pub use parity::TraceWaterfallEl;
pub use parity::TypeScaleEl;
pub use parity::UploadDropzoneEl;
pub use parity::{
    Act, ActivityFeedEl, AgentApprovalsEl, AgentPlanEl, AgentRunEl, AgentStatusEl, AgentTrailEl,
    AgentTreeEl, ApprovalGatesEl, ApprovalPagerEl, BreakdownEl, ChecklistEl, ConfirmMorphEl,
    ConfirmPopoverEl, CountdownEl, CounterEl, FleetEl, FlowStepperEl, HoldConfirmEl,
    OrchestratorEl, PermissionRequestEl, ReadinessEl, TaskRowsEl, TreeTableEl,
};
pub use parity::{
    ActivityHeatmapEl, ActivityHomeEl, AttachmentStripEl, DocGridEl, ExperimentGridEl,
    PollingCostEl, RunModeMenuEl, ThreadListEl, TokenUsageChartEl,
};
pub use parity::{
    AgentCommentThreadEl, AgentRunPanelEl, CommentsPanelEl, ModelPickerMenuEl, ThreadMsg,
    file_badge,
};
pub use parity::{
    AgentCrewEl, AgentCursorsEl, AgentFindingsEl, AgentGraphEl, AgentHandoffEl, AgentInboxEl,
    AgentLanesEl, AgentLedgerEl, AgentMonitorEl, AgentQueueEl, AgentRecapEl, AgentSequenceEl,
    AutonomyDialEl, AwaySummaryEl, CollabStreamEl, ContextMapEl, CostWaterfallEl, DecisionTreeEl,
    DelegateBoardEl, FallbackChainEl,
};
pub use parity::{
    AiComposerEl, AvatarUploadEl, ChatComposerEl, ContextFieldEl, ContextGridEl, ElasticSliderEl,
    FileDropzoneEl, GaugeEl, ModelMenuEl, ModelPillEl, MorphSelectEl, NumberFieldEl, Pick,
    RichTextEl, RunControlsEl, ScrubInputEl, SplitBarEl, UsageMeterEl,
};
pub use parity::{
    AiInlineEditEl, ChangeSummaryEl, CodeSuggestionEl, CommentThreadEl, DiffTableEl,
    DiffTimelineEl, DiffViewerEl, HunkAct, IdAct, ImageCompareEl, InlineDiffEl, ReviewCardEl,
    ReviewCenterEl,
};
pub use parity::{
    AnswerCardEl, CitedAnswerEl, MessageActionsEl, ThinkingPanelEl, ThinkingPillEl,
    WritingAssistantEl,
};
pub use parity::{ConflictRadarEl, MergeAct, MergeConflictEl, RadarAct};
pub use parity::{
    ExpandableCardEl, JsonViewerEl, LinkUnfurlEl, ToolCallCardEl, ToolCallEl, ToolChipsEl,
};
pub use parity::{ImeEvent, ImeInput, bind_ime_keys};
pub use parity::{LoadingStateEl, ShimmerEl, SlotTextEl, ThinkingLoaderEl};
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
    SheetEl, SheetStackEl, ShortcutRecorderEl, SkeletonEl, SkeletonMorphEl, ToasterEl, dialog_size,
};

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
