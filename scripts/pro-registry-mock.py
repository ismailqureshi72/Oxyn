#!/usr/bin/env python3
"""A local stand-in for the Worker's Pro registry, for the cli-e2e CI job and local testing.

    python3 scripts/pro-registry-mock.py <registry dir, e.g. site/public> <port>

Serves <dir>/r/* publicly, <dir>/_pro/r/<slug>.json at /api/r/<slug>.json only to
`Authorization: Bearer oxyn_e2e`, and /api/cli/whoami. Like site/worker/pro.ts it counts each
distinct item once (a ?for=<parent> dependency of an opened item is free) and reports the count in
x-oxyn-usage / x-oxyn-counted. Every Pro request is appended to <dir>/_pro/mock.log.
"""
import json
import os
import sys
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from urllib.parse import parse_qs, urlparse

ROOT, PORT = sys.argv[1], int(sys.argv[2])
TOKEN = "oxyn_e2e"
INDEX = json.load(open(os.path.join(ROOT, "r", "index.json")))
KIND = {i["slug"]: ("component" if i["kind"] in ("component", "style") else "screen") for i in INDEX["items"]}
DEPS = {i["slug"]: sorted({d for v in i.get("oxyn_deps", {}).values() for d in v}) for i in INDEX["items"]}
opened = []


def closure(slug):
    out, todo = set(), [slug]
    while todo:
        for d in DEPS.get(todo.pop(), []):
            if d not in out:
                out.add(d)
                todo.append(d)
    return out


def usage():
    c = sum(1 for s in opened if KIND.get(s) == "component")
    return f"{c} of 20 components, {len(opened) - c} of 8 screens, resets 2099-01-01"


class H(BaseHTTPRequestHandler):
    def send(self, status, body, extra=None):
        b = body if isinstance(body, bytes) else json.dumps(body).encode()
        self.send_response(status)
        self.send_header("content-type", "application/json")
        self.send_header("content-length", str(len(b)))
        for k, v in (extra or {}).items():
            self.send_header(k, v)
        self.end_headers()
        self.wfile.write(b)

    def do_GET(self):
        u = urlparse(self.path)
        authed = self.headers.get("authorization") == f"Bearer {TOKEN}"
        if u.path.startswith("/r/"):
            p = os.path.normpath(os.path.join(ROOT, u.path.lstrip("/")))
            if p.startswith(os.path.join(ROOT, "r")) and os.path.isfile(p):
                return self.send(200, open(p, "rb").read())
            return self.send(404, {"ok": False})
        if u.path == "/api/cli/whoami":
            if not authed:
                return self.send(401, {"ok": False, "error": {"code": "unauthorized", "message": "Sign in first"}})
            return self.send(200, {"ok": True, "email": "e2e@example.com", "tier": "pro", "pro": True, "admin": False, "usage": {"summary": usage()}})
        if u.path.startswith("/api/r/") and u.path.endswith(".json"):
            slug = u.path[len("/api/r/"):-len(".json")]
            if not authed:
                return self.send(401, {"ok": False, "error": {"code": "unauthorized", "message": "Sign in first"}})
            p = os.path.join(ROOT, "_pro", "r", f"{slug}.json")
            if "/" in slug or not os.path.isfile(p):
                return self.send(404, {"ok": False, "error": {"code": "not_found", "message": f"no Pro item {slug}"}})
            parent = parse_qs(u.query).get("for", [""])[0]
            counted = slug not in opened and not (parent in opened and slug in closure(parent))
            if counted:
                opened.append(slug)
            with open(os.path.join(ROOT, "_pro", "mock.log"), "a") as f:
                f.write(f"{slug} for={parent or '-'} counted={int(counted)}\n")
            return self.send(200, open(p, "rb").read(), {"x-oxyn-counted": "1" if counted else "0", "x-oxyn-usage": usage()})
        return self.send(404, {"ok": False})

    def log_message(self, *a):
        pass


ThreadingHTTPServer(("127.0.0.1", PORT), H).serve_forever()
