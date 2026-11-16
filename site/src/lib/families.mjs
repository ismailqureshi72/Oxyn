// Component families: every component sits in exactly one family, and each family holds many
// variants of one idea (Thinking Selector, Context Visualizer, Attachments …). The library sidebar
// groups components by family. Full map with the variants still to build: docs/families.md.

/** Families in sidebar order. `members` are demo slugs; a demo whose category is a family slug joins it too. */
export const FAMILY_NAV = [
  { slug: "composer", name: "Composer", blurb: "The prompt box itself, the field and what sits in it.", members: ["composer", "chat-composer", "ai-composer", "rich-text-editor", "mention-input", "mode-switch", "run-mode-menu", "tools-menu", "queue-stack", "agent-steering", "starter-prompts", "writing-assistant", "ghost-text"] },
  { slug: "send-run", name: "Send & Run Controls", blurb: "Send, stop, queue and steer: the button that changes with the run.", members: ["send-control", "run-controls", "action-swap"] },
  { slug: "model-picker", name: "Model Picker", blurb: "Choosing the model: pills, menus and full pickers.", members: ["model-picker", "model-menu", "model-pill"] },
  { slug: "thinking-selector", name: "Thinking Selector", blurb: "How hard the model thinks: rulers, sliders, menus and pickers.", members: ["effort-control", "effort-ruler", "reasoning-panel"] },
  { slug: "context-visualizer", name: "Context Visualizer", blurb: "How full the context window is and what fills it.", members: ["context-meter", "context-field", "context-grid", "context-map", "context-cards"] },
  { slug: "attachments", name: "Attachments", blurb: "Files, images and links going into a prompt.", members: ["attachments", "attachment-strip", "file-dropzone", "avatar-upload", "link-unfurl", "document-extraction"] },
  { slug: "storage", name: "Storage", blurb: "Disk and context usage drawn as dot matrices: capacity, cleanup, waffle and forecast.", members: [] },
  { slug: "voice", name: "Voice", blurb: "Speaking to the agent: record, dictate, listen.", members: ["voice-recorder", "dictation-button"] },
  { slug: "approval", name: "Approval & Permissions", blurb: "The agent asks before it acts; how much it may do alone.", members: ["approval-card", "approval-pager", "decision-card", "approval-questions", "hold-to-confirm", "confirm-morph", "confirm-popover", "approval-gates", "permission-request", "agent-approvals", "permission-selector", "autonomy-dial", "tool-access-map"] },
  { slug: "messages", name: "Messages & Answers", blurb: "The conversation: bubbles, answers, streaming text and actions.", members: ["message-bubble", "chat-thread", "ai-chat", "ai-answer", "cited-answer", "message-actions", "response-feedback", "streaming-markdown", "streaming-text", "support-conversation", "recommendation-card"] },
  { slug: "reasoning", name: "Reasoning", blurb: "What the model thought: blocks, traces and live thinking states.", members: ["reasoning-block", "thinking-traces", "thinking-state", "thinking-loader"] },
  { slug: "sources", name: "Sources & Research", blurb: "Citations, source cards and research runs.", members: ["source-cards", "research-run", "experiment-grid"] },
  { slug: "loaders", name: "Loaders", blurb: "One busy signal per region: glyphs, spinners and skeletons.", members: ["glyph-loader", "spinner", "loading-state", "status-indicator", "skeleton", "skeleton-morph"] },
  { slug: "text-motion", name: "Text Motion", blurb: "Shimmering, cycling, revealing and counting text.", members: ["text-shimmer", "shimmer-label", "verb-cycler", "text-reveal", "word-reveal", "text-morph", "slot-text", "in-view-title", "elapsed-timer", "number-ticker", "animated-counter", "countdown"] },
  { slug: "tool-calls", name: "Tool Calls", blurb: "What the agent called and what came back.", members: ["tool-call", "tool-stack", "tool-call-card", "tool-call-transcript", "tool-chips", "tool-kind-map", "json-viewer"] },
  { slug: "plans", name: "Plans & Tasks", blurb: "Plans, to-dos and checklists, and how the run kept to them.", members: ["todo-list", "plan-card", "agent-plan", "task-rows", "task-breakdown", "stepper", "step-timeline", "onboarding-checklist", "release-readiness", "plan-vs-actual", "decision-tree"] },
  { slug: "run-trace", name: "Run & Trace", blurb: "One run end to end: status, trail, waterfall and root cause.", members: ["agent-run", "agent-trail", "trace-waterfall", "activity-feed", "agent-sequence", "root-cause-tree", "running-now", "agent-status", "agent-monitor"] },
  { slug: "multi-agent", name: "Multi-agent", blurb: "Many agents at once: trees, crews, lanes, fan-outs and judges.", members: ["subagent-tree", "agent-tree", "agent-fleet", "agent-orchestrator", "agent-crew", "agent-cursors", "agent-graph", "agent-handoff", "agent-lanes", "delegate-board", "fanout-grid", "parallel-attempts", "judge-bracket", "fallback-chain", "collaboration-stream"] },
  { slug: "inbox", name: "Inbox & Recap", blurb: "What happened while you were away and what needs you now.", members: ["agent-inbox", "agent-queue", "agent-recap", "away-summary", "agent-findings", "notification-center", "notification-stack", "thread-list", "activity-home"] },
  { slug: "usage-cost", name: "Usage & Cost", blurb: "Tokens, money and limits: meters, charts and planners.", members: ["usage-meter", "token-usage-chart", "polling-cost", "cost-waterfall", "agent-ledger", "rate-limit-planner", "gauge", "activity-heatmap", "split-bar"] },
  { slug: "memory", name: "Memory", blurb: "What the agent remembers about you and the work.", members: ["memory-panel"] },
  { slug: "diff", name: "Diff & Review", blurb: "Changes the agent made, line by line, and the review around them.", members: ["diff-viewer", "inline-diff", "diff-table", "diff-timeline", "change-summary", "code-suggestion", "ai-inline-edit", "review-center", "review-card", "image-compare"] },
  { slug: "merge", name: "Merge Conflict", blurb: "Resolving two versions of the same change.", members: ["merge-conflict", "conflict-radar"] },
  { slug: "checkpoints", name: "Checkpoints", blurb: "Rewind a run to any point; branch new attempts from it.", members: ["checkpoint-lanes", "checkpoint-list", "checkpoint-marker", "restore-toast"] },
  { slug: "comments", name: "Comments & Reactions", blurb: "Talking about the work: threads, panels, reactions.", members: ["comment-thread", "comments-panel", "threaded-comments", "reactions", "reaction-bar"] },
  { slug: "schedule", name: "Schedule & Time", blurb: "Dates, times and when agents run.", members: ["scheduled-runs", "calendar", "date-picker", "date-range-picker", "time-picker", "time-dial", "date-reel"] },
  { slug: "theme-selector", name: "Theme Selector", blurb: "Backgrounds, textures, theme swatches and font size.", members: [] },
  { slug: "buttons", name: "Buttons", blurb: "Buttons, icon buttons, split buttons and the groups they sit in.", members: ["button", "icon-button", "action-button", "split-button", "button-group", "floating-button-group", "expanding-button-group", "copy-button"] },
  { slug: "fields", name: "Fields", blurb: "Typing things in: text, numbers, codes, passwords, tags.", members: ["text-field", "text-area", "labeled-input", "labeled-textarea", "number-field", "scrub-input", "password-field", "password-strength", "otp-input", "phone-input", "inline-edit", "tag-input", "smart-form-fill", "stepper-form", "multi-step-form"] },
  { slug: "choice", name: "Choice", blurb: "Picking one or many: switches, radios, selects, chips and ratings.", members: ["switch", "checkbox", "radio-group", "radio-cards", "segmented-control", "toggle-group", "chip-group", "rating", "select", "morph-select", "combobox", "multi-select", "filter-menu", "reorderable-list"] },
  { slug: "sliders", name: "Sliders", blurb: "Dragging a value: plain, elastic and scrubbed.", members: ["slider", "elastic-slider"] },
  { slug: "display", name: "Data Display", blurb: "Badges, chips, avatars, keys, code and empty states.", members: ["badge", "chip", "value-pill", "entity-chip", "avatar", "kbd", "separator", "empty-state", "empty-states", "code-block", "tree-table"] },
  { slug: "overlays", name: "Overlays", blurb: "Tooltips, popovers, menus, dialogs, sheets and drawers.", members: ["tooltip", "popover", "dropdown-menu", "hover-card", "dialog", "sheet", "sheet-stack", "bottom-sheet", "drawer", "control-center"] },
  { slug: "feedback", name: "Toasts & Alerts", blurb: "Telling you something happened.", members: ["toaster", "alert"] },
  { slug: "search", name: "Search & Commands", blurb: "Palettes, AI search, settings search and shortcuts.", members: ["command-palette", "ai-command", "ai-search-bar", "settings-command", "search-field", "expanding-search", "shortcut-recorder", "pagination"] },
  { slug: "navigation", name: "Navigation & Layout", blurb: "Tabs, breadcrumbs, accordions, panes, trees and docks.", members: ["tabs", "breadcrumb", "accordion", "resizable-panels", "tree-view", "toc-progress", "page-header", "dock", "expandable-text", "sidebar-nav", "terminal-tabs", "scroll-area", "expandable-card", "card"] },
  { slug: "app-shell", name: "App Shell", blurb: "Sidebars, rails, side panels and account chrome.", members: ["sidebar", "workspace-sidebar", "sidebar-rail", "inbox-sidebar", "app-shell", "ai-side-panel", "harness", "ai-chat-screens", "doc-grid", "user-menu", "team-switcher"] },
];

const bySlug = new Map();
for (const f of FAMILY_NAV) for (const m of f.members) bySlug.set(m, f);
/** The family a demo belongs to (`null` only for a component not yet placed). */
export const navFamilyOf = (d) => bySlug.get(d.slug) ?? FAMILY_NAV.find((f) => f.slug === d.category) ?? null;
