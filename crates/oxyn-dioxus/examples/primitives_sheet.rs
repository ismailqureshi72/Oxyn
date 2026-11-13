//! Core primitives review sheet (plan #23): every item, its states and its Oxyn-plus extras.
//! `cargo run -p oxyn-dioxus --example primitives_sheet -- tokens/dist/oxyn.css > sheet.html`
//! Add `light` as a second argument for the light theme.

use dioxus::prelude::*;
use oxyn_core::{
    AgentState, ClockMs, RequestRef, ToolRef,
    primitives::{
        button::{ActionButton as ActionModel, GroupItem},
        display::{
            AgentRing, Avatar as AvatarModel, AvatarShape, Badge as BadgeModel, EmptyLayout,
            EmptyState as EmptyModel, EntityChip as EntityModel, Fill, Monogram as MonogramModel,
            Presence, Tone,
        },
        menu::MenuItem,
        segmented::Segment,
        select::SelectOption,
        slider::{Mark, Slider as SliderModel},
        ticker::{CountRule, NumberFormat, NumberTicker as TickerModel, Reaction, TickerMode},
        toggle::RadioOption,
    },
};
use oxyn_dioxus::{
    ActionButton, Avatar, AvatarGroup, Badge, Button, ButtonGroup, Checkbox, Chip, DropdownMenu,
    EmptyState, EntityChip, ExpandingButtonGroup, FloatingButtonGroup, IconButton, Kbd, Monogram,
    NumberTicker, PRIMITIVES_CSS, Popover, RadioGroup, ReactionBar, SegmentedControl, Select,
    Separator, Slider, Spinner, SplitButton, Switch, TextField, Tooltip, TooltipProvider,
    ValuePill,
};
use oxyn_tokens::ColorRole;

fn badge(label: &str, tone: Tone) -> BadgeModel {
    BadgeModel::new(label, tone)
}

fn person(name: &str, size: f32, status: Option<Presence>) -> AvatarModel {
    let mut a = AvatarModel::person(name);
    a.size = size;
    a.status = status;
    a
}

fn agent(name: &str, size: f32, state: AgentState) -> AvatarModel {
    let mut a = AvatarModel::agent(name, &state);
    a.size = size;
    a
}

fn tool() -> AgentState {
    AgentState::ToolRunning {
        tool: ToolRef {
            id: "t1".into(),
            name: "bash".into(),
            activity: oxyn_core::ActivityKind::Run,
            target: Some("cargo test".into()),
        },
        concurrent: 1,
    }
}

fn menu_items() -> Vec<MenuItem> {
    vec![
        MenuItem::label("Thread"),
        MenuItem::action("rename", "Rename")
            .icon("pencil")
            .shortcut("Mod+R"),
        MenuItem::action("dup", "Duplicate")
            .icon("copy")
            .shortcut("Mod+D"),
        MenuItem::action("pin", "Pin to top")
            .icon("star")
            .disabled(),
        MenuItem::submenu(
            "move",
            "Move to",
            vec![
                MenuItem::action("inbox", "Inbox").icon("inbox"),
                MenuItem::action("archive", "Archive").icon("archive"),
            ],
        )
        .icon("folder"),
        MenuItem::divider(),
        MenuItem::checkbox("wrap", "Wrap long lines", true),
        MenuItem::radio("fast", "Fast model", "model", true),
        MenuItem::radio("careful", "Careful model", "model", false),
        MenuItem::divider(),
        MenuItem::action("del", "Delete thread")
            .icon("trash")
            .shortcut("Mod+Backspace")
            .danger(),
    ]
}

#[component]
fn Card(title: String, children: Element) -> Element {
    rsx! { div { class: "card", h2 { "{title}" } {children} } }
}

#[component]
fn Sheet() -> Element {
    let now = ClockMs(10_000);
    let mut action_ok = ActionModel::new("Deploy", "Deploying…", "Deployed");
    action_ok.press(ClockMs(9_000));
    action_ok.resolve(true, ClockMs(9_600));
    let mut action_pending = ActionModel::new("Run evals", "Running…", "Done");
    action_pending.press(ClockMs(9_800));
    let mut action_err = ActionModel::new("Sync", "Syncing…", "Synced");
    action_err.backoff = std::time::Duration::from_secs(4);
    action_err.press(ClockMs(8_000));
    action_err.resolve(false, ClockMs(8_500));

    let group = vec![
        GroupItem::new("bold", "Bold", Some("text")),
        GroupItem::new("link", "Link", Some("link")),
        GroupItem::new("code", "Code", Some("code")),
        GroupItem::new("files", "Files", Some("folder")).count(12),
        GroupItem::new("img", "Image", Some("image")),
        GroupItem::new("table", "Table", Some("table")),
    ];
    let tray = vec![
        GroupItem::new("copy", "Copy", Some("copy")),
        GroupItem::new("retry", "Regenerate", Some("retry")),
        GroupItem::new("good", "Good response", Some("check")),
        GroupItem::new("share", "Share", Some("share")),
    ];
    let expand = vec![
        GroupItem::new("copy", "Copy", Some("copy")).done("Copied"),
        GroupItem::new("branch", "Branch", Some("branch")).done("Branched"),
        GroupItem::new("share", "Share", Some("share")).done("Link copied"),
        GroupItem::new("dl", "Export", Some("download")).done("Saved"),
    ];

    let mut live = badge("Running", Tone::Working);
    live.live = true;
    let mut count = badge("unread", Tone::Neutral);
    count.count = Some(128);
    count.fill = Fill::Solid;
    let mut dot = badge("Passed", Tone::Success);
    dot.dot = true;
    let mut outline = badge("Needs review", Tone::Warning);
    outline.fill = Fill::Outline;
    let mut mono = badge("v0.4.2", Tone::Neutral);
    mono.fill = Fill::Mono;

    let mut ent = EntityModel::new("Cone King");
    ent.status = Some(Presence::Online);
    ent.card = vec![
        ("Type".into(), "Supplier".into()),
        ("Since".into(), "March 2024".into()),
        ("Orders".into(), "1,204".into()),
    ];
    let mut ws = person("Northwind Labs", 32.0, None);
    ws.shape = AvatarShape::Square;
    let mut prog = agent("Indexer", 40.0, AgentState::Idle);
    prog.shape = AvatarShape::Circle;
    prog.ring = AgentRing::Progress(0.64);
    let mut modeled = agent("Coder", 40.0, AgentState::Streaming);
    modeled.model = Some("S".into());

    let mut ticker_up = TickerModel::new(9_998.0, NumberFormat::Integer);
    ticker_up.set(10_240.0, ClockMs(5_000));
    let mut ticker_mid = TickerModel::new(1_204.0, NumberFormat::Integer);
    ticker_mid.set(1_212.0, ClockMs(9_700));
    let mut ticker_cost = TickerModel::new(0.42, NumberFormat::Currency("$".into(), 2));
    ticker_cost.set(1.87, ClockMs(5_000));
    let mut ticker_compact = TickerModel::new(1_200_000.0, NumberFormat::Compact);
    ticker_compact.mode = TickerMode::CountUp;
    let reactions = vec![
        Reaction::new("like", "Like", "heart", 1_203, ColorRole::FgDefault),
        Reaction::action("comment", "Comment", "comment", 48),
        Reaction::new("repost", "Repost", "repost", 12_480, ColorRole::FgDefault),
        Reaction::action("share", "Share", "share", 0),
    ];
    let mut liked = reactions.clone();
    liked[0].active = true;

    let segs = |n: usize| -> Vec<Segment> {
        let mut v = vec![
            Segment::new("ask", "Ask"),
            Segment::new("plan", "Plan"),
            Segment::new("agent", "Agent"),
            Segment::new("debug", "Debug"),
            Segment::new("review", "Review"),
            Segment::new("research", "Research"),
            Segment::new("spec", "Spec"),
            Segment::new("mission", "Mission"),
        ];
        v.truncate(n);
        v
    };
    let mut seg_icons = segs(3);
    seg_icons[0].icon = Some("thread".into());
    seg_icons[1].icon = Some("checklist".into());
    seg_icons[2].icon = Some("agents".into());
    seg_icons[2].count = Some(3);
    let mut seg_tabs = vec![
        Segment::new("all", "All"),
        Segment::new("running", "Running"),
        Segment::new("done", "Done"),
    ];
    seg_tabs[1].count = Some(4);
    seg_tabs[2].badge = Some("New".into());

    let models = vec![
        SelectOption::new("haiku", "Haiku")
            .desc("Fastest · $0.25 / M")
            .icon("bolt")
            .group("Anthropic"),
        SelectOption::new("sonnet", "Sonnet")
            .desc("Balanced · $3 / M")
            .icon("spark")
            .group("Anthropic"),
        SelectOption::new("opus", "Opus")
            .desc("Deepest · $15 / M")
            .icon("brain")
            .group("Anthropic"),
        SelectOption::new("mini", "GPT mini")
            .desc("Cheap and quick")
            .icon("bolt")
            .group("OpenAI"),
    ];
    let mut models_dis = models.clone();
    models_dis[2].disabled = true;

    let mut effort = SliderModel::new(1.0, 0.0, 3.0, 0.01);
    effort.snap = 0.25;
    effort.marks = ["Low", "Medium", "High", "Max"]
        .iter()
        .enumerate()
        .map(|(i, l)| Mark {
            value: i as f64,
            label: (*l).into(),
            detent: true,
        })
        .collect();
    let mut temp = SliderModel::new(0.7, 0.0, 2.0, 0.1);
    temp.marks = vec![
        Mark {
            value: 0.0,
            label: "0".into(),
            detent: false,
        },
        Mark {
            value: 1.0,
            label: "1".into(),
            detent: false,
        },
        Mark {
            value: 2.0,
            label: "2".into(),
            detent: false,
        },
    ];
    let mut budget = SliderModel::range(20.0, 60.0, 0.0, 100.0, 5.0);
    budget.min_steps_between = 2;
    let mut vol = SliderModel::new(60.0, 0.0, 100.0, 1.0);
    vol.vertical = true;
    let mut off = SliderModel::new(30.0, 0.0, 100.0, 1.0);
    off.disabled = true;

    let radio = vec![
        RadioOption {
            description: Some("Asks before every edit".into()),
            ..RadioOption::new("ask", "Ask first")
        },
        RadioOption {
            description: Some("Edits files, asks before shell commands".into()),
            ..RadioOption::new("edits", "Auto-edit")
        },
        RadioOption {
            description: Some("Runs everything; you review at the end".into()),
            disabled: Some("Your admin turned this off".into()),
            ..RadioOption::new("full", "Full auto")
        },
    ];

    let mut empty_prompts = EmptyModel {
        title: "No runs yet".into(),
        text: "Start an agent on this repo and its runs show up here.".into(),
        action: Some("New run".into()),
        secondary: Some("Read the guide".into()),
        icon: "agents".into(),
        ..Default::default()
    };
    empty_prompts.prompts = vec![
        "Summarise this repo".into(),
        "Find flaky tests".into(),
        "Draft release notes".into(),
    ];

    rsx! {
        TooltipProvider {
            div { class: "page",
                header { h1 { "Oxyn · core primitives" } p { "Plan #23 · 27 components · Dioxus renderer · review sheet, not the site layout. States are forced where a sheet cannot hover." } }
                div { class: "cols",
                    div { class: "col",
                        Card { title: "button · variants (source 4 + Oxyn plus 4)",
                            div { class: "line",
                                Button { label: "Primary" } Button { label: "Secondary", variant: "secondary" } Button { label: "Ghost", variant: "ghost" } Button { label: "Delete", variant: "destructive", icon: "trash" }
                            }
                            div { class: "line",
                                Button { label: "Outline", variant: "outline" } Button { label: "Link", variant: "link" } Button { label: "Quiet", variant: "quiet" } Button { label: "Saved", variant: "success", icon: "check" }
                            }
                        }
                        Card { title: "button · sizes xs–xl · states",
                            div { class: "line",
                                Button { label: "xs", size: "xs" } Button { label: "sm", size: "sm" } Button { label: "md" } Button { label: "lg", size: "lg" } Button { label: "xl", size: "xl", icon: "spark" }
                            }
                            div { class: "line",
                                Button { label: "Hover", variant: "secondary", state: "hover" } Button { label: "Pressed", state: "pressed" } Button { label: "Focus", variant: "secondary", state: "focus" }
                                Button { label: "Disabled", disabled: true } Button { label: "Send", disabled: true, reason: "Waiting for your approval", icon: "up" }
                            }
                            div { class: "line",
                                Button { label: "Generating", loading: true } Button { label: "Saving", variant: "secondary", loading: true, loading_style: "spinner" }
                                Button { label: "Uploading 64%", variant: "secondary", loading: true, loading_style: "progress", progress: 0.64 }
                            }
                            div { class: "line",
                                Button { label: "Hold to delete", variant: "destructive", hold: true, icon: "trash" }
                                Button { label: "Run", icon: "play", shortcut: "Mod+Enter" }
                                Button { label: "Open", variant: "secondary", icon_right: "external" }
                            }
                        }
                        Card { title: "icon button · ghost / secondary / primary · 5 sizes · toggle morph · badge",
                            div { class: "line",
                                IconButton { icon: "copy", label: "Copy", size: "xs" } IconButton { icon: "copy", label: "Copy", size: "sm" } IconButton { icon: "copy", label: "Copy" }
                                IconButton { icon: "copy", label: "Copy", size: "lg", variant: "secondary" } IconButton { icon: "plus", label: "New", size: "xl", variant: "primary" }
                                IconButton { icon: "audio", label: "Mute", pressed: true, pressed_icon: "stop" } IconButton { icon: "bell", label: "Notifications", badge: 3 } IconButton { icon: "inbox", label: "Inbox", badge: 0, variant: "secondary" }
                                IconButton { icon: "pencil", label: "Edit", state: "hover" } IconButton { icon: "trash", label: "Delete", disabled: true }
                            }
                        }
                        Card { title: "action button · idle / pending / success / retry back-off (Oxyn plus)",
                            div { class: "line",
                                ActionButton { model: ActionModel::new("Deploy", "Deploying…", "Deployed"), now, shortcut: "Mod+Shift+D" }
                                ActionButton { model: action_pending, now }
                                ActionButton { model: action_ok, now }
                                ActionButton { model: action_err, now, icon: "retry" }
                            }
                        }
                        Card { title: "split button · primary / secondary · remembered choice (Oxyn plus)",
                            div { class: "line menuroom",
                                SplitButton { actions: vec![("merge".into(), "Merge".into()), ("squash".into(), "Squash and merge".into()), ("rebase".into(), "Rebase and merge".into())], icon: "merge", remember: true, open: true }
                                SplitButton { actions: vec![("run".into(), "Run with Sonnet".into()), ("opus".into(), "Run with Opus".into())], variant: "secondary", icon: "play" }
                                SplitButton { actions: vec![("x".into(), "Disabled".into()), ("y".into(), "Other".into())], disabled: true }
                            }
                        }
                        Card { title: "button group · outline / solid · vertical · counts · overflow menu (Oxyn plus)",
                            div { class: "line",
                                ButtonGroup { items: group[..3].to_vec(), highlight: 1 }
                                ButtonGroup { items: group[..3].to_vec(), variant: "solid", selected: "code", collapse_labels: true }
                                ButtonGroup { items: group.clone(), max_visible: 4, size: "sm", menu: vec![MenuItem::action("settings", "Toolbar settings")] }
                            }
                            div { class: "line",
                                ButtonGroup { items: group[..3].to_vec(), vertical: true, size: "sm" }
                                ButtonGroup { items: group[..4].to_vec(), size: "ide", collapse_labels: true }
                            }
                        }
                        Card { title: "floating button group · floating / muted · pill / underline / glow highlight · auto-hide",
                            div { class: "line tall",
                                FloatingButtonGroup { items: tray.clone(), highlight: 1 }
                                FloatingButtonGroup { items: tray.clone(), variant: "muted", highlight_style: "underline", size: "sm" }
                            }
                            div { class: "line",
                                FloatingButtonGroup { items: tray.clone(), highlight_style: "glow", icon_only: false, size: "sm" }
                                FloatingButtonGroup { items: tray.clone(), vertical: true, auto_hide: true, tooltip_side: "right" }
                            }
                        }
                        Card { title: "expanding button group · one open · confirmed · labels always on (Oxyn plus)",
                            div { class: "stack",
                                ExpandingButtonGroup { items: expand.clone(), default_expanded: 1 }
                                ExpandingButtonGroup { items: expand.clone(), confirmed: 0, size: "sm" }
                                ExpandingButtonGroup { items: expand.clone(), labels: true, delay_ms: 150 }
                            }
                        }
                        Card { title: "dropdown menu · icons, shortcuts, check / radio, danger, submenu, gliding highlight",
                            div { class: "line menuroom big",
                                DropdownMenu { items: menu_items(), trigger: "Thread", icon: "thread", open: true, highlight: 4, title: "Thread actions" }
                            }
                        }
                        Card { title: "dropdown menu · searchable + compact (Oxyn plus) · danger highlight · empty",
                            div { class: "line menuroom big",
                                DropdownMenu { items: menu_items(), trigger: "Filter", searchable: true, density: "compact", open: true, highlight: 10, align: "start" }
                                div { class: "push" }
                                DropdownMenu { items: vec![], trigger: "Recent", open: true, align: "end" }
                            }
                        }
                        Card { title: "context menu mode (Arc) · right-click or Shift+F10",
                            div { class: "ctxcard", DropdownMenu { items: menu_items(), context: true, open: true, highlight: 1,
                                div { class: "ctxbox", "Right-click this message" }
                            } }
                        }
                    }
                    div { class: "col",
                        Card { title: "badge · six tones + agent tones · soft / outline / solid / mono · live · count",
                            div { class: "line",
                                for t in Tone::SOURCE { Badge { badge: badge(t.id(), t) } }
                            }
                            div { class: "line",
                                for t in [Tone::Working, Tone::Waiting, Tone::Queued, Tone::Stalled] { Badge { badge: badge(t.id(), t), size: "md" } }
                            }
                            div { class: "line",
                                Badge { badge: live } Badge { badge: dot } Badge { badge: outline } Badge { badge: count } Badge { badge: mono } Badge { badge: badge("Edited", Tone::Info), icon: "pencil", size: "lg" }
                            }
                        }
                        Card { title: "chip · static / toggle / removable · counts · status dot",
                            div { class: "line",
                                Chip { label: "Rust" } Chip { label: "Toggle", kind: "toggle" } Chip { label: "Selected", kind: "toggle", selected: true, count: 12 }
                                Chip { label: "Running", dot: "working" } Chip { label: "Failed", dot: "danger", count: 2 } Chip { label: "src/lib.rs", kind: "removable" } Chip { label: "Disabled", kind: "toggle", disabled: true }
                            }
                        }
                        Card { title: "value pill (Beautiful UI) · tones · delta arrows (Oxyn plus) · entity chip + monogram",
                            div { class: "line",
                                ValuePill { value: "7 days" } ValuePill { value: "Healthy", tone: "green" } ValuePill { value: "Slow", tone: "orange" } ValuePill { value: "Over budget", tone: "red" } ValuePill { value: "Pick", tone: "accent" }
                                ValuePill { value: "+12.4%", delta: (10.0, 11.24) } ValuePill { value: "$4.10", delta: (3.2, 4.1), good_up: false } ValuePill { value: "320 ms", delta: (410.0, 320.0), good_up: false }
                            }
                            p { class: "prose",
                                "Restock " EntityChip { entity: EntityModel::new("Cone King") } " in " ValuePill { value: "7 days" } " — cheaper than " EntityChip { entity: ent.clone(), card_open: true } " or " EntityChip { entity: EntityModel::new("Ada Lovelace") } "."
                            }
                            div { class: "line",
                                for i in 0..12 { Monogram { monogram: MonogramModel::of("x", Some(i), Some(["AK","BR","CK","DL","EM","FN","GO","HP","IQ","JR","KS","LT"][i])), size: 22.0, square: i % 2 == 1 } }
                            }
                        }
                        Card { title: "avatar · people, workspace, image fallback · presence · 7 sizes (Oxyn plus)",
                            div { class: "line",
                                for (i, s) in [12.0f32, 16.0, 24.0, 32.0, 40.0, 56.0, 96.0].into_iter().enumerate() {
                                    Avatar { avatar: person(["Ada Lovelace","Grace Hopper","Alan Turing","Katherine Johnson","Linus T","Margaret Hamilton","Barbara Liskov"][i], s, if s >= 24.0 { Some([Presence::Online, Presence::Away, Presence::Busy, Presence::Offline][i % 4]) } else { None }) }
                                }
                                Avatar { avatar: ws }
                            }
                        }
                        Card { title: "avatar with agent ring · state-driven ring · pixel mark · model badge · progress ring",
                            div { class: "line",
                                Avatar { avatar: agent("Planner", 40.0, AgentState::Thinking { header: None }) }
                                Avatar { avatar: agent("Coder", 40.0, AgentState::Streaming) }
                                Avatar { avatar: agent("Runner", 40.0, tool()) }
                                Avatar { avatar: agent("Deployer", 40.0, AgentState::AwaitingApproval { request: RequestRef { id: "r1".into(), tool: None } }) }
                                Avatar { avatar: agent("Tester", 40.0, AgentState::Done { elapsed: Default::default(), waited: Default::default() }) }
                                Avatar { avatar: agent("Fetcher", 40.0, AgentState::Error { kind: oxyn_core::ErrorKind::Tool }) }
                                Avatar { avatar: agent("Idle", 40.0, AgentState::Idle) }
                                Avatar { avatar: prog }
                                Avatar { avatar: modeled }
                            }
                            div { class: "line",
                                AvatarGroup { avatars: vec![person("Ada L", 28.0, None), agent("Coder", 28.0, AgentState::Streaming), person("Grace H", 28.0, None), person("Alan T", 28.0, None), person("Kat J", 28.0, None), person("Linus", 28.0, None)], max: 4 }
                                AvatarGroup { avatars: vec![person("Ada L", 24.0, None), person("Grace H", 24.0, None)], total: 14 }
                            }
                        }
                        Card { title: "kbd · keycap / flat / inline · Mac vs other (Oxyn plus) · sequences · pressed",
                            div { class: "line",
                                Kbd { keys: "Mod+K" } Kbd { keys: "Mod+Shift+P" } Kbd { keys: "Mod+K", other: true } Kbd { keys: "G I" } Kbd { keys: "Esc", style: "flat" } Kbd { keys: "Enter", style: "inline" } Kbd { keys: "Mod+Enter", pressed: true } Kbd { keys: "Up" } Kbd { keys: "Alt+Shift+Down" }
                            }
                        }
                        Card { title: "separator · horizontal / vertical / labelled / dashed / fade / inset",
                            div { class: "stack",
                                Separator {} Separator { label: "Today", decorative: false } Separator { dashed: true } Separator { fade: true } Separator { inset: 32.0 }
                                div { class: "line h24", span { "Files" } Separator { vertical: true } span { "Edits" } Separator { vertical: true, dashed: true } span { "Runs" } }
                            }
                        }
                        Card { title: "spinner · arc (source) + dots / ray / orbit (Oxyn plus) · sizes · speed · arc length",
                            div { class: "line",
                                Spinner {} Spinner { size: 20.0, label: "Loading runs" } Spinner { size: 32.0, arc: 0.6 } Spinner { size: 48.0, speed_ms: 1600 }
                                Spinner { style: "dots", size: 20.0 } Spinner { style: "ray", size: 20.0 } Spinner { style: "orbit", size: 20.0 }
                                Button { label: "Inside a button", variant: "secondary", loading: true, loading_style: "spinner" }
                            }
                        }
                        Card { title: "number ticker · roll / count-up · currency · compact · new column slides in",
                            div { class: "line",
                                NumberTicker { ticker: ticker_up, now }
                                NumberTicker { ticker: ticker_mid, now, size: 22.0 }
                                span { class: "note", "← mid-roll frame" }
                                NumberTicker { ticker: ticker_cost, now, size: 22.0 }
                                NumberTicker { ticker: ticker_compact, now, size: 22.0 }
                                NumberTicker { ticker: TickerModel::new(0.986, NumberFormat::Percent(1)), now, size: 22.0 }
                            }
                        }
                        Card { title: "reaction bar · counts roll · pop + fill · compact · count rules (Oxyn plus)",
                            div { class: "stack",
                                ReactionBar { reactions: reactions.clone(), now }
                                ReactionBar { reactions: liked, now: ClockMs(10_100) }
                                ReactionBar { reactions: reactions.clone(), compact: true, rule: CountRule { from: 0, hide_zero: true, ..Default::default() } }
                            }
                        }
                        Card { title: "empty state · framed / bare / tall / inline · secondary action · starter prompts",
                            div { class: "stack",
                                EmptyState { state: empty_prompts }
                                EmptyState { state: EmptyModel { title: "No files".into(), text: "Drop files here or attach from the composer.".into(), layout: EmptyLayout::Bare, icon: "folder".into(), ..Default::default() } }
                                EmptyState { state: EmptyModel { title: "No matching rows".into(), text: "Clear the filter to see all 42.".into(), action: Some("Clear filter".into()), layout: EmptyLayout::Inline, icon: "filter".into(), ..Default::default() } }
                            }
                        }
                    }
                }
                div { class: "cols",
                    div { class: "col",
                        Card { title: "switch · start / end row · sm / md / lg · pressed stretch · pending · restart note",
                            div { class: "stack",
                                Switch { label: "Auto-approve reads", description: "Agents read files without asking", checked: true }
                                Switch { label: "Stream reasoning", placement: "end", description: "Show the model's thinking as it arrives" }
                                div { class: "line",
                                    Switch { label: "sm", size: "sm", checked: true } Switch { label: "md" } Switch { label: "lg", size: "lg", checked: true }
                                    Switch { label: "Pressed", state: "pressed" } Switch { label: "Focus", state: "focus", checked: true }
                                    Switch { label: "Applying", pending: true, checked: true } Switch { label: "Disabled", disabled: true }
                                }
                                Switch { label: "GPU acceleration", restart_note: true, placement: "end" }
                            }
                        }
                        Card { title: "checkbox · plain / card · indeterminate · error · sizes · tri-state (Oxyn plus)",
                            div { class: "stack",
                                Checkbox { label: "Select all files", value: "indeterminate" }
                                Checkbox { label: "src/main.rs", value: "checked", description: "+42 −7" }
                                Checkbox { label: "I understand this deletes the branch", error: "Tick this to continue" }
                                Checkbox { label: "Run tests after every edit", description: "Slower, but catches breakage early", variant: "card", value: "checked" }
                                div { class: "line", Checkbox { label: "sm", size: "sm", value: "checked" } Checkbox { label: "lg", size: "lg", value: "checked", speed: 2.0 } Checkbox { label: "Tri-state", tri_state: true, value: "indeterminate" } Checkbox { label: "Disabled", disabled: true } }
                            }
                        }
                        Card { title: "radio group · list / cards / inline · descriptions · disabled with reason",
                            div { class: "stack",
                                RadioGroup { label: "Approval mode", options: radio.clone(), value: "edits" }
                                RadioGroup { label: "Approval mode (cards)", options: radio.clone(), value: "ask", layout: "cards" }
                                RadioGroup { label: "Effort", options: vec![RadioOption::new("l", "Low"), RadioOption::new("m", "Medium"), RadioOption::new("h", "High")], value: "m", layout: "inline" }
                            }
                        }
                        Card { title: "segmented control · tray / underline · icons, counts, badges · sizes · 8 options · vertical",
                            div { class: "stack",
                                SegmentedControl { options: segs(3), value: "plan", aria_label: "Mode" }
                                SegmentedControl { options: seg_icons, value: "agent", size: "lg", aria_label: "Mode" }
                                SegmentedControl { options: seg_tabs, value: "running", variant: "underline", kind: "tabs", aria_label: "Filter runs" }
                                SegmentedControl { options: segs(8), value: "review", size: "sm", full_width: true, aria_label: "Mode" }
                                div { class: "line", SegmentedControl { options: segs(4), value: "agent", vertical: true, aria_label: "Mode" } SegmentedControl { options: segs(3), disabled: true, aria_label: "Mode" } }
                            }
                        }
                    }
                    div { class: "col",
                        Card { title: "select · placeholder / value / error / disabled · open with groups, icons, second lines",
                            div { class: "grid2",
                                Select { options: models.clone(), label: "Model", hint: "Applies to new runs" }
                                Select { options: models.clone(), value: "sonnet", label: "Model", optional: true }
                                Select { options: models.clone(), label: "Fallback model", error: "Choose a fallback", required: true }
                                Select { options: models.clone(), value: "haiku", label: "Locked", disabled: true }
                            }
                            div { class: "grid2 menuroom big",
                                Select { options: models_dis.clone(), value: "sonnet", label: "Open", open: true }
                                Select { options: models_dis.clone(), label: "Searchable (Oxyn plus)", searchable: true, open: true, size: "sm" }
                            }
                            div { class: "grid2 menuroom",
                                Select { options: models.clone(), label: "Loading (Oxyn plus)", loading: true, open: true, size: "lg" }
                            }
                        }
                        Card { title: "slider · single / range · marks · named detents + cost tag (Oxyn plus) · vertical · disabled",
                            div { class: "stack gap16",
                                Slider { slider: effort, label: "Effort", tag: "≈ $0.42 / run", active: 0 }
                                Slider { slider: temp, label: "Temperature", decimals: 1 }
                                Slider { slider: budget, label: "Monthly budget", format: "$#", thumb_labels: vec!["Minimum".into(), "Maximum".into()] }
                                div { class: "line", Slider { slider: vol, label: "Volume", format: "#%" } Slider { slider: off, label: "Disabled" } }
                            }
                        }
                        Card { title: "text field · hint / error / prefix / suffix / clear / copy / password / counter",
                            div { class: "grid2",
                                TextField { label: "Repository", placeholder: "owner/name", hint: "We clone it read-only", prefix: "icon:branch" }
                                TextField { label: "Email", value: "ada@", kind: "email", error: "Enter an email like name@example.com" }
                                TextField { label: "Budget", value: "250", prefix: "$", suffix: "/ month", optional: true }
                                TextField { label: "Search runs", kind: "search", value: "flaky", clearable: true, prefix: "icon:search", state: "focus" }
                                TextField { label: "API key", value: "sk-live-8f2a…", copyable: true, state: "hover" }
                                TextField { label: "Password", kind: "password", value: "hunter22", auto_hide_s: 10, hint: "Hides again after 10 s" }
                                TextField { label: "Commit message", value: "fix: retry flaky network calls in the sync worker", counter: true, max: 50, soft: 40 }
                                TextField { label: "Disabled", value: "read-only", disabled: true, size: "sm" }
                            }
                        }
                        Card { title: "tooltip · sides · shortcut · rich (Oxyn plus) · popover · arrow · hover intent",
                            div { class: "line tips",
                                Tooltip { content: "Copy", open: true, IconButton { icon: "copy", label: "Copy" } }
                                Tooltip { content: "Search", shortcut: "Mod+K", side: "bottom", open: true, IconButton { icon: "search", label: "Search" } }
                                Tooltip { content: "Branch from here", side: "right", open: true, IconButton { icon: "branch", label: "Branch" } }
                                Tooltip { content: "Runs the full suite on a fresh worktree.", title: "Run tests", side: "bottom", open: true, delay: 300, Button { label: "Run tests", variant: "secondary", size: "sm" } }
                            }
                            div { class: "line popspace",
                                Popover { trigger: rsx! { Button { label: "Share", variant: "secondary", size: "sm", icon: "share" } }, trigger_label: "Share", open: true, title: "Share this thread", description: "Anyone with the link can read it.", width: 260.0, arrow: true,
                                    TextField { label: "Link", value: "oxyn.dev/t/8f2a", copyable: true, size: "sm" }
                                }
                                div { class: "push" }
                                Popover { trigger: rsx! { IconButton { icon: "info", label: "Details" } }, trigger_label: "Details", mode: "hover", title: "Hover intent (Oxyn plus)", description: "Opens after 150 ms, closes 300 ms after you leave.", side: "left",
                                    span {}
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

const PAGE_CSS: &str = r#"
html,body{margin:0;background:var(--oxyn-color-bg-base)}
.page{padding:32px;width:1500px;font-family:Inter,system-ui,sans-serif;color:var(--oxyn-color-fg-default)}
header h1{margin:0;font-size:24px;font-weight:600}
header p{margin:4px 0 22px;color:var(--oxyn-color-fg-subtle);font-size:13px}
.cols{display:grid;grid-template-columns:1fr 1fr;gap:18px;margin-bottom:18px;align-items:start}
.col{display:flex;flex-direction:column;gap:18px;min-width:0}
.card{background:var(--oxyn-color-surface-base);border:1px solid var(--oxyn-color-border-subtle);border-radius:16px;padding:18px}
.card h2{margin:0 0 14px;font-size:11.5px;letter-spacing:.06em;text-transform:uppercase;color:var(--oxyn-color-fg-muted);font-weight:600}
.line{display:flex;flex-wrap:wrap;align-items:center;gap:10px;margin-bottom:12px}
.line:last-child{margin-bottom:0}
.line.tall{min-height:60px;align-items:flex-end}
.stack{display:flex;flex-direction:column;gap:12px;align-items:flex-start}
.stack>.oxyn-sep,.stack>.oxyn-empty,.stack>.oxyn-switch.end,.stack>.oxyn-check.card,.stack>.oxyn-radio,.stack>.oxyn-seg.full,.stack>.oxyn-slider{align-self:stretch}
.gap16{gap:22px}
.grid2{display:grid;grid-template-columns:1fr 1fr;gap:14px;margin-bottom:12px}
.menuroom{min-height:240px;align-items:flex-start}
.menuroom.big{min-height:400px}
.push{flex:1}
.note{font-size:11.5px;color:var(--oxyn-color-fg-subtle)}
.h24{height:24px;font-size:13px;color:var(--oxyn-color-fg-muted)}
.prose{font-size:14px;line-height:1.9;margin:6px 0 14px;color:var(--oxyn-color-fg-muted);min-height:96px}
.ctxbox{display:flex;align-items:center;justify-content:center;height:90px;border:1px dashed var(--oxyn-color-border-default);border-radius:10px;color:var(--oxyn-color-fg-subtle);font-size:13px}
.ctxcard{min-height:520px}
.tips{gap:110px;padding:34px 8px 60px}
.popspace{min-height:230px;align-items:flex-start}
"#;

fn main() {
    let mut args = std::env::args().skip(1);
    let tokens = args
        .next()
        .and_then(|p| std::fs::read_to_string(p).ok())
        .unwrap_or_default();
    let theme = args.next().unwrap_or_else(|| "dark".into());
    let mut dom = VirtualDom::new(Sheet);
    dom.rebuild_in_place();
    let body = dioxus_ssr::render(&dom);
    println!(
        "<!doctype html><html data-oxyn-theme=\"{theme}\"><head><meta charset=utf-8><style>{tokens}{PRIMITIVES_CSS}{PAGE_CSS}</style></head><body>{body}</body></html>"
    );
}
