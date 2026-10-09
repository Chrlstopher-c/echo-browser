#!/usr/bin/env bash
# Reglages lisibles : aucune cle interne, sommaire, theme Clair/Sombre/Systeme, moteur choisi dans une liste et suivi
# par la recherche, suggestions « Rechercher … » ; historique date du jour (plus de 1970). Instance isolee.
cd "$(dirname "$0")/.."
source tools/banc.sh
uv run -q --with websocket-client python - <<'PY'
import time
from banc import *
PAGES = "echo://ui/pages.html"
setting("search.suggest", "flag", False)
alpha = page("alpha.html", "<p>alpha", "alpha reglages")
call(op="open", url=alpha); time.sleep(2)
ui({"kind": "openPage", "page": "reglages"})
until(lambda: "Moteur de recherche" in (ev("document.body.innerText", PAGES) or ""), "page Reglages absente")
texte = ev("document.body.innerText", PAGES)
for interdit in ["Autres", "shield.enabled", "onboarding", "profiles.list", "Restore_tabs", "%s"]:
    assert interdit not in texte, f"cle interne visible : {interdit}"
sommaire = ev("[...document.querySelectorAll('nav[aria-label=\"Sommaire des réglages\"] button')].map(b=>b.textContent)", PAGES)
assert {"Apparence", "Navigation", "Vie privée", "Onglets"} <= set(sommaire), sommaire
print(f"reglages propres, sommaire : {sommaire}")
ev("document.querySelector('[aria-label=\"Retirer discord.com\"]').click()", P := PAGES)
until(lambda: ev("!document.querySelector('[aria-label=\"Retirer discord.com\"]')", PAGES), "pastille non retiree")
print("sites eveilles en pastilles : retrait d'un clic")
ev("[...document.querySelectorAll('[role=radio]')].find(b=>b.textContent==='DuckDuckGo').click()", PAGES)
until(lambda: ev("[...document.querySelectorAll('[role=radio]')].find(b=>b.textContent==='DuckDuckGo').getAttribute('aria-checked')", PAGES) == "true", "choix du moteur non retenu")
call(op="activate", id=[t for t in tabs() if "alpha" in t["url"]][0]["id"]); time.sleep(1)
type_address("meteo paris")
until(lambda: options() and options()[0].startswith("search|"), f"pas de ligne de recherche en tete : {options()}")
assert "duckduckgo.com" in options()[0], options()[0]
print(f"ligne de recherche : {options()[0]}")
press_in_address("Enter")
until(lambda: active()["url"].startswith("https://duckduckgo.com/?q=meteo+paris"), f"recherche hors du moteur choisi : {active()['url']}")
print("recherche envoyee au moteur choisi")
ev("[...document.querySelectorAll('[role=tab]')].find(b=>b.textContent==='Clair').click()", PAGES)
until(lambda: ev("localStorage.getItem('echo.scheme.now')") == '"light"', "la barre n'a pas suivi le theme choisi dans la page")
ev("[...document.querySelectorAll('[role=tab]')].find(b=>b.textContent==='Sombre').click()", PAGES)
until(lambda: ev("localStorage.getItem('echo.scheme.now')") == '"dark"', "retour au sombre non suivi")
print("theme choisi dans la page applique a la barre")
ui({"kind": "openPage", "page": "bibliotheque"})
until(lambda: "Historique" in (ev("document.body.innerText", PAGES) or ""), "bibliotheque absente")
ev("[...document.querySelectorAll('[role=tab]')].find(b=>b.textContent.startsWith('Historique')).click()", PAGES)
until(lambda: "alpha reglages" in ev("document.body.innerText", PAGES), "page visitee absente de l'historique")
hist = ev("document.body.innerText", PAGES)
assert "1970" not in hist and "aujourd" in hist.lower(), hist[:400]
print("OK : reglages, moteur, theme, dates de l'historique")
PY
