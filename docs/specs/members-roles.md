**Status:** built v0.1 (2026-11-20, S15: Arc screens IV and Agent / Tool calls).

# Spec: Members (`members-roles`)

Family: Arc screens (pages at `/screens/members-roles/` and `/full`). Reference: the Arc demo app's members page, behaviour and layout only, under our own design; no source, artwork or copy taken. Every company, person, key, endpoint and amount is fictional; dates are absolute ISO dates and times are 24-hour UTC.
Components: `MembersRolesScreen` (Dioxus) · `MembersRolesEl` (GPUI), rules in `oxyn_core::parity::arcscr4`.

## 1. Purpose
Bring people in with the right role and keep the seat count honest.

## 2. Parts
The Arc frame (shared with the S12–S14 screens): a 248 px sidebar (Corvel Labs workspace, Search ⌘K, nav rows Overview · Audit log · Status · API keys · Members · Billing, an **Agents** group with Harbor and its Claude mark, a context card with the plan and seats, the account row with name and **email** and its menu), the page as an inset card (8 px gutter) with a 53 px header (breadcrumb, meta, actions) and an optional second row, and an optional right card in the gutter.

| Part | Look |
|---|---|
| Header | Role filter Everyone · Admins · Members · Viewers, Invite |
| Seats | A meter: seats used (members and pending invites) of the plan's seats, amber from 90% |
| Invite form | Email (checked: format, already a member or invited, seats left) and Role (Admin, Member, Viewer), Send invite |
| Table | Avatar, name and email, the role as a button (steps down a role; the only owner keeps Owner), two-step sign-in On / Off, last active ISO date, Remove (asks again) |
| Pending invites | Email, role and sent date, Resend, Revoke |

## 3. Behaviour
`Members` in `oxyn_core::parity::arcscr4`: `invite_error`, `send()`, `cycle_role(i)`, `remove(i)` (two presses, never the only owner), `resend(id)`, `revoke(id)`.

## 4. Variants
States: Data · Loading · Empty (Just you so far) · **Invite open** (a half-typed email with its error) · **Seats full** (the invite refuses) · **Admins only**. Frame: Inset · Flat.

## 5. Keyboard and accessibility
The seat bar is role meter with its value; invite errors are alerts; role buttons name the person, the role and the change. Every action says itself back in the frame's polite live region. Focus rings show for the keyboard only.

## 6. Responsive
Authored on the 1440 × 900 stage and scaled to its column; `/full` renders at the real viewport. Under 960 px the sidebar is a drawer from the header's menu button and the right card hides; under 640 px header actions become icon buttons. scripts/ovl.py checks text overlap at 1440, 768 and 390 in light and dark.

## 7. Data
`MembersRolesScreen` takes `layout` and `state`; sample data comes from `oxyn_core::parity::arcscr4`.

## 8. Known gaps
No email is sent.

## 9. GPUI
`MembersRolesEl` draws one state; with `.arc4(Arc4Ui)` it draws the live state. In `ScreenView` (set S15 Arc IV): `mb-invite`, `mb-email`, `mb-irole-<n>`, `mb-send`, `mb-role-<i>`, `mb-rm-<i>`, `mb-resend-<id>`, `mb-revoke-<id>`, `mb-filter-<n>`, `mb-all`. `cargo run --manifest-path examples/gpui-agent-apps/Cargo.toml` opens it from the strip.
