# Brief — echo-browser
**Quoi** : navigateur de bureau Linux (Rust + CEF/Chromium), barre latérale façon Zen/Arc, qui remplace Chrome au quotidien.
**Pour qui** : Chris seul ; comptes connectés, automatisations, Claude Code (tmux) intégré, joignable depuis Quart.
**Stack** : Rust (coque, onglets, bouclier) · CEF · React/TS/Tailwind (interface) ; Bun/pnpm selon le lockfile du dossier.
**Réussite** : (1) 10 onglets ≤ RAM de Chrome sur les mêmes pages (`tools/bench-ram.sh`), et ≤ 500 Mo avec la veille ;
(2) design Zen/Arc sans défaut visible (marge, angles, nouvel onglet, bulles) ; (3) un jour entier sans retour à Chrome : Google connecté, mots de passe, vidéo.
**Hors périmètre** : synchronisation cloud propre, version Windows/macOS, magasin d'extensions maison.
