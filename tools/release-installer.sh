#!/usr/bin/env bash
# Installe Echo Browser pour l'utilisateur, depuis le dossier extrait de l'archive : copie dans ~/.local/opt/echo-browser,
# entree dans le menu des applications (avec icone), commande `echo-browser`. Relancer apres une nouvelle version la met
# a jour. `./installer.sh --retirer` desinstalle (le profil, ~/.local/share/echo-browser, est garde).
set -euo pipefail
here="$(cd "$(dirname "$0")" && pwd)"
dest="$HOME/.local/opt/echo-browser"
apps="$HOME/.local/share/applications"
icons="$HOME/.local/share/icons/hicolor/scalable/apps"
bin="$HOME/.local/bin"

if [ "${1:-}" = "--retirer" ]; then
  rm -rf "$dest" "$apps/echo-browser.desktop" "$icons/echo-browser.svg"
  [ -L "$bin/echo-browser" ] && rm -f "$bin/echo-browser"
  echo "Echo est désinstallé. Vos données restent dans ~/.local/share/echo-browser."
  exit 0
fi

# Seule la copie installee compte : un autre Echo (version de developpement, autre dossier) peut rester ouvert.
if pgrep -u "$USER" -f "^$dest/echo-browser( |$)" >/dev/null; then
  echo "Echo est ouvert : fermez-le avant d'installer ou de mettre à jour." >&2
  exit 1
fi

if [ "$here" != "$dest" ]; then
  mkdir -p "$(dirname "$dest")"
  rm -rf "$dest.nouveau"
  cp -a "$here" "$dest.nouveau"
  rm -rf "$dest"
  mv "$dest.nouveau" "$dest"
fi
mkdir -p "$apps" "$icons" "$bin"
cp "$dest/echo-browser.svg" "$icons/echo-browser.svg"
ln -sf "$dest/echo-browser.sh" "$bin/echo-browser"
cat > "$apps/echo-browser.desktop" <<DESKTOP
[Desktop Entry]
Type=Application
Name=Echo
GenericName=Navigateur web
Comment=Navigateur rapide et économe
Exec=$dest/echo-browser.sh
Icon=echo-browser
StartupWMClass=echo-browser
Categories=Network;WebBrowser;
Terminal=false
DESKTOP
command -v update-desktop-database >/dev/null && update-desktop-database "$apps" 2>/dev/null || true
echo "Echo est installé : cherchez « Echo » dans le menu des applications, ou lancez la commande echo-browser."
