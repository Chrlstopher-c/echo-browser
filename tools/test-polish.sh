#!/usr/bin/env bash
# Finitions du second contre-audit : Ctrl+T laisse l'adresse vide ; les surlignages de Ctrl+F partent aussi quand un
# lien ouvre un nouvel onglet ; un domaine tape sans schema, sans https, revient en http. Instance isolee.
cd "$(dirname "$0")/.."
source tools/banc.sh
uv run -q --with websocket-client python - <<'PY'
import time
from banc import *
call(op="key", code=84, ch="t", mods=["ctrl"]); time.sleep(2)
assert ev(f"{ADDRESS}.value") == "", f"Ctrl+T : adresse non vide ({ev(f'{ADDRESS}.value')!r})"
print("Ctrl+T : adresse vide")
url = page("f.html", "<style>a{display:block;height:50vh}</style><p>chat chat chat</p><a href='g.html' target=_blank>lien</a>", "f")
page("g.html", "<p>g", "g")
call(op="open", url=url)
until(lambda: active()["url"] == url and not active()["loading"], "page non chargee")
ui({"kind": "find", "text": "chat", "forward": True, "next": False}); time.sleep(1)
call(op="click", x=300, y=400); time.sleep(2)
until(lambda: active()["title"] == "g", "le lien n'a pas ouvert d'onglet")
call(op="activate", id=[t for t in tabs() if t["url"] == url][0]["id"]); time.sleep(1)
sel = ev("getSelection().toString()", url)
assert sel == "", f"surlignage reste dans la page : {sel!r}"
print("Ctrl+F : recherche fermee quand un lien ouvre un onglet")
print("OK : finitions (adresse vide, recherche fermee)")
PY
