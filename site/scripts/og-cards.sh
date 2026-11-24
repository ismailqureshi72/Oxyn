#!/usr/bin/env bash
# Text-only section link-preview cards (public/og/<section>.png, 1200 x 630): the Oxyn lockup, the
# section name and one line under it. The landing, Components, Screens, Blocks, Pricing and Docs cards
# are real screenshots instead (scripts/og-shots.py, which also writes public/og/t/*.webp). Used as og:image, image-sitemap entries and the <img>
# previews on the landing. Rerun after editing a line: bash scripts/og-cards.sh (needs ImageMagick).
set -euo pipefail
cd "$(dirname "$0")/.."
B=/usr/share/fonts/truetype/liberation/LiberationSans-Bold.ttf
R=/usr/share/fonts/truetype/liberation/LiberationSans-Regular.ttf
lock=$(mktemp --suffix=.png)
magick scripts/assets/og-lockup.png -resize x80 "$lock"
card() { # slug, title, line
  magick -size 1200x630 xc:'#FFFFFF' \
    \( "$lock" \) -geometry +88+76 -composite \
    -fill '#F2F2F0' -draw 'roundrectangle 64,190 1136,566 28,28' \
    -fill '#0B0B0B' -font "$B" -pointsize 76 -annotate +120+330 "$2" \
    -fill '#55554F' -font "$R" -pointsize 34 -annotate +122+410 "$3" \
    -fill '#8A8A84' -font "$R" -pointsize 26 -annotate +122+510 "oxynui.com  ·  Oxyn UI" \
    -strip -define png:compression-level=9 "public/og/$1.png"
}
card cli         'oxyn CLI'            'cargo install oxyn-cli, then oxyn add <component>'
card mcp         'MCP server'          'Let your coding agent add Oxyn UI components'
card dioxus      'Dioxus components'   'Shadcn-style AI agent UI components for Dioxus'
card gpui        'GPUI components'     'Native desktop agent UI components for GPUI'
card about       'About Oxyn UI'       'The Rust UI component library for AI agents'
card guides      'Guides'              'Build an AI agent UI in Rust, step by step'
rm -f "$lock"
