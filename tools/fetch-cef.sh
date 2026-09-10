#!/usr/bin/env bash
# Recupere les binaires Chromium (CEF) dont le navigateur a besoin pour tourner.
# Environ 1 Go telecharge, 1,5 Go sur disque. A relancer quand la version du crate `cef` change.
set -euo pipefail
DEST="${CEF_PATH:-$HOME/.local/share/cef}"
echo "[cef] destination : $DEST"
cargo install --quiet export-cef-dir 2>/dev/null || true
if command -v export-cef-dir >/dev/null; then
  export-cef-dir --force "$DEST"
else
  echo "export-cef-dir introuvable. Installation : cargo install export-cef-dir" >&2
  exit 1
fi
echo "[cef] pret : $(du -sh "$DEST" | cut -f1)"
