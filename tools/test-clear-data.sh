#!/usr/bin/env bash
# Effacer les donnees de navigation : Ctrl+Maj+Suppr ouvre le panneau ; « Tout » + historique + cookies vide
# l'historique du profil et deconnecte ses sites (cookie efface). Instance isolee.
cd "$(dirname "$0")/.."
source tools/banc.sh
uv run -q --with websocket-client python - <<'PY'
import time
from banc import *
url = page("site.html", "<p>site<script>document.title='site '+(document.cookie||'vide');"
           "document.cookie='session=1; max-age=3600'</script>", "site")
call(op="open", url=url)
until(lambda: active()["title"] == "site vide", f"premier chargement : {active()['title']}")
call(op="navigate", id=active()["id"], url=url)
until(lambda: active()["title"] == "site session=1", f"cookie non pose : {active()['title']}")
site = active()["id"]
call(op="key", code=0x2E, ch="", mods=["ctrl", "shift"])
P = "echo://ui/pages.html"
until(lambda: any(t["url"].startswith(P + "#effacer") for t in targets()), "Ctrl+Maj+Suppr n'ouvre pas le panneau")
until(lambda: ev("!!document.querySelector('section[aria-label=\"Effacer les données de navigation\"]')", P), "panneau absent")
ev("[...document.querySelectorAll('[role=tab]')].find(b=>b.textContent==='Tout').click()", P)
ev("document.querySelector('[role=switch][aria-label=\"Cookies et sessions\"]').click()", P); time.sleep(0.5)
ev("[...document.querySelectorAll('section[aria-label=\"Effacer les données de navigation\"] button')]"
   ".find(b=>b.textContent.trim()==='Effacer').click()", P)
time.sleep(2)
ui({"kind": "openPage", "page": "bibliotheque"}); time.sleep(2)
ev("[...document.querySelectorAll('[role=tab]')].find(b=>b.textContent.startsWith('Historique')).click()", P); time.sleep(1)
hist = ev("document.body.innerText", P)
assert "site.html" not in hist and "\nsite\n" not in hist, f"historique non efface : {hist[:300]}"
call(op="activate", id=site); time.sleep(1)
call(op="navigate", id=site, url=url)
until(lambda: active()["title"].startswith("site "), "rechargement")
time.sleep(1)
assert active()["title"] == "site vide", f"cookie toujours present apres effacement : {active()['title']}"
print("OK : effacer les donnees (Ctrl+Maj+Suppr, historique et cookies du profil)")
PY
