# TODO — echo-browser

## Fait
- [x] Moteur de blocage natif (Rust) sur les listes uBlock Origin — 17/17, 0 faux positif, 6,5 µs/requête
- [x] Scriptlets : 151 extraits d'uBO, injectés là où les listes les ciblent
- [x] Ressources de remplacement (`$redirect=`) — 47
- [x] Rafraîchissement automatique des listes (24 h) + cache binaire du moteur
- [x] Contrat d'échange cœur ↔ interface (miroirs Rust et TypeScript)

## En cours
- [ ] Coque CEF : fenêtre, disposition interface/contenu, boucle de messages
- [ ] Interface TypeScript (barre d'onglets, barre d'adresse, panneau bouclier) — agent dédié
- [ ] Onglets : cycle de vie, navigation, titres et favicons

## Critères de recette — à vérifier dans le navigateur, pas au niveau du moteur
- [ ] **Aucune publicité sur YouTube**, pré-roll compris, sur une vraie vidéo
- [ ] Les scriptlets s'exécutent AVANT le premier script de la page (sinon YouTube gagne)
- [ ] Aucun site courant cassé par un faux positif (Le Monde, Leboncoin, GitHub, Gmail, banque)
- [ ] Connexion Google / Gmail fonctionnelle
- [ ] Vidéo plein écran, lecture fluide

## Ensuite
- [ ] Extensions : téléchargement depuis le catalogue Chrome, dépaquetage, activation
      (rappel : le chargement se fait au démarrage, ajouter une extension impose un redémarrage)
- [ ] Bibliothèque : favoris, historique, téléchargements (SQLite)
- [ ] Réglages persistés
- [ ] Empreinte de version sur les fichiers statiques servis à l'interface
- [ ] Filtres procéduraux : le rapport en compte 0 partout, à confirmer ou corriger

## Reporté
- [ ] Mode lecture
- [ ] Réorganisation des onglets par glisser-déposer
- [ ] Gestion des permissions (caméra, micro, notifications)
