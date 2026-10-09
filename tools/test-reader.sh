#!/usr/bin/env bash
# Mode lecture : clic droit → « Lire en mode lecture » n'affiche que l'article (titre, texte), sans menu ni publicite ;
# le menu propose alors « Quitter la lecture » ; Ctrl+Alt+R rend la page d'origine. Une page sans article reste
# intacte. Instance isolee.
set -euo pipefail
cd "$(dirname "$0")/.."
W="$(mktemp -d)"; PORT=$((29000 + RANDOM % 900))
python3 - "$W" <<'GEN'
import sys
para = "".join(f"<p>Paragraphe {i} : la machine analytique de Babbage aurait pu calculer bien plus que des nombres, "
               "Ada Lovelace l'avait compris et l'a ecrit dans ses notes celebres, publiees en mille huit cent "
               "quarante-trois.</p>" for i in range(12))
open(f"{sys.argv[1]}/article.html", "w").write(
    "<!doctype html><meta charset=utf-8><title>Ada et la machine</title><nav>MENU-DU-SITE Accueil Rubriques</nav>"
    "<aside class=pub>PUBLICITE-ACHETEZ</aside><article><h1>Ada et la machine</h1><p class=byline>Par Charles</p>"
    + para + "</article><footer>PIED-DE-PAGE</footer>")
open(f"{sys.argv[1]}/court.html", "w").write("<!doctype html><meta charset=utf-8><title>court</title><p>Bonjour.</p>")
GEN
python3 tools/serveur-test.py "$PORT" "$W" >/dev/null 2>&1 & SERVEUR=$!
export ECHO_RUN_DIR="$W/run" ECHO_DATA_DIR="$W/data" ECHO_CONTROL_NAME="lecture-$$" ECHO_NO_WELCOME=1 W PORT
export ECHO_DEVTOOLS_PORT=$((30000 + RANDOM % 9000))
mkdir -p "$W/run"
trap './stop.sh >/dev/null 2>&1 || true; kill $SERVEUR 2>/dev/null || true; rm -rf "$W"' EXIT
./start.sh release >/dev/null; sleep 8
uv run -q --with websocket-client python - <<'PY'
import json, os, socket, time, urllib.request, websocket
PORT, DT = os.environ["PORT"], os.environ["ECHO_DEVTOOLS_PORT"]
s = socket.socket(socket.AF_UNIX); s.connect(f"{os.environ['XDG_RUNTIME_DIR']}/echo-browser/{os.environ['ECHO_CONTROL_NAME']}.sock")
f = s.makefile("rw")
def call(**r):
    f.write(json.dumps(r) + "\n"); f.flush(); return json.loads(f.readline())
def cdp(prefix, method, params):
    t = [t for t in json.load(urllib.request.urlopen(f"http://127.0.0.1:{DT}/json/list")) if t["url"].startswith(prefix)][0]
    ws = websocket.create_connection(t["webSocketDebuggerUrl"], suppress_origin=True)
    ws.send(json.dumps({"id": 1, "method": method, "params": params}))
    while (m := json.loads(ws.recv())).get("id") != 1: pass
    ws.close(); return m.get("result", {})
ev = lambda prefix, js: cdp(prefix, "Runtime.evaluate", {"expression": js, "returnByValue": True})["result"].get("value")
def until(cond, label, tries=30):
    for _ in range(tries):
        try:
            if cond(): return
        except Exception:
            pass
        time.sleep(0.5)
    raise AssertionError(label)
def menu(page):
    for kind in ("mousePressed", "mouseReleased"):
        cdp(page, "Input.dispatchMouseEvent", {"type": kind, "x": 400, "y": 300, "button": "right", "clickCount": 1})
    time.sleep(1.2)
    return ev("echo://ui/menu", "document.body.innerText")
article = f"http://localhost:{PORT}/article.html"
call(op="open", url=article); time.sleep(3)
assert "Lire en mode lecture" in menu(article), "entree absente du clic droit"
call(op="ui", request={"kind": "runContextMenu", "action": "reader"})
until(lambda: ev(article, "document.documentElement.dataset.echoLecture") == "1", "vue de lecture absente")
text = ev(article, "document.body.innerText")
assert "Ada et la machine" in text and "Paragraphe 11" in text, "article incomplet"
assert not any(x in text for x in ("MENU-DU-SITE", "PUBLICITE", "PIED-DE-PAGE")), f"bruit garde : {text[:200]}"
assert "min de lecture" in text
assert text.count("Ada et la machine") == 1, "titre en double"
fonts = ev(article, "[...new Set(['h1','.meta','article p'].map(s=>getComputedStyle(document.querySelector(s)).fontFamily))]")
assert len(fonts) == 1, f"plusieurs polices : {fonts}"
taille = lambda: ev(article, "getComputedStyle(document.querySelector('article p')).fontSize")
avant = taille()
ev(article, "document.querySelector('[data-do=plus]').click()")
until(lambda: taille() != avant, "A+ sans effet")
ev(article, "document.querySelector('[data-do=ton]').click()")
assert ev(article, "document.documentElement.dataset.ton") == "clair"
print(f"lecture : article seul, titre unique, une police, taille {avant} -> {taille()}, couleurs")
assert "Quitter la lecture" in menu(article), "le menu ne propose pas d'en sortir"
call(op="key", code=82, ch="r", mods=["ctrl", "alt"])
until(lambda: "MENU-DU-SITE" in ev(article, "document.body.innerText"), "Ctrl+Alt+R ne rend pas la page")
call(op="key", code=82, ch="r", mods=["ctrl", "alt"])
until(lambda: ev(article, "document.documentElement.dataset.echoLecture") == "1", "retour en lecture impossible")
assert ev(article, "document.documentElement.dataset.ton") == "clair", "reglages de lecture oublies"
assert ev(article, "getComputedStyle(document.querySelector('article p')).fontSize") == taille()
print("reglages de lecture retenus d'une ouverture a l'autre")
call(op="key", code=82, ch="r", mods=["ctrl", "alt"])
until(lambda: "MENU-DU-SITE" in ev(article, "document.body.innerText"), "seconde sortie impossible")
assert "Lire en mode lecture" in menu(article), "etat de lecture garde apres la sortie"
print("Ctrl+Alt+R : page d'origine rendue")
court = f"http://localhost:{PORT}/court.html"
call(op="navigate", id=[t for t in call(op="tabs")["tabs"] if t["active"]][0]["id"], url=court); time.sleep(2.5)
call(op="key", code=82, ch="r", mods=["ctrl", "alt"]); time.sleep(1.5)
assert ev(court, "document.documentElement.dataset.echoLecture") is None, "page sans article transformee"
assert "Bonjour." in ev(court, "document.body.innerText")
print("OK : mode lecture (clic droit, article seul, sortie Ctrl+Alt+R, page sans article intacte)")
PY
