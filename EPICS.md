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
- S6 Onglets des autres machines : chaque machine publie ses onglets ouverts (chiffrés, 100 au plus) ; Bibliothèque →
  « Sur vos autres machines » (nom de la machine, onglets cliquables). Critère : deux instances, les onglets de A
  apparaissent sur B, A ne se voit pas elle-même.

## E9 — Tableau de bord des créateurs + signaux partagés (demande de Chris, 08/10)
Règle : on voit ce qui se passe (comptes, usage, santé du service) sans jamais pouvoir lire les données de quelqu'un.
Le coffre est chiffré sur la machine : le serveur ne peut pas le lire, le tableau de bord non plus.
- S1 Tableau de bord, DANS Echo (page « Administration », demande de Chris) pour les comptes portant le drapeau admin
  (vérifié par le serveur à chaque appel), plus une page web de secours (`/admin`, clé d'administration en secret Wrangler, comparée en temps
  constant ; Cloudflare Access peut s'ajouter devant sans code). Chiffres : comptes (total, inscriptions par jour sur
  30 j, actifs 1/7/30 j), sessions ouvertes, volume du coffre par type, requêtes et erreurs par jour, versions d'Echo
  en usage. Comptes listés avec leur e-mail complet (choix de Chris), recherche par e-mail.
  Actions : déconnecter un compte partout, supprimer un compte. Neumorphisme clair/sombre comme Echo.
  Critère : service local, deux comptes créés par le test → le tableau de bord montre 2 comptes, leurs tailles, la
  version d'Echo ; la suppression d'un compte depuis le tableau de bord l'empêche de se reconnecter ; sans la clé : 401.
- S1b Administration détaillée (demande de Chris, 08/10) : usage global (utilisateurs actifs par jour, synchros par jour,
  écritures par type), usage PAR utilisateur (requêtes par jour sur 30 j, par type d'action, machines connectées avec
  date de connexion, dernière activité et version, coffre par type avec version et date), les plus actifs sur 7 j.
  Métadonnées seulement (comptages, dates, tailles), jamais de contenu ni de nom de machine ; usage gardé 90 jours.
  Critère : service local, un compte qui synchronise → sa fiche montre ses requêtes du jour, sa machine et ses types.
- S2 Signaux anonymes (socle des idées collectives : sites malveillants, habitudes, etc.). Réglage « Partager des signaux
  anonymes », coupé par défaut (consentement). Envoyés sans compte ni identifiant stable, par lots, domaine seulement
  (jamais l'adresse complète), agrégés par jour ; un chiffre n'est montré qu'au-delà de k installations distinctes ;
  conservation limitée (90 j). Critère : sans opt-in, rien ne part (vérifié sur le réseau) ; avec, le tableau de bord
  montre les agrégats et rien d'individuel.
- S3+ Fonctions collectives bâties sur S2 (une story chacune, à cadrer avec IDEES.md).

## E10 — Idées de Chris, première vague (validée 08/10 : 10+14, 6, puis 1+5)
- S1 Capture réseau par onglet : chaque requête (domaine, type, tiers ou non, méthode, statut, octets, durée, verdict du
  bouclier) gardée en mémoire par onglet (500 dernières), résumée par domaine ; diffusée seulement quand le panneau est
  ouvert (rien ne coûte quand il est fermé). Critère : page de test → le résumé montre ses domaines, octets et blocages.
- S2 Panneau « Réseau » (barre latérale) : domaines de l'onglet actif en direct (requêtes, octets, bloquées, tiers),
  détail des requêtes d'un domaine ; neumorphisme clair/sombre. Critère : capture claire et sombre lisible, mise à jour live.
- S3 Actions à la volée : bloquer un domaine sur ce site, « isolement strict » d'un site (aucun tiers) ; gardé entre
  deux lancements. Critère : domaine bloqué → ses requêtes suivantes annulées ; isolement → zéro requête tierce.
- S4 Journal d'accès par site : premières connexions à un tiers, permissions demandées et décidées, téléchargements ;
  gardé (200 derniers par site). Critère : une page qui demande la position → entrée au journal avec la décision.
- S5 Reprise de session exacte : champs de saisie (jamais les mots de passe) et position des vidéos restaurés après
  relance et réveil d'onglet. Critère : texte tapé + vidéo à 30 s → relance → texte et position revenus.
- S6 Empreinte de comportements : séquences de navigation qui reviennent (empreinte des suites de domaines) → proposition
  de routine (ouvrir la suite d'un geste). Critère : même suite 3 fois → proposition ; acceptée → la routine l'ouvre.
- S8 Rapport de performance (idée 12) : dans le panneau Réseau, vue « Poids » — part des tiers, poids par type, les
  requêtes les plus lourdes et les plus lentes, alertes (image > 500 Ko, script tiers > 100 Ko). Critère : page de test
  avec une grosse image et un script tiers lourd → les deux signalés.
- S9 Surveiller une page (idée 15) : clic droit « Surveiller cette page » ; à chaque visite, empreinte du texte comparée à
  la précédente ; si elle a changé, bandeau « Cette page a changé » avec les lignes ajoutées et retirées. Critère : page
  surveillée, contenu modifié côté serveur, nouvelle visite → bandeau avec la ligne ajoutée.
- S7 Mémoire de structure : « Masquer cet élément » (clic droit) retenu par empreinte de structure de page, appliqué aux
  pages de même gabarit. Critère : élément masqué sur un article → masqué sur un autre article du même site.

## E11 — Découvrabilité et profils (retour de Chris, 08/10 matin)
- S1 Page Aide (F1, bouton « ? ») : fonctions, où les trouver, bouton qui y mène, raccourcis ; bandeau « À découvrir ».
- S2 Cadenas de l'adresse → Sécurité et réseau (domaines, poids, journal, blocages).
- S3 Profils : créer, renommer, teinte, réinitialiser, supprimer (sessions effacées tout de suite, données au
  lancement suivant). Critère : `tools/test-help-profiles.sh` vert.

## E12 — Quotidien et lecture (autonomie validée par Chris, 08/10 après-midi)
Mots de passe écartés : Chris utilise Proton Pass dans Echo ; un gestionnaire maison ferait doublon et ajouterait du
risque dans un projet open source.
- S1 Palette d'adresse : en tapant, onglets ouverts, favoris et historique sous la barre ; ↑/↓, Entrée, Échap ; un
  onglet ouvert s'active au lieu d'être rouvert ; Ctrl+K comme Ctrl+L. Critère : test qui tape, voit les trois
  sections, choisit au clavier.
- S2 Formulaires réutilisables (idée 3) : fiches (identité, adresse) dans Réglages → Formulaires, clic droit
  « Remplir le formulaire » dans un champ ; champs reconnus par leurs attributs `autocomplete`/nom/libellé ; jamais un
  mot de passe ni une carte. Synchronisées chiffrées avec le compte. Critère : formulaire de test rempli.
- S3 Mode lecture (idée 20) : clic droit / Ctrl+Alt+R « Lire en mode lecture » : l'article seul, typographie d'Echo,
  clair/sombre, sans publicité ni menus ; « Quitter la lecture ». Critère : page de test avec pub et menu → seuls
  titre et paragraphes restent.
- Chaque story : point d'entrée visible + ligne dans l'Aide.
- Fait le 08/10 : S1 (`test-palette.sh`), S2 (`test-forms.sh`), S3 (`test-reader.sh`), captures clair/sombre.

## E13→E17 — Viser 10/10 sur les cinq axes de l'audit UX du 08/10 (demande de Chris, 09/10, autonomie)
Source : audit Opus (scratchpad/audit/RAPPORT.md, résumé dans STATE.md). Notes de départ : navigation 5, esthétique 7,5,
grand public 4, finition 4,5, fonctions face à Chrome/Firefox/Zen/Arc 6,5. Écartés volontairement : gestionnaire de
mots de passe (Proton Pass), traduction de page (service tiers). Chaque story : point d'entrée visible + ligne d'Aide,
clair ET sombre, test `tools/test-*.sh` isolé. Fin : régression complète, contre-audit, release 0.11.0.

### E13 — Navigation
- [x] S1 Recherche dans la page : Ctrl+F (et clic droit « Rechercher dans la page ») ouvre une barre flottante, compteur
  « 3/12 », Entrée / Maj+Entrée, Échap ferme et efface. VERIFY: `test-find.sh` (compte, suivant, fermeture).
- [x] S2 Palette : 1re ligne « Rechercher « … » sur <moteur> », suggestions du moteur (réglage, coupable), police du
  texte (pas monospace), pas de cadenas pendant la frappe, liste large qui déborde de la barre. VERIFY: `test-palette.sh`.
- [x] S3 Moteur au choix dans une liste (Google, DuckDuckGo, Qwant, Ecosia, Bing, Startpage, Brave) au lieu d'un champ
  « %s ». VERIFY: test qui change le moteur → URL de recherche suivie.
- [x] S4 Positions : Ctrl+Maj+T rouvre à sa place ; popup et lien « nouvel onglet » juste sous le parent ; passage en
  conteneur garde la place. VERIFY: test d'index.
- [x] S5 Plein écran : Échap en sort ; bord à bord (ni marge ni coins). VERIFY: test + capture.
- [x] S6 Zoom : paliers de Chrome (90/100/110/125/150…), pastille qui réserve sa place, Ctrl+0 / Ctrl+à. VERIFY: test.
- [x] S7 Fenêtre étroite : barre repliée d'office sous 900 px, poignée visible, raccourci Ctrl+Alt+S. VERIFY: test.
- [x] S8 Navigation demandée pendant le réveil d'un onglet endormi : jamais perdue. VERIFY: test.

### E14 — Finition
- [x] S1 Dates de l'historique et des fichiers (secondes ≠ millisecondes). VERIFY: test vitest + capture.
- [x] S2 Réglages : section « Autres » supprimée ; thème Clair / Sombre / Système ; sommaire. VERIFY: test DOM.
- [x] S3 Bouclier : compteur de l'onglet actif ; vrais nombres de règles dès le 1er lancement ; accents ; listes sous
  « Avancé » ; bandeaux de cookies coupés par défaut. VERIFY: test deux onglets → deux chiffres.
- [x] S4 Favoris : Ctrl+D confirme (bulle « Ajouté aux favoris ») + étoile dans l'adresse ; pages internes et pages
  d'erreur ni en favori ni dans l'historique. VERIFY: test.
- [x] S5 Téléchargements : progression visible, bulle cliquable (Ouvrir / Afficher le dossier). VERIFY: test.
- [x] S6 Chromium en français (pages d'erreur, DevTools). VERIFY: page d'erreur lue en français.
- [x] S7 Mode lecture : titre unique, une seule famille de police, taille et largeur réglables. VERIFY: test-reader.
- [x] S8 Icônes de repli : initiale teintée pour un épinglé ou un onglet sans favicon, logo Echo pour les pages
  internes. VERIFY: test DOM + capture.

### E15 — Grand public et accessibilité
- [x] S1 Contraste ≥ 4,5:1 pour tout texte (clair et sombre). VERIFY: test qui calcule les contrastes des jetons.
- [x] S2 Onglets au clavier : `tablist`/`tab`, `aria-selected`, flèches, Entrée, Suppr. VERIFY: test DOM.
- [x] S3 Profils : cible ≥ 24 px, nom du profil actif visible, libellés aria justes. VERIFY: test DOM.
- [x] S4 Cadenas honnête : certificat invalide = « Non sécurisé » rouge ; http = « Non sécurisé » en toutes lettres ;
  résumé en langage courant en tête du panneau. VERIFY: test (page http + état d'erreur de certificat).
- [x] S5 Veille selon la mémoire réellement libre, plus un nombre d'onglets ; réglage qui dit vrai ; icône « garder
  éveillé » distincte du thème. VERIFY: test 6 onglets, rien ne dort avant le délai réglé.
- [x] S6 Bouton Claude Code montré seulement si `claude` est installé ; extensions : « Catalogue » en bouton principal,
  phrase juste ; interrupteurs étiquetés selon leur état. VERIFY: test DOM.
- [x] S7 Accueil qui configure : thème, moteur, import, puis compte ; pas de bulle d'astuce pendant l'accueil ; l'accueil
  ne reste pas dans l'historique ; afficher le mot de passe. VERIFY: test du parcours.
- [x] S8 Recherche dans l'Aide. VERIFY: test DOM.

### E16 — Profils = identités
- [x] S1 Un seul profil au premier lancement (les profils existants ne bougent pas). VERIFY: test premier lancement.
- [x] S2 Historique, favoris, dossiers, suggestions et nouvel onglet propres à chaque profil. VERIFY: test
  Personnel/Travail sans fuite.
- [x] S3 Conteneur visible (couleur + nom sur la ligne d'onglet) ; clic droit de lien « Ouvrir dans un conteneur ».
- [x] S4 Navigation privée : Ctrl+Maj+N, onglet en mémoire seule (rien sur disque, pas d'historique), aspect distinct.
  VERIFY: test (aucun cookie ni historique après fermeture).

### E17 — Face aux concurrents
- [x] S1 Import Chrome / Firefox / Chromium (favoris + historique) depuis l'accueil et les Réglages. VERIFY: test avec
  profils factices.
- [x] S2 Effacer les données de navigation (Ctrl+Maj+Suppr) : période, historique, cookies, cache. VERIFY: test.
- [x] S3 Nouvel onglet : tuiles avec icônes, plus de champ en double. VERIFY: capture.
- [x] S4 Clic droit : « Rechercher « sélection » », bascules cochées, lecture seulement si article ; Ctrl+Maj+C copie
  l'adresse. VERIFY: test-contextmenu.
- [x] S5 Couper le son d'un onglet depuis son indicateur audio. VERIFY: test.
- [x] S6 Fiches de formulaire proposées au focus d'un champ reconnu. VERIFY: test-forms.

- Fait le 09/10 (autonomie) : toutes les stories, chacune avec son test isole (`tools/test-*.sh` : find, settings,
  palette, positions, fullscreen, zoom, narrow, wake-navigate, shield-count, bookmarks, downloads, french, reader,
  marks, contrast en CI, tabs-keyboard, profile-strip, security, sleep-pressure, terminal-button, welcome,
  help-search, profile-isolation, container-visible, private, clear-data, menu-extras, mute, forms-offer).
- Ecarts assumes : la liste de la palette reste dans la barre (la vue de la barre ne peut pas deborder sur la page) ;
  « lecture seulement si article » non fait (l'entree reste sur toute page web, la vue dit « pas d'article ») ; le
  selecteur « Demander où enregistrer » (portail) n'est pas rejouable sur le banc ; Ctrl+0 sur un vrai clavier AZERTY
  couvert par test unitaire seulement.

### E18 — Contre-audit du 09/10 (notes : navigation 7,5, esthétique 8, grand public 6,5, finition 6,5, concurrents 7,5)
Rapport : scratchpad `contre-audit/RAPPORT.md`. Bugs N1→N10 et manques classés par impact.
- [x] S1 N1 : Précédent ne rouvre plus l'accueil (l'accueil n'entre pas dans le fil de l'onglet) ; `test-welcome`
  vérifie vraiment `canGoBack`.
- [x] S2 N2 : changer d'onglet efface les surlignages de Ctrl+F.
- [x] S3 N3/N4 : page en échec ni dans l'historique ni en tuile ; cadenas « Page non chargée », jamais « chiffrée ».
- [x] S4 N5 : un site http seul reste joignable (mise à niveau https sans repli coupée).
- [x] S5 N6/N7 : pages internes au bon thème dans tous les profils et en privé ; sélecteur de thème synchronisé.
- [x] S6 N8 : onglet privé titré « Nouvel onglet » ; son nouvel onglet neutre (ni historique, ni suggestions du moteur).
- [x] S7 N9/N10 et textes : version 0.11.0, « Enregistrer cette page » expliqué, textes périmés, Aide (thème,
  téléchargements), bulle d'astuce seulement après une première vraie page.
- [x] S8 Proximité et clarté : fiche proposée sous l'adresse (en haut), marque d'Echo qui ne ressemble pas au son,
  « Rechercher « … » sur <moteur> », mode lecture absent du menu d'un champ, icônes distinctes, palette : favoris et
  historique avant les suggestions du moteur.
- [x] S9 Fenêtre étroite : Ctrl+L / Ctrl+F montrent la barre par-dessus la page sans la pousser, elle repart après.
- [x] S10 Concurrents : « Traduire la page » au clic droit ; Réglages → Mots de passe (Proton Pass, Bitwarden en un clic).
- Fait le 09/10 au matin : S1→S10 (tests : welcome, security, theme-everywhere, private, menu-extras, narrow,
  forms-offer, marks, palette). Écarts restants : outils de développement de CEF en anglais (leurs traductions ne sont
  pas embarquées : réglage « browserLanguage » posé sans effet, essai retiré) ; pages d'erreur de Chromium qui citent
  « Chrome » ; menus contextuels carrés (une surimpression ne peut pas être transparente) ; palette dans la largeur de
  la barre ; extensions actives au redémarrage ; historique en initiales (pas de favicon par visite).
