# Site search (F1)

The ⌘K palette finds any component, screen, agent app, docs page or library page from part of its
name, a word inside it or a misspelling of it.

## Index
- `site/src/pages/search-index.json.ts` writes `/search-index.json` at build time from the same data
  the pages use (`src/lib/data.ts`, the Starlight docs collection). One row per item:
  `t` name, `u` url, `g` group (`c` component, `s` screen, `a` agent app, `d` docs, `p` page),
  `k` category or app, `d` one short line, `w` keywords (slug words, family, old category, app name,
  aliases, docs headings).
- About 85 KB raw, 24 KB gzipped for 411 items. The page fetches it once when idle (or on open) and
  keeps it in memory; it revalidates with an ETag like every other unhashed file.

## Matching (public/site.js)
The query and every name are lowercased with punctuation folded to spaces. Score, highest first:
1. exact name, 2. name starts with the query, 3. a word of the name starts with the query,
4. the name contains the query (3+ letters), then token by token over name and keywords:
   exact word, word prefix, inside a word (3+ letters), one typo for 5+ letters or two for 8+
   (Damerau, so swapped letters count as one; the word must be at least as long as the token).
   Every token has to land somewhere; a 4+ letter token found only in the short line adds a little.
Components get a small boost over screens, agent apps, docs and library pages; shorter names
win ties. Groups are ordered by their best hit, capped at 8 components and 4–5 of the others.

## Palette
- Opens from every search button, ⌘K / Ctrl+K and `/` (when not typing), on every page including
  the Starlight docs. ⌘K is bound on `window` in the capture phase so a live demo cannot take it.
- Rows: group icon, name with the matched letters underlined, short line, category on the right.
- ↑ ↓ move, ↵ opens, ⌘↵ opens in a new tab, esc closes; the pointer moves the active row.
- Empty query: Recent (last five opened, `localStorage['oxyn-recent-search']`) and Suggestions.
  No results: the query echoed and suggestion chips that fill the field.
- A11y: input `role=combobox` with `aria-expanded`, `aria-controls`, `aria-autocomplete=list` and
  `aria-activedescendant`; results `role=listbox` → `role=group` (labelled by the group heading) →
  `role=option`; a polite status line reads the result count.
- Phones (≤ 640 px): a full-screen sheet with a Cancel button; 16 px input so iOS does not zoom.

## Why not Pagefind
Pagefind indexed page bodies by whole stemmed words: partial names and typos missed or hit noise,
component pages had no type so the Components filter came back empty, and the palette waited on a
wasm index before it could show anything. Pagefind no longer runs in the build.
