#!/usr/bin/env bash
# Section link-preview cards (public/og/<section>.png, 1200 x 630): the Oxyn lockup from og.png,
# the section name and one line under it. Used as og:image, image-sitemap entries and the <img>
# previews on the landing. Rerun after editing a line: bash scripts/og-cards.sh (needs ImageMagick).
set -euo pipefail
cd "$(dirname "$0")/.."
B=/usr/share/fonts/truetype/liberation/LiberationSans-Bold.ttf
R=/usr/share/fonts/truetype/liberation/LiberationSans-Regular.ttf
lock=$(mktemp --suffix=.png)
magick public/og.png -crop 590x160+305+210 +repage -resize 50% "$lock"
card() { # slug, title, line
  magick -size 1200x630 xc:'#FFFFFF' \
    \( "$lock" \) -geometry +88+76 -composite \
    -fill '#F2F2F0' -draw 'roundrectangle 64,190 1136,566 28,28' \
    -fill '#0B0B0B' -font "$B" -pointsize 76 -annotate +120+330 "$2" \
    -fill '#55554F' -font "$R" -pointsize 34 -annotate +122+410 "$3" \
    -fill '#8A8A84' -font "$R" -pointsize 26 -annotate +122+510 "oxynui.com  ·  Oxyn UI" \
    -strip -define png:compression-level=9 "public/og/$1.png"
}
card components  'Components'          'Rust UI components for AI agents, Dioxus and GPUI'
card screens     'Screens'             'Whole agent app screens, built from the components'
card blocks      'Blocks'              'Agent and AI surfaces, ready to drop in'
card pricing     'Pricing'             'Free components, MIT. Pro for every screen and block'
card docs        'Docs'                'Install, theme and ship Oxyn UI in Rust'
card cli         'oxyn CLI'            'cargo install oxyn-cli, then oxyn add <component>'
card mcp         'MCP server'          'Let your coding agent add Oxyn UI components'
card dioxus      'Dioxus components'   'Shadcn-style AI agent UI components for Dioxus'
card gpui        'GPUI components'     'Native desktop agent UI components for GPUI'
card about       'About Oxyn UI'       'The Rust UI component library for AI agents'
card guides      'Guides'              'Build an AI agent UI in Rust, step by step'
rm -f "$lock"
