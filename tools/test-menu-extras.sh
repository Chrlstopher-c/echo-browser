#!/usr/bin/env bash
# Clic droit et raccourcis de confort : l'entree du bouclier dit ce qu'elle fera, « Rechercher dans la page » est
# proposee, Ctrl+Maj+C copie l'adresse (bulle « Adresse copiée »). Instance isolee.
cd "$(dirname "$0")/.."
source tools/banc.sh
uv run -q --with websocket-client python - <<'PY'
import time
from banc import *
url = page("p.html", "<style>body{height:100vh}</style><p>texte", "p")
call(op="open", url=url)
until(lambda: active()["url"] == url and not active()["loading"], "page non chargee")
time.sleep(1)
texte = context_menu(url, 600, 500)
assert "Désactiver le bouclier sur ce site" in texte, texte
assert "Rechercher dans la page" in texte, texte
call(op="click", x=800, y=600); time.sleep(0.5)
call(op="key", code=0x43, ch="c", mods=["ctrl", "shift"])
until(lambda: "Adresse copiée" in (ev("[...document.querySelectorAll('[role=status]')].map(e=>e.innerText).join('|')") or ""),
      "Ctrl+Maj+C sans effet")
print("OK : clic droit explicite (bouclier, recherche), Ctrl+Maj+C")
PY
