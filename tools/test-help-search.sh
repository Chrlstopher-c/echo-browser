#!/usr/bin/env bash
# Recherche dans l'Aide : « securite » (sans accent) trouve la section Sécurité, « ctrl+f » trouve le raccourci, un mot
# absent le dit. Instance isolee.
cd "$(dirname "$0")/.."
source tools/banc.sh
uv run -q --with websocket-client python - <<'PY'
import json, time
from banc import *
P = "echo://ui/pages.html"
ui({"kind": "openPage", "page": "aide"})
FIELD = "document.querySelector('input[aria-label=\"Chercher dans l’Aide\"]')"
until(lambda: ev(f"!!{FIELD}", P), "champ de recherche de l'Aide absent")
def tape(text):
    ev(f"(()=>{{const i={FIELD};const set=Object.getOwnPropertyDescriptor(HTMLInputElement.prototype,'value').set;"
       f"set.call(i,{json.dumps(text)});i.dispatchEvent(new Event('input',{{bubbles:true}}));return 1}})()", P)
titles = lambda: ev("[...document.querySelectorAll('section p.text-ink')].map(p=>p.textContent)", P)
tape("securite du site"); time.sleep(0.5)
assert titles() == ["Sécurité du site"], titles()
tape("ctrl+f"); time.sleep(0.5)
assert "Rechercher dans la page" in ev("document.body.innerText", P)
tape("zzzz"); time.sleep(0.5)
assert "Rien ne correspond" in ev("document.body.innerText", P)
print("OK : recherche dans l'Aide (accents, raccourcis, rien trouve)")
PY
