# Contributing to Oxyn

Oxyn is built by two maintainers — **@ismailqureshi72** and **@Khubaib7-del**. These rules keep the
history tidy and make sure both of us are credited on every change.

## 1. Never push to `main`

`main` is protected by convention: all work lands through a pull request.

```bash
git switch main && git pull
git switch -c <type>/<short-topic>     # e.g. docs/composer-spec, feat/core-state
```

Branch types: `feat/`, `fix/`, `docs/`, `design/`, `research/`, `chore/`, `ci/`.

## 2. Commit format

[Conventional Commits](https://www.conventionalcommits.org/) + a co-author trailer on **every** commit.

```
<type>(<scope>): <summary in imperative, ≤ 72 chars>

<optional body: what and why>

Co-authored-by: Khubaib Nazeer <khubaibnazeer8@gmail.com>
```

When Khubaib is the author, the trailer credits Ismail instead:

```
Co-authored-by: Muhammad Ismail Qureshi <254414836+ismailqureshi72@users.noreply.github.com>
```

Types: `feat`, `fix`, `docs`, `design`, `research`, `refactor`, `perf`, `test`, `chore`, `ci`.
Scopes: `core`, `tokens`, `gpui`, `dioxus`, `cli`, `codegen`, `site`, `studio`, `registry`, `brand`, or a
component name (`composer`, `loader`, ...).

### Commit template (recommended)

Save this as `~/.gitmessage-oxyn` and run `git config commit.template ~/.gitmessage-oxyn` inside the repo:

```
type(scope): summary


Co-authored-by: Khubaib Nazeer <khubaibnazeer8@gmail.com>
```

## 3. Pull requests

- One topic per PR; keep commits small and meaningful (no "wip" commits on merge).
- Fill in the PR template, including the **co-author checklist**.
- **Merge with "Create a merge commit"** (preferred) so each commit and its trailer survive. If you
  squash, keep every `Co-authored-by:` line in the final squash message.
- The other maintainer reviews and merges. Alternate who authors PRs, so both of us collect
  co-author credit on merged PRs over time.

> Why so strict about trailers? GitHub's *Pair Extraordinaire* achievement counts merged PRs that
> contain co-authored commits (tiers at 1, 10, 24 and 48 PRs), and the trailer email must belong to the
> co-author's GitHub account. It also keeps the history honest about who built what.

## 4. CI rules

- No `actions/upload-artifact` in any workflow (artifact storage is not available on our plan).
- Builds that must be kept go to **GitHub Releases** or Cloudflare R2, never Actions artifacts.
- Add `[skip ci]` to the commit message for docs-only commits once CI exists.

## 5. Licensing of references

Read [docs/licensing.md](docs/licensing.md) before porting anything. Short version: only MIT/Apache code
may be ported, with attribution; everything else (paid kits, AGPL, unclear licences) is study-only.

## 6. Style

- Rust: `cargo fmt`, `cargo clippy -- -D warnings`, stable toolchain.
- Docs: sentence-case headings, absolute dates (`2026-11-11`), link sources.
- Every component: all agent states, reduced-motion fallback, keyboard access, light + dark.
