#!/usr/bin/env bash
# Son d'un onglet : « Couper le son » au clic droit de l'onglet ; l'onglet montre alors son haut-parleur barre, un
# clic dessus remet le son. (Aucun son reel n'est joue : le banc partage la sortie audio de la machine.)
cd "$(dirname "$0")/.."
source tools/banc.sh
uv run -q --with websocket-client python - <<'PY'
import time
from banc import *
url = page("m.html", "<p>m", "page muette")
call(op="open", url=url)
until(lambda: active()["url"] == url and not active()["loading"], "page non chargee")
ui({"kind": "setTabMuted", "id": active()["id"], "muted": True}); time.sleep(1)
BTN = "document.querySelector('[role=tab][aria-label=\"page muette\"] button[aria-pressed]')"
until(lambda: ev(f"{BTN}?.getAttribute('aria-pressed')") == "true", "onglet muet sans indicateur")
ev(f"{BTN}.click()"); time.sleep(1)
until(lambda: ev(f"{BTN}") is None, "le clic ne remet pas le son")
print("OK : couper / remettre le son d'un onglet")
PY
