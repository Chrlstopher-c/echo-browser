# Idées d'innovation (Chris, 08/10)

Philosophie retenue : léger par défaut, fonctions lourdes en opt-in dans les réglages, budgets CPU et cache
configurables, pas de GPU ni de VRAM imposés.

## Apprentissage et automatisation (sans ML lourd)
1. Empreinte de comportements : hash de séquences d'actions, proposition d'automatisation quand une séquence revient ;
   enrichissement optionnel par une synchro anonyme.
2. Workflows visuels enregistrés, éditables avant lancement.
3. Détection de formulaires et profils de données réutilisables.
4. Prédiction de navigation : préchargement ciblé selon l'historique.
5. Mémoire de structure DOM : empreinte structurelle (hash), réutilisation des stratégies sur les pages de même structure.
6. Reprise de session : état exact de la page restauré au redémarrage.

## Performance et réseau
7. Négociation de contenu côté client : réécriture des requêtes (ex. WebP au lieu de JPEG) selon la connexion.
8. Rendu adaptatif selon le réseau et l'écran.
9. Cache de snapshots DOM pré-calculés.
10. Tableau de bord réseau temps réel : initiateur, destination, coût, blocage ou modification à la volée.
11. Mutualisation des bibliothèques JS courantes entre sites (empreinte des dépendances).
12. Rapport de performance granulaire : scripts tiers bloquants, images surdimensionnées.
13. Optimisation de protocole HTTP/3 et compression personnalisée.

## Sécurité et transparence
14. Bac à sable par domaine avec journal transparent des accès.
15. Détection des changements de contenu (empreinte cryptographique, diff exact).
16. Gestion proactive des erreurs : contournement d'un script qui plante systématiquement.
17. Isolation de rendu : iframes ou workers instables dans un processus séparé.

## Expérience et accessibilité
18. Rendu accessible d'abord : texte et structure sans JavaScript, enrichissement ensuite.
19. Suivi d'attention par interactions DOM (défilement, survol, clic), sans webcam.
20. Réinterprétation sémantique des pages : thème sombre, colonnes, sans publicité, reconstruites localement.
21. Replay temporel du DOM : diffs horodatés, retour à un état passé.

## Plus expérimental (à évaluer)
22. Réseau de confiance entre navigateurs (signatures d'état).
23. Partage coopératif entre instances locales.
24. Grammaire de formulaire canonique, indépendante du HTML.

Avis de Chris : 1, 5, 6, 10 et 14 les plus réalisables à court terme ; 17, 22 et 23 demandent le plus de travail et de
précautions de sécurité. Non cadré : à reprendre après le compte Echo synchronisé.
