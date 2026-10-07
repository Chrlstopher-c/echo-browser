# echo-browser

Navigateur de bureau Linux, souverain et sobre : coque Rust, moteur Chromium (CEF), interface React.
Barre latérale verticale façon Zen/Arc, bouclier anti-pub natif, veille des onglets pour la mémoire, Claude Code intégré.

Objectifs, dans l'ordre : **design** (neumorphisme clair et sombre), **rendu**, **performance** (RAM par onglet), puis usage quotidien.

## Installer (release)

Télécharger `echo-browser-<version>-linux-x64.tar.xz` depuis les *Releases*, puis :

```bash
tar -xJf echo-browser-*-linux-x64.tar.xz && cd echo-browser-*-linux-x64 && ./echo-browser.sh
```

Lecture vidéo : VP9, AV1, Opus et Vorbis d'emblée (YouTube…). Le H.264/AAC (Twitch, certains MP4) n'est pas
distribué dans la release : ces formats sont brevetés. Le moteur compilé localement avec `proprietary_codecs` les lit.

Fabriquer l'archive : `tools/package-release.sh <libffmpeg.so libre>` (décodeur compilé avec `ffmpeg_branding=Chromium`).

## Lancer (développement)

```bash
./start.sh release     # lance (profil dans ~/.local/share/echo-browser) ; journal : logs/browser.log
./stop.sh              # arrête par identifiant enregistré (jamais par nom de processus)
```

Première fois sur une machine : `bash tools/fetch-cef.sh` (binaires Chromium, ~1,5 Go), `cd ui && bun install && bun run build`,
puis `cargo build -p echo-shell --release` (cargo doit être dans le PATH ; `export PATH=$HOME/.cargo/bin:$PATH`).

## Stack

| Couche | Techno |
|---|---|
| Coque, onglets, bouclier, pilotage | Rust (crate `cef` 152, binaires CEF 154) |
| Interface | React + TypeScript + Tailwind v4 + Framer Motion (Bun) |
| Données | SQLite (favoris, historique, téléchargements, réglages, permissions) |
| Terminal Claude Code | PTY Rust + xterm.js, session tmux partagée avec Quart |

Ports : sonde de présence `127.0.0.1:4330` ; port de débogage local `127.0.0.1` (aléatoire, DevTools ancrés et onglets ouverts par les extensions — seule l'origine `devtools://devtools` est admise en WebSocket). Pilotage local : prise Unix `$XDG_RUNTIME_DIR/echo-browser/control.sock`.

## Tester

- `./tools/test-control.sh` — prise de pilotage ; `./tools/test-ipc-origin.sh` — une page web ne pilote pas le cœur.
- `cargo test --workspace --exclude echo-shell` — crates sans Chromium (CI GitHub).
- `./tools/bench-ram.sh echo|chrome [attente_s]` — mémoire (PSS) sur des pages réelles, comparée à Chrome.
- **Les scripts de test lancent une instance isolée** (`ECHO_RUN_DIR`, `ECHO_CONTROL_NAME`) : ils ne touchent jamais au navigateur de l'utilisateur.

## Variables utiles (bancs et diagnostic)

`ECHO_DATA_DIR`, `ECHO_RUN_DIR`, `ECHO_LOG=debug`, `ECHO_SLEEP_AFTER_S`, `ECHO_BENCH_URLS`, `ECHO_BENCH_UI`, `ECHO_BENCH_JS`,
`ECHO_FLAGS`, `ECHO_ROUND=0`, `ECHO_TERM_CMD`, `ECHO_CONTROL=0`.

## Documentation

`BRIEF.md` (le quoi) · `EPICS.md` (stories, critères exécutables) · `STATE.md` (état, décisions, pièges) ·
`TODO.md` (backlog) · `ARCHITECTURE.md` (domaines et frontières) · `ARBORESCENCE.md` (carte des fichiers, générée).
