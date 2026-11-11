# Website stack

**Decision (proposed 2026-11-11): hybrid.** A static **Astro + Starlight** site as the shell, with
**Rust → WASM** demos that load only when they scroll into view, and a **Dioxus web app** for the
Studio. Components stay 100 % Rust; the site shell is the only non-Rust part.

Precedent: gpui-kit (the main GPUI component library) moved its docs to Astro with Pagefind search,
WASM previews and `llms-full.txt`.

## Why not all-Rust (Dioxus / Leptos / Zola for the whole site)?
- Docs tooling is weaker: no MDX, thinner search/i18n/a11y plugins.
- Dioxus SSG had an open `dx bundle --ssg` bug (issue #5050) at the time of research, and ships WASM on
  every page, which hurts first paint and SEO.
- We still use Rust where it matters: every demo and the whole Studio.

## Stack by layer

| Layer | Choice | Why | Alternative |
|---|---|---|---|
| Framework | Astro 7, static output | Zero JS by default, fast first paint, good SEO | Dioxus 0.7 SSG, Zola |
| Docs / MDX | Starlight + custom pages (landing, gallery, pricing) | Mature MDX, search, i18n, a11y | Fumadocs, mdBook |
| Demo runtime | 3 levels: **(0)** PNG/WebM poster rendered headlessly from GPUI, **(1)** live Dioxus web demo, **(2)** GPUI-on-web canvas on click | GPUI on the web works but is heavy (a full app measured ~3.9 MB compressed) and needs WebGPU | egui (WebGL fallback), Rune later |
| Embedding | `<oxyn-demo>` custom element, mounts when visible, one cached WASM per framework, pauses off-screen | No React runtime on docs pages | Astro `client:visible` island |
| Studio | Dioxus web app at `/studio`; shares `oxyn-codegen` with the CLI so exports match byte for byte; full state in the URL | One source of truth, live preview without recompiling | React + Leva shell (Paper Shaders style) |
| Search | Pagefind with family / category / state / gesture filters | Static, free | Algolia DocSearch |
| Tokens | `tokens/*.json` → Rust theme, CSS variables, GPUI theme JSON via build script | Docs, Studio and code share one file | Tailwind-first theming |
| Registry / CLI | Rust `oxyn` CLI via GitHub Releases; free registry as static JSON at `/r/{name}.json` (shadcn-like item shape) | Free, cacheable, agent-friendly | Registry served from a GitHub repo |
| Pro registry | Cloudflare Worker checks `Authorization: Bearer <licence>` against Polar, caches, streams source from private R2 | Same pattern as shadcn private registries | Self-signed Ed25519 keys |
| Auth | None in v1 — the licence key is the credential; Polar's customer portal handles buyers | Zero auth code | Better Auth / Clerk later |
| Payments | **Polar** (merchant of record, built-in licence keys, 5 % + 50¢, +1.5 % non-US cards) | Pays out to Pakistan via Stripe Connect Express; Stripe direct does not onboard Pakistani sellers | Paddle; Lemon Squeezy not recommended (moving to Stripe Managed Payments without licence keys) |
| Analytics | Cloudflare Web Analytics + Polar sales + install counter in the registry Worker | Free, cookieless | Plausible / Umami |
| Hosting | Cloudflare Workers static assets + R2 (posters, Pro source) | Free tier allows commercial use; 25 MiB per file | GitHub Pages (repo is public), Netlify. Vercel Hobby is non-commercial only |
| CI | GitHub Actions — each job builds and deploys itself (`wrangler deploy`); posters → R2; binaries → Releases | **Never uses `upload-artifact`** | Cloudflare Workers Builds |
| AI-agent access | `starlight-llms-txt`, "Copy prompt" / "Copy Markdown" per page, remote MCP on a Worker + `oxyn mcp` stdio mode | Coding agents can find and install components | MCP in CLI only |

## Monorepo layout (target)

```
oxyn/
├─ Cargo.toml  rust-toolchain.toml
├─ crates/     oxyn-core · oxyn-tokens · oxyn-registry-schema · oxyn-codegen · oxyn-gpui · oxyn-dioxus · oxyn-cli
├─ registry/   free/<comp>/{meta.toml,gpui.rs,dioxus.rs,story.rs} · pro/ (private submodule)
├─ apps/       gallery-dioxus-web · gallery-gpui-web · studio
├─ tokens/
├─ site/       Astro + Starlight (OxynDemo.astro, gallery, public/wasm)
├─ workers/    registry-pro · mcp
├─ xtask/      build-registry | render-posters | build-wasm | release
└─ .github/workflows/  ci · deploy-site · posters · release-cli
```

## Risks
- **GPUI on the web is experimental** (single-threaded on stable, treat as WebGPU-only, ~82–89 % browser
  support). Posters + Dioxus demos are the default; GPUI web is opt-in.
- **Two renderers won't match pixel-for-pixel.** GPUI posters are the reference look.
- **Free-tier limits:** Workers 100k requests/day and 10 ms CPU each.
- **Polar payouts** need a Pakistani bank account in PKR.
- **Pro code can be shared after install** — handled by licence terms and activation limits.

Full research with sources: kept in the project research notes (`18-website-stack`).
