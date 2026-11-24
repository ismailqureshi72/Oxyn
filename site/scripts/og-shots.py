#!/usr/bin/env python3
# Link-preview cards from real screenshots (public/og.png and public/og/<card>.png, 1200 x 630, PNG,
# each under 300 KB). Screenshots the live components and screens on a running site, then lays out
# the Oxyn lockup, a title and a line on the left and the screenshot on a grey panel on the right.
# Bump OG_V in src/lib/site.mjs after rerunning so apps that cached the old cards fetch the new ones.
#
#   OXYN_PREVIEW=https://oxynui.com python3 scripts/og-shots.py      (needs playwright + Pillow)
#   CHROMIUM=/usr/bin/chromium picks a browser; --compose-only reuses the last screenshots;
#   --thumbs-only only rewrites the 640-wide WebP previews (public/og/t/) after og-cards.sh.
import asyncio, json, os, sys
from PIL import Image, ImageDraw, ImageFilter, ImageFont

HERE = os.path.dirname(os.path.abspath(__file__))
PUB = os.path.join(HERE, '..', 'public')
SHOTS = os.environ.get('OXYN_OG_SHOTS', '/tmp/oxyn-og-shots')
BASE = os.environ.get('OXYN_PREVIEW', 'https://oxynui.com').rstrip('/')
FREE = len(json.load(open(os.path.join(HERE, '..', 'src', 'lib', 'free-tier.json')))['free'])

# What to screenshot: whole screens (the /full/ view at 1440 x 900) and single components (the
# component itself on its item page, without the playground controls).
SCREENS = ['chat-agents', 'kanban'] if os.environ.get('OXYN_OG_SCREENS', '1') == '1' else []
COMPONENTS = ['components/agents/approval-card', 'components/agents/agent-plan', 'components/agents/tool-call',
              'components/chat/reasoning-block', 'blocks/agents/agent-routines', 'blocks/agents/background-agents']

def font(weight, size):
    for f in [f'/usr/share/fonts/opentype/inter/Inter-{weight}.otf',
              f'/usr/share/fonts/truetype/liberation/LiberationSans-{"Bold" if weight != "Regular" else "Regular"}.ttf']:
        if os.path.exists(f): return ImageFont.truetype(f, size)
    return ImageFont.load_default()

async def shoot():
    from playwright.async_api import async_playwright
    os.makedirs(SHOTS, exist_ok=True)
    async with async_playwright() as p:
        exe = os.environ.get('CHROMIUM')
        b = await p.chromium.launch(**({'executable_path': exe} if exe else {}), args=['--no-sandbox'] + (['--no-proxy-server'] if '127.0.0.1' in BASE or 'localhost' in BASE else []))
        ctx = await b.new_context(viewport={'width': 1440, 'height': 900}, device_scale_factor=2, color_scheme='light')
        # Only the site itself: a third-party beacon must not hold up the load event.
        await ctx.route(lambda u: not u.startswith(BASE), lambda r: r.abort())
        pg = await ctx.new_page()
        async def open_(u):
            await pg.goto(BASE + u, wait_until='domcontentloaded', timeout=60000)
            await pg.evaluate("sessionStorage.setItem('oxyn-intro','1');localStorage.setItem('oxyn-announce','closed');localStorage.removeItem('starlight-theme')")
            await pg.goto(BASE + u, wait_until='load', timeout=60000)
            await pg.wait_for_timeout(2500)
        for s in SCREENS:
            await open_(f'/screens/{s}/full/')
            # The screen's own frame, without the gallery's state bar under it.
            bb = await pg.evaluate("(()=>{const e=document.querySelector('oxyn-demo .oxd-mount > *')||document.querySelector('oxyn-demo');const r=e.getBoundingClientRect();return {x:r.x,y:r.y,width:r.width,height:Math.min(r.height,innerHeight-r.y)}})()")
            await pg.screenshot(path=f'{SHOTS}/screen-{s}.png', clip=bb)
        for c in COMPONENTS:
            await open_(f'/{c}/')
            bb = await pg.evaluate("(()=>{const d=document.querySelector('oxyn-demo');const e=d.querySelector('.oxd-mount > *')||d;scrollTo(0,scrollY+e.getBoundingClientRect().top-140);const r=e.getBoundingClientRect();const P=20;return {x:r.x-P,y:r.y-P,width:r.width+2*P,height:Math.min(r.height+2*P,innerHeight-r.y+P)}})()")
            await pg.wait_for_timeout(1200)
            await pg.screenshot(path=f'{SHOTS}/comp-{c.split("/")[-1]}.png', clip=bb)
        await b.close()

W, H = 1200, 630
INK, MUTED, FAINT, PANEL, LINE = (11, 11, 11), (85, 85, 79), (138, 138, 132), (242, 242, 240), (226, 226, 222)

def rounded(im, r):
    m = Image.new('L', im.size, 0)
    ImageDraw.Draw(m).rounded_rectangle([0, 0, im.size[0] - 1, im.size[1] - 1], r, fill=255)
    out = Image.new('RGBA', im.size); out.paste(im, (0, 0), m); return out

def place(card, shot, x, y, w, crop_h=None, r=14):
    """A screenshot scaled to width w at (x, y), cut at crop_h, with a hairline and a soft shadow."""
    im = Image.open(shot).convert('RGB')
    h = round(im.size[1] * w / im.size[0])
    im = im.resize((w, h), Image.LANCZOS)
    if crop_h and crop_h < h: im = im.crop((0, 0, w, crop_h)); h = crop_h
    sh = Image.new('RGBA', (w + 60, h + 60), (0, 0, 0, 0))
    ImageDraw.Draw(sh).rounded_rectangle([30, 36, 30 + w, 36 + h], r, fill=(0, 0, 0, 34))
    card.alpha_composite(sh.filter(ImageFilter.GaussianBlur(14)), (x - 30, y - 30))
    framed = Image.new('RGB', (w, h), LINE); framed.paste(im.crop((1, 1, w - 1, h - 1)), (1, 1))
    card.alpha_composite(rounded(framed, r), (x, y))

def wrap(d, text, f, width):
    lines, cur = [], ''
    for word in text.split():
        t = (cur + ' ' + word).strip()
        if d.textlength(t, font=f) <= width: cur = t
        else: lines.append(cur); cur = word
    return lines + [cur]

def card(name, title, line, shots):
    c = Image.new('RGBA', (W, H), (255, 255, 255, 255))
    d = ImageDraw.Draw(c)
    d.rounded_rectangle([520, 40, W + 40, H + 40], 32, fill=PANEL)
    for args in shots: place(c, *args)
    # The lockup, title and line sit on white, left of the panel.
    lock = Image.open(os.path.join(HERE, 'assets', 'og-lockup.png')).convert('RGBA')
    lock = lock.resize((round(lock.size[0] * 40 / lock.size[1]), 40), Image.LANCZOS)
    c.alpha_composite(lock, (64, 72))
    ft = font('Bold', 60 if len(title) < 12 else 50)
    d.text((62, 210), title, font=ft, fill=INK)
    fl = font('Regular', 27)
    for i, ln in enumerate(wrap(d, line, fl, 410)): d.text((64, 300 + i * 38), ln, font=fl, fill=MUTED)
    d.text((64, 548), 'oxynui.com', font=font('Medium', 23), fill=FAINT)
    out = os.path.join(PUB, 'og.png' if name == 'home' else f'og/{name}.png')
    c.convert('RGB').quantize(colors=256, method=Image.Quantize.MEDIANCUT, dither=Image.Dither.NONE).save(out, optimize=True)
    kb = os.path.getsize(out) / 1024
    print(f'{out}: {kb:.0f} KB')
    assert kb < 300, f'{out} is {kb:.0f} KB (limit 300)'

def compose():
    S = lambda n: os.path.join(SHOTS, n + '.png')
    card('home', 'Oxyn UI', 'Rust UI components for AI agents, for Dioxus and GPUI.', [(S('screen-chat-agents'), 572, 96, 860)])
    card('components', 'Components', 'Chat, tool calls, approvals and plans for Dioxus and GPUI.',
         [(S('comp-approval-card'), 572, 96, 520), (S('comp-tool-call'), 572, 418, 520), (S('comp-reasoning-block'), 572, 490, 520)])
    card('screens', 'Screens', 'Whole agent app screens, built from the components.', [(S('screen-kanban'), 572, 96, 860)])
    card('blocks', 'Blocks', 'Agent and AI surfaces, ready to drop in.',
         [(S('comp-background-agents'), 572, 96, 560, 300), (S('comp-agent-routines'), 660, 330, 520, 280)])
    card('pricing', 'Pricing', f'{FREE} components free under MIT. Pro opens every screen and block.',
         [(S('comp-agent-plan'), 572, 96, 520), (S('comp-approval-card'), 700, 380, 440)])
    card('docs', 'Docs', 'Install, theme and ship Oxyn UI in Rust.',
         [(S('comp-reasoning-block'), 572, 96, 560), (S('comp-approval-card'), 640, 300, 480)])
    card('docs-page', 'Oxyn UI docs', 'Guides and reference for agent interfaces in Rust.',
         [(S('comp-agent-plan'), 572, 96, 600)])

def thumbs():
    """640-wide WebP copies of every card (public/og/t/), for the <img> previews on the landing."""
    os.makedirs(os.path.join(PUB, 'og', 't'), exist_ok=True)
    for f in sorted(os.listdir(os.path.join(PUB, 'og'))):
        if not f.endswith('.png'): continue
        out = os.path.join(PUB, 'og', 't', f[:-4] + '.webp')
        Image.open(os.path.join(PUB, 'og', f)).convert('RGB').resize((640, 336), Image.LANCZOS).save(out, 'WEBP', quality=82, method=6)
        print(f'{out}: {os.path.getsize(out) / 1024:.0f} KB')

if __name__ == '__main__':
    if '--compose-only' not in sys.argv and '--thumbs-only' not in sys.argv: asyncio.run(shoot())
    if '--thumbs-only' not in sys.argv: compose()
    thumbs()
