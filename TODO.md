# TODO — echo-browser

## RETOURS DE CHRIS DU 06/10 SOIR — liste de reprise (ne rien perdre ; détail et critères dans EPICS.md E6)

Ordre voulu par Chris : angles → favicons → mémoire → décodeur vidéo. Puis clic droit, fermeture, dossiers, comptes.

1. **Faits** : favicons · fermeture Super+Q instantanée · texte centré / séparation adresse-onglets · session conservée entre deux lancements · tests isolés (plus jamais
   l'instance de Chris tuée) · angles de page (masques) · identité Chrome 154 cohérente (captcha) · neumorphisme clair/sombre.
2. **Faits cette nuit (tests dans `tools/test-*.sh`)** : animation barre repliée (page repoussée, 14 étapes) · clic droit réel dans les pages ·
   F12 / Ctrl+Maj+I/J (DevTools complets) · clic droit barre (onglet, dossier, fond de liste) · dossiers d'onglets persistés · conteneurs
   (cookies isolés, persistés) · défilement restitué au réveil · purge V8 des onglets d'arrière-plan (-9 %) · démarrage 0,3 s.
3. **Mémoire** : drapeaux Chromium testés, aucun > 4 % (voir STATE.md) ; le levier est la veille. Piste restante : état des onglets endormis
   sur disque (déjà : fiche + défilement), délai de veille plus court, extensions par conteneur.
3b. **RETOURS DE CHRIS 07/10 après-midi (ordre de traitement)** :
   1. Menu clic droit des pages : un clic gauche ailleurs le ferme ; un clic droit ailleurs ferme l'ancien et rouvre au nouvel endroit.
   2. Téléchargements : ne fonctionnent pas (aucun système visible) — à réparer et vérifier de bout en bout.
   3. Copier une image (et l'adresse d'une vidéo) au clic droit, pour coller directement ailleurs (Ctrl+V) sans télécharger.
   4. **Profils façon Arc** : les points en bas de la barre (graphite, sable, rose…) ne changent que la couleur ; ils doivent être des
      profils/espaces : chacun ses onglets (et ses comptes, via un conteneur), nom modifiable, couleur propre.
   5. Pubs Twitch : essayer d'intercepter le flux (pas prioritaire).
   6. Release publique : quand prêt (Chris la délègue).
   7. Veille pendant une lecture : un onglet qui lit une vidéo/son (même muet) ne dort jamais.
   8. Bouclier : les interrupteurs n'affichent pas l'état réel après bascule (pub YouTube alors que « actif ») — vérifier tous les réglages.
   9. Réglages, téléchargements, bibliothèque, thèmes : en PAGE pleine (onglet spécial comme « Nouvel onglet ») au lieu de la barre ;
      le bouclier et les extensions verticales peuvent rester dans la barre.
   10. Extensions : page dédiée ; installer depuis le Chrome Web Store dans un onglet (bouton « Ajouter ») et/ou catalogue intégré
       (API gratuite ?) ; à défaut, intégration native propre du Web Store.
4. **Reste (petit)** : extensions dans les conteneurs (Chromium les rattache au profil) ; menu contextuel des sites aux couleurs
   neumorphiques ; « Examiner l'élément » qui sélectionne l'élément cliqué (aujourd'hui : ouvre seulement les DevTools ancrés) ;
   largeur des DevTools mémorisée ; pubs Twitch (insérées côté serveur dans le flux, le bouclier ne les voit pas).
5. **Fait 07/10** : moteur CEF compilé avec codecs installé (Twitch/H.264/AAC/HEVC OK, testé par Chris en 1080p). Aucun build lourd
   prévu. À refaire seulement à une montée de version de CEF (pièges et commandes : STATE.md).
6. **Release publique (quand Chris le décide)** : moteur sans codecs brevetés + `libffmpeg.so` H.264 téléchargé à la demande depuis un
   tiers (emplacement séparé déjà prévu, `is_component_ffmpeg`) ; paquet (binaire + CEF + UI), CI verte, dépôt public à relire
   (aucun identifiant réel). Rien n'est encore poussé sur GitHub.
7. **Système (non persistant)** : zram1 10 Go ajouté à chaud ; la config zram (12 Go) prendra effet au prochain redémarrage.
8. **Règles retenues** : ne jamais arrêter l'instance de Chris (tests isolés) · ne jamais perdre ses onglets · NE PAS toucher à sa souris
   (captures par `grim` sur fenêtre flottante de test, demandes jouées par `ECHO_BENCH_UI` / `ECHO_BENCH_JS`).

---

Relevé du 10/09/2026, sur retour de Chris capture à l'appui. L'ordre est celui qu'il a donné.

---

## 1. L'écart entre la barre latérale et la page — **prioritaire**

Une gouttière sombre sépare la colonne de gauche du cadre de la page : elle ne porte ni la
teinte de la barre ni celle de la page, elle donne deux blocs posés côte à côte au lieu d'une
fenêtre. Chez Zen, la marge existe aussi mais elle est *la même matière* que la barre, et la
page y flotte dedans.

- [ ] Le fond de la fenêtre prend la teinte de la barre, pas le noir par défaut
- [ ] Marge égale des quatre côtés de la page, valeur unique et assumée
- [ ] La barre et la marge forment une seule surface continue

## 2. Le clic droit — fait le 10/09

Menu construit par le cœur selon la cible, dessiné par nous, en français.

- [x] Menu selon la cible : lien, image, sélection, champ de saisie, page nue
- [x] Lien : nouvel onglet, arrière-plan, copier l'adresse, enregistrer la cible
- [x] Image : ouvrir, copier l'adresse, enregistrer
- [x] Sélection : copier, rechercher, ouvrir si c'est une adresse
- [x] Champ : couper, copier, coller, coller sans mise en forme, tout sélectionner
- [x] Page : précédent, suivant, recharger, copier l'adresse, favori, enregistrer,
      imprimer, bouclier sur ce site, code source, examiner l'élément
- [x] Entrées grisées plutôt que masquées — une action qui disparaît déplace les autres
- [ ] Se ferme-t-il bien au clic dans la page ? À constater à l'usage
- [ ] Sous-menus (langue, encodage) et raccourcis affichés à droite des libellés

## 3. Le nouvel onglet — page et suggestions

Le clic sur « Nouvel onglet » ouvre une page nue. Il doit ouvrir une palette centrée : champ
de recherche, et en dessous les dernières adresses consultées, les onglets déjà ouverts
(« aller à cet onglet »), et l'adresse présente dans le presse-papiers.

- [x] Page locale `ui/public/nouvel-onglet.*` (06/10) : champ centré, actif au clavier, sans framework ;
      c'est la page d'accueil (`search::HOME`) — gain mesuré : repos 536 → 460 Mo, et chaque nouvel onglet ne
      coûte plus un rendu Google (~100 Mo). Moteur de recherche dupliqué dans le JS : à rapprocher de `search.rs`
- [ ] Palette : suggestions ci-dessous (nécessite un pont page → cœur pour lire historique/onglets)
- [ ] Suggestions : historique récent, onglets ouverts, presse-papiers, favoris
- [ ] Chaque ligne porte son favicon et son action à droite
- [ ] Navigation entière au clavier, entrée pour valider, échap pour fermer
- [ ] Même palette sur Ctrl+L et Ctrl+K

## 4. Les bulles d'aide

L'infobulle est celle du système : rectangle gris, police par défaut, apparition sèche.

- [x] Bulles maison sur tous les boutons (06/10, `ui/src/shared/design/tooltip-layer.tsx`) : matière, délai, apparition animée
- [x] Le raccourci clavier affiché à côté du libellé
- [x] Barre repliée : plus de rail de 56 px, les bulles maison fonctionnent partout
- [ ] Positionnement qui évite les bords

## 5. La barre repliée

Repliée, la barre laisse un rail de 56 px qui pousse toujours la page. Attendu : la page prend
toute la fenêtre, et le rail revient en surimpression quand la souris longe le bord.

- [x] Largeur réelle quasi nulle une fois repliée (1 px d'espaceur)
- [x] Barre en surimpression au-dessus de la page, révélée au bord gauche (liseré de 6 px `bord.html`)
- [ ] Retour animé de la barre (aujourd'hui elle apparaît d'un coup)

## 6. Les bords de la fenêtre

Les quatre angles sont mal rendus : la page reste un rectangle net dans un cadre arrondi, le
liseré d'accent coupe les coins, et le haut de la page passe sous la bordure. Zen a des angles
pleins et une page qui épouse exactement son cadre.

- [x] Angles de la page réellement arrondis (06/10, rayon 12 px)
- [ ] Liseré continu, de la même épaisseur sur les quatre côtés (à constater)
- [x] Aucun débordement de la page sous la bordure

## 6 bis. Les fenêtres d'extension — **ultra important** (retour du 10/09)

Proton Pass est installé et actif, mais rien ne permet de l'ouvrir : le panneau des extensions
ne fait que lister et activer. Une extension moderne vit dans sa fenêtre — celle de Proton Pass
est le gestionnaire de mots de passe lui-même. Sans elle, l'extension est décorative.

- [x] Barre d'icônes d'extensions sous l'adresse, une par extension qui en déclare une
- [x] Icône réelle de l'extension, servie par le cœur depuis le paquet
- [x] Clic : la fenêtre de l'extension s'ouvre ancrée sous son icône
- [x] La fenêtre se ferme au second clic, à Échap, et au changement d'onglet
- [ ] Elle ne se ferme pas encore au clic dans la page : la vue web ne nous prévient pas
- [x] Le menu de l'icône : réglages de l'extension, activer/désactiver, retirer
- [x] Activation et retrait effectifs pour **toutes** les extensions, catalogue compris
- [ ] Épinglage : choisir les icônes visibles, les autres dans un dépassement
- [ ] Les extensions qui n'ont pas de fenêtre déclenchent leur action au clic
- [ ] La fenêtre se dimensionne à sa page (taille fixe 380 × 600 aujourd'hui)
- [ ] Pastille de compteur sur l'icône, quand l'extension en pose une

Technique : la fenêtre d'une extension est une page `chrome-extension://<id>/…`. Elle s'affiche
dans une vue superposée (chantier 7), ancrée sous l'icône.

## 7. Vues superposées — socle en place

- [x] Vue posée au-dessus de la page, position et taille au pixel
- [x] Sert déjà : menu contextuel, fenêtres d'extension
- [ ] **Elle ne sait pas être transparente** : Chromium peint un rectangle plein
      dessous, les angles arrondis se voient découpés (mesuré le 10/09, trois essais).
      Tout ce qui s'affiche au-dessus de la page est donc opaque et rectangulaire.
- [ ] Reste à en tirer : palette du nouvel onglet, rail flottant, masques d'angle

## 8. Ce qui doit être réellement conservé d'une session à l'autre

Constaté sur le profil : 52 cookies persistants, l'historique Chromium et la base de mots de
passe existent sur le disque. Ce qui n'est pas établi, c'est le chemin complet vu par Chris.

- [ ] **Connexion Google** — se connecter, fermer, rouvrir, constater qu'on est encore connecté
- [ ] **Mots de passe** — aucune proposition d'enregistrement n'apparaît aujourd'hui : le
      gestionnaire de Chromium n'est pas branché en mode Alloy. À brancher ou à remplacer.
- [ ] Remplissage automatique des formulaires
- [ ] Cookies : durée de vie respectée, tiers, effacement par site depuis les réglages
- [ ] Historique : celui de Chromium et le nôtre disent la même chose
- [ ] Un écran de confidentialité : ce qui est gardé, et de quoi se débarrasser

---

## À élucider — la première page échoue à chaque lancement

`ERR_PROXY_CONNECTION_FAILED` sur la page d'accueil, une fois par démarrage, puis la page
se recharge et s'affiche. Aucun proxy n'est configuré sur la machine (`gsettings` dit
« none », aucune variable d'environnement). Deux pistes écartées par la mesure : le
drapeau `--no-proxy-server` ne change rien, retarder la navigation après l'ouverture de
la fenêtre non plus. Reste le service réseau de Chromium, qui n'est peut-être pas encore
debout à la première requête.

- [ ] Mesurer le temps réellement perdu entre l'échec et l'affichage
- [ ] Si c'est le service réseau : attendre son signal plutôt que de naviguer à l'aveugle

## Reste du chantier, hors retour du jour

- [ ] Pagination de l'historique (`searchHistory` sans décalage : soixante entrées au plus)
- [x] Veille automatique des onglets inactifs (06/10) — 1 145 → 408 Mo sur 10 onglets
- [ ] Veille : réglage du délai (réglages), mise en veille manuelle (clic droit sur l'onglet), exclure les onglets
      (fait le 06/10 : saisie utilisateur détectée par la page → onglet jamais endormi)
- [ ] **Mémoire — plancher fixe** : interface React ~80–100 Mo, accueil ~100 Mo, principal ~150–190 Mo ;
      viser un plancher sous Chrome. Pistes : accueil sans renderer dédié, limite de processus de rendu, UI allégée
- [ ] Codecs H.264/AAC : construire/obtenir un CEF avec codecs propriétaires (Twitch, Netflix, Spotify)
- [ ] Mise en sourdine d'un onglet
- [ ] Les extensions déclarées en ligne de commande demandent encore une relance
- [ ] Mode lecture
- [x] Permissions : caméra, micro, notifications, position, presse-papiers (06/10)
- [ ] Écran de gestion des permissions retenues par site (aujourd'hui retenues sans pouvoir les revoir ; `permissions::forget_site` existe)
- [ ] Empreinte de version sur les fichiers servis à l'interface
- [ ] Filtres procéduraux : le rapport en compte zéro partout, à confirmer ou corriger

## Fait
- [x] Blocage natif sur les listes uBlock Origin — 17/17, 6,5 µs par requête, aucune publicité
      sur YouTube, pré-roll compris
- [x] Scriptlets exécutés avant le premier script de la page, ressources de remplacement
- [x] Onglets : cycle de vie, épinglage, déplacement, zoom, trace de navigation
- [x] Bibliothèque SQLite : favoris, historique, téléchargements réels, dix-neuf réglages
- [x] Extensions : inventaire lu dans le profil Chromium, installation depuis la boutique
- [x] Empreinte Chrome de bureau français, vérifiée côté serveur
- [x] Relance du processus avec restitution des onglets
- [x] Fiche `.echoforge.yml` et sonde de présence pour le centre de contrôle

## Notes du 06/10 (terminal et pages internes)
- [x] Le champ d'adresse n'affiche plus « ui » pour les pages `echo://` : il reste une invite
- [ ] Raccourci clavier pour le terminal (ex. Ctrl+Maj+K) et mention dans la bulle
- [ ] Terminal : sélection/copie (Ctrl+Maj+C/V), liens cliquables, redimensionnement vérifié sous tmux
- [ ] `ECHO_TERM_CMD` ne gère pas les guillemets (découpe sur les espaces)

- [ ] Angles arrondis : pages en `color-scheme: dark` sans fond déclaré s'affichent en blanc (le fond par défaut est forcé) ; à traiter si ça se voit
