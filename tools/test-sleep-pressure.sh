#!/usr/bin/env bash
# Veille honnete : avec 6 onglets et le delai regle (5 min), rien ne dort apres 75 s tant que la memoire ne manque pas ;
# memoire qui manque (simulee : ECHO_PRESSURE=1), les onglets inactifs dorment au bout d'une minute. Deux instances
# isolees, environ 3 minutes : a lancer en tache de fond.
cd "$(dirname "$0")/.."
run() {
  ( source tools/banc.sh
    PRESSURE=$1 uv run -q --with websocket-client python - <<'PY'
import os, time
from banc import *
for i in range(6):
    call(op="open", url=page(f"p{i}.html", f"<p>{i}", f"p{i}")); time.sleep(0.8)
time.sleep(75)
asleep = [t["title"] for t in tabs() if t["asleep"]]
if os.environ["PRESSURE"] == "1":
    assert len(asleep) >= 4, f"memoire qui manque : onglets inactifs encore eveilles ({asleep})"
    print(f"memoire qui manque : {len(asleep)} onglets endormis apres 75 s")
else:
    assert asleep == [], f"onglets endormis avant le delai regle : {asleep}"
    print("memoire suffisante : aucun onglet endormi apres 75 s (delai regle 5 min)")
PY
  )
}
run 0 && ECHO_PRESSURE=1 run 1 && echo "OK : veille selon la memoire, pas selon le nombre d'onglets"
