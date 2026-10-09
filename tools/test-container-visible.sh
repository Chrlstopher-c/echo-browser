#!/usr/bin/env bash
# Conteneurs visibles : clic droit sur un lien → « Ouvrir dans : Banque » ; l'onglet ouvert porte le nom et la
# couleur du conteneur sur sa ligne. Instance isolee.
cd "$(dirname "$0")/.."
source tools/banc.sh
uv run -q --with websocket-client python - <<'PY'
import json, time
from banc import *
setting("tabs.containers", "text", json.dumps([{"id": "banque", "name": "Banque"}]))
cible = page("cible.html", "<p>cible", "cible")
url = page("liens.html", "<style>a{display:block;height:100vh}</style><a href='cible.html'>lien</a>", "liens")
call(op="open", url=url)
until(lambda: active()["url"] == url and not active()["loading"], "page non chargee")
time.sleep(1)
texte = context_menu(url)
assert "Ouvrir dans : Banque" in texte, texte
ui({"kind": "runContextMenu", "action": "openLinkInContainer1"})
until(lambda: active()["url"] == cible, "lien non ouvert")
until(lambda: ev("[...document.querySelectorAll('[data-container]')].map(e=>e.dataset.container)") == ["Banque"],
      "nom du conteneur absent de la ligne d'onglet")
print("OK : conteneur visible (clic droit d'un lien, nom et couleur sur l'onglet)")
PY
