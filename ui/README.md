# ui — chrome d'Echo Browser

Bande d'interface servie au cœur Rust. La page web n'est pas rendue ici : Chromium l'affiche
nativement sous cette bande.

## Commandes

```bash
bun install
bun run dev        # port 4310, faux cœur en mémoire
bun run build      # tsc --noEmit puis vite build → dist/
bun run typecheck
```

## Ce que le cœur doit fournir

`window.echo` conforme à `CoreBridge` (`src/shared/contract.ts`, figé) avant le chargement du
bundle. En son absence, `src/shared/fake-core.ts` prend le relais et un marqueur « simulation »
s'affiche en haut à droite.

Le chrome mesure 78 px au repos et réclame sa hauteur au cœur par `setChromeHeight` à chaque
ouverture ou fermeture de panneau.

## Sortie de build

`dist/` — `index.html`, JS et CSS empreintés, polices IBM Plex en woff2. Chemins relatifs
(`base: './'`), servable depuis n'importe quelle racine locale.

## Domaines

| Dossier | Rôle |
|---|---|
| `shared/` | contrat figé, pont, état, système de design |
| `chrome/` | bande d'onglets, barre de navigation, adresse, progression |
| `shield/` | bouton et panneau du bouclier |
| `library/` | favoris, historique, téléchargements |
| `extensions/` | liste des extensions |
| `shell/` | assemblage, panneaux, hauteur réclamée |
