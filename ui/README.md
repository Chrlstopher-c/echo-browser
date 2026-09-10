# ui — barre latérale d'Echo Browser

Colonne d'interface servie au cœur Rust. La page web n'est pas rendue ici : Chromium l'affiche
nativement à droite de cette colonne, dans un cadre que le cœur peint lui-même.

## Commandes

```bash
bun install
bun run dev        # port 4310, faux cœur en mémoire, scène de page simulée à droite
bun run build      # tsc --noEmit puis vite build → dist/
bun run typecheck
```

## Ce que le cœur doit fournir

`window.echo` conforme à `CoreBridge` (`src/shared/contract.ts`, figé) avant le chargement du
bundle. En son absence, `src/shared/fake-core.ts` prend le relais, un marqueur « simulation »
s'affiche dans la rangée de contrôles, et la place de la page est dessinée par `src/stage/`.

## Ce que l'interface réclame au cœur

| Requête | Quand | Valeur |
|---|---|---|
| `setChromeWidth` | au démarrage, à chaque repli ou dépli | 240 px dépliée, 56 px en rail |
| `setSidebarCollapsed` | à chaque repli ou dépli | l'état demandé |
| `setAccent` | au démarrage, à chaque changement d'espace | la couleur plate `shell` de l'espace |

Au repli, la largeur est envoyée **après** l'animation ; au dépli, **avant**. La page ne
chevauche jamais la barre. Le bord droit de la barre est plat, de la couleur envoyée par
`setAccent` : le cadre peint par le cœur s'y fond sans couture.

## Espaces

Six teintes dans `src/spaces/space-palette.ts` — cinq sombres (Graphite par défaut, Sable,
Rose, Forêt, Ardoise) et une claire (Lin). Chaque espace est un jeu complet de jetons, posé
sur `<html>` à l'exécution ; le choix est persisté localement.

## Sortie de build

`dist/` — `index.html`, JS et CSS empreintés, polices IBM Plex en woff2, aucune requête réseau.
Chemins relatifs (`base: './'`), servable depuis n'importe quelle racine locale.

## Domaines

| Dossier | Rôle |
|---|---|
| `shared/` | contrat figé, pont, état, faux cœur, système de design |
| `spaces/` | palette des espaces, application des jetons, sélecteur |
| `sidebar/` | assemblage de la barre, largeur réclamée, feuilles, rangée d'outils |
| `address/` | contrôles de navigation, champ d'adresse, sécurité, progression |
| `tabs/` | essentiels épinglés (grille) et liste verticale des onglets |
| `shield/` | bouton et feuille du bouclier |
| `library/` | favoris, historique, téléchargements |
| `extensions/` | liste des extensions |
| `settings/` | feuille des réglages |
| `stage/` | scène de la page, en simulation uniquement |
