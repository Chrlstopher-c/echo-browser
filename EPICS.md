# EPICS — echo-browser (écrit le 06/10, ordre = priorité de Chris : perf, design, quotidien)

## E1 — Mémoire
- [x] E1.S1 — Banc comparatif Echo/Chrome — VERIFY: `BENCH_URLS=about:blank tools/bench-ram.sh echo 15`
- [x] E1.S2 — Veille auto des onglets inactifs — VERIFY: `ECHO_SLEEP_AFTER_S=20 tools/bench-ram.sh echo 110` < 500 Mo
- [x] E1.S3 — Autoplay bloqué, accueil local — VERIFY: repos < 470 Mo
- [x] E1.S4 — Veille manuelle (menu de l'onglet « Endormir », op `sleep`) + réglages `tabs.sleepEnabled` / `tabs.sleepAfterMinutes` (5 min par défaut) — VERIFY: réglage à 1 min → onglets endormis, désactivé → aucun ; op `sleep` sur la prise (fait le 06/10)
- [x] E1.S5 — Ne jamais endormir un onglet avec saisie non envoyée — VERIFY: page qui émet `echo:dirty`, veille 15 s, onglet resté éveillé (fait ; la frappe réelle `isTrusted` n'est pas encore rejouée automatiquement)
- [ ] E1.S6 — Plancher : processus principal et interface sous Chrome — VERIFY: repos ≤ 360 Mo (Chrome 351)

## E2 — Design
- [x] E2.S1 — Marge et gouttière de la teinte de la barre — VERIFY: capture, pixels de la gouttière = teinte `shell`
- [x] E2.S2 — Bulles d'aide maison sur tous les boutons (couche unique `tooltip-layer.tsx`, 380 ms, raccourci en touche ; barre repliée = bulle système, la vue y est trop étroite) — VERIFY: capture Playwright au survol
- [x] E2.S3 — Nouvel onglet : suggestions (onglets ouverts, favoris, historique dédoublonné, filtre à la frappe) — VERIFY: capture à 6 lignes + op `suggest` filtrée (fait le 06/10 ; flèches/Entrée écrites mais pas rejouées au clavier réel)
- [x] E2.S4 — Barre repliée : la page prend toute la fenêtre, la barre (colonne entière, plus de rail) revient par-dessus au bord gauche — VERIFY: `ECHO_BENCH_UI` rejoue repli/survol/dépli, captures (page à x=9 repliée, barre au-dessus au survol) (fait le 06/10)
- [x] E2.S5 — Angles arrondis de la page : `clip-path` sur la racine + fond de vue de la teinte fenêtre + barres de défilement en surimpression (`roundness.rs`) — VERIFY: capture des quatre coins sur Wikipédia et Hacker News (fait le 06/10 ; `ECHO_ROUND=0` pour comparer)

## E3 — Quotidien
- [x] E3.S1 — Cookies persistants conservés après relance (mécanisme ; la session Google réelle reste à constater par Chris) — VERIFY: serveur local `Set-Cookie: Max-Age`, relance, `document.cookie` le contient (fait le 06/10 ; cookie de session perdu, comme Chrome)
- [ ] E3.S2 — Mots de passe = extension Proton Pass (le gestionnaire de Chromium n'existe pas en mode Alloy) : vérifier qu'elle remplit un formulaire — VERIFY: capture de la fenêtre de l'extension sur une page de connexion
- [ ] E3.S3 — Codecs H.264/AAC — VERIFY: `canPlayType('video/mp4; codecs="avc1.42E01E"')` ≠ "" sur http://127.0.0.1 (page de test). Essai du 06/10 : le `libffmpeg.so` d'Electron (45-alpha.8 et 44.5.1) est sans effet, ce CEF lie ffmpeg en statique. Reste : un CEF compilé `proprietary_codecs=true ffmpeg_branding=Chrome` (gros chantier) ou un build tiers
- [ ] E3.S4 — Permissions caméra/micro/notifications/position — VERIFY: page de test, invite visible

## E4 — Claude Code dans le navigateur
- [x] E4.S1 — Terminal Claude Code dans un onglet (`core/terminal` PTY + xterm.js, bouton `>_` de la barre) — VERIFY: `cargo test -p echo-terminal` + capture de la fenêtre avec Claude Code (fait le 06/10)
- [x] E4.S2 — Même session tmux que Quart : `tmux -L claude`, session `claude-navigateur`, survit à la fermeture du navigateur — VERIFY: `tmux -L claude ls | grep navigateur` (fait le 06/10)
- [x] E4.S3 — Claude lit les pages via le MCP `echo-browser` (6 outils : tabs, read, open, navigate, activate, close ; enregistré au niveau utilisateur, chargé à la prochaine session) — VERIFY: client MCP stdio : `browser_read` rend « Example Domain » (fait le 06/10)

## E5 — Automatisations
- [x] E5.S1 — Prise de pilotage locale (Unix 0600, pas de TCP) — VERIFY: `tools/test-control.sh`
- [ ] E5.S2 — Comptes connectés : profils/espaces par compte — VERIFY: deux espaces, cookies isolés

## E0 — Sécurité
- [x] E0.S1 — Une page web ne pilote pas le cœur par `echo://ui/ipc` — VERIFY: `tools/test-ipc-origin.sh`
- [ ] E0.S2 — Auditer les autres surfaces `echo://` (icônes, fichiers) et la sonde 4330 — VERIFY: page de test qui tente chaque URL, aucune donnée lisible
