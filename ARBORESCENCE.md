# Arborescence

Un fichier par ligne, avec son rôle. Le découpage en domaines et les frontières entre eux
sont dans `ARCHITECTURE.md` ; ici, c'est la carte plate.

```
echo-browser/
├── .echoforge.yml                      — fiche lue par le centre de contrôle (type, port de sonde, scripts)
├── Cargo.toml                          — espace de travail Rust : les cinq caisses ci-dessous
├── Cargo.lock                          — versions figées
├── rust-toolchain.toml                 — version du compilateur imposée au dépôt
├── start.sh / stop.sh / restart.sh     — lancement, arrêt par identifiant enregistré, cycle complet
├── STATE.md · TODO.md · ARCHITECTURE.md · README.md · ARBORESCENCE.md
│
├── core/contract/
│   ├── Cargo.toml
│   └── src/lib.rs                      — contrat figé cœur ↔ interface, miroir de ui/src/shared/contract.ts
│
├── core/shield/                        — blocage publicitaire, moteur adblock-rust
│   ├── src/lib.rs                      — façade : charger, décider, traiter une page, basculer un site
│   ├── src/engine.rs                   — moteur de règles et son cache binaire
│   ├── src/catalog.rs                  — listes de filtres, abonnements, rafraîchissement
│   ├── src/resources.rs                — scriptlets et redirections extraits d'uBlock Origin
│   ├── src/exceptions.rs               — sites où le bouclier est levé
│   ├── src/verdict.rs                  — verdict rendu par requête
│   ├── src/tally.rs                    — comptage par onglet et par catégorie
│   └── examples/rapport.rs             — mesure de parité hors interface
│
├── core/extensions/                    — extensions Chromium
│   ├── src/lib.rs                      — façade : inventaire, installation, activation
│   ├── src/profile.rs                  — lecture du profil Chromium (source de vérité)
│   ├── src/catalog.rs                  — extensions connues, métadonnées
│   ├── src/crx.rs                      — dépaquetage d'une archive CRX
│   ├── src/store.rs                    — récupération depuis la boutique
│   └── examples/installer.rs           — installation en ligne de commande
│
├── core/library/                       — favoris, historique, téléchargements, réglages (SQLite WAL)
│   ├── src/lib.rs                      — façade et ouverture de la base
│   ├── src/schema.rs                   — quatre tables et leurs migrations
│   ├── src/bookmarks.rs · history.rs · downloads.rs · settings.rs
│   └── tests/library.rs                — dix cas sur une base réelle
│
├── core/shell/                         — la coque : Chromium, fenêtre, onglets, pont
│   ├── src/main.rs                     — point d'entrée, processus principal et processus enfants
│   ├── src/app.rs                      — application CEF, schéma echo://
│   ├── src/flags.rs                    — commutateurs Chromium (dont l'exclusion de Vulkan)
│   ├── src/window.rs                   — fenêtre, disposition, largeur du bandeau, teinte
│   ├── src/session.rs                  — état vivant, cantonné au fil de l'interface
│   ├── src/tabs.rs                     — onglets, trace de navigation, adoption et détachement des vues
│   ├── src/client.rs                   — rappels Chromium par vue
│   ├── src/filtering.rs                — branchement du bouclier sur les requêtes
│   ├── src/injection/mod.rs            — traitement cosmétique et scriptlets
│   ├── src/injection/filter.rs         — insertion dans le flux HTML, avant le premier script de la page
│   ├── src/bridge/mod.rs               — réception des demandes de l'interface, publication des états
│   ├── src/bridge/library.rs           — demandes touchant la bibliothèque
│   ├── src/bridge/script.rs            — envoi d'évènements vers l'interface
│   ├── src/transfers.rs                — téléchargements réels et dossier de destination
│   ├── src/identity.rs                 — empreinte Chrome (en-têtes, langues)
│   ├── src/search.rs                   — page d'accueil et moteur de recherche
│   ├── src/shortcuts.rs                — raccourcis clavier
│   ├── src/restart.rs                  — relance du processus avec restitution des onglets
│   ├── src/presence.rs                 — sonde de présence : le seul port ouvert, lu par le centre de contrôle
│   ├── src/assets.rs                   — service des fichiers de l'interface
│   └── src/selftest.rs                 — scénario rejoué sans interface (ECHO_SELFTEST=1)
│
├── ui/                                 — interface, Bun + React + TypeScript + Tailwind
│   ├── index.html · vite.config.ts · tsconfig.json · package.json · bun.lock
│   ├── dist/                           — sortie de construction, servie par echo://
│   └── src/
│       ├── main.tsx · app.tsx          — montage et assemblage général
│       ├── shared/contract.ts          — miroir TypeScript du contrat (ne jamais modifier seul)
│       ├── shared/core-bridge.ts       — envoi des demandes, réception des évènements
│       ├── shared/core-state.ts        — état reçu du cœur
│       ├── shared/use-core.ts          — accès au cœur depuis les composants
│       ├── shared/format.ts · url-shape.ts · local-store.ts
│       ├── shared/design/              — primitives visuelles (thème, boutons, listes, anneaux, bascules)
│       ├── shared/fake/                — cœur simulé pour le développement hors navigateur
│       ├── sidebar/                    — colonne latérale, rail replié, feuilles, raccourcis, géométrie
│       ├── tabs/                       — liste, grille des épinglés, ligne d'onglet, menu contextuel
│       ├── address/                    — champ d'adresse, cadenas, progression, rechargement
│       ├── stage/                      — cadre de la page et barre de développement
│       ├── shield/                     — bouclier : compteurs, listes de filtres, feuille
│       ├── library/                    — favoris, historique, téléchargements
│       ├── extensions/                 — inventaire, installation, activation
│       ├── settings/                   — catalogue des réglages et leur rendu
│       ├── spaces/                     — espaces colorés et leur sélecteur
│       └── restart/                    — écran et bandeau de relance
│
├── tools/
│   ├── fetch-cef.sh                    — récupération de la distribution CEF
│   └── build-resources.mjs             — extraction des scriptlets et redirections d'uBlock Origin
│
├── data/
│   ├── filter-lists/                   — listes de filtres téléchargées
│   ├── shield-resources.json           — scriptlets et redirections
│   ├── shield-engine.bin               — moteur sérialisé, relu au démarrage
│   └── extensions/                     — extensions dépaquetées, un dossier par identifiant
│
├── logs/                               — journal du navigateur, remis à zéro à chaque lancement
└── target/                             — sortie de compilation Rust
```
