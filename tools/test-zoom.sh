#!/usr/bin/env bash
# Zoom : paliers de Chrome (110 %, 125 %, 90 %), Ctrl+0 remet a 100 %, la pastille ne recouvre pas l'adresse.
cd "$(dirname "$0")/.."
source tools/banc.sh
uv run -q --with websocket-client python - <<'PY'
import time
from banc import *
url = page("z.html", "<p>zoom", "z")
call(op="open", url=url)
until(lambda: active()["url"] == url and not active()["loading"], "page non chargee")
zoom = lambda: round(active()["zoom"], 2)
plus = lambda: call(op="key", code=0xBB, ch="=", mods=["ctrl"])
plus(); until(lambda: zoom() == 1.1, f"premier palier : {zoom()}")
plus(); until(lambda: zoom() == 1.25, f"second palier : {zoom()}")
badge = ev("(()=>{const b=[...document.querySelectorAll('button')].find(x=>/^Zoom /.test(x.getAttribute('aria-label')||''));"
           "const h=document.querySelector('input[aria-label=\"Adresse\"]').parentElement.querySelector('.pointer-events-none');"
           "if(!b||!h)return null;return [b.getBoundingClientRect().left, h.getBoundingClientRect().right, b.textContent]})()")
assert badge and badge[1] <= badge[0] + 1, f"la pastille recouvre l'adresse : {badge}"
print(f"125 % : pastille {badge[2]!r} a cote de l'adresse")
# Avec Ctrl, Chromium livre « 0 » en code de controle 16 : ce Ctrl+0 ouvrait l'impression (Ctrl+P).
call(op="key", code=0x30, ch="à", mods=["ctrl"]); until(lambda: zoom() == 1.0, f"Ctrl+0 : {zoom()}")
call(op="key", code=0xBD, ch="-", mods=["ctrl"]); until(lambda: zoom() == 0.9, f"Ctrl+- : {zoom()}")
print("OK : zoom (paliers, Ctrl+0, pastille)")
PY
