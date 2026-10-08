#!/usr/bin/env bash
# Recherche dans la page : Ctrl+F ouvre la barre avec le focus, compte les occurrences (« 1/3 »), Entree passe a la
# suivante, Maj+Entree revient, un mot absent dit « Aucun », Echap ferme. Instance isolee.
cd "$(dirname "$0")/.."
source tools/banc.sh
uv run -q --with websocket-client python - <<'PY'
import json, time
from banc import *
url = page("texte.html", "<p>chat noir</p><p>un chat blanc</p><p>le chat dort</p><p>chien</p>", "texte")
call(op="open", url=url)
until(lambda: active()["url"] == url and not active()["loading"], "page non chargee")
FIELD = "document.querySelector('input[aria-label=\"Rechercher dans la page\"]')"
call(op="key", code=70, ch="f", mods=["ctrl"])
until(lambda: ev(f"!!{FIELD}"), "Ctrl+F n'ouvre pas la barre de recherche")
until(lambda: ev(f"document.activeElement==={FIELD}"), "le champ de recherche n'a pas le focus")
print("Ctrl+F : barre ouverte, champ actif")
def tape(text):
    ev(f"(()=>{{const i={FIELD};const set=Object.getOwnPropertyDescriptor(HTMLInputElement.prototype,'value').set;"
       f"set.call(i,{json.dumps(text)});i.dispatchEvent(new Event('input',{{bubbles:true}}));return 1}})()")
def key(k, shift=False):
    ev(f"{FIELD}.dispatchEvent(new KeyboardEvent('keydown',{{key:'{k}',shiftKey:{str(shift).lower()},bubbles:true}}))")
counter = lambda: ev("document.querySelector('[role=search] [aria-live]')?.textContent")
tape("chat")
until(lambda: counter() == "1/3", f"compteur attendu 1/3 : {counter()}")
key("Enter")
until(lambda: counter() == "2/3", f"Entree : attendu 2/3 : {counter()}")
key("Enter", shift=True)
until(lambda: counter() == "1/3", f"Maj+Entree : attendu 1/3 : {counter()}")
sel = ev("getSelection().toString()", url)
print(f"occurrences : {counter()}, selection dans la page : {sel!r}")
tape("zebre")
until(lambda: counter() == "Aucun", f"mot absent : {counter()}")
key("Escape")
until(lambda: not ev(f"!!{FIELD}"), "Echap ne ferme pas la barre")
print("OK : recherche dans la page (Ctrl+F, compteur, suivant/precedent, Echap)")
PY
