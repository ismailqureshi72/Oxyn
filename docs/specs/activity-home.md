**Status:** built v0.1 (2026-11-15, plan #28).

# Spec: Activity home (`activity-home`)

Family: App shells & screens. Closes the parity row(s): X #44 @flohoeller AI chat composer & overview; X #53 Threadbear Librarian home. Behaviour-only rebuild; nothing is copied.

## 1. Purpose
A home under the composer slot. `overview`: Activity (the compact heat map with totals), Recent chats, Usage against the plan (rolling number, a bar that turns to the warning tone at 90 %, reset date), Files made for you and This month with rolling values and signed deltas. `librarian`: a background agent's card (Idle, Paused or Looking at everything, with Pause / Resume and Look at everything, which runs for 2.4 s and then adds to the looks count), a quote that refreshes in place, and the year heat map with navigation. Cards stagger in once (60 ms apart).

## 2. States
| State | Looks | Leaves on |
|---|---|---|
| overview | Five cards | — |
| near limit | Usage bar in the warning tone | credits below 90 % |
| librarian idle | Grey dot | Pause / Look |
| looking | Pulsing dot, button disabled | 2.4 s |
| paused | Amber dot, Resume | Resume |

## 3. Timing and motion
- Cards rise 320 ms, 60 ms apart; numbers roll 260 ms; the usage bar grows over 900 ms.
- Reduced motion: everything lands at once.

## 4. Variants
| Variant | What differs |
|---|---|
| overview | Composer over the five cards |
| near-limit | Usage past 90 % |
| librarian | Greeting, status, quote, year heat map |

## 5. Keyboard
| Key | Does |
|---|---|
| Tab | Through the cards' buttons and lists |

## 6. Accessibility
- Cards are sections; usage is role=meter with the percent in its label; the Librarian status is role=status.

## 7. Responsive
- Container query on a full-width root: six columns, four under 860 px, two under 620 px.

## 8. Props (Dioxus)
| Prop | Type | Default | Notes |
|---|---|---|---|
| variant | overview · librarian | overview |  |
| greeting / date | String | — | empty hides |
| credits | (u32, u32) | (1284, 2000) |  |
| counts | Vec<u32> | — | per day, last is today |
| year | i32 | 2026 |  |
| now | ClockMs | 0 | Librarian timing |
| children | Element | — | composer slot |
| on_open | EventHandler<String> | — | recent chat or file |

## 9. Gaps against the source
- The composer menus in the reference (model, add, approval mode, folder) are the existing composer parts and are slotted, not rebuilt here.
- Recent chats and files are demo content.

## 10. GPUI
ActivityHomeEl renders the Activity, Usage and This month cards from the same helpers.
