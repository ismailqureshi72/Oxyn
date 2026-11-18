**Status:** built v0.1 (2026-11-18, S7: Settings II).

# Spec: Settings · Security (`settings-security`)

Family: Screens. Reference: affectUI Settings / Security (`/screens/settings-security`), layout and behaviour only; no source or assets copied.
Components: `SecuritySettingsScreen` (Dioxus) · `SecuritySettingsEl` (GPUI), models in `oxyn_core::parity::setscr2`.

## 1. Purpose
How you and everyone here sign in, and what happened lately.

## 2. Parts
| Part | Look |
|---|---|
| Sidebar (272) | Settings shell: Back to app, a search over every setting that counts matches per page (Enter opens the first), then the Personal group and the Workspace group of pages with this one current, and the account row with its menu |
| Column | Centred, 696 px: heading, one-line lead, then sections of rows in cards |
| Your sign-in | Two-step sign-in with a three-step setup: a real, scannable QR code (the `otpauth://totp/…` link for the demo key, encoded with the `qrcodegen` crate, MIT, and drawn as one SVG path in Dioxus and a painted path in GPUI) and its typed key, a six-digit code check, then ten recovery codes to copy or download; Recovery codes (show, new codes); passkeys with Remove and Add passkey |
| Workspace sign-in | Require two-step for everyone; Require single sign-on (needs a verified domain, else "Verify a domain first" blocks Save) with the Google Workspace mark and Entity ID / Reply URL to copy; Session length; Join by domain |
| Email domains | Each domain Verified or Pending with its TXT record and Verify; Add domain with checks |
| Signed-in devices | Person, device, place and when; Sign out, and Sign out everyone else (asks first) |
| Audit log | Filter chips (All, Sign-in, Members, Keys, Billing) and events with a status dot, who, where and when; Export |

Chrome is monochrome neutral; colour only means something: official marks (model providers, Google), identity hues and status (needs-you amber, done green, failed red, running blue).

## 3. Behaviour
Policies go through the save bar; setup, passkeys, domains and devices act at once and are announced. Every other settings page opens in place from the sidebar.
prefers-reduced-motion: no fills or transitions.

## 4. Variants
States: Data · Loading · Empty · **Setting up (the two-step setup open at its first step)**. Frame: Inset · Flat. Oxyn extras: `compact` and `autosave`.

## 5. Keyboard and accessibility
The sidebar is a labelled nav with aria-current on this page. Every control has a name; a field with a problem is aria-invalid with its message beside it. Segments and filters are radiogroups; destructive actions ask first and focus the safe choice. Every action reports what it did in a polite live region; skeletons are aria-hidden.

## 6. Responsive
Authored on the 1440 × 900 stage (rule R1) and scaled to its column; the full view (`/screens/settings-security/full`) renders at the real viewport with the state bar under it (R2). Under 640 px the settings sidebar slides in from a menu button. Under 760 px plan cards stack and meters go two by two; under 560 px tables drop their extra columns and rows put their control under the label. Every label ellipsises in a flex child with min-width 0 (R5); scripts/ovl.py finds no text overlap at 1440, 768 and 390 in light and dark.

## 7. Known gaps
GPUI is a static snapshot: `.extra(...)` draws the screen's own state or `compact`; typing, menus and timers belong to the host. Sample people, keys, cards, domains and the workspace are fictional; model versions are fictional, marks are the providers' official ones. There is no backend: invites, keys, payments and sign-in changes are simulated.
