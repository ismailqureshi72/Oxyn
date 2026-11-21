**Status:** built v0.1 (2026-11-19, S14: Arc screens III).

# Spec: Sign-in split (`sign-in-split`)

Family: Arc screens (pages at `/screens/sign-in-split/` and `/full`). Reference: the sign-in split screen of Arc UI (uiarc.dev), behaviour and layout only, under our own design; no source, artwork or copy taken. Every company, ticker, product, client and person is fictional; dates are absolute ISO dates and times are 24-hour.
Components: `SignInSplitScreen` (Dioxus) · `SignInSplitEl` (GPUI), rules in `oxyn_core::kit::arcscr3`.

## 1. Purpose
Sign in to a workspace with SSO or email and password, with clear errors.

## 2. Parts
A split page of its own (no sidebar): the form side and an art panel, inside the same `oxs12` chrome variables.

| Part | Look |
|---|---|
| Top bar | Harrowgate mark and name, "No account?" and Sign up |
| Form (380 px) | Welcome back, a line, **Continue with Google** (official colour mark) and **Continue with GitHub** (official mark in the text colour), "or", Email and Password with labels, Forgot password?, Show password, field errors under each field, the form error, **Sign in**, the demo account with Fill in, the terms line with an ISO date |
| Art panel | A neutral panel of rings with a customer quote card (fictional) |

## 3. Behaviour
`SignIn` in `oxyn_core::kit::arcscr3`: checks run on submit (email shape, password of 8 or more characters); the demo account signs in, anything else counts a wrong try ("2 tries left"), and three wrong tries lock the form ("Reset your password to continue"). An SSO press shows that provider busy and disables the others; pressing it again finishes.

## 4. Variants
States: Data (the demo email filled) · Loading · Empty (a blank form) · **Invalid** (bad email, short password) · **Wrong password** · **Signing in** (Continuing with GitHub). Frame: Inset · Flat.

## 5. Keyboard and accessibility
Inputs have labels, aria-invalid and their message tied by aria-describedby; the form error is an alert; busy buttons set aria-busy; the spinner slows under reduced motion; Enter submits.

## 6. Responsive
Authored on the 1440 × 900 stage and scaled to its column; `/full` renders at the real viewport. scripts/ovl.py finds no text overlap at 1440, 768 and 390 in light and dark. Under 860 px the art panel steps aside and the form fills the page; under 420 px the "No account?" text hides.

## 7. Data
`SignInSplitScreen` takes `layout` and `state`. The demo account is `DEMO_EMAIL` / `DEMO_PASS`.

## 8. Known gaps
No real authentication; SSO buttons only show the busy state.

## 9. GPUI
`SignInSplitEl` draws one state; fields show their values (Fill in sets them). In `ScreenView`: `si-fill`, `si-show`, `si-submit`, `si-sso-<n>`, `si-finish`. The set's tabs, state bar, command bar and account menu work as for the other sets; `cargo run --manifest-path examples/gpui-agent-apps/Cargo.toml` opens it from the strip (S14 Arc III).
