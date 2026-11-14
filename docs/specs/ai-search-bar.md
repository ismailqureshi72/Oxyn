**Status:** v1 — implemented (plan #26).

# Spec: AI search bar

> Implemented in `oxyn-core::primitives::command` (`AiSearch`, `AnswerStream`), `oxyn-dioxus::AiSearchBar`, `oxyn-gpui::AiSearchBarEl`.
> Source: Arc UI AI search bar block (behaviour only, no source used).

## 1. Purpose
Search over docs, customers and issues where a short synthesized answer is often better than ten links, but the links must stay one step away.

## 2. State machine
```
Empty ─▶ recent searches + suggested questions
Typing ─▶ grouped results, first row "Ask AI about …"
Tab / Enter on the Ask row ─▶ Thinking ─▶ Streaming ─▶ Done (citations badge their results)
Esc: answer → results; results → clear
```

## 3. States table
| State | Shows | Keyboard | ARIA |
|---|---|---|---|
| empty | Recent, Ask AI suggestions | ↑ ↓ Enter | combobox |
| results | Ask row + grouped results | ↑ ↓ Enter Tab | options |
| answer | streamed text with [n] citations | — | region `aria-busy` |
| cited result | number badge, lit while its citation is hovered | — | "Cited as n" |

## 4. Micro-interactions
The card keeps its size; the answer slides in above the results. Hovering or focusing a citation lights the result it points to.

## 5. Timing table
| Transition | Duration | Easing | Reduced motion |
|---|---|---|---|
| Answer in | 200 ms | `ease.out` | instant |
| Stream | ~45 chars / s | linear | full text |
| Citation light | 160 ms | linear | same |

## 6. Variants
Free: recent, suggested questions, grouped results, answer.

**Oxyn plus:** result badges for citations, remembered queries, shared answer engine with AI command.

## 7. Edge cases
Queries with no results still offer Ask AI. Recent searches de-duplicate and keep five. Long result labels truncate.

## 8. Accessibility
The field is a combobox; results are options in labelled groups; the answer region is busy while streaming; citations and badges are labelled.

## 9. Props sketch
```rust
AiSearchBar { results: Vec<Command>, recent, suggestions, model, placeholder, query, autoplay,
              answer: ScriptedAnswer, height, width, now: ClockMs, on_select: EventHandler<String> }
```

## 10. Related components and references
`ai-command.md`, `search-field.md`, `citations-sources.md`. Arc UI blocks `/docs/blocks/ai-search-bar` — behaviour only.
