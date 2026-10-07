# echo-browser — État

**Statut** : navigateur fonctionnel, utilisable pour naviguer et bloquer — pas encore pour remplacer Zen au quotidien.
**Stack** : Rust (coque, onglets, bouclier) · CEF 152 / Chromium 152 · React + TypeScript + Tailwind (interface)
**Centre de contrôle** : fiche `.echoforge.yml` à jour — type `desktop`, sonde `4330`, `./start.sh` / `./stop.sh`.
**Branche** : `refonte/rust-cef` — la branche `main` porte encore le MVP Electron abandonné.

## Ce qui fonctionne, vérifié dans le navigateur
- Fenêtre native, colonne latérale à gauche, page dans un cadre à droite
- Onglets multiples : ouverture, bascule sans rechargement, fermeture
- Navigation : saisie d'adresse, précédent/suivant, rechargement
- **Bouclier** : blocage réseau (180 000 règles, 6,5 µs/requête), remplacement de scripts publicitaires,
  masquage d'éléments, scriptlets d'uBlock Origin exécutés **avant le premier script du site**
- **Aucune publicité sur YouTube**, pré-roll compris
- Le navigateur se présente comme un Chrome de bureau français (empreinte vérifiée côté serveur)
- Raccourcis clavier, onglets épinglés, déplacement dans la liste, zoom par onglet
- Extensions : inventaire lu dans le profil Chromium, installation depuis la boutique
- Favoris, historique daté, téléchargements réels, dix-neuf réglages — le tout en base SQLite
- Interface complète : colonne latérale, feuilles bouclier / bibliothèque / extensions / réglages,
  progression de chargement, plein écran qui démonte la barre
- **Clic droit complet** : menu construit selon la cible — lien, image, sélection, champ,
  page — en français, dessiné par nous au-dessus du contenu
- **Extensions** : installées par déclaration à Chromium, qui les télécharge et les
  maintient ; leur fenêtre s'ouvre au-dessus de la page, clic droit sur l'icône pour les
  gérer, panneau dépliant sur les permissions
- **Sonde de présence sur 127.0.0.1:4330** — le seul port ouvert, servant une page d'état au centre
  de contrôle. Aucun en-tête d'origine croisée : une page web peut la solliciter, jamais la lire.

## Mémoire — mesures du 06/10 (`tools/bench-ram.sh echo|chrome`, PSS cumulé, 10 pages réelles)
- Avant : Echo 1 770 Mo contre Chrome 1 126 Mo (profil vierge). Après veille + lecture auto bloquée : voir ci-dessous.
- **Veille des onglets** (`core/shell/src/sleep.rs`, `Tabs::put_to_sleep`) : un onglet inactif depuis
  300 s (`ECHO_SLEEP_AFTER_S`) voit son navigateur détruit, la fiche (URL, titre, fil) reste ; il se recharge
  à la sélection. 10 onglets : **1 145 Mo → 408 Mo** (seul l'actif et l'interface restent). Réveil vérifié.
  Jamais endormis : onglet actif, qui joue du son, en chargement, ou avec une saisie de l'utilisateur
  (la page prévient le cœur par `console.debug('echo:dirty')`, voir `sleep.rs`).
- `--autoplay-policy=document-user-activation-required` (comme Chrome) : −155 Mo sur YouTube.
- Mesuré, hors cause : le masquage/scriptlets du bouclier (1 202 vs 1 221 Mo sans).
- Reste un plancher fixe plus lourd que Chrome : ~540 Mo au repos (interface React ≈ 80–100 Mo de rendu
  + onglet d'accueil ≈ 100 Mo + processus principal ≈ 150–190 Mo), et ~67 Mo/onglet contre ~45 Mo chez Chrome.
- Accueil = page locale `echo://ui/nouvel-onglet.html` (plus Google) : repos 536 → 460 Mo.
- Variables de banc : `ECHO_BENCH_URLS`, `ECHO_BENCH_NO_INJECT`, `ECHO_BENCH_WAKE_AT_S`, `ECHO_FLAGS`.
- **Codecs propriétaires absents** du CEF officiel (H.264/AAC) : Twitch, Netflix, Spotify web ne liront pas.
  Remplacer `libffmpeg.so` par celui d'Electron ne change rien (ffmpeg est lié en statique dans ce CEF, essayé le 06/10).

## Claude Code dans le navigateur (06/10)
- Bouton `>_` de la barre → onglet `echo://ui/terminal.html` (xterm.js) branché sur un PTY (`core/terminal`).
- Commande par défaut : `tmux -L claude … new-session -A -s claude-navigateur -- claude` (même serveur tmux et
  même config que Quart, donc la session se retrouve depuis Quart et survit à la fermeture). `ECHO_TERM_CMD` la remplace.
- Routes `echo://ui/term/{read,write,resize}`, refusées à toute page qui n'est pas en `echo://`.
- Un onglet terminal endormi se recharge et rejoue la sortie récente (tampon 512 Ko côté cœur).

## Design : neumorphisme clair et sombre (06/10, consigne de Chris)
- `ui/src/spaces/space-palette.ts` : 5 teintes × 2 schémas, tous les jetons déduits de la teinte ; une seule matière
  (`shell`), le relief vient de deux ombres (`hi` en haut à gauche, `lo` en bas à droite). Ombres : `shadow-card` (relief),
  `shadow-pressed` (enfoncé : onglet actif, bouton actif), `shadow-field` (champs), `shadow-lift` (flottant).
- Bascule soleil/lune au pied de la barre ; choix dans le stockage local (`echo.scheme`, `echo.space`), jetons recopiés
  dans `echo.theme` pour les pages statiques (`ui/public/theme.js` : accueil, terminal, liseré).
- Le cœur retient la teinte de la fenêtre (`appearance.shell`) : fenêtre et angles de page justes dès le démarrage.
  Une page déjà ouverte garde l'ancienne teinte dans ses angles jusqu'au rechargement.


## Profils = identités (règle de Chris, 07/10)
- Comptes, cookies ET extensions jamais partagés entre profils. Cookies/comptes : un `RequestContext` par profil
  (`profile/conteneur-profil-<id>`) ; les conteneurs choisis dans un profil lui appartiennent (`profil-<id>--<conteneur>`,
  `profiles::scope_container` ; profil par défaut : identifiant nu, contextes existants gardés).
- Extensions : Chromium installe une extension déclarée dans TOUS ses profils (déclaration externe globale). Registre
  d'Echo `extensions/profils.json` (`echo_extensions::profiles`, migration : tout l'existant → profil par défaut). Le pont
  (chargé dans chaque profil Chromium) désactive ce qui n'appartient pas au profil, d'après le cookie `extensions` posé par
  Echo dans chaque contexte (`extension_profiles.rs`). Pièges mesurés : un cookie posé par CEF ne déclenche PAS
  `cookies.onChanged` → après un ajout/retrait, Echo ouvre un instant `appliquer.html` (page du pont) dans chaque contexte
  du profil, invisible ; Chromium garde l'ancien script du service worker même si la version change → le nom du script suit
  son contenu (`pont-<hash>.js`) ; `cookies.getAll({url})` ne rend rien, `cookies.get` oui.
  Test : `tools/test-extension-profiles.sh` (Proton depuis le catalogue, réseau requis).
- Tests : lancer sur un sway sans écran (`~/.local/share/jev-desktop/sway`, `WAYLAND_DISPLAY` dédié) pour ne pas ouvrir de
  fenêtres sur l'écran de Chris ; test-contextmenu/occlusion/popup interrogent Hyprland.

## Release publique 0.3.0 (07/10)
- `tools/package-release.sh <libffmpeg.so libre>` → `dist-release/*.tar.xz` + sha256. Décodeur libre : `out/FfmpegLibre` dans l'arbre
  CEF (`ffmpeg_branding="Chromium"`, `proprietary_codecs=false`, cible `third_party/ffmpeg`, ~5 min).
- Piège : « h264 » figure aussi dans le décodeur libre (table des descripteurs) ; le garde-fou teste `h264_has_num_reorder_frames` /
  `AAC decoder`, avec `grep -c` (un `grep -q` sous `pipefail` laisse tout passer : SIGPIPE de `strings`).
- Le moteur (compilé avec codecs) annonce H.264/AAC même avec le décodeur libre : sans GPU, une vidéo H.264 échoue (erreur 4) ; l'AAC
  n'a aucun chemin matériel, donc `codecs.rs` le retire de `canPlayType`, `MediaSource.isTypeSupported` et `mediaCapabilities`
  (injecté dans le flux HTML, avant les scripts du site). Vérifié : YouTube passe en AV1 + Opus.

- Décodeur complet à la demande (0.3.1) : Réglages → Vidéo → Installer télécharge `libffmpeg.so` de nwjs-ffmpeg-prebuilt (même
  branche Chromium, ABI vérifiée : symboles + lecture H.264/AAC sans GPU), garde le fichier seulement si son SHA-256 est celui
  épinglé, le range dans `<données>/codecs/` avec la branche (`chromium`). Au lancement, `codecs::adopt_installed` relance le
  processus avec ce dossier en tête de `LD_LIBRARY_PATH` (le moteur lie libffmpeg avant `main`). Test : `tools/test-codecs-install.sh`.

## Session et robustesse (06/10 soir)
- **Session** : `persist.rs` enregistre les onglets (historique, position, épinglé, titre, icône) dans `last-session.json`, 1,2 s après
  chaque changement et avant toute fermeture ; au démarrage `app.rs::restore_or_open` les restitue — seul l'actif est chargé, les
  autres sont **endormis** (démarrage rapide, RAM sobre). Réglage `session.restore` (actif). Une relance voulue (`restart.rs`) reprend tout en direct.
- **Fermeture** : `can_close` renvoie 1 après `persist::flush` — toute demande de fermeture de la fenêtre ferme l'application entière.
  (Avant : seule la vue de l'interface se fermait, laissant un fond gris.) Essai `hyprctl dispatch closewindow` : voir EPICS E6.S8.
- **Tests isolés** : `ECHO_RUN_DIR` (pid + journal) et `ECHO_CONTROL_NAME` (prise) — ne jamais lancer `./stop.sh` par défaut pendant que
  l'utilisateur utilise le navigateur ; mes tests ont tué son instance le 06/10 et lui ont fait perdre ses onglets.
- **Identité** (`identity.rs`) : en-têtes `sec-ch-ua*` ET `navigator.userAgentData` (CDP `Emulation.setUserAgentOverride` à chaque
  navigateur créé) annoncent Chrome 154.0.8037.94 avec la marque « Google Chrome » ; `platformVersion` vide comme Chrome Linux.
- **Angles** : `roundness.rs` pose 4 masques de coin (div fixes) dans chaque page, couleur = teinte de la fenêtre, recolorés à chaud
  par `window.__echoCorners` à chaque `setAccent`. Remplace le `clip-path` (qui laissait la teinte de création de la vue).
- **Barre repliée** : l'espaceur est animé (`DOCK_NOW` → `target_dock`, tick de 16 ms) et la page est repoussée, pas recouverte.
- **Favicons** : `on_favicon_urlchange` → `Tab.favicon` → `TabView.favicon` (CSP UI : `img-src` https: et http:).

## Pilotage par outil (06/10)
- Prise Unix `$XDG_RUNTIME_DIR/echo-browser/control.sock` (dossier 0700, prise 0600, `ECHO_CONTROL=0` la coupe) :
  une ligne JSON par demande — `tabs`, `read`, `open`, `navigate`, `activate`, `close`. Aucun port TCP.
- MCP `echo-browser` (`mcp/echo_browser_mcp.py`, venv `.venv-mcp`) par-dessus : Claude lit et ouvre des pages.
  Tout processus de l'utilisateur peut lire les pages ouvertes par cette prise — c'est voulu (comme le port de
  débogage de Chrome), à resserrer si un jour plusieurs utilisateurs partagent la machine.

## Ce qui n'existe pas encore
- Mode lecture
- Pagination de l'historique : la recherche ne rend que les soixante premières entrées
- Mise en sourdine ; veille manuelle et réglage du délai dans l'interface (la veille auto existe)
- Les extensions déclarées en ligne de commande demandent encore une relance

## Décisions structurantes
- **CEF plutôt que Tauri.** Tauri sur Linux utilise WebKitGTK : aucune extension possible, moteur en retrait,
  et cette configuration (Nvidia + Wayland) le fait planter. CEF embarque un vrai Chromium.
- **Style Alloy obligatoire** pour les vues. Une vue en style Chrome dans une fenêtre sur mesure cherche
  l'infrastructure d'onglets du vrai Chrome et fait planter le processus.
- **Schéma interne `echo://` plutôt qu'un serveur local.** Un port ouvert serait joignable par n'importe
  quelle page affichée dans le navigateur. **Mais** le schéma est aussi joignable en POST depuis une page web
  (la réponse est illisible, la demande part) : le pont `echo://ui/ipc` vérifie donc que l'appelant est une page
  `echo://` (faille trouvée et corrigée le 06/10, test : `tools/test-ipc-origin.sh`).
- **Blocage natif plutôt qu'extension.** uBlock Origin n'existe plus sur Chromium depuis juillet 2026
  (Manifest V2 supprimé). Le moteur lit les mêmes listes, sans dépendre du bon vouloir de Google.
- **Les scriptlets viennent d'uBlock Origin (GPL-3.0)** : distribuer le navigateur imposerait d'en publier
  le code source.

## Limites mesurées, à ne pas re-tenter

- **Angles arrondis de la page** : contournement par `clip-path` sur `<html>` (injecté au début du chargement) + fond de
  la vue = teinte de la fenêtre (les pages sans fond déclaré reçoivent `#fff` à `DOMContentLoaded`) + barres de défilement
  en surimpression (la colonne native n'est pas rognée). Seul le contenu de page est rogné, jamais l'interface.
- **Une vue posée au-dessus de la page ne peut pas être transparente.** Chromium peint un
  rectangle plein dessous : les angles arrondis se voient découpés. Trois essais le
  10/09 — fond transparent sur les réglages du navigateur, sur la vue, sur la page.
- **`--load-extension` est inopérant.** L'extension est listée, annoncée active, et
  toutes ses adresses répondent `ERR_BLOCKED_BY_CLIENT`. Ni le mode développeur ni
  `--disable-extensions-except` n'y changent rien. Passer par la déclaration externe.
- **Ne jamais écrire dans les préférences de Chromium.** Sa protection d'intégrité
  invalide l'entrée et l'extension cesse d'être chargée, tout en restant listée active.
  C'est ce qui a cassé Proton Pass le 10/09.
- **Le gestionnaire d'extensions de Chromium ne s'affiche pas** dans ce mode : page
  blanche. Toute la gestion doit être la nôtre.

## Pièges vérifiés, à ne pas réintroduire
- **La barre est une surimpression, pas un enfant de la disposition** (`window.rs`) : un espaceur réserve sa place.
  Une vue de **largeur nulle sort de la disposition** et garde son ancienne taille (largeur ≥ 1) ; la taille préférée de
  l'espaceur est mise en cache, il faut invalider *l'espaceur* (pas seulement la fenêtre).
- **Ne jamais tenir l'accès à l'état pendant un appel à Chromium** : il rappelle le programme au milieu.
  A causé un arrêt brutal à l'ouverture d'onglet, puis un figeage à la fermeture.
- **Ne pas fermer explicitement le navigateur d'un onglet** : la demande remonte à la fenêtre et la ferme.
  Retirer la vue du conteneur suffit.
- **Ne pas détacher une vue avant de la fermer** : Chromium remonte jusqu'à la fenêtre pour fermer.
- **L'application doit être fournie à `execute_process`**, sinon les processus de rendu ignorent `echo://`
  et l'interface s'affiche sans scripts ni styles.
- **Vulkan est incompatible avec Wayland** : sans `--disable-features=Vulkan`, le processus s'arrête (code 28).
- **`accept_language_list` ne se règle que dans la configuration**, pas dans un gestionnaire de requête,
  et doit être fourni sans pondération.

## Lancement
`./start.sh release` · journal `logs/browser.log` · `ECHO_LOG=debug` pour le détail
Binaires Chromium : `bash tools/fetch-cef.sh` (~1,5 Go, une fois)
Paquet de scriptlets : `bun tools/build-resources.mjs`
Autotest des onglets : `ECHO_SELFTEST=1 ./start.sh release`

## Session du 06/10 nuit — décisions et mesures
- Clic droit : cause = `on_before_context_menu` vidait le modèle → Chromium n'appelait plus `run_context_menu`. Modèle laissé intact. Menu des sites coupé dans la barre (`is_interface_page`). F12 / Ctrl+Maj+I/J = DevTools complets (`toggle_devtools`, fenêtre séparée).
- Dossiers d'onglets : `Tab.folder` (session) + réglage `tabs.folders` (JSON). Conteneurs : `Tab.container`, un RequestContext par conteneur, profil `profile/conteneur-<id>` (enfant DIRECT de la racine du cache, sinon « Cannot create profile »), init asynchrone → `containers::is_ready` + `later` (nouvel onglet/réveil différés de 40 ms). Les extensions ne suivent pas dans les conteneurs.
- Mémoire (10 pages, PSS cumulé, 70-100 s) : base 1786 ; `--in-process-gpu` 1733 ; `NetworkServiceInProcess2` 1711 ; site-per-process coupé 1704 ; `--renderer-process-limit=4` 1668 ; `--disable-gpu` 1587 (refusé : rendu) ; combo 1617. Aucun drapeau > 4 % hors désactivation GPU → non retenus. Purge V8 (`Memory.forciblyPurgeJavaScriptMemory`) RETIRÉE le 07/10 : elle détruit le contexte JavaScript de la page (onglet affiché mais mort : éditeur de claude.ai inerte, collage cassé, texte par-dessus le placeholder, Ctrl+R obligatoire). Le « -9 % » venait de pages tuées. Remplacée par `Memory.simulatePressureNotification` critique (caches + GC, page vivante, `tools/test-trim-alive.sh`) : 1603 Mo, gain ~1 %. Le vrai levier reste la veille (410–550 Mo pour 10 pages).
- Défilement restitué au réveil (message console `echo:scroll:`), sauvegardé en session.
- Démarrage : fenêtre affichée 0,3 s après le lancement, bouclier opérationnel à ~1,2 s.
- Pilotage de test : ops `click` (target chrome|page, button), `wheel`, `layout`, `ui`, `menu`, `devtools`. Tests : `tools/test-{contextmenu,folders,containers,sleep-scroll,sidebar-anim}.sh`.
- Build CEF avec codecs : sources en téléchargement dans `/mnt/backup/cef-build` (`build.sh`, BUILD=1 pour compiler, nice 19).
- Veille sous pression : au-delà de 4 pages en mémoire, délai ramené à 60 s (`sleep.rs::under_pressure`). 10 pages, 130 s : 1527 → 1032 Mo (4 pages gardées vivantes + onglet actif). Les onglets audibles, en chargement ou avec saisie ne dorment jamais.
- Recherche perf (agent Opus, 06/10) : changer de moteur NON justifié (Servo/Ladybird/WebKitGTK/Gecko : rien de mesuré en faveur, extensions et compat perdues). L'écart Echo/Chrome vient du build (CEF officiel = is_official_build, LTO, PGO) et des expériences Finch que CEF n'a pas. Sources dans le rapport de session ; chiffres tiers : Edge Sleeping Tabs -32 % mémoire, Chrome timers bridés ≤ 5× moins de CPU.
- Fait (sans nouveau build) : réveil anticipé au survol (`WarmTab`, clic en 2 ms) ; fenêtre masquée (autre espace Hyprland) ⇒ pages `hidden` via `occlusion.rs` (Wayland ne le signale pas) ; liste « jamais endormi » (messageries) ; bancs en PSS+SwapPss. Mesuré inutile : `ProcessPerSiteUpToMainFrameThreshold` (1549→1538, bruit ±5 %), VRAM (86 Mo pour 10 pages), CPU au repos (~0).
- Bloqué sans sudo (voir TODO) : décodage NVDEC (`libva-nvidia-driver`), zram 4 Go plein ⇒ pas de gel+pageout (cgroup `memory.reclaim` possible : le cgroup délégué `user@1000.service` est inscriptible, contrôleur memory actif).
- Build CEF : GN = `/mnt/backup/cef-build/gn-defines.txt` (is_official_build=true, codecs, symbol_level=0, fieldtrial_testing_config coupé, concurrent_links=2) + `--with-pgo-profiles` ; ninja limité à -j10 par un shim `bin/autoninja` (RAM libre ~16 Go) ; `chain.sh` lance la compilation après le téléchargement.
- Décodage vidéo GPU (NVDEC via VA-API) : `libva-nvidia-driver` installé (sudo), `LIBVA_DRIVER_NAME=nvidia` posé par `flags::prepare_environment`, features `AcceleratedVideoDecodeLinuxGL,VaapiOnNvidiaGPUs,VaapiIgnoreDriverChecks` ; `ECHO_HWDEC=0` coupe. vainfo : H.264, HEVC (Main/10/12), VP9, AV1. Mesure 4K VP9 (`tools/bench-video.sh`) : CPU 12 s → 4 s sur ~6 s de lecture, décodeur GPU actif. H.264/HEVC suivront avec le build codecs.
- Swap : zram1 (10 Go) ajouté à chaud ; config persistante `/etc/systemd/zram-generator.conf` passée à min(RAM/2, 12 Go) (effective au prochain démarrage, ancien fichier en .bak) (zram0 4 Go était plein).
- **Comparatif final (06/10 nuit, mêmes 10 pages, PSS+SwapPss cumulé)** : Chrome 1058 Mo (30 processus) ; Echo éveillé (sans veille) 1236 Mo (20 processus) ; Echo réglages par défaut après 130 s 736 Mo (12 processus). Gain allocateur (`MALLOC_ARENA_MAX=1`, `MALLOC_TRIM_THRESHOLD_=65536`, désactivable `ECHO_MALLOC=0`) : ~-15 %, CPU inchangé (69 s vs 69-71 s). Echo est ~30 % sous Chrome avec la veille, ~17 % au-dessus éveillé (écart attribué au build non officiel / Finch, à corriger par le build CEF officiel).
- Captcha Google (07/10) : (1) corrigé — `identity::apply` AJOUTAIT 9 en-têtes Client Hints à chaque requête (Chrome n'en envoie que 3, les autres sur demande `Accept-CH`) : il ne fait plus que corriger ceux que Chromium envoie ; (2) mesuré — un Chrome officiel avec profil NEUF reçoit aussi `/sorry` depuis cette IP après quelques recherches : un profil sans cookies Google (instances de test, bancs) déclenche le captcha, indépendamment d'Echo. Ne plus lancer de recherches Google depuis des profils jetables (ça dégrade la réputation de l'IP).
- Session 07/10 (nuit, Chris présent) :
  - Popups (`window.open`, liens `_blank`, popup de popup) → onglets (`on_popup_browser_view_created` + `delegate_for_popup_browser_view`), `window.opener` gardé ; `do_close` : `window.close()` ferme l'onglet, plus la fenêtre. Popup de connexion Google coupée par COOP (LinkedIn) → fermée et page d'origine rechargée (`finish_orphan_signin`, détecté sur l'erreur `postMessage` des scripts `/gsi/`).
  - Thème des pages = thème d'Echo (`scheme.rs`, CDP `Emulation.setEmulatedMedia prefers-color-scheme`).
  - DevTools ancrés à droite (`devtools.rs`) : frontend `devtools://devtools/bundled/devtools_app.html` dans une vue Alloy reliée par le port de débogage local (127.0.0.1, `--remote-allow-origins=devtools://devtools`), barre avec croix (`outils-barre.html`), poignée de redimensionnement (`outils-poignee.html`, overlay dans l'espace entre page et panneau). Les DevTools natifs CEF en style Chrome plantent dans notre fenêtre (ChromeBrowserWidget::Init) — ne pas réessayer. Le serveur `/json/list` répond sur le fil UI : l'interroger depuis un autre fil.
  - Bouclier : exception de site complète (décision sur l'URL principale de l'onglet), rechargement sans cache après bascule, coupure globale persistée (`shield.enabled`). Injection exclue des pages internes (corrompait les traductions des DevTools).
  - `echo://` branché dans chaque conteneur ; réglage `tabs.containers` déclaré (sa création était refusée) ; code source en onglet ; glisser un onglet sur un dossier.
- Build CEF codecs (07/10 01:45) : compilation lancée (80 301 étapes, siso -j10, garde-fou mémoire `garde-memoire.sh` qui suspend/reprend sous 2,5 Go dispo). Pièges rencontrés : `--no-build` crée `out/` → relancer avec `--force-config --force-build` ; `concurrent_links` interdit avec ThinLTO (build officiel) ; profils PGO absents (download sans `--with-pgo-profiles`) → `tools/update_pgo_profiles.py --target=linux update --gs-url-base=chromium-optimization-profiles/pgo_profiles` et `v8/tools/builtins-pgo/download_profiles.py download --depot-tools third_party/depot_tools --check-v8-revision` ; vérifier à blanc avec `autoninja -C out/Release_GN_x64 -n cefclient`.
  - Suite des pièges (07/10 matin) : `treat_warnings_as_errors=false` (bindgen/rustc `unnecessary_transmutes`) ; `use_sysroot=true` + `build/linux/sysroot_scripts/install-sysroot.py --arch=amd64` (sans sysroot : `-lffi_pic` introuvable au lien de SwiftShader) ; étapes Python tuées au hasard par un signal (cause non trouvée, ni OOM ni cgroup) → `relance.sh` relance tant que l'échec est un signal, s'arrête sur une vraie erreur.
- **Moteur CEF compilé installé (07/10 16:00)** : `~/.local/share/cef` = build local (codecs propriétaires, ffmpeg Chrome en `libffmpeg.so` séparé, officiel + PGO, sysroot Debian) ; l'ancien CEF officiel est gardé dans `~/.local/share/cef-officiel` (retour : renommer les dossiers). Vérifié : `canPlayType` H.264/AAC/HEVC « probably », MP4 H.264+AAC lu (avec et sans GPU ; HEVC via GPU seulement), tests de la coque verts. Mesures A/B (mêmes pages, l'une après l'autre) : mémoire +15 à +25 % avec Twitch/YouTube (leur vidéo se lance désormais), +6 à +12 % sans sites vidéo ; CPU identique (69 s vs 70-72 s). Le gain « build officiel » annoncé n'existe pas : le CEF de Spotify est déjà un build officiel. Seul apport réel : les codecs. `libcef.so` 533 Mo (contre 1 455).
- Profils façon Arc (07/10) : les 5 pastilles du bas = profils (`profiles.rs`) ; `Tab.space` (session) ; `SetSpace` affiche les onglets du profil (dernier utilisé, sinon accueil) ; profil par défaut `graphite` = contexte commun (connexions existantes), les autres = conteneur `profil-<id>` (cookies à part) ; noms modifiables (réglage `profiles.names`, section Profils). Test : `tools/test-profiles.sh`.
- Menu clic droit des pages : fermé au clic gauche dans la page (`echo:press` injecté) ou dans la barre ; focus donné à l'ouverture ; fermeture tardive de l'ancien menu ignorée 250 ms. Téléchargements : fonctionnaient, maintenant annoncés (début, fin). Copier l'image : `clipboard.rs` (ureq + crate image → PNG → wl-copy/xclip), entrées vidéo (ouvrir, copier l'adresse, enregistrer).
- Pages pleine largeur (07/10) : `ui/pages.html` (entrée Vite `pages`, `src/pages/`) — Réglages et Bibliothèque dans un onglet (`OpenPage`, retour sur l'onglet existant du profil) ; le cœur diffuse ses événements aussi aux onglets `echo://ui/pages.html` ; l'état de chargement des pages d'Echo en onglet est suivi. La teinte se choisit toujours dans la barre (changer de teinte = changer de profil). Bouclier et extensions restent dans la barre.
- Bouclier : l'état était publié pour l'onglet 0 alors que l'interface le lit pour l'onglet actif → interrupteurs toujours « actifs ». Corrigé. Veille : signal de lecture (`echo:media:`) — un onglet qui lit (vidéo avec son, même en arrière-plan) ne dort ni n'est purgé ; Chromium met lui-même en pause les vidéos muettes cachées.
- Web Store (07/10) : la boutique renvoie vers le Chrome du système ; `store.rs` pose sur les fiches `chromewebstore.google.com/detail/…` un bouton « Ajouter à Echo » → message console `echo:install:<fiche>` (accepté seulement si la page ET la fiche sont du catalogue) → installation par le gestionnaire d'Echo (paquet déclaré, installé au démarrage suivant). Vérifié : Proton Pass présent dans `profile/Default/Extensions/` après relance.
- Extensions et onglets (07/10) : les API d'extension (`tabs.create`, `windows.create`) échouaient (« No current window ») — la fenêtre d'Echo n'est pas une fenêtre Chrome. `anchor.rs` crée une fenêtre Chrome jamais affichée (point d'ancrage) ; `anchor_watch.rs` écoute le protocole de débogage (`Target.setDiscoverTargets`), repère les pages qui ont une fenêtre Chrome (`Browser.getWindowForTarget` réussit seulement pour l'ancrage), les ferme et les rouvre comme onglets d'Echo. Features `Glic,GlicActor,GlicActorUi` désactivées (plantage TabInterface::GetFromContents en ajoutant un onglet à l'ancrage). `tabs.query` (07/10) : réglé par `extension_tabs/` — pont interne (`pont/`, id fixe par clé, permission `debugger` : `debugger.getTargets` donne l'id réel des onglets ; répond seulement aux extensions qui ont `tabs` ou un accès à tous les sites) + `tabs.query` remplacé dans les pages d'extension (ordre et onglet actif d'Echo, filtres url/title/active…). L'id réel suffit au reste (`tabs.get`, `sendMessage`, `scripting`). `--load-extension` refonctionne (CEF 154) : réservé au pont et aux tests. Test : `tools/test-extension-query.sh`. Test : `tools/test-extension-tabs.sh`.
