**Status:** v1.1 — implemented (plan #23, Arc gaps closed in plan #26).

# Spec: avatar with agent ring (and avatar group)

> Implemented in `oxyn-core::primitives::display` (`Avatar`, `AvatarShape`, `Presence`,
> `AgentRing`, `agent_mark`, `agent_cell_opacity`, `avatar_overflow`, `AVATAR_SIZES`),
> `oxyn-dioxus::{Avatar, AvatarGroup}`, `oxyn-gpui::AvatarEl`. Sources: affectUI Avatar, Arc
> Avatar (behaviour only).

## 1. Purpose
Who is acting: a person, a workspace or an agent. In agent apps several agents and people share a
thread, so the avatar also has to say *what the agent is doing right now* without words: the ring
around it follows `AgentState`.

## 2. State machine
Image: `NoSrc → Initials`; `Src → Loading (initials shown) → Loaded (picture fades in over them)`;
`Error → Initials`. Ring: `AgentRing::of(&AgentState)`:

| AgentState | Ring | Motion |
|---|---|---|
| Idle, Composing, Cancelled | none | — |
| Submitted, Thinking, Steering | `orbit`: a 22 % arc going round | 1.2 s / turn |
| Streaming | `breathe`: full ring | opacity 1 ↔ 0.35, 2 s |
| ToolRunning | `dashed` ring turning | 3 s / turn |
| AwaitingApproval, Queued | `waiting` (waiting colour), steady | — |
| Done | `done` (success), one flash in | 600 ms once |
| Error | `error` (danger), steady | — |
| Stalled | `stalled` (stall colour) | — |
| (host) | `progress(p)`: arc from 12 o'clock | follows `p` on `spring.follow` |

The agent's pixel mark twinkles while `working` (ring is orbit, breathe or dashed).

## 3. States table
| State | Shows | ARIA |
|---|---|---|
| Person, no picture | initials on the seeded monogram colour | `role=img`, name |
| Person, picture loading / loaded | initials, then picture fades in from a 4 px blur | name |
| Workspace | rounded square (25 % radius) | name |
| Agent | 5×5 mirrored pixel mark from the seed on a rounded square | "Coder, working" |
| Presence | dot bottom-right: online green, away amber, busy red, offline hollow | ", online" appended |
| Ring | per the table above | ring word appended ("waiting for you", "failed") |
| Model badge | corner tag ("S", "4o") | hidden (decorative; name it in the label) |
| Group | overlapping avatars, `+N` tile | "Ada, Grace and 2 more" |

## 4. Micro-interactions
- Picture fades in once loaded, over the initials — never an empty circle while it loads; a cached
  picture shows at once (Arc).
- Agent mark: cells fade down and back in a diagonal wave, 1.2 s (affectUI), each cell offset by
  (row + col) × 150 ms.
- Presence dot pops in on `spring.snap` when it appears or changes.
- Group members joining or leaving slide the stack instead of jumping; the `+N` count rolls.

## 5. Timing table
| Transition | Duration | Easing | Reduced motion |
|---|---|---|---|
| Picture fade-in | 180 ms + blur 4 → 0 | `ease.out` | 80 ms fade, no blur |
| Agent wave | 1200 ms loop | sine | still (full opacity) |
| Orbit ring | 1200 ms / turn | linear | static arc |
| Dashed ring | 3000 ms / turn | linear | static dashes |
| Breathe ring | 2000 ms loop | sine | steady ring |
| Done flash | 600 ms once | `ease.out` | none |
| Progress ring | 362 ms (`spring.follow`) | spring | jumps |
| Presence pop | 214 ms (`spring.snap`) | spring | none |

## 6. Variants
Source: `name`, `src`, `size` (32), `shape` circle / square, `status` online / away / busy (Arc
offline), `kind` person / agent, `seed`, `color`, `working`; group `max` (4), `size`, `total`.

**Oxyn plus:**
1. State-driven agent ring (8 looks) — the "agent ring" of plan #23.
2. Progress ring around any avatar (`AgentRing::Progress`).
3. Seeded agent identicon (`agent_mark`), mirrored, never empty or solid.
4. Seven sizes 12 / 16 / 24 / 32 / 40 / 56 / 96 (`AVATAR_SIZES`).
5. Model badge in the corner; offline presence (Arc) alongside the affectUI three.

## 7. Edge cases
Tiny sizes (12–16 px): the presence dot is at least 6 px; the ring pad shrinks to 2 px; initials
drop to one letter's width. Broken image URLs fall back to initials. Very long names only affect the
accessible name. Groups with `total` larger than the list show the remainder in `+N`. In dense
lists, the ring animations count against the shared motion clock (orbit at 60 Hz; the clock's
unfocused / low-power caps apply).

## 8. Accessibility
`role=img` with a full name: "{name}, {presence}, {ring word}". Pictures have empty `alt` (the
container is named). Groups are `role=group` named with the visible names and "and N more". Ring
colours always come with the word, never colour alone.

## 9. Props sketch
```rust
Avatar { avatar: Avatar { name, src, size, shape, status, agent, seed, colour, working, ring, model } }
Avatar::person("Ada"), Avatar::agent("Coder", &AgentState::Streaming)
AvatarGroup { avatars, max: 4, total }
```

## 10. Related components and references
`status-indicator.md` (same state words), `badges-chips.md` (monogram colours), `animated-marks.md`,
`acting-indicator.md`. affectUI `/components/avatar`, Arc `/components/avatar` — behaviour only.

## Plan #26 additions
- Arc UI size presets via `Avatar::preset_size`: `sm` 28, `md` 40, `lg` 56, `xl` 88 px.
- A photo that fails to load is removed and the initials stay (Arc "falls back to initials").
- Presence `online` / `offline` (and Oxyn's `away` / `busy`) appends the state to the label; the dot
  is decorative.
