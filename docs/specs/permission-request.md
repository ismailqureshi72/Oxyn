**Status:** built v0.1 (2026-11-14, plan #28).

# Spec: Permission request (`permission-request`)

Family: G Approvals ([approvals.md](approvals.md)). Closes the parity row(s): affectUI · Permission Request.

## 1. Purpose
One agent asks for one specific right: who (agent, model), what (“wants to push to main”), where (target with the tool's mark), why (its own words) and exactly what will happen (preview rows with right-aligned meta, “N more”). Allow it once, for a while (default 60 min) or always; after a decision it folds into one line with the time left and Revoke (or Ask again once expired or denied). Destructive requests turn Allow red and leave out Always.

Anatomy:
```
R Release bot wants to push to main               risky
  ⌁ acme/web · Claude Sonnet 5.5
│ CI is green on release/2.4 …
WHAT WILL HAPPEN
  ⑂ Merge release/2.4 into main          +412 −96
  ✓ Tag v2.4.0
  2 more
( Once | For 1 h | Always )            Deny [Allow]
— folded —
✓ Release bot · push to main · acme/web     Allowed · 58 min left  Revoke
```

## 2. States
| State | Looks | Leaves on |
|---|---|---|
| waiting | Full card | allow → folded; deny → folded (Denied) |
| folded active | Check, line, time left, Revoke | revoke → waiting; expiry → Expired |
| folded denied / expired | Ban icon, Ask again | ask again → waiting |

## 3. Timing and motion
- Folding fades the line in (240 ms); time left ticks with the host clock.
- Reduced motion: no fade.

## 4. Variants
| Variant | What differs |
|---|---|
| risky | Three scopes (default) |
| destructive | Red Allow, no Always |
| timed | Folded with time left |
| once | One scope hides the switch |

## 5. Keyboard
| Key | Does |
|---|---|
| Tab | Scope radios, Deny, Allow |
| Escape | Deny |

## 6. Accessibility
- Group named “<agent> wants to <action>”; scopes are a radiogroup; folded line is role=status.

## 7. Responsive
- Max 440 px; the footer wraps when narrow.

## 8. Props
| Prop | Type | Default | Notes |
|---|---|---|---|
| req | PermissionRequest | — | agent, model, action, target, risk, reason, preview, preview_limit, scopes, scope, duration_min, decision |
| now | ClockMs | — |  |
| preview_title | String | What will happen |  |
| on_scope | EventHandler<Scope> | — |  |
| on_allow / on_deny / on_revoke / on_more | EventHandler<()> | — |  |

## 9. Edge cases
- Choosing Always then switching to destructive falls back to Once.
- Timed grants under a minute show seconds.

## 10. Implementation
- Core: `PermissionRequest`, `PermRisk`, `Scope`, `PermDecision`, `PreviewRow` with tests.
- Dioxus: `PermissionRequest`.
- GPUI: `PermissionRequestEl`.

## 11. References
- affectUI Permission Request (https://www.affectui.com/components/permission-request) — behaviour-only rebuild.
