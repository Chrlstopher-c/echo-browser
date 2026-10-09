#!/usr/bin/env bash
# Theme partout : en clair, le nouvel onglet d'un onglet prive et d'un autre profil est clair aussi ; le theme choisi
# dans les Reglages d'un autre profil s'applique a la barre ; la bascule soleil/lune met le selecteur a jour.
cd "$(dirname "$0")/.."
source tools/banc.sh
uv run -q --with websocket-client python - <<'PY'
import time
from banc import *
NEW = "echo://ui/nouvel-onglet"
P = "echo://ui/pages.html"
ev("document.querySelector('[aria-label=\"Passer au thème clair\"]').click();1"); time.sleep(1.5)
assert ev("localStorage.getItem('echo.scheme.now')") == '"light"'
scheme_of = lambda url: ev("getComputedStyle(document.documentElement).colorScheme", url)
call(op="key", code=0x4E, ch="n", mods=["ctrl", "shift"]); time.sleep(3)
privee = [t for t in tabs() if t["container"] == "prive"][-1]
until(lambda: scheme_of(NEW) == "light", "nouvel onglet prive sombre en theme clair")
print("prive : clair")
ev("document.querySelector('[aria-label=\"Nouveau profil\"]').click()"); time.sleep(3)
until(lambda: [t for t in targets() if t["url"].startswith(NEW)] and scheme_of([t["url"] for t in targets() if t["url"].startswith(NEW)][-1]) == "light", "nouvel onglet d'un autre profil sombre")
ui({"kind": "openPage", "page": "reglages"}); time.sleep(3)
until(lambda: ev("[...document.querySelectorAll('[role=tab]')].find(b=>b.textContent==='Clair')?.getAttribute('aria-selected')", P) == "true", "Reglages d'un autre profil : choix non affiche")
ev("[...document.querySelectorAll('[role=tab]')].find(b=>b.textContent==='Sombre').click()", P)
until(lambda: ev("localStorage.getItem('echo.scheme.now')") == '"dark"', "choix fait dans les Reglages d'un autre profil sans effet")
print("autre profil : Reglages appliques a la barre")
ev("[...document.querySelectorAll('[role=tab]')].find(b=>b.textContent==='Système').click()", P); time.sleep(1)
ev("document.querySelector('[aria-label^=\"Passer au thème\"]').click();1"); time.sleep(1.5)
until(lambda: ev("[...document.querySelectorAll('[role=tab]')].find(b=>b.textContent==='Système')?.getAttribute('aria-selected')", P) == "false", "selecteur reste sur Systeme apres la bascule")
print("OK : theme partout (prive, autre profil, Reglages, bascule)")
PY
