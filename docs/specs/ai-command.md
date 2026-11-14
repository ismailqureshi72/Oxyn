**Status:** v1 — implemented (plan #26).

# Spec: AI command

> Implemented in `oxyn-core::primitives::command` (`AiCommand`, `AiMode`, `AnswerStream`, `AnswerPhase`, `AiAction`, `Source`), `oxyn-dioxus::AiCommand`, `oxyn-gpui::AiCommandEl`.
> Source: Arc UI AI command (behaviour only, no source used).

## 1. Purpose
One shortcut for both finding and asking: jump to a page or run an action, or ask a question about the workspace and act on the answer without leaving the palette.

## 2. State machine
```
Search(query) ─Tab─▶ Ask(query as question) ─Enter─▶ Thinking(900 ms) ─▶ Streaming ─▶ Done
Done ─↑ ↓─▶ next step active ─Enter─▶ Running(900 ms) ─▶ Done(label → done label)
Ask ─Tab─▶ Search
Esc: answer open → close answer; Ask → Search; Search → close
```

## 3. States table
| State | Shows | Keyboard | ARIA |
|---|---|---|---|
| search | grouped pages and actions; "Ask AI: …" row | ↑ ↓ Enter Tab | `combobox` + `listbox` |
| ask, empty | suggested questions | ↑ ↓ Enter | — |
| thinking | shimmering status | — | answer region `aria-busy` |
| streaming | sources, text with a caret, citations | — | footer status polite |
| done | next steps with Enter hints | ↑ ↓ Enter | — |
| step running / done | spinner → check, label → done label | — | — |

## 4. Micro-interactions
The palette never resizes: views crossfade inside a fixed body. Sources pop in as they are first cited; citations pop as they stream. Steps show a spinner then a check.

## 5. Timing table
| Transition | Duration | Easing | Reduced motion |
|---|---|---|---|
| View crossfade | 160 ms + 60 ms delay | `ease.out` | instant |
| Thinking | 900 ms | — | same |
| Stream | ~45 chars / s | linear | full text at once |
| Step run | 900 ms | — | same |

## 6. Variants
Free: search mode, ask mode, suggested questions, model name.

**Oxyn plus:** citations that light their source, runnable next steps with done labels, scripted answers for demos, shared clock for web and desktop.

## 7. Edge cases
Empty questions don't ask. Asking again replaces the answer. Very long answers scroll inside the fixed body. Running a step twice is prevented.

## 8. Accessibility
Combobox for the field; the answer is a region with `aria-busy` while streaming; the footer status announces Thinking, Answer ready and step results. Citations are buttons named "Source n: <title>".

## 9. Props sketch
```rust
AiCommand { items: Vec<Command>, suggested_questions, model, default_mode: "search"|"ask",
            default_query, thinking_label, height, elevated, answer: ScriptedAnswer, autoplay,
            now: ClockMs, width, on_select: EventHandler<String>, on_ask: EventHandler<String> }
```

## 10. Related components and references
`command-palette.md`, `ai-search-bar.md`, `citations-sources.md`. Arc UI `/docs/components/ai-command` — behaviour only.
