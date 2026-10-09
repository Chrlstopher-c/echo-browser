#!/usr/bin/env bash
# Un profil = une identite : l'historique, les favoris, les dossiers, les onglets proposes et les tuiles du nouvel
# onglet du profil principal n'apparaissent pas dans un autre profil, et inversement. Instance isolee.
cd "$(dirname "$0")/.."
source tools/banc.sh
uv run -q --with websocket-client python - <<'PY'
import json, time
from banc import *
perso = page("perso.html", "<p>perso", "page perso unique")
travail = page("travail.html", "<p>travail", "page travail unique")
call(op="open", url=perso)
until(lambda: active()["url"] == perso and not active()["loading"], "page perso non chargee")
time.sleep(1)
call(op="key", code=0x44, ch="d", mods=["ctrl"]); time.sleep(1)
folders = json.dumps([{"id": "dperso", "name": "Dossier perso", "collapsed": False, "space": "graphite"}])
setting("tabs.folders", "text", folders); time.sleep(1)
until(lambda: "Dossier perso" in ev("document.body.innerText"), "dossier perso absent dans son profil")
ev("document.querySelector('[aria-label=\"Nouveau profil\"]').click()"); time.sleep(3)
sidebar = lambda: ev("document.body.innerText")
assert "Dossier perso" not in sidebar(), "dossier du profil principal visible dans le nouveau profil"
setting("search.suggest", "flag", False)
type_address("unique")
time.sleep(1.5)
opts = options()
assert not any("perso" in o for o in opts), f"suggestions du profil principal dans le nouveau : {opts}"
NEW = [t for t in targets() if t["url"].startswith("echo://ui/nouvel-onglet")]
if NEW:
    tuiles = ev("document.getElementById('tuiles').innerText + document.getElementById('recents').innerText", NEW[-1]["url"])
    assert "perso" not in tuiles, f"nouvel onglet du nouveau profil montre le principal : {tuiles}"
call(op="navigate", id=active()["id"], url=travail)
until(lambda: active()["url"] == travail and not active()["loading"], "page travail non chargee")
time.sleep(1)
ui({"kind": "openPage", "page": "bibliotheque"}); time.sleep(2)
P = "echo://ui/pages.html"
ev("[...document.querySelectorAll('[role=tab]')].find(b=>b.textContent.startsWith('Historique')).click()", P); time.sleep(1)
hist = ev("document.body.innerText", P)
assert "page travail unique" in hist and "page perso unique" not in hist, hist[:400]
print("nouveau profil : rien du principal (dossier, suggestions, historique)")
ev("[...document.querySelectorAll('[role=radiogroup][aria-label=Profils] [role=radio]')][0].click()"); time.sleep(3)
type_address("uniqu"); time.sleep(1.5)
opts = options()
assert any("perso" in o for o in opts) and not any("travail" in o for o in opts), f"retour au principal : {opts}"
print("OK : historique, favoris, dossiers, suggestions et nouvel onglet propres a chaque profil")
PY
