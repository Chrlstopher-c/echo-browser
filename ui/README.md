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
bundle. En son absence, `src/shared/fake/` prend le relais, un marqueur « simulation » s'affiche
dans la rangée de contrôles, et la place de la page est dessinée par `src/stage/`, avec une
barre de leviers (son, sommeil, téléchargement, plein écran) pour juger chaque état.

Raccourcis en simulation : `Ctrl+L` (focus adresse, relayé comme `focusAddressRequested`),
`F11` (plein écran), `Échap` (sortie du plein écran, fermeture d'une feuille).

## Ce que l'interface réclame au cœur

| Requête | Quand | Valeur |
|---|---|---|
| `setChromeWidth` | au démarrage, à chaque repli ou dépli | 248 px dépliée, 56 px en rail |
| `setSidebarCollapsed` | à chaque repli ou dépli | l'état demandé |
| `setAccent` | au démarrage, à chaque changement d'espace | la couleur plate `shell` de l'espace |

Au repli, la largeur est envoyée **après** l'animation ; au dépli, **avant**. La page ne
chevauche jamais la barre. Le bord droit de la barre est plat, de la couleur envoyée par
`setAccent` : le cadre peint par le cœur s'y fond sans couture.

En plein écran (`fullscreenChanged { active: true }`), l'interface se démonte entièrement ;
elle revient à la sortie avec son état intact.

## Espaces

Six teintes dans `src/spaces/space-palette.ts` — cinq sombres (Graphite par défaut, Sable,
Rose, Forêt, Ardoise) et une claire (Lin). Chaque espace est un jeu complet de jetons, posé
sur `<html>` à l'exécution ; le choix est persisté localement. Bande de points au pied de la
barre, sélecteur complet dans les réglages, `Ctrl+Alt+↑/↓` pour passer de l'un à l'autre.

## Réglages

Les clés que le cœur livre (`settingsChanged`) sont habillées par `src/settings/setting-catalogue.ts` :
groupe, libellé, explication, forme du contrôle. Une clé absente du catalogue reste affichée,
dans « Autres », avec un contrôle déduit de son type.

## Sortie de build

`dist/` — `index.html`, JS et CSS empreintés, polices Instrument Sans et JetBrains Mono en
woff2, aucune requête réseau. Chemins relatifs (`base: './'`), servable depuis n'importe quelle
racine locale.

## Domaines

| Dossier | Rôle |
|---|---|
| `shared/` | contrat figé, pont, état, faux cœur (`fake/`), système de design (`design/`) |
| `spaces/` | palette des espaces, application des jetons, sélecteur, bande de points |
| `sidebar/` | assemblage de la barre, largeur réclamée, feuilles, rangée d'outils, clavier |
| `address/` | contrôles de navigation, champ d'adresse, sécurité, progression, zoom |
| `tabs/` | onglets épinglés (pastilles), liste réordonnable, menu contextuel, marque d'état |
| `shield/` | bouton et feuille du bouclier, listes de filtres |
| `library/` | favoris, historique, téléchargements |
| `extensions/` | inventaire, installation par le catalogue, retrait |
| `settings/` | catalogue des réglages et feuille |
| `restart/` | bande et écran de relance |
| `stage/` | scène de la page et leviers, en simulation uniquement |
