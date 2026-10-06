# Architecture — echo-browser

## Deux moitiés, une frontière

| Dossier | Rôle | Langage |
|---|---|---|
| `core/` | Le navigateur : fenêtre, onglets, réseau, blocage | Rust |
| `ui/` | Ce que l'utilisateur voit et manipule | TypeScript / React |
| `tools/` | Scripts de préparation (binaires Chromium, paquet de scriptlets) | Bash / JS |

La frontière est un contrat explicite, écrit en double : `core/contract/src/lib.rs` et
`ui/src/shared/contract.ts`. **Les deux se modifient ensemble, jamais l'un sans l'autre.**

## Les domaines du cœur

Chaque dossier de `core/` correspond à une préoccupation du produit, pas à une couche technique.

| Domaine | Responsabilité | Ne fait pas |
|---|---|---|
| `shield/` | Décider ce qui est bloqué, masqué, remplacé ; tenir les listes à jour | Ne parle jamais à Chromium |
| `shell/` | Tout ce qui touche Chromium : fenêtre, vues, onglets, requêtes, pont | Ne décide pas des blocages |
| `contract/` | Le vocabulaire commun avec l'interface | Aucune logique |
| `library/` | Favoris, historique, téléchargements, réglages, permissions (SQLite) | Ne parle jamais à Chromium |
| `extensions/` | Inventaire et installation des extensions Chrome | Ne dessine rien |
| `terminal/` | Pseudo-terminal qui survit à la page (tampon borné, reprise par position) | Ne connaît ni Chromium ni l'interface |

`shield` est délibérément ignorant de CEF : il se teste seul
(`cargo run -p echo-shield --example rapport --release`) et pourrait servir ailleurs.

### À l'intérieur de `shell/`

| Fichier | Responsabilité |
|---|---|
| `main.rs` | Démarrage, configuration, boucle de messages |
| `app.rs` | Contact avec Chromium : drapeaux, création de la fenêtre |
| `window.rs` | La fenêtre et la place respective de l'interface et du contenu |
| `tabs.rs` | Les onglets et la scène qui les porte |
| `client.rs` | Les rappels de Chromium pendant la vie d'un navigateur |
| `filtering.rs` | Soumettre chaque requête au bouclier |
| `injection/` | Poser le traitement du bouclier dans la page, avant ses scripts |
| `identity.rs` | Ce que le navigateur déclare de lui-même aux sites |
| `assets.rs` | Servir l'interface sous `echo://` |
| `bridge/` | Le canal entre l'interface et le cœur |
| `session.rs` | L'état vivant, accessible depuis le seul thread interface |
| `flags.rs` | Emplacements sur disque et drapeaux de démarrage |
| `selftest.rs` | Rejouer sans main les manipulations d'onglets, et les leviers de banc (`ECHO_BENCH_*`) |
| `sleep.rs` | Endormir les onglets inactifs (détruire leur navigateur, garder la fiche) |
| `persist.rs` | Enregistrer les onglets au fil de l'eau (dernière session), vidage avant fermeture |
| `permissions.rs` | Demandes de permission des sites : décision retenue ou question à l'utilisateur |
| `roundness.rs` | Angles arrondis de la page : masques de coin recolorables à chaud |
| `control.rs` | Prise Unix de pilotage (liste, lecture, ouverture d'onglets) pour les outils et le MCP |
| `suggest.rs` | Suggestions de la page « nouvel onglet » (onglets, favoris, historique) |
| `terminal.rs` | Routes `echo://ui/term/*` et commande du terminal Claude Code |
| `overlay.rs` | Vues posées au-dessus de la page (menus, fenêtres d'extension, liseré de bord) |

## Règles de frontière

- `shield` ne dépend d'aucun type CEF. Une dépendance à `cef` dans `core/shield/` est un défaut.
- `ui` ne connaît du cœur que le contrat. Aucun appel direct, aucun port réseau.
- L'état vivant (`session`) n'est accessible que sur le thread interface, **et jamais tenu pendant
  un appel à Chromium** — c'est la cause de deux arrêts brutaux corrigés le 2026-09-10.

## Frontières ajoutées le 06/10

- **`echo://` est joignable par n'importe quelle page web** (POST sans réponse lisible, iframes). Toute route sensible vérifie que
  l'appelant est une page `echo://` en cadre principal (`assets.rs`) ; test : `tools/test-ipc-origin.sh`.
- **La barre latérale est une surimpression** (`window.rs`) : un espaceur réserve sa place, elle glisse par-dessus ; repliée, la page
  prend toute la fenêtre. Une vue de largeur nulle sort de la disposition (largeur ≥ 1).
- **Hors du cœur** : `mcp/` (serveur MCP Python, parle à la prise Unix), `ui/public/` (pages statiques servies sous `echo://`).
- **Aucun test ne touche au navigateur de l'utilisateur** : instances de test isolées (`ECHO_RUN_DIR`, `ECHO_CONTROL_NAME`).
