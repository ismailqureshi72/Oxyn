**Status:** v1 — implemented (plan #27).

# Spec: code block

> Implemented in `oxyn-core::primitives::chrome (`highlight`, `Tok`, `code_collapses`, `language_label`)`, `oxyn-dioxus::CodeBlock`, `oxyn-gpui::CodeBlockEl`.
> Source: Arc UI Code block (behaviour only, rebuilt from the public docs; no source used). Header layout ported from Beautiful UI Code Block (MIT, © 2026 Shane Levine).

## 1. Purpose
Code in answers and tool output: readable, copyable, collapsible, and still streaming.

## 2. State machine
```
Collapsed(max_lines) ─Show all─▶ Expanded ─Show less─▶ Collapsed
Wrap off ─toggle─▶ wrap on      streaming ─▶ caret after the last line
```

## 3. States table
| State | Shows | Keyboard | ARIA |
|---|---|---|---|
| collapsed | first n lines, fade | Tab to Show all | `aria-expanded=false` |
| expanded | all lines | same | `aria-expanded=true` |
| streaming | blinking caret | — | caret hidden |

## 4. Micro-interactions
New lines rise in; the toggle label rolls and its chevron rotates; copy feedback comes from the copy button.

## 5. Timing table
| Transition | Duration | Easing | Reduced motion |
|---|---|---|---|
| Line in | 240 ms | `ease.out` | instant |
| Toggle roll | 200 ms | `ease.out` | swap |
| Caret | 1 s steps | — | static |

## 6. Variants
Free: filename header, language label, copy, Show all.

**Oxyn plus:** line numbers, highlighted lines, `diff` colouring, wrap toggle, streaming caret, `bare` (no header).

## 7. Edge cases
Long lines scroll horizontally unless wrapped. The highlighter is a light tokenizer (strings, numbers, comments, keywords, calls, types); unknown languages fall back to plain text.

## 8. Accessibility
A `section` labelled “file source code”; the `pre` is focusable; Show all has `aria-controls`.

## 9. Props sketch
```rust
CodeBlock { code, filename: Option<String>, language: String, max_lines: usize, line_numbers: bool,
            highlight_lines: Vec<usize>, wrap: bool, streaming: bool, variant: "default"|"bare" }
```

## 10. Related components and references
`streaming-markdown.md`, `diff-viewer.md`, `copy-button.md`. Arc UI Code block — behaviour only; Beautiful UI Code Block — MIT.
