#!/usr/bin/env bash
# Places des onglets : Ctrl+Maj+T rouvre a la place d'origine (et l'active), une fenetre ouverte par une page arrive
# juste apres elle (derriere ses autres ouvertures), pas en fin de liste. Instance isolee.
cd "$(dirname "$0")/.."
source tools/banc.sh
uv run -q --with websocket-client python - <<'PY'
import time
from banc import *
urls = {n: page(f"{n}.html", f"<p>{n}", n) for n in "abcd"}
page("ouvreur.html", "<style>body{margin:0;height:100vh}</style><p>ouvreur<script>"
     "document.addEventListener('click',()=>window.open('fille.html'))</script>", "ouvreur")
page("fille.html", "<p>fille", "fille")
for n in "abcd":
    call(op="open", url=urls[n]); time.sleep(1.2)
order = lambda: [t["title"] for t in tabs() if t["title"] in ("a", "b", "c", "d", "ouvreur", "fille")]
until(lambda: order() == list("abcd"), f"ordre initial : {order()}")
b = [t for t in tabs() if t["title"] == "b"][0]
call(op="close", id=b["id"]); time.sleep(1)
assert order() == list("acd"), order()
call(op="key", code=84, ch="t", mods=["ctrl", "shift"])
until(lambda: order() == list("abcd"), f"Ctrl+Maj+T ne remet pas l'onglet a sa place : {order()}")
until(lambda: active()["title"] == "b", "l'onglet rouvert n'est pas actif")
print(f"Ctrl+Maj+T : {order()}, actif {active()['title']}")
call(op="activate", id=[t for t in tabs() if t["title"] == "a"][0]["id"]); time.sleep(0.5)
call(op="navigate", id=active()["id"], url=f"{BASE}/ouvreur.html")
until(lambda: active()["title"] == "ouvreur", "page ouvreuse non chargee")
time.sleep(1)
call(op="click", x=700, y=400)
until(lambda: "fille" in order(), "la page n'a pas ouvert de fenetre")
o = order()
assert o.index("fille") == o.index("ouvreur") + 1, f"fenetre de la page pas juste apres elle : {o}"
print(f"OK : places des onglets ({o})")
PY
