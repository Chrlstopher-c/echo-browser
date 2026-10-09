#!/usr/bin/env bash
# Bouton Claude Code : absent quand Claude Code n'est pas installe (PATH sans `claude`), present sinon. Instance isolee.
cd "$(dirname "$0")/.."
export PATH="/usr/bin:/bin"
source tools/banc.sh
uv run -q --with websocket-client python - <<'PY'
from banc import *
until(lambda: ev("document.querySelector('[aria-label=Bibliothèque]')!==null"), "barre non chargee")
assert ev("document.querySelector('[aria-label=\"Claude Code\"]')") is None, "bouton Claude Code montre sans Claude Code"
print("OK : bouton Claude Code cache sans Claude Code installe")
PY
