#!/usr/bin/env bash
# Favoris : Ctrl+D confirme (bulle « Ajouté aux favoris » avec « Retirer »), l'etoile de l'adresse suit l'etat, une
# page d'Echo ne se met pas en favori, une page en erreur n'entre pas dans l'historique. Instance isolee.
cd "$(dirname "$0")/.."
source tools/banc.sh
uv run -q --with websocket-client python - <<'PY'
import time
from banc import *
url = page("fav.html", "<p>favori", "page favorite")
call(op="open", url=url)
until(lambda: active()["url"] == url and not active()["loading"], "page non chargee")
STAR = "document.querySelector('button[aria-pressed][aria-label*=\"favoris\"]')"
until(lambda: ev(f"{STAR}?.getAttribute('aria-pressed')") == "false", "etoile absente de l'adresse")
call(op="key", code=0x44, ch="d", mods=["ctrl"])
until(lambda: "Ajouté aux favoris" in (ev("[...document.querySelectorAll('[role=status]')].map(e=>e.innerText).join('|')") or ""), "Ctrl+D sans confirmation")
until(lambda: ev(f"{STAR}?.getAttribute('aria-pressed')") == "true", "l'etoile ne montre pas le favori")
print("Ctrl+D : bulle de confirmation, etoile pleine")
ev("[...document.querySelectorAll('[role=status] button')].find(b=>b.textContent==='Retirer').click()")
until(lambda: ev(f"{STAR}?.getAttribute('aria-pressed')") == "false", "« Retirer » n'enleve pas le favori")
print("Retirer depuis la bulle : favori enleve")
ui({"kind": "openPage", "page": "bibliotheque"}); time.sleep(2)
call(op="key", code=0x44, ch="d", mods=["ctrl"]); time.sleep(1)
marks = ev("[...document.querySelectorAll('[role=status]')].map(e=>e.innerText).join('|')")
assert "Ajouté" not in marks, f"une page d'Echo a ete mise en favori : {marks}"
mort = f"http://localhost:1/rien"
call(op="open", url=mort); time.sleep(3)
ui({"kind": "searchHistory", "terms": ""}); time.sleep(1)
call(op="open", url="echo://ui/pages.html#bibliotheque"); time.sleep(2)
ev("[...document.querySelectorAll('[role=tab]')].find(b=>b.textContent.startsWith('Historique')).click()", "echo://ui/pages.html")
time.sleep(1)
hist = ev("document.body.innerText", "echo://ui/pages.html")
assert "page favorite" in hist, hist[:300]
assert "localhost:1" not in hist, f"page en erreur dans l'historique : {hist[:400]}"
print("OK : favoris (Ctrl+D, etoile, Retirer), pages internes et en erreur hors historique")
PY
