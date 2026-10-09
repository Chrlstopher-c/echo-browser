#!/usr/bin/env bash
# Marques d'onglet : un onglet epingle sans favicon (page locale, favicon introuvable) montre l'initiale teintee de
# son site, jamais une tuile vide ; une page d'Echo porte la marque d'Echo. Instance isolee.
cd "$(dirname "$0")/.."
source tools/banc.sh
uv run -q --with websocket-client python - <<'PY'
import time
from banc import *
url = page("sans-icone.html", "<link rel=icon href=/introuvable.ico><p>sans icone", "sans icone")
call(op="open", url=url)
until(lambda: active()["url"] == url and not active()["loading"], "page non chargee")
ui({"kind": "pinTab", "id": active()["id"], "pinned": True}); time.sleep(1.5)
marks = lambda: ev("[...document.querySelectorAll('[data-mark=initiale]')].map(e=>e.textContent)")
until(lambda: "l" in [m.lower() for m in marks()], f"pas d'initiale pour l'epingle sans favicon : {marks()}")
ui({"kind": "openPage", "page": "aide"}); time.sleep(2)
echo_marks = ev("document.querySelectorAll('[data-mark=echo]').length")
assert echo_marks >= 1, "page d'Echo sans sa marque"
print(f"OK : marques d'onglet (initiales {marks()}, marque d'Echo)")
PY
