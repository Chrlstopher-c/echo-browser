# TODO — echo-browser

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

## 2. Le clic droit — **inexistant**

Le menu affiché est celui de Chromium par défaut : quatre entrées, en anglais, sans rapport
avec ce qui a été cliqué. Un navigateur doit servir un menu complet et contextuel.

- [ ] Menu selon la cible : lien, image, vidéo, texte sélectionné, champ de saisie, page nue
- [ ] Lien : ouvrir dans un nouvel onglet, en arrière-plan, copier l'adresse
- [ ] Image : ouvrir, copier, copier l'adresse, enregistrer
- [ ] Sélection : copier, rechercher sur le web, ouvrir si c'est une adresse
- [ ] Champ : couper, copier, coller, coller sans mise en forme, tout sélectionner
- [ ] Page : précédent, suivant, recharger, enregistrer, imprimer, code source, inspecter
- [ ] Entrées à nous : bloquer cet élément, lever le bouclier sur ce site
- [ ] Tout en français
- [ ] **Dessiné par nous**, pas par Chromium — voir le chantier 7 (vues superposées)

## 3. Le nouvel onglet — page et suggestions

Le clic sur « Nouvel onglet » ouvre une page nue. Il doit ouvrir une palette centrée : champ
de recherche, et en dessous les dernières adresses consultées, les onglets déjà ouverts
(« aller à cet onglet »), et l'adresse présente dans le presse-papiers.

- [ ] Palette centrée sur la page, champ actif au clavier
- [ ] Suggestions : historique récent, onglets ouverts, presse-papiers, favoris
- [ ] Chaque ligne porte son favicon et son action à droite
- [ ] Navigation entière au clavier, entrée pour valider, échap pour fermer
- [ ] Même palette sur Ctrl+L et Ctrl+K

## 4. Les bulles d'aide

L'infobulle est celle du système : rectangle gris, police par défaut, apparition sèche.

- [ ] Bulles maison sur tous les boutons : matière, délai, apparition et disparition animées
- [ ] Le raccourci clavier affiché à côté du libellé
- [ ] Positionnement qui évite les bords

## 5. La barre repliée

Repliée, la barre laisse un rail de 56 px qui pousse toujours la page. Attendu : la page prend
toute la fenêtre, et le rail revient en surimpression quand la souris longe le bord.

- [ ] Largeur réelle zéro une fois repliée
- [ ] Rail en surimpression au-dessus de la page, révélé au bord gauche
- [ ] Retour animé, pas d'à-coup sur la page

## 6. Les bords de la fenêtre

Les quatre angles sont mal rendus : la page reste un rectangle net dans un cadre arrondi, le
liseré d'accent coupe les coins, et le haut de la page passe sous la bordure. Zen a des angles
pleins et une page qui épouse exactement son cadre.

- [ ] Angles de la page réellement arrondis
- [ ] Liseré continu, de la même épaisseur sur les quatre côtés
- [ ] Aucun débordement de la page sous la bordure

## 6 bis. Les fenêtres d'extension — **ultra important** (retour du 10/09)

Proton Pass est installé et actif, mais rien ne permet de l'ouvrir : le panneau des extensions
ne fait que lister et activer. Une extension moderne vit dans sa fenêtre — celle de Proton Pass
est le gestionnaire de mots de passe lui-même. Sans elle, l'extension est décorative.

- [ ] Barre d'icônes d'extensions près de l'adresse, une par extension qui en déclare une
- [ ] Icône réelle de l'extension, et sa pastille de compteur quand elle en pose une
- [ ] Clic : la fenêtre de l'extension s'ouvre ancrée sous son icône, à la bonne taille
- [ ] La fenêtre se ferme au clic ailleurs, à Échap, et se redimensionne à la demande de la page
- [ ] Épinglage : choisir les icônes visibles, les autres dans un dépassement
- [ ] Le menu de l'icône : options de l'extension, retirer, gérer
- [ ] Les extensions qui n'ont pas de fenêtre déclenchent leur action au clic

Technique : la fenêtre d'une extension est une page `chrome-extension://<id>/…`. Elle s'affiche
dans une vue superposée (chantier 7), ancrée sous l'icône.

## 7. Vues superposées — le socle technique des chantiers 2, 3, 5 et 6

Ce qui manquait jusqu'ici : rien ne pouvait s'afficher **au-dessus** de la page, la vue web
étant une surface native opaque. `Window::add_overlay_view` le permet — une vue ancrée
librement au-dessus de la page, dont on pilote position, taille et visibilité.

- [ ] Vue superposée transparente chargée en `echo://`, à établir d'abord sur un cas simple
- [ ] Un canal de rendu partagé avec l'interface principale (même contrat, même thème)
- [ ] S'en servir pour : menu contextuel, palette du nouvel onglet, rail flottant, masques
      d'angle, **fenêtres d'extension**

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
- [ ] Mise en sourdine et mise en veille d'un onglet — l'état est affiché, pas pilotable
- [ ] Les extensions déclarées en ligne de commande demandent encore une relance
- [ ] Mode lecture
- [ ] Permissions : caméra, micro, notifications, position
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
