**Status:** built v0.1 (2026-11-18, S8: Onboarding wizard).

# Spec: Onboarding · Ready (`onboarding-ready`)

Family: Screens. Reference: affectUI Onboarding / Ready (`/screens/onboarding-ready`), layout and behaviour only; no source or assets copied.
Components: `OnboardingReadyScreen` (Dioxus) · `OnboardingReadyEl` (GPUI), rules in `oxyn_core::kit::onbscr`.

## 1. Purpose
Step 4 of 4: check what the assistant knows and make it live.

## 2. Parts
| Part | Look |
|---|---|
| Top bar (56) | Product mark, Back (from step 2), the signed-in address and Log out |
| Steps | Four dashes under the top, the done ones filled; each done step is a button back to it (Oxyn `named_steps` names them) |
| Form column (720) | Centred, 440 px: heading, one-line lead, then the step's fields |
| Preview (720) | The assistant's dashboard as a picture: a 232 px side with the brand mark and company, then Overview tiles (Knowledge, Conversations, Status), Your assistant, Knowledge and Test chat. It glides to the part this step changes; the others dim |
| Footer (48) | © 2026 Oxyn, Terms, Privacy |
| Summary | Rows: Learns from, Brand, Name (with the mark), Voice, Hands over to; each with Edit (Add when that step still needs something) |
| Open dashboard | Full width; disabled while a step is missing, with a note saying which and why |

Chrome is monochrome neutral; colour only means something: the brand colour the user picks, and status (learning blue, ready green, failed red, warnings amber).

## 3. Behaviour
Edit goes back to that step with everything kept; the dashes let you jump forward again to any step already reached. Open dashboard turns the assistant from Draft to Live in the preview (status tile and pill turn green), shows a confirmation and calls `on_done`.
Back and Continue move through the four steps in place, keeping every value; Log out reports and calls `on_logout`. A polite live region reports what each action did.
prefers-reduced-motion: the preview jumps instead of gliding; pulses and typing dots stop.

## 4. Variants
States: Data · Loading · Empty · **Live (published)**. Frame: Inset · Flat (the dashboard picture as a rounded panel or edge to edge). Oxyn extras: `named_steps` (each step named under its dash) and `form_only` (no preview).

## 5. Keyboard and accessibility
The steps are an ordered list with aria-current=step on the current one, each named "Step n of 4: …". Fields have labels; a problem sets aria-invalid and shows its message as an alert. Colour, mark, tone and length are radiogroups; the handover is a switch. The preview is decorative (aria-hidden): everything in it is in the form too. Skeletons are aria-hidden.

## 6. Responsive
Authored on the 1440 × 900 stage (rule R1) and scaled to its column; the full view (`/screens/onboarding-ready/full`) renders at the real viewport with the state bar under it (R2). Under 960 px the preview leaves the page and opens over the form from Preview in the top bar (with Close). Under 620 px the signed-in address hides, marks go two by two and summary rows tighten. Labels ellipsise in flex children with min-width 0 (R5); scripts/ovl.py finds no text overlap at 1440, 768 and 390 in light and dark.

## 7. Known gaps
GPUI is a static snapshot: `.extra(...)` draws the own state, `named` or `form-only`; typing and ticks belong to the host. There is no backend: reading sites, uploading a logo and publishing are simulated with fictional page counts. The company, people and addresses are fictional (`kestrel.example`).
In a window, `ScreenView` (oxyn-gpui, `kit::screenview`) runs this screen live from `oxyn_core::kit::screenui`: the sidebar and page links open their screens (across sets), buttons open their state, the bar above it switches the set's screens and the state bar, a command bar (a real text field) jumps to any screen or state, the account row opens its menu, and Left / Right, Up / Down and Escape step states and screens. Editing the page's own fields in place is still the host's.
