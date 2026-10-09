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


## Audit UX → 10/10 (E13→E17, autonomie du 09/10) — release 0.11.0
Audit Opus du 08/10 (notes : navigation 5, esthétique 7,5, grand public 4, finition 4,5, concurrents 6,5) traité en
entier, une story = un test isolé (`tools/banc.sh` + `tools/banc.py` : prélude et outils communs des tests).
- **Navigation** : Ctrl+F (barre sous l'adresse, compteur, Échap ; `find.rs`) ; palette avec ligne « Rechercher … »
  et suggestions du moteur (`search.suggest`) ; moteur au choix (7, `search.rs`, enfin branché : il était codé en dur) ;
  rouvrir/popups/conteneurs à leur place ; plein écran bord à bord + Échap ; paliers de zoom de Chrome ; barre
  repliée d'elle-même sous 900 px + poignée + Ctrl+Alt+S ; navigation pendant un réveil gardée.
- **Bug clavier trouvé** : avec Ctrl, Chromium rend le caractère en code de contrôle (`& 0x1F`) : Ctrl+0 devenait
  Ctrl+P (impression), Ctrl+& Ctrl+F. Le code de contrôle ne vaut plus que pour une touche lettre (`shortcuts.rs`).
- **Finition** : dates (le cœur horodate en secondes : `fromCoreTime`) ; Réglages sans clés internes (seules les
  entrées du catalogue s'affichent ; 4 réglages affichés n'agissaient pas : ne-pas-suivre, tout effacer à la
  fermeture, demander où enregistrer, moteur — tous branchés), sommaire, thème Clair/Sombre/Système (portail XDG,
  `system_theme.rs`) ; bouclier compté par onglet et par page ; Ctrl+D confirmé + étoile ; notifications à boutons ;
  téléchargements avec progression ; Chromium en français (`LANGUAGE=fr` : Linux ignore `locale`) ; lecture réglable ;
  initiales teintées quand une icône manque.
- **Grand public** : contrastes ≥ 4,5:1 (garde CI `ui/scripts/contrast.ts`) ; onglets au clavier (tablist) ; profils
  en cibles de 24 px avec le nom actif ; cadenas honnête (certificat refusé = « Non sécurisé », résumé en clair) ;
  veille selon la mémoire libre (`sleep.rs`, plus selon le nombre d'onglets) ; bouton Claude Code seulement si
  installé ; accueil qui configure (thème, moteur, import) ; recherche dans l'Aide.
- **Profils = identités jusque dans les données** : base v3 (`space` sur `history` et `bookmarks`), dossiers,
  suggestions, nouvel onglet par profil ; la synchro porte le profil (`s` / `space`, absents = principal). Une
  installation neuve n'a qu'un profil ; les profils d'origine qui ont un dossier restent (`profiles.legacy`).
- **Face aux concurrents** : import Chrome/Chromium/Brave/Edge/Vivaldi/Firefox/Zen (`core/library/src/import.rs`) ;
  navigation privée (contexte en mémoire `prive`, Ctrl+Maj+N) ; effacer les données (Ctrl+Maj+Suppr) ; nouvel onglet
  en tuiles ; conteneurs nommés sur l'onglet + « Ouvrir dans : … » ; couper le son ; fiches proposées au focus (la
  page ne peut que déclencher l'offre, le remplissage part d'un clic dans Echo).

## Releases 0.11.0 et 0.11.1 (09/10 matin) — main à jour
Second contre-audit après E18 : navigation 8, esthétique 8, grand public 7, finition 7,5, concurrents 7,5. Ses défauts
corrigeables sont livrés dans la 0.11.1 : adresse vide sur un nouvel onglet, recherche Ctrl+F fermée à tout changement
d'onglet (`find::active_changed`), repli en http d'un domaine tapé sans https (`http_fallback`), « Traduire » seulement
pour une page publique hors privé, vraies icônes des sites (`history::set_favicon`, historique et favoris), marque
d'Echo non alphabétique, intitulés 11 px, Aide par mots-clés. Page récapitulative (artefact privé) :
https://claude.ai/artifact/CSBRVVyqCoWwGUSjSJbr3c. Reste pour 10/10 : voir TODO.md.

## Contre-audit du 09/10 et E18
Contre-audit Opus (agent isolé, mêmes règles) : navigation 7,5, esthétique 8, grand public 6,5, finition 6,5,
concurrents 7,5, avec 10 bugs (N1→N10). Tous corrigés (E18) : Précédent qui ramenait à l'accueil (l'accueil sort du fil
de l'onglet), surlignages Ctrl+F effacés au changement d'onglet, pages en échec ni dans l'historique ni en tuile et
cadenas « Page non chargée » (`Security::Failed`), `HttpsUpgrades` coupé (CEF ne revenait pas en http), thème servi
par le cœur aux pages des autres profils et du privé (`echo://ui/data/theme`, `SetPageTheme`, `SetSchemeChoice`),
onglet privé neutre, fiche proposée sous l'adresse (`FormOffer`), barre flottante en fenêtre étroite (`floating()`),
« Traduire la page », Réglages → Mots de passe, textes ≥ 10,5 px. Écarts : voir EPICS E18.

## Popups, reprise de session, arrêt (08/10 soir)

- **Popups** : depuis la 0.4.0, toute fenêtre `window.open` qui garde `window.opener` restait sur about:blank. Cause
  mesurée : l'auto-attachement du débogueur au niveau du navigateur (pause des service workers d'extensions,
  `extension_tabs/workers.rs`) fige ces fenêtres dans CEF, quel que soit le filtre (sessions non aplaties refusées,
  « tab » + « page » refusés ensemble, relance des cibles sans effet). **Choix de Chris** : les extensions gardent tout,
  les popups s'ouvrent en onglet normal (`on_before_popup` → `bridge::page_opens_window`), sans `window.opener`,
  rattachées à leur page (retour à la fermeture, connexion Google finie par rechargement).
- **Reprise de session** : dossier, épinglage, historique d'un onglet rouvert vivant allaient à l'onglet actif (pas
  au rouvert) — `open_tab_in` rend maintenant l'onglet créé.
- **Arrêt par signal** (`shutdown.rs`) : SIGTERM/SIGINT/SIGHUP enregistrent d'abord les onglets, puis rendent la main
  au gestionnaire de Chromium (qui finit proprement : état local, profils).

## E12 — palette, formulaires, lecture (autonomie, 08/10 après-midi) — release 0.10.0, main à jour

- **Palette d'adresse** (`ui/src/address/use-suggestions.ts`, `suggest::for_address`) : en tapant, onglets ouverts,
  favoris, historique sous l'adresse ; ↓/↑ Entrée ; un onglet ouvert est rejoint. Ctrl+K = Ctrl+L, qui donnent
  désormais le vrai focus clavier à la barre (avant : la frappe restait dans la page). Test `tools/test-palette.sh`.
- Bug corrigé au passage : le titre d'une page était écrit sur la visite précédente dans l'historique (le titre arrive
  avant l'adresse) — `set_tab_title` lit l'adresse dans la trame.
- **Formulaires** (`core/shell/src/forms/`, `ui/src/forms/`) : fiches dans le réglage synchronisé `forms.cards`
  (chiffré avec le reste), Réglages → Formulaires ; clic droit dans un champ → « Remplir : fiche » (3 au menu) ;
  seuls les champs vides reconnus (autocomplete, nom, libellé) ; jamais mot de passe, carte, IBAN. `tools/test-forms.sh`.
- **Mode lecture** (`core/shell/src/reader/`, Readability 0.6.0 de Mozilla, Apache-2.0, embarqué) : clic droit
  « Lire en mode lecture » / Ctrl+Alt+R ; vue construite DANS la page (jamais dans une page echo://, que le contenu
  d'un site ne doit pas toucher) ; sortie = rechargement. Moins de 250 caractères = pas d'article. `tools/test-reader.sh`.
- Banc : le sway headless n'a pas de clavier (seat sans capacité) — aucune fenêtre n'a le focus système ; les tests
  qui dépendent de focus/blur React simulent `focusin`/`focusout`.

## Découvrabilité et profils (retour de Chris, 08/10 matin)

- Page **Aide** (`ui/src/help`, bouton « ? » en bas de la barre, F1) : chaque fonction, où la trouver, bouton qui y
  mène (ouvre les panneaux de la barre via `OpenSidebarSheet`), raccourcis. Bandeau « À découvrir » une fois par machine.
- Le **cadenas** de l'adresse ouvre « Sécurité et réseau » (ex-panneau Réseau).
- **Profils** : liste réglable `profiles.list` (synchronisée ; migration des 5 d'origine et de `profiles.names`) —
  créer (« + » des pastilles, Réglages → Profils), renommer, teinte, réinitialiser, supprimer. Côté cœur
  `profiles::forget` : onglets fermés, cookies effacés tout de suite, dossiers effacés au lancement suivant
  (`profils-a-effacer.json`), extensions du profil retirées. Le profil principal (`graphite`) ne se supprime pas.
- Test : `tools/test-help-profiles.sh`.

## Synchro automatique (consigne de Chris, 08/10)

- Modes (`sync.mode`, par machine) : temps réel (lot 5 s, contrôle `/v1/etat` chaque minute), automatique (défaut :
  lot 60 s, contrôle 5 min), manuelle. Synchro au lancement (1,5 s). Onglets et visites : lots espacés (2 min / 10 min).
- Jamais de perte : une modification locale faite pendant l'aller-retour n'est pas écrasée (le type garde son ancienne
  base et repart à la passe suivante) ; conflit 409 → nouvelle passe 5 s après ; erreur → nouvel essai 30 s.
- Drapeau admin gardé dans `compte.json` : la section est là dès le lancement. Alerte en bas de la barre si pas à jour
  (erreur, synchro trop ancienne, ou modifications en attente en manuel). Test : `tools/test-account-auto.sh`.
- Budget gratuit Cloudflare (100 000 requêtes/jour, 100 000 lignes écrites/jour) : une seule écriture de compteur par
  requête, `/v1/etat` n'écrit rien. Estimation par machine et par jour : ~2 500 requêtes en temps réel (8 h actives),
  ~500 en automatique → 10 utilisateurs × 2 machines tiennent même tous en temps réel (~50 000 requêtes, ~60 000
  écritures). Au-delà de ~15 machines en temps réel permanent, surveiller le tableau de bord.

## Historique synchronisé, données du compte, réveil des extensions (08/10)

- Historique dans le coffre (type `historique`, réglage `sync.history` actif par défaut) : les 1 000 adresses les plus
  récentes + les effacements (table `history_forgotten`, `*` = tout effacé) ; fusion « la date la plus récente
  l'emporte », un effacement fait sur une machine s'applique partout. Adresses > 400 caractères non transmises.
- Réglages → Compte : interrupteur historique, « Données stockées » (le coffre lu sur le serveur, déchiffré sur la
  machine, type par type), « Supprimer le compte » (tout effacer côté serveur, confirmation en place).
- Extensions : celles qui écoutent les onglets sont réveillées quand les onglets de leur profil changent ; au réveil
  elles reçoivent l'état d'avant la veille puis l'actuel (onglet fermé pendant la veille : non annoncé).
- Tests : `tools/test-account-sync.sh` (historique, effacement, données montrées, suppression),
  `tools/test-extension-events.sh` (réveil après 45 s de veille).

## Extensions : onglets dans les service workers, pubs Twitch (08/10)
- Service workers d'extension (`extension_tabs/workers.rs` + `worker.js`) : connexion au protocole de débogage, mise en pause
  au démarrage (`setAutoAttach waitForDebuggerOnStart`, filtre service_worker), `chrome` n'existe qu'à l'arrêt
  `Debugger.setInstrumentationBreakpoint beforeScriptExecution` → `worker.js` + liste des onglets, puis reprise et
  détachement (un débogueur attaché empêche la veille). Changements d'onglets poussés aux service workers éveillés (attache,
  `__echoTabs`, détache) → `onCreated/onUpdated/onRemoved/onActivated` + `tabs.query`. Seulement les extensions qui voient
  déjà les onglets ; chacune ne voit que son profil (contexte appris par URL puis gardé ; le pont filtre par `tabs.get`).
  Pièges : se détacher juste après un ordre le perd (worker figé avant son script) → détacher à la réponse ; un accesseur sur
  `self.chrome` ne voit rien (défini par DefineOwnProperty) ; service workers lancés avant la connexion → `ServiceWorker.stopAllWorkers`
  une fois par contexte. Limite : un service worker endormi ne reçoit pas les événements (au réveil il reçoit l'état complet).
  Vérifié : TTV LOL PRO pose/retire son proxy à l'ouverture/fermeture de Twitch. Tests : `test-extension-events.sh`.
- Pubs Twitch : sur profil neuf, pub d'arrivée ~16 s, bouclier actif ou non ; aucun marqueur « stitched » observé. vaft
  (`injection/twitch/`, MIT, archivé 03/2026) intégré au bouclier pour twitch.tv : 0 pub, lecture continue (bandeau « Blocking
  ads »), mesures `tools/bench-twitch-ads.py`. Si Twitch change, re-mesurer ; vaft ne se met plus à jour seul.

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

## Ce qui n'existe pas encore
- Pagination de l'historique : la recherche ne rend que les 200 premières entrées
- Gestionnaire de mots de passe (choix : Proton Pass en extension), traduction de page
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

Historique plus ancien : `STATE-ARCHIVE.md`.
