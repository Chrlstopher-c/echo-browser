#!/usr/bin/env bash
# Fabrique l'archive publique d'Echo Browser : binaire, moteur CEF, interface, lanceur.
# Le decodeur video embarque est la variante LIBRE (sans H.264/AAC brevetes) : la release ne distribue
# jamais le decodeur complet. Usage : tools/package-release.sh <libffmpeg.so libre> [dossier de sortie]
set -euo pipefail
cd "$(dirname "$0")/.."

FFMPEG_LIBRE="${1:?chemin du libffmpeg.so libre (ffmpeg_branding=Chromium)}"
OUT="${2:-$PWD/dist-release}"
CEF="${CEF_PATH:-$HOME/.local/share/cef}"
VERSION="$(sed -n 's/^version = "\(.*\)"/\1/p' Cargo.toml | head -1)"
NAME="echo-browser-$VERSION-linux-x64"
STAGE="$OUT/$NAME"

# Le nom « h264 » figure aussi dans la table des descripteurs du decodeur libre : on teste les vrais decodeurs.
if [ "$(strings "$FFMPEG_LIBRE" | grep -cE 'h264_has_num_reorder_frames|^AAC decoder$')" != 0 ]; then
  echo "refus : $FFMPEG_LIBRE contient le decodeur H.264 (variante Chrome)" >&2
  exit 1
fi

echo "compilation (release)…"
cargo build --release -p echo-shell >/dev/null
(cd ui && bun run build >/dev/null)

rm -rf "$STAGE" && mkdir -p "$STAGE/cef/locales" "$STAGE/ui"
cp target/release/echo-browser "$STAGE/"
strip --strip-unneeded "$STAGE/echo-browser"
cp data/shield-resources.json "$STAGE/"
cp -r ui/dist/. "$STAGE/ui/"

for f in libcef.so libvk_swiftshader.so libvulkan.so.1 vk_swiftshader_icd.json icudtl.dat v8_context_snapshot.bin \
         chrome_100_percent.pak chrome_200_percent.pak resources.pak chrome-sandbox; do
  cp "$CEF/$f" "$STAGE/cef/"
done
cp "$CEF"/locales/*.pak "$STAGE/cef/locales/"
cp "$FFMPEG_LIBRE" "$STAGE/cef/libffmpeg.so"
strip --strip-unneeded "$STAGE/cef/libcef.so" "$STAGE/cef/libffmpeg.so"

cat > "$STAGE/echo-browser.sh" <<'EOF'
#!/usr/bin/env bash
# Lance Echo Browser depuis ce dossier. Profil : ~/.local/share/echo-browser (ECHO_DATA_DIR pour le changer).
here="$(cd "$(dirname "$0")" && pwd)"
export CEF_PATH="$here/cef"
export LD_LIBRARY_PATH="$here/cef${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}"
export ECHO_UI_DIR="$here/ui"
cd "$here"
exec "$here/echo-browser" "$@"
EOF
chmod +x "$STAGE/echo-browser.sh"

cat > "$STAGE/LISEZMOI.txt" <<EOF
Echo Browser $VERSION — Linux x86-64

Lancer : ./echo-browser.sh

Vidéo : VP9, AV1, Opus et Vorbis sont lus d'emblée (YouTube, la plupart des sites). Les décodeurs logiciels
H.264 et AAC (Twitch, certains MP4) ne sont pas inclus : ces formats sont couverts par des brevets. Pour les
lire : Réglages → Vidéo → Installer (téléchargé depuis un tiers, vérifié par empreinte), puis Redémarrer.
EOF

(cd "$OUT" && tar -cf - "$NAME" | xz -T0 -6 > "$NAME.tar.xz")
sha256sum "$OUT/$NAME.tar.xz" > "$OUT/$NAME.tar.xz.sha256"
ls -la "$OUT/$NAME.tar.xz"
