#!/usr/bin/env bash
# Chromium en francais : une page d'erreur (site injoignable) est redigee en francais. Instance isolee.
cd "$(dirname "$0")/.."
source tools/banc.sh
uv run -q --with websocket-client python - <<'PY'
import time
from banc import *
call(op="open", url="http://localhost:1/rien"); time.sleep(4)
cible = [t for t in targets() if t["url"].startswith("chrome-error://") or "localhost:1" in t["url"]]
assert cible, [t["url"] for t in targets()]
texte = ev("document.body.innerText", cible[0]["url"])
print(texte[:160].replace("\n", " | "))
assert "Ce site est inaccessible" in texte or "inaccessible" in texte, "page d'erreur pas en francais"
assert ev("navigator.language", cible[0]["url"]).startswith("fr"), "langue du navigateur pas francaise"
print("OK : pages de Chromium en francais")
PY
