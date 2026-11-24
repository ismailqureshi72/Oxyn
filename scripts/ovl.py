#!/usr/bin/env python3
# Overlap + overflow guard for the screen demos on a running site preview (Playwright + Chromium).
# Usage: OXYN_PREVIEW=http://localhost:4321 python3 scripts/ovl.py [slugs,...|all|full] [themes] [widths]
#   OXYN_PAGES="name=/path/ ..." adds any page; OXYN_COOKIES="k=v; k2=v2" sets cookies (signed-in checks).
#   OXYN_FAKE_API=file.json answers /api/* GETs from that file ({"/api/auth/me": {...}, ...}) so signed-in
#   pages can be checked against a static preview (pair it with OXYN_COOKIES="oxyn_signed_in=1").
#   all  = every screen page plus the agent orchestrator page; full = every screen's /full/ view;
#   pages = only the OXYN_PAGES below; landing = the landing page: its demos plus the site chrome marked data-ovl (navbar, question pills, footer).
#   Slugs come from OXYN_SLUGS (space separated), else from the links on the preview's /screens/ index.
#   Writes ovl_<all|full|some>.json to OXYN_OVL_OUT (default: current directory); exit 1 when anything is found.
# Per rendered demo (oxyn-demo): spill = elements outside the demo box; overlap = pairs of visible text
# runs whose boxes intersect (each run clipped by its overflow ancestors; ancestor/descendant pairs and
# aria-hidden / role=menu|dialog popovers skipped); clip = text cut off without an ellipsis.
import asyncio, json, os, re, sys, urllib.request
from playwright.async_api import async_playwright
BASE = os.environ.get('OXYN_PREVIEW', 'http://localhost:4321').rstrip('/')
OUT = os.environ.get('OXYN_OVL_OUT', '.')
def screens():
    if os.environ.get('OXYN_SLUGS'): return os.environ['OXYN_SLUGS'].split()
    html = urllib.request.urlopen(BASE + '/screens/').read().decode()
    seen = []
    for m in re.findall(r'href="/screens/([a-z0-9-]+)/"', html):
        if m not in seen: seen.append(m)
    return seen
arg = sys.argv[1] if len(sys.argv) > 1 else 'all'
SCREENS = [] if arg in ('landing', 'pages') else screens()
slugs = SCREENS if arg == 'all' else arg.split(',')
def path(s):
    if s == 'orchestrator': return '/components/tools/agent-orchestrator/'
    if s.endswith(':full'): return '/screens/' + s[:-5] + '/full/'
    return '/screens/' + s + '/'
if arg == 'full':
    slugs = [x + ':full' for x in SCREENS]
if arg in ('landing', 'pages'):  # pages: only OXYN_PAGES
    slugs = []
PAGES = {s: path(s) for s in slugs}
if arg == 'landing':
    PAGES['landing'] = '/'
# Any other pages: OXYN_PAGES="login=/login/ account=/account/" (checked as well as the above).
for kv in os.environ.get('OXYN_PAGES', '').split():
    k, _, u = kv.partition('='); PAGES[k] = u
if arg == 'all':
    PAGES['orchestrator'] = '/components/tools/agent-orchestrator/'
# OXYN_OVL_SEL="main" checks more than the demos and data-ovl chrome (e.g. a whole page's main panel).
SEL = ','.join(['oxyn-demo', '[data-ovl]'] + ([os.environ['OXYN_OVL_SEL']] if os.environ.get('OXYN_OVL_SEL') else []))
FAKE = json.load(open(os.environ['OXYN_FAKE_API'])) if os.environ.get('OXYN_FAKE_API') else None
THEMES = sys.argv[2].split(',') if len(sys.argv) > 2 else ['light', 'dark']
WIDTHS = [int(x) for x in sys.argv[3].split(',')] if len(sys.argv) > 3 else [1440, 768, 390]
TH = "(t)=>{const d=document.documentElement;d.dataset.theme=t;d.setAttribute('data-oxyn-theme',t);document.querySelectorAll('.pv').forEach(p=>{p.dataset.theme=t;p.setAttribute('data-oxyn-theme',t)})}"
JS = r"""(SEL)=>{
const out=[];
const vis=e=>{for(let x=e;x&&x.nodeType==1;x=x.parentElement){const s=getComputedStyle(x);if(s.display=='none'||s.visibility=='hidden'||+s.opacity===0)return false;}return true};
const name=e=>{const c=(e.className&&e.className.baseVal===undefined)?e.className:'';return (c||e.tagName.toLowerCase()).toString().split(' ')[0]};
for(const pv of document.querySelectorAll(SEL)){
 const b=pv.getBoundingClientRect(); if(!b.width) continue;
 const spill=new Set(), clip=new Set(), runs=[];
 for(const e of pv.querySelectorAll('*')){
  const x=e.getBoundingClientRect(); if(!x.width||!x.height) continue;
  if(e.closest('[aria-hidden=true],[role=menu],[role=dialog],[role=listbox],[role=tooltip]')) continue;
  if(e.closest('details:not([open]) > :not(summary)')) continue; // collapsed answer: not rendered
  if(!vis(e)) continue;
  let xl=x.left,xr=x.right;for(let a=e.parentElement;a&&a!==pv;a=a.parentElement){const cs=getComputedStyle(a);if(cs.overflowX!='visible'){const q=a.getBoundingClientRect();xl=Math.max(xl,q.left);xr=Math.min(xr,q.right);}}
  if(xr>b.right+1||xl<b.left-1) spill.add(name(e)+' '+Math.round(x.left-b.left)+'..'+Math.round(x.right-b.left)+'/'+Math.round(b.width));
  const s=getComputedStyle(e);
  const own=[...e.childNodes].filter(n=>n.nodeType==3&&n.textContent.trim());
  if(!own.length) continue;
  if(e.scrollWidth>e.clientWidth+2&&s.overflowX!='visible'&&s.textOverflow!='ellipsis'&&s.clipPath=='none'&&!(e.clientWidth<=1)) clip.add(name(e));
  // each line box of the element's own text, clipped by every overflow ancestor inside the demo
  const cl=[];for(let a=e;a&&a!==pv;a=a.parentElement){const cs=getComputedStyle(a);if(cs.overflowX!='visible'||cs.overflowY!='visible')cl.push(a.getBoundingClientRect());}
  const rs=[];
  for(const n of own){const r=document.createRange();r.selectNodeContents(n);for(const q of r.getClientRects()){let L=q.left,T=q.top,R=q.right,B=q.bottom;for(const c of cl){L=Math.max(L,c.left);T=Math.max(T,c.top);R=Math.min(R,c.right);B=Math.min(B,c.bottom);}if(R-L>=1&&B-T>=1)rs.push([L,T,R,B]);}}
  if(rs.length) runs.push({e,rs,t:own.map(n=>n.textContent.trim()).join(' ').slice(0,28)});
 }
 const ov=[];
 for(let i=0;i<runs.length;i++)for(let j=i+1;j<runs.length;j++){
  const a=runs[i],c=runs[j]; if(a.e.contains(c.e)||c.e.contains(a.e)) continue;
  let hit=null;for(const p of a.rs)for(const q of c.rs){const w=Math.min(p[2],q[2])-Math.max(p[0],q[0]),h=Math.min(p[3],q[3])-Math.max(p[1],q[1]);if(w>2&&h>2)hit=[w,h];}
  if(hit) ov.push(`"${a.t}" x "${c.t}" ${Math.round(hit[0])}x${Math.round(hit[1])}`);
 }
 out.push({v:pv.getAttribute('data-variant')??(pv.tagName.toLowerCase()+'.'+String(pv.className).split(' ').pop()),spill:[...spill].slice(0,8),clip:[...clip].slice(0,8),overlap:ov.slice(0,10),n:ov.length});
}
return {doc:document.documentElement.scrollWidth,W:innerWidth,demos:out}}"""
async def main():
    res = {}; bad = 0
    async with async_playwright() as p:
        exe = os.environ.get('CHROMIUM')
        b = await p.chromium.launch(**({'executable_path': exe} if exe else {}), args=['--no-sandbox'] + os.environ.get('OXYN_CHROMIUM_ARGS', '').split())
        for th in THEMES:
            for w in WIDTHS:
                ctx = await b.new_context(viewport={'width': w, 'height': 900}, color_scheme=th)
                # OXYN_COOKIES="name=value; name2=value2": e.g. a session cookie to check signed-in pages.
                ck = [c.split('=', 1) for c in os.environ.get('OXYN_COOKIES', '').split(';') if '=' in c]
                if ck: await ctx.add_cookies([{'name': k.strip(), 'value': v.strip(), 'url': BASE} for k, v in ck])
                await ctx.route('**/cloudflareinsights.com/**', lambda r: r.abort())  # analytics beacon
                if FAKE:
                    async def api(route):
                        path = route.request.url.split('://', 1)[1].split('/', 1)[1].split('?')[0]
                        body = FAKE.get('/' + path)
                        if body is None: return await route.fulfill(status=404, content_type='application/json', body='{"ok":false}')
                        await route.fulfill(status=200, content_type='application/json', body=json.dumps(body))
                    await ctx.route('**/api/**', api)
                pg = await ctx.new_page()
                for k, u in PAGES.items():
                    await pg.goto(BASE + u); await pg.wait_for_timeout(1000)
                    await pg.evaluate(TH, th)
                    for y in range(0, 24000, 700):
                        await pg.evaluate(f"scrollTo(0,{y})"); await pg.wait_for_timeout(50)
                    await pg.wait_for_timeout(900)
                    r = await pg.evaluate(JS, SEL); res[f'{k} {w} {th}'] = r
                    issues = [d for d in r['demos'] if d['spill'] or d['overlap'] or d['clip']]
                    if r['doc'] > r['W'] + 1 or issues: bad += 1
                    print(k, w, th, 'doc', r['doc'], '/', r['W'], 'demos', len(r['demos']),
                          '|', '; '.join(f"v{d['v']}: spill={d['spill'][:3]} clip={d['clip'][:3]} ov{d['n']}={d['overlap'][:3]}" for d in issues) or 'clean', flush=True)
                await ctx.close()
        await b.close()
    json.dump(res, open(os.path.join(OUT, 'ovl_') + (arg if arg in ('all', 'full', 'landing') else 'some') + '.json', 'w'), indent=1)
    print('BAD', bad)
    return bad
sys.exit(1 if asyncio.run(main()) else 0)
