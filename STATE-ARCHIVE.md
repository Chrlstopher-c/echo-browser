# STATE — archive (sections déplacées de STATE.md, du plus récent au plus ancien)

## Garder éveillé (07/10)
- `Tab.keep_awake` (session + `TabSnapshot`), `UiRequest::KeepTabAwake` ; épargné par `sleep_candidates` et
  `take_trim_targets` ; restitué endormi au démarrage → réveillé au premier passage de la veille (15 s).
  Menu : `MenuItem checked`, pastille `AwakePip` (soleil) sur `TabMark`.

## Inspecteur et menu des sites (07/10)
- « Examiner l'élément » : la page marque `window.__echoInspect = elementFromPoint(clic / zoom)` AVANT l'ouverture (le
  panneau rétrécit la page), puis `devtools::reveal_marked` exécute dans l'inspecteur un script qui importe ses modules
  (`devtools://devtools/bundled/core/sdk/sdk.js`, `common.js`), résout l'objet en nœud et `Common.Revealer.reveal`.
- Largeur des outils : réglage `devtools.width`, enregistré 600 ms après la fin du glissement.
- Menu des sites : la surimpression ne sait toujours pas être transparente (revérifié : coin noir) → carré, relief par
  liserés `--color-hi/--color-lo` + icônes par action.

## Release 0.4.0 : proposition du décodeur, installateur (08/10)
- Proposition : le shim des codecs (décodeur libre seulement) envoie `echo:codecs` quand une page demande de l'AAC ou
  qu'une vidéo échoue au décodage (code 3, ou 4 hors webm/ogg) → `codecs::page_needs_codecs` (une fois par session,
  réglage `video.codecsPrompt` pour « Jamais ») → bandeau `sidebar/codecs-strip.tsx` : Installer / Plus tard / Jamais,
  puis Redémarrer. Test du parcours : `tools/test-codecs-prompt.sh` (archive, sans GPU).
- Archive : `installer.sh` (copie dans ~/.local/opt/echo-browser, entrée de menu + icône `data/echo-browser.svg`,
  commande `echo-browser`, `--retirer`), ne bloque que si la copie installée est ouverte.

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

<!-- archive du 09/10 -->
## Raccourcis et fichiers locaux (demande de Chris, 08/10)

- Lettres lues dans le caractère produit (disposition du clavier), plus dans le code de touche : en AZERTY, le code
  suit souvent la position US (Ctrl+W arrivait comme Ctrl+Z). Ajoutés : Ctrl+O (sélecteur du système via le portail
  du bureau — crate `rfd`, dans un fil à part : le dialogue intégré de CEF faisait tomber Echo), Ctrl+Maj+T, Ctrl+D,
  Ctrl+H/J, Ctrl+P, Ctrl+S, Ctrl+U, Alt+←/→, Ctrl+Page↑/↓, Ctrl +/−/0.
- Barre d'adresse : `/chemin`, `~/…` → `file://` (dossier listé, fichier ouvert) ; `localhost:3000`, IP:port → http.
- `echo-browser fichier…` ouvre les pages ; Echo déjà lancé → confiées par la prise de pilotage. Entrée de bureau :
  `%U` + types (HTML, PDF, images, texte, dossiers). Pilotage : op `key` (frappe simulée).
- Piège de banc : un dialogue de fichiers passe par le portail de la session D-Bus de l'utilisateur et s'ouvre sur SON
  écran → tests avec un bus D-Bus privé (`test-shortcuts.sh`). Tests : `test-shortcuts.sh`, `test-launch.sh`.

## Signaux anonymes partagés (E9 S2, nuit du 08/10) — PAS ENCORE DÉPLOYÉ sur le service de production

- Réglage « Partager des signaux anonymes » (`signals.share`, Vie privée), coupé par défaut. Activé seulement : domaines
  visités et hôtes bloqués par le bouclier comptés par jour (`signals_day`) ; chaque jour fini part en un lot sans
  compte, jeton aléatoire neuf (aucun identifiant stable), puis est effacé ; coupé → tout ce qui attendait est effacé.
- Service : `POST /v1/signaux` (jour fini de moins de 8 j, doublon refusé, 3 lots par adresse et par jour via empreinte
  HMAC jour+IP effacée après 2 j), agrégats `signaux` (installations, total), gardés 90 j. Administration : section
  « Signaux partagés », clés montrées seulement au-delà de k installations un même jour (`SEUIL_K`, 3).
- À faire au déploiement : `wrangler d1 execute --remote --file schema.sql` (3 tables ajoutées) puis `wrangler deploy`.
- Test : `tools/test-signals.sh` (rien sans accord, lot envoyé puis effacé, seuil k).

## E10 — idées de Chris, première vague (nuit du 08/10)

- Réseau (`core/network`, `shell/src/network.rs`, `ui/src/network`) : chaque requête notée par onglet (500 dernières,
  résumé par domaine, tiers via le domaine enregistrable), diffusée seulement panneau ouvert. Règles par site
  (`reseau.json`) : domaine bloqué sur ce site, isolement strict ; appliquées avant le bouclier, même site en exception.
- Vue « Poids » (idée 12) : part des tiers, poids par type, requêtes les plus lourdes / lentes, alertes (image > 500 Ko,
  script tiers > 100 Ko). Piège de banc : une image invalide est coupée par Chromium (0 octet compté).
- Journal d'accès par site (table `site_journal`, 200 par site) : premier contact avec un tiers, permissions et décision,
  téléchargements. Onglet « Journal » du panneau Réseau.
- Reprise exacte (`page_state.rs`) : saisies (jamais mots de passe / carte) + position des médias, dans la session,
  rejouées au réveil et à la relance sans écraser une saisie.
- Routines (`routines.rs`, table `sequences`/`routines`) : suites de 2 à 4 sites par empreinte, proposées au 3e passage.
- Mémoire de structure (`page_memory.rs`, table `hidden_elements`) : « Masquer cet élément » retenu par empreinte de
  gabarit (squelette balises + classes stables, répétitions écrasées), appliqué aux pages de même gabarit DU MÊME SITE.
  Sécurité : une page peut écrire dans la console comme nos scripts — un masquage n'est cru que dans les 3 s qui suivent
  un vrai « Masquer » de l'utilisateur, et un autre site qui annonce le même gabarit ne reçoit rien.
- Surveiller une page (idée 15, `watch.rs`, table `watched_pages`) : clic droit « Surveiller cette page », texte lu à
  chaque visite (seulement à notre demande, fenêtre de 15 s), lignes ajoutées/retirées montrées en bas de la barre.
- Tests : `test-network.sh`, `test-page-state.sh`, `test-routines.sh`, `test-hide-element.sh`, `test-watch.sh`.

## Tableau de bord des créateurs (E9 S1, 08/10)

- Dans Echo : page « Administration » (pages pleine largeur + lien dans Réglages → Compte), visible seulement si le
  compte connecté porte le drapeau admin (table `admins` ; relu à chaque synchro via `/v1/moi`). Le serveur vérifie la
  session admin à CHAQUE appel : cacher la section n'est que de l'affichage. Test : `tools/test-admin.sh`.
- Détail (S1b) : usage par compte et par jour (table `usage`, 90 j), machines par session (table `machines`, sans nom
  de machine), fiche d'un compte (requêtes/jour, actions, machines, coffre par type), actifs/jour, synchros/jour,
  envois par type, plus actifs. Compte de Chris admin en production (08/10).
- Pages pleine largeur : elles défilent (le `body` garde `overflow: hidden` pour la barre).
- Secours web : `/admin` sur le service (clé `ADMIN_KEY` en secret Wrangler ; sert aussi à nommer le 1er admin). Comptes
  (e-mail complet, choix de Chris), actifs 1/7/30 j, sessions, coffre par type (tailles), requêtes et erreurs par jour,
  routes, versions d'Echo (en-tête `X-Echo-Version` envoyé par le navigateur). Actions : déconnecter, supprimer.
- Jamais le contenu du coffre : chiffré sur les machines. Tables ajoutées : `activite`, `compteurs` (CREATE IF NOT EXISTS).
- Test : `compte/test/compte.test.mjs` (avec `ADMIN_KEY`).

## Onglets des autres machines + pont (0.7.0, 08/10)
- Type `onglets` du coffre : une entrée par machine (`<données>/machine-id` aléatoire, nom = /etc/hostname, 100 pages web
  max) ; chaque machine n'écrit que la sienne (fusion par clé). Bibliothèque → « Machines » (section absente sans autre
  machine) ; clic = nouvel onglet. `shell/src/account/machine.rs`, `ui/src/library/devices/`.
- Pont (`bridge/script.rs`) : l'état de départ n'était rejoué qu'au PREMIER abonné ; la page pleine largeur a plusieurs
  abonnés → certains états manquaient jusqu'à leur prochaine publication. L'amorce garde le dernier événement de chaque
  sorte d'état et le rejoue à chaque abonné (pas les ponctuels ni les incrémentaux par onglet).

## Releases publiées le 08/10 (nuit)
- v0.5.0 (mise à jour automatique + correctif profil principal), v0.6.0 (compte Echo), v0.7.0 (onglets des autres
  machines, pont), rattachées au commit de la
  branche `nuit/2026-10-06` (le mode nuit interdit de pousser `main` : à faire avancer au retour de Chris).
- Vérifié pour de vrai : une 0.5.0 téléchargée de GitHub et installée (`installer.sh`, HOME temporaire) a trouvé la 0.6.0,
  l'a vérifiée, préparée, puis a basculé au redémarrage (0.5.0 gardée en `.precedent`, essai validé) ; idem 0.6.0 → 0.7.0.

## Compte Echo synchronisé (0.6.0, 08/10)
- Service : `compte/` (Worker + D1, offre gratuite), déployé (`pnpm exec wrangler deploy` dans compte/), adresse dans
  `.env.local` (ECHO_SYNC_URL) → `start.sh` l'exporte, `package-release.sh` la met dans `release.json` (`sync`).
  Routes `/v1/{sel,inscription,connexion,deconnexion,coffre,coffre/:type,compte}` ; 10 échecs/15 min → 429 ; sel fictif
  stable pour une adresse inconnue (pas d'énumération) ; écriture du coffre avec version de base (409 si concurrence).
  Calcul serveur minimal (SHA-256) : l'offre gratuite limite à 10 ms de CPU par requête. Tests : `cd compte && pnpm dev`
  puis `node test/compte.test.mjs` (COMPTE_URL pour viser le service en ligne ; nettoyer les comptes `essai-%`).
- Chiffrement (`core/account/crypto.rs`, ring) : PBKDF2-SHA256 600 000 → HKDF → clé d'accès (envoyée) / clé de chiffrement
  (jamais envoyée) ; AES-256-GCM avec le type en données associées. Fusion à trois voies (`merge.rs`) : réglages par clé,
  favoris par url, extensions par profil en ensembles. Synchro : connexion, +10 s au démarrage, toutes les 10 min, bouton.
- Synchronisé : 14 réglages choisis (`account/local.rs`), favoris, extensions par profil (déclarées sur la machine qui ne les
  a pas). Jamais : cookies, mots de passe, historique. Compte local : `<données>/compte.json` (0600).
- Premier lancement (aucune session, `onboarding.done` faux) : `pages.html#bienvenue` (3 écrans + compte, « Passer ») ;
  `ECHO_NO_WELCOME=1` pour les essais. Tests : `tools/test-account-sync.sh` (deux instances), `cargo test -p echo-account
  -- --ignored` avec COMPTE_URL (deux machines au niveau de la crate).

## Mise à jour automatique (0.5.0, 08/10)
- `core/shell/src/update/` : archive installée reconnue par `release.json` (version + dépôt, écrit par package-release) ;
  vérification 30 s après le démarrage puis toutes les 6 h (`ECHO_UPDATE_URL`, `ECHO_UPDATE_DELAY_S` pour les essais) ;
  téléchargement en flux + SHA-256 (digest GitHub ET fichier `.sha256`, qui doivent concorder), extraction dans
  `<installation>.maj` ; bandeau « prêt → Redémarrer ». La bascule et le retour en arrière sont dans le LANCEUR
  (`echo-browser.sh`) : un binaire cassé ne peut pas revenir seul ; la relance interne passe par le lanceur.
  Version courante = celle de `release.json`. Réglage `updates.auto`. Test : `tools/test-update.sh`.

## INCIDENT 08/10 : comptes déconnectés (corrigé)
- Cause : sans `cache_path`, CEF (runtime Chrome) ouvre au démarrage le « dernier profil utilisé » de `Local State`
  (`profile.last_used`) ; une fenêtre Chrome d'extension ouverte dans un conteneur le change → au redémarrage, le contexte
  commun = un conteneur vide (`conteneur-profil-sable`), tous les comptes du profil principal « déconnectés ». Données
  intactes dans `profile/Default` (cookies vérifiés sur copie). Correctif : `--profile-directory=Default` (flags.rs).
  Test qui reproduit puis vérifie : `tools/test-default-profile.sh` (échoue sans le correctif).

