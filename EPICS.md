# EPICS — echo-browser (écrit le 06/10, ordre = priorité de Chris : perf, design, quotidien)

## E1 — Mémoire
- [x] E1.S1 — Banc comparatif Echo/Chrome — VERIFY: `BENCH_URLS=about:blank tools/bench-ram.sh echo 15`
- [x] E1.S2 — Veille auto des onglets inactifs — VERIFY: `ECHO_SLEEP_AFTER_S=20 tools/bench-ram.sh echo 110` < 500 Mo
- [x] E1.S3 — Autoplay bloqué, accueil local — VERIFY: repos < 470 Mo
- [x] E1.S4 — Veille manuelle (menu de l'onglet « Endormir », op `sleep`) + réglages `tabs.sleepEnabled` / `tabs.sleepAfterMinutes` (5 min par défaut) — VERIFY: réglage à 1 min → onglets endormis, désactivé → aucun ; op `sleep` sur la prise (fait le 06/10)
- [x] E1.S5 — Ne jamais endormir un onglet avec saisie non envoyée — VERIFY: page qui émet `echo:dirty`, veille 15 s, onglet resté éveillé (fait ; la frappe réelle `isTrusted` n'est pas encore rejouée automatiquement)
- [~] E1.S6 — Plancher : repos ~430 Mo contre ~380 pour Chrome (écart ~13 %), onglets éveillés +30 % (≈50 Mo/onglet contre ≈32). Essayé sans gain notable : `--renderer-process-limit=6` (−4 %, isolation perdue), `BackForwardCache` coupé (−1 %), `--no-sandbox` n'est pas en cause, bouclier hors de cause, onglets masqués bien signalés `hidden`. Piste restante : purge mémoire des renderers en arrière-plan. — VERIFY: `tools/bench-ram.sh`

## E2 — Design
- [x] E2.S1 — Marge et gouttière de la teinte de la barre — VERIFY: capture, pixels de la gouttière = teinte `shell`
- [x] E2.S2 — Bulles d'aide maison sur tous les boutons (couche unique `tooltip-layer.tsx`, 380 ms, raccourci en touche ; barre repliée = bulle système, la vue y est trop étroite) — VERIFY: capture Playwright au survol
- [x] E2.S3 — Nouvel onglet : suggestions (onglets ouverts, favoris, historique dédoublonné, filtre à la frappe) — VERIFY: capture à 6 lignes + op `suggest` filtrée (fait le 06/10 ; flèches/Entrée écrites mais pas rejouées au clavier réel)
- [x] E2.S4 — Barre repliée : la page prend toute la fenêtre, la barre (colonne entière, plus de rail) revient par-dessus au bord gauche — VERIFY: `ECHO_BENCH_UI` rejoue repli/survol/dépli, captures (page à x=9 repliée, barre au-dessus au survol) (fait le 06/10)
- [x] E2.S5 — Angles arrondis de la page : `clip-path` sur la racine + fond de vue de la teinte fenêtre + barres de défilement en surimpression (`roundness.rs`) — VERIFY: capture des quatre coins sur Wikipédia et Hacker News (fait le 06/10 ; `ECHO_ROUND=0` pour comparer)

- [x] E2.S6 — Neumorphisme clair ET sombre (consigne de Chris, 06/10) : une matière, double ombrage, 5 teintes × 2 schémas, bascule soleil/lune, pages statiques et angles suivent le thème — VERIFY: captures réelles dark/light (sidebar, accueil, angles) (fait le 06/10)
- [x] E2.S7 — Neumorphisme appliqué aux feuilles (réglages : champs en creux, interrupteurs, pastilles de teinte), au menu contextuel (thème clair vérifié) ; les surimpressions restent plates (opaques, pas d'ombre extérieure possible) — VERIFY: captures réglages + menu en clair (fait le 06/10)

## E3 — Quotidien
- [x] E3.S1 — Cookies persistants conservés après relance (mécanisme ; la session Google réelle reste à constater par Chris) — VERIFY: serveur local `Set-Cookie: Max-Age`, relance, `document.cookie` le contient (fait le 06/10 ; cookie de session perdu, comme Chrome)
- [ ] E3.S2 — Mots de passe = extension Proton Pass (le gestionnaire de Chromium n'existe pas en mode Alloy) : vérifier qu'elle remplit un formulaire — VERIFY: capture de la fenêtre de l'extension sur une page de connexion
- [ ] E3.S3 — Codecs H.264/AAC — VERIFY: `canPlayType('video/mp4; codecs="avc1.42E01E"')` ≠ "" sur http://127.0.0.1 (page de test). Essai du 06/10 : le `libffmpeg.so` d'Electron (45-alpha.8 et 44.5.1) est sans effet, ce CEF lie ffmpeg en statique. Reste : un CEF compilé `proprietary_codecs=true ffmpeg_branding=Chrome` (gros chantier) ou un build tiers
- [x] E3.S4 — Permissions caméra/micro/position/notifications/presse-papiers : question dans la barre (affichée même repliée), décision retenue par site (table `permissions`) — VERIFY: page de test locale : notifications accordées, position refusée, micro accordé ; capture de la question (fait le 06/10)

## E4 — Claude Code dans le navigateur
- [x] E4.S1 — Terminal Claude Code dans un onglet (`core/terminal` PTY + xterm.js, bouton `>_` de la barre) — VERIFY: `cargo test -p echo-terminal` + capture de la fenêtre avec Claude Code (fait le 06/10)
- [x] E4.S2 — Même session tmux que Quart : `tmux -L claude`, session `claude-navigateur`, survit à la fermeture du navigateur — VERIFY: `tmux -L claude ls | grep navigateur` (fait le 06/10)
- [x] E4.S3 — Claude lit les pages via le MCP `echo-browser` (6 outils : tabs, read, open, navigate, activate, close ; enregistré au niveau utilisateur, chargé à la prochaine session) — VERIFY: client MCP stdio : `browser_read` rend « Example Domain » (fait le 06/10)

## E5 — Automatisations
- [x] E5.S1 — Prise de pilotage locale (Unix 0600, pas de TCP) — VERIFY: `tools/test-control.sh`
- [ ] E5.S2 — Comptes connectés : profils/espaces par compte — VERIFY: deux espaces, cookies isolés

## E0 — Sécurité
- [x] E0.S1 — Une page web ne pilote pas le cœur par `echo://ui/ipc` — VERIFY: `tools/test-ipc-origin.sh`
- [x] E0.S2 — Surfaces `echo://` : une page web ne peut plus afficher une page interne dans un cadre (le terminal se lançait) — VERIFY: page avec iframe `echo://ui/terminal.html` : refus journalisé, pas de terminal lancé (fait le 06/10) ; reste la sonde 4330 (page fixe, sans en-tête d'origine croisée : OK)

## E6 — Retours de Chris du 06/10 soir (nuit 06→07/10) — ne rien oublier
- [x] E6.S1 — Texte centré dans le champ d'adresse et les onglets ; adresse (pilule en creux) distincte des onglets (section « Onglets », carte active en relief + repère) — VERIFY: capture de la vraie fenêtre
- [x] E6.S2 — Session conservée : onglets enregistrés au fil de l'eau (`persist.rs`), restitués au démarrage (actif chargé, autres endormis) — VERIFY: lancer, ouvrir 3 pages, fermer, relancer : les onglets sont revenus
- [x] E6.S3 — Les tests ne touchent jamais au navigateur de l'utilisateur (`ECHO_RUN_DIR`, `ECHO_CONTROL_NAME`, scripts de test isolés) — VERIFY: lancer `tools/test-control.sh` pendant qu'une instance tourne : elle reste vivante
- [x] E6.S4 — Angles de la page : masques de coin recolorables à chaud (clair et sombre, onglets déjà ouverts compris) — VERIFY: captures des 4 coins avant/après bascule de thème
- [x] E6.S5 — Identité cohérente pour éviter les captchas Google : version 154 partout, marque « Google Chrome » dans les en-têtes ET `navigator.userAgentData`, `platformVersion` vide — VERIFY: page d'empreinte vs Chrome headless (seule différence : GPU réel) — à confirmer à l'usage (le captcha peut aussi venir de l'IP/IPv6)
- [x] E6.S6 — Barre repliée : réapparition ANIMÉE et qui REPOUSSE la page (pas par-dessus) — code fait (`window.rs` : `DOCK_NOW`, `animate_dock`) — VERIFY: mesure image par image de la position de la page pendant l'ouverture (rafale `grim`)
- [x] E6.S7 — Favicons dans les onglets, conservés en session (`on_favicon_urlchange` lu par l'interface C directe : `CefStringList::clone().into_iter()` rendait une liste vide) — VERIFIÉ le 06/10 : capture GitHub, Wikipédia, kernel.org
- [x] E6.S8 — Fermeture par le gestionnaire de fenêtres (Super+Q) : `can_close` renvoie 1 après `persist::flush` ET les vues sont libérées avant `shutdown()` (sinon vérification Chromium en échec et ~12 s de rapport de plantage) — VERIFIÉ : `hyprctl dispatch closewindow` → processus terminé en 0,1 s, session enregistrée
- [x] E6.S9 — Clic droit réel dans les pages (page, lien, image, sélection, champ) : AUJOURD'HUI NE MARCHE PAS pour Chris sur Google — VERIFY: clic droit simulé (`send_mouse_click_event`) ouvre le menu
- [x] E6.S10 — F12 / Ctrl+Maj+I : DevTools complets (éléments, console, réseau, mémoire, performances) — VERIFY: la fenêtre DevTools s'ouvre sur l'onglet actif
- [x] E6.S11 — Clic droit dans la barre : zone des onglets (nouvel onglet, nouveau dossier, fermer les autres/à droite…), onglet (déjà là), dossiers d'onglets — VERIFY: capture des menus
- [x] E6.S12 — Dossiers d'onglets (groupes repliables) persistés en session — VERIFY: créer un dossier, y glisser 2 onglets, relancer
- [x] E6.S13 — Un compte par onglet (conteneurs) : cookies indépendants entre conteneurs (plusieurs comptes du même site en même temps) — risque connu : les extensions sont par profil Chromium — VERIFY: cookie posé dans le conteneur A absent du B
- [x] E6.S14 — MÉMOIRE RADICALE : battre Chrome, Firefox, Zen. Mesurer par processus, essayer : `--process-per-site`, isolation de site réduite, GPU et réseau dans le processus principal, `MALLOC_ARENA_MAX`, drapeaux V8, désactiver les sous-systèmes inutiles ; hibernation plus fine (délai court, défilement restauré) ; « micro-fichiers » : tout ce qui peut sortir de la RAM (état des onglets endormis sur disque) — VERIFY: `tools/bench-ram.sh` avant/après, 10 pages et repos, vs Chrome
- [x] E6.S15 — Démarrage rapide : temps jusqu'à la première page mesuré, < 3 s — VERIFY: horodatages du journal
- [ ] E6.S16 — Décodeur H.264/AAC : build CEF avec `proprietary_codecs=true ffmpeg_branding=Chrome` (voir TODO « Build CEF avec codecs ») — VERIFY: `canPlayType('video/mp4; codecs="avc1.42E01E"')` ≠ ""

## E7 — Mise à jour automatique (demande de Chris, 08/10)
- S1 Echo installé (`~/.local/opt/echo-browser`, marqueur `release.json`) interroge au démarrage puis toutes les 6 h la
  dernière release GitHub ; version plus récente → état « disponible ». Jamais pour la version de développement.
  Critère : faux serveur (`ECHO_UPDATE_URL`) annonçant une version plus haute → état disponible ; égale → rien.
- S2 Téléchargement en arrière-plan, empreinte SHA-256 vérifiée (fichier `.sha256` de la release), extraction dans
  `~/.local/opt/echo-browser.maj`. Critère : archive altérée → refusée, rien n'est préparé.
- S3 Bandeau « Echo x.y prêt → Redémarrer » ; au redémarrage, le lanceur bascule les dossiers (l'ancien gardé en
  `.precedent`) ; si la nouvelle version ne démarre pas, retour à l'ancienne. Critère : après redémarrage, version = x.y,
  onglets gardés.
- S4 Réglages → À propos : version, « Vérifier maintenant », mise à jour automatique oui/non.

## E8 — Compte Echo synchronisé (idée de Chris, 08/10)
- S1 Service gratuit Cloudflare (Worker + D1) : inscription, connexion, jetons de session. Adresse hors dépôt public
  (`.env.local`, injectée à la fabrication de l'archive). Critère : inscription puis connexion depuis un autre poste.
- S2 Chiffrement de bout en bout : clé dérivée du mot de passe sur la machine (PBKDF2), séparée en clé d'accès (envoyée,
  hachée côté serveur) et clé de chiffrement (jamais envoyée) ; données en AES-256-GCM. Critère : le serveur ne stocke
  aucune donnée lisible.
- S3 Synchro : réglages, noms de profils, favoris, conteneurs, extensions par profil ; fusion simple (le plus récent
  gagne, par type). Critère : deux instances, même compte, un favori ajouté sur l'une apparaît sur l'autre.
- S4 Premier lancement façon Arc : présentation en quelques écrans, puis créer un compte / se connecter / passer.
- S5 Réglages → Compte : état, synchroniser maintenant, se déconnecter.
