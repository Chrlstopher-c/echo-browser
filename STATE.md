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
