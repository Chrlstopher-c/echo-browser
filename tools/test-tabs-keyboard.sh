#!/usr/bin/env bash
# Onglets au clavier : la liste est une `tablist`, l'onglet actif porte `aria-selected` et le focus ; fleche bas passe
# au suivant, Entree l'ouvre, Suppr le ferme. Instance isolee.
cd "$(dirname "$0")/.."
source tools/banc.sh
uv run -q --with websocket-client python - <<'PY'
import time
from banc import *
for n in "abc":
    call(op="open", url=page(f"{n}.html", f"<p>{n}", f"page {n}")); time.sleep(1.2)
rows = lambda: ev("[...document.querySelectorAll('[role=tablist] [role=tab]')].map(e=>e.getAttribute('aria-label')+'|'+e.getAttribute('aria-selected'))")
until(lambda: "page c|true" in rows(), f"onglet actif non annonce : {rows()}")
assert ev("document.querySelector('[role=tablist][aria-label=Onglets]')!==null")
focused = lambda: ev("document.activeElement?.getAttribute('aria-label')")
ev("document.querySelector('[role=tab][aria-selected=true]').focus()")
key = lambda k: ev(f"document.activeElement.dispatchEvent(new KeyboardEvent('keydown',{{key:'{k}',bubbles:true}}))")
key("ArrowUp"); assert focused() == "page b", focused()
key("Enter"); until(lambda: active()["title"] == "page b", "Entree n'ouvre pas l'onglet")
n = len(tabs())
key("Delete"); until(lambda: len(tabs()) == n - 1, "Suppr ne ferme pas l'onglet")
assert "page b" not in [t["title"] for t in tabs()]
print(f"OK : onglets au clavier ({rows()})")
PY
