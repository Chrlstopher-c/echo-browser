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
| `selftest.rs` | Rejouer sans main les manipulations d'onglets |

## Règles de frontière

- `shield` ne dépend d'aucun type CEF. Une dépendance à `cef` dans `core/shield/` est un défaut.
- `ui` ne connaît du cœur que le contrat. Aucun appel direct, aucun port réseau.
- L'état vivant (`session`) n'est accessible que sur le thread interface, **et jamais tenu pendant
  un appel à Chromium** — c'est la cause de deux arrêts brutaux corrigés le 2026-09-10.
