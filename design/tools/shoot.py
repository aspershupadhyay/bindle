"""Render .dc.html artboards to PNG with a tiny stand-in for the x-dc runtime.

usage: python shoot.py <project dir> <out dir> Name.dc.html[:suffix=json-state] ...
env: W (viewport width), DPR, BLOBMAP
"""
import re, sys, pathlib, asyncio, os, json
from playwright.async_api import async_playwright

ROOT = pathlib.Path(sys.argv[1]); OUT = pathlib.Path(sys.argv[2]).resolve(); jobs = sys.argv[3:]
F = pathlib.Path.home() / "fonts"
FACES = f"""@font-face{{font-family:Inter;src:url('file://{F}/Inter.ttf');font-weight:100 900}}
@font-face{{font-family:Newsreader;src:url('file://{F}/Newsreader.ttf');font-weight:200 800}}
@font-face{{font-family:'JetBrains Mono';src:url('file://{F}/JBMono.ttf');font-weight:100 800}}
@font-face{{font-family:Silkscreen;src:url('file://{F}/Silkscreen.woff2');font-weight:400}}
@font-face{{font-family:Silkscreen;src:url('file://{F}/Silkscreen-700.woff2');font-weight:700}}
@font-face{{font-family:VT323;src:url('file://{F}/VT323.woff2')}}
@font-face{{font-family:Caveat;src:url('file://{F}/Caveat.ttf');font-weight:400 700}}"""
BM = json.load(open(os.environ['BLOBMAP'])) if os.environ.get('BLOBMAP') else {}
W = int(os.environ.get('W', '1440'))

RUNTIME = r"""
class DCLogic { constructor(p){ this.props = p || {}; this.state = {}; } setState(p){ this.state = Object.assign({}, this.state, typeof p === 'function' ? p(this.state) : p); window.__render(); } forceUpdate(){ window.__render(); } }
function look(scope, path){ path = path.trim(); if (path === 'true') return true; if (path === 'false') return false; if (/^-?\d+(\.\d+)?$/.test(path)) return +path; if (/^'.*'$/.test(path)) return path.slice(1,-1);
  return path.split('.').reduce((o,k)=> o == null ? undefined : o[k], scope); }
const HOLE = /\{\{([^}]+)\}\}/g;
function interp(s, scope){ return s.replace(HOLE, (_, p) => { const v = look(scope, p); return v == null ? '' : String(v); }); }
function proc(node, scope){
  if (node.nodeType === 3){ if (HOLE.test(node.nodeValue)) { HOLE.lastIndex = 0; node.nodeValue = interp(node.nodeValue, scope); } HOLE.lastIndex = 0; return; }
  if (node.nodeType !== 1) return;
  const tag = node.localName;
  if (tag === 'sc-for'){
    const list = look(scope, node.getAttribute('list').replace(/[{}]/g,'')) || []; const as = node.getAttribute('as') || 'item';
    const frag = document.createDocumentFragment();
    list.forEach((it, i) => { const s = Object.assign({}, scope, {[as]: it, $index: i});
      Array.from(node.childNodes).forEach(c => { const cl = c.cloneNode(true); frag.appendChild(cl); proc(cl, s); }); });
    node.replaceWith(frag); return;
  }
  if (tag === 'sc-if'){
    const v = look(scope, node.getAttribute('value').replace(/[{}]/g,''));
    if (!v){ node.remove(); return; }
    const kids = Array.from(node.childNodes); const frag = document.createDocumentFragment();
    kids.forEach(k => frag.appendChild(k)); node.replaceWith(frag); kids.forEach(k => proc(k, scope)); return;
  }
  Array.from(node.attributes).forEach(a => {
    if (!a.value.includes('{{')) return;
    const whole = a.value.match(/^\s*\{\{([^}]+)\}\}\s*$/);
    if (/^on[A-Z]/.test(a.name) || /^on/.test(a.name) && whole){ const fn = look(scope, whole[1]); node.removeAttribute(a.name); if (typeof fn === 'function') node.addEventListener(a.name.slice(2).toLowerCase(), fn); return; }
    node.setAttribute(a.name, interp(a.value, scope));
  });
  Array.from(node.childNodes).forEach(c => proc(c, scope));
}
window.__render = function(){ const root = document.getElementById('root'); root.innerHTML = '';
  const t = document.getElementById('tpl').content.cloneNode(true); const vals = window.__comp.renderVals();
  Array.from(t.childNodes).forEach(c => proc(c, vals)); root.appendChild(t); };
"""

def plain(src):
    styles = "\n".join(re.findall(r"<style>(.*?)</style>", src, re.S))
    body = re.search(r"</helmet>(.*)</x-dc>", src, re.S).group(1)
    logic = re.search(r'data-dc-script[^>]*>(.*?)</script>', src, re.S).group(1)
    for k, v in BM.items():
        body = body.replace(k, pathlib.Path(v).resolve().as_uri())
        logic = logic.replace(k, pathlib.Path(v).resolve().as_uri())
    return (f"<!doctype html><html><head><meta charset='utf-8'><style>{FACES}{styles}</style></head><body>"
            f"<template id='tpl'>{body}</template><div id='root'></div>"
            f"<script>{RUNTIME}\n{logic}\nwindow.__comp = new Component({{}}); window.__render();</script></body></html>")

async def main():
    OUT.mkdir(parents=True, exist_ok=True)
    async with async_playwright() as p:
        b = await p.chromium.launch()
        pg = await b.new_page(viewport={"width": W, "height": 900}, device_scale_factor=float(os.environ.get('DPR', '1')))
        pg.on('pageerror', lambda e: print('PAGE ERROR', e))
        for job in jobs:
            name, _, rest = job.partition(':')
            suffix, _, state = rest.partition('=')
            t = OUT / f"_{name}.html"; t.write_text(plain((ROOT / name).read_text()))
            await pg.goto(t.as_uri())
            if state:
                await pg.evaluate(f"window.__comp.setState({state})")
            await pg.evaluate("document.fonts.ready"); await pg.wait_for_timeout(300)
            h = await pg.evaluate("document.documentElement.scrollHeight")
            fn = name.replace('.dc.html', '') + (('-' + suffix) if suffix else '') + '.png'
            await pg.screenshot(path=str(OUT / fn), full_page=True)
            print(fn, h); t.unlink()
        await b.close()

asyncio.run(main())
