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
cp data/shield-resources.json data/echo-browser.svg "$STAGE/"
cp tools/release-installer.sh "$STAGE/installer.sh"
cp -r ui/dist/. "$STAGE/ui/"

for f in libcef.so libvk_swiftshader.so libvulkan.so.1 vk_swiftshader_icd.json icudtl.dat v8_context_snapshot.bin \
         chrome_100_percent.pak chrome_200_percent.pak resources.pak chrome-sandbox; do
  cp "$CEF/$f" "$STAGE/cef/"
done
cp "$CEF"/locales/*.pak "$STAGE/cef/locales/"
cp "$FFMPEG_LIBRE" "$STAGE/cef/libffmpeg.so"
strip --strip-unneeded "$STAGE/cef/libcef.so" "$STAGE/cef/libffmpeg.so"

# Marqueur de l'archive : sa version, et le depot ou chercher les suivantes (mise a jour automatique).
# Nom canonique du depot (un ancien nom ne marche que tant que GitHub redirige) ; a defaut, le remote git.
REPO="$(gh repo view --json nameWithOwner -q .nameWithOwner 2>/dev/null || git remote get-url origin | sed -E 's#(git@github.com:|https://github.com/)##; s#\.git$##')"
# Adresse du service de compte : dans .env.local (hors depot), jamais dans un fichier suivi.
SYNC="$(sed -n 's/^ECHO_SYNC_URL=//p' .env.local 2>/dev/null | tail -1)"
printf '{"version":"%s","repo":"%s","sync":"%s"}\n' "$VERSION" "$REPO" "$SYNC" > "$STAGE/release.json"

cat > "$STAGE/echo-browser.sh" <<'EOF'
#!/usr/bin/env bash
# Lance Echo Browser depuis ce dossier. Profil : ~/.local/share/echo-browser (ECHO_DATA_DIR pour le changer).
# La commande `echo-browser` est un lien vers ce fichier : on suit le lien pour trouver le dossier.
here="$(dirname "$(readlink -f "$0")")"
parent="$(dirname "$here")"; name="$(basename "$here")"
staged="$parent/$name.maj"; previous="$parent/$name.precedent"; trial="$here/.essai-demarrage"
# Mise a jour preparee par Echo : bascule (l'ancienne version est gardee), la nouvelle est a l'essai.
if [ -x "$staged/echo-browser" ] && [ -f "$staged/release.json" ]; then
  rm -rf "$previous"
  if mv "$here" "$previous" && mv "$staged" "$here"; then
    echo 0 > "$here/.essai-demarrage"
    exec "$here/echo-browser.sh" "$@"
  fi
fi
# Une version a l'essai qui n'a pas demarre deux fois de suite : retour a la precedente.
if [ -f "$trial" ]; then
  tries=$(( $(cat "$trial" 2>/dev/null || echo 0) + 1 ))
  if [ "$tries" -gt 2 ] && [ -x "$previous/echo-browser" ]; then
    rm -rf "$parent/$name.echec"
    mv "$here" "$parent/$name.echec" && mv "$previous" "$here" && exec "$here/echo-browser.sh" "$@"
  fi
  echo "$tries" > "$trial"
fi
export CEF_PATH="$here/cef"
export LD_LIBRARY_PATH="$here/cef${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}"
export ECHO_UI_DIR="$here/ui"
cd "$here"
exec "$here/echo-browser" "$@"
EOF
chmod +x "$STAGE/echo-browser.sh"

cat > "$STAGE/LISEZMOI.txt" <<EOF
Echo Browser $VERSION — Linux x86-64

Installer : ./installer.sh  (menu des applications + commande echo-browser ; relancer pour mettre à jour)
Essayer sans installer : ./echo-browser.sh
Désinstaller : ~/.local/opt/echo-browser/installer.sh --retirer  (vos données sont gardées)

Vidéo : VP9, AV1, Opus et Vorbis sont lus d'emblée (YouTube, la plupart des sites). Le H.264/AAC (Twitch, certains
MP4) est couvert par des brevets et n'est pas distribué : la première fois qu'une page en a besoin, Echo propose de
télécharger le décodeur (depuis un tiers, vérifié par empreinte) puis de redémarrer. Aussi dans Réglages → Vidéo.
EOF

(cd "$OUT" && tar -cf - "$NAME" | xz -T0 -6 > "$NAME.tar.xz")
# Nom seul dans le fichier d'empreinte : `sha256sum -c` doit marcher la ou l'utilisateur a telecharge.
(cd "$OUT" && sha256sum "$NAME.tar.xz" > "$NAME.tar.xz.sha256")
ls -la "$OUT/$NAME.tar.xz"
