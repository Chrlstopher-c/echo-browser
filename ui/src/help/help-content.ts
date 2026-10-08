// Responsabilite : le contenu de l'Aide — ce qu'Echo sait faire, ou le trouver, et un bouton qui y mene. Les textes
// decrivent le geste exact (icone, clic droit, raccourci) : l'Aide sert a trouver, pas a expliquer la technique.

import type { UiRequest } from '../shared/contract'

export interface HelpAction {
  label: string
  request: UiRequest
}

export interface HelpTopic {
  title: string
  /** Comment y aller, en une ou deux phrases. */
  how: string
  action?: HelpAction
}

export interface HelpSection {
  id: string
  title: string
  intro: string
  topics: HelpTopic[]
}

const sheet = (name: string): UiRequest => ({ kind: 'openSidebarSheet', sheet: name })
const page = (name: 'reglages' | 'bibliotheque' | 'extensions'): UiRequest => ({ kind: 'openPage', page: name })

export const HELP: HelpSection[] = [
  {
    id: 'securite', title: 'Sécurité et réseau',
    intro: 'Voir avec qui chaque page communique, ce qu’elle charge, et ce qu’Echo bloque pour vous.',
    topics: [
      { title: 'Sécurité du site', how: 'Cliquez le cadenas à gauche de l’adresse : domaines contactés, tiers, octets, '
        + 'requêtes bloquées.', action: { label: 'Ouvrir', request: sheet('network') } },
      { title: 'Poids de la page', how: 'Panneau Sécurité et réseau → Poids : part des tiers, requêtes les plus lourdes '
        + 'et les plus lentes, images et scripts trop lourds.' },
      { title: 'Journal d’accès', how: 'Panneau Sécurité et réseau → Journal : premiers contacts avec des tiers, '
        + 'permissions demandées (caméra, position…), téléchargements.' },
      { title: 'Bloquer un tiers ou isoler un site', how: '« Bloquer » à côté d’un domaine tiers, ou « Isolement strict » '
        + 'pour qu’un site ne contacte plus aucun autre site. Gardé d’une session à l’autre.' },
      { title: 'Bouclier', how: 'Le bouclier en bas à gauche bloque publicités et traqueurs ; il se coupe par site.',
        action: { label: 'Ouvrir', request: sheet('shield') } },
    ],
  },
  {
    id: 'profils', title: 'Profils et conteneurs',
    intro: 'Un profil est une identité complète : ses onglets, ses comptes, ses extensions.',
    topics: [
      { title: 'Changer de profil', how: 'Les pastilles en bas de la barre ; Ctrl+↓/↑ dans la barre pour passer au '
        + 'suivant.' },
      { title: 'Créer, renommer, supprimer', how: 'Le « + » à côté des pastilles crée un profil ; Réglages → Profils '
        + 'pour renommer, changer la teinte, réinitialiser ou supprimer.', action: { label: 'Réglages', request: page('reglages') } },
      { title: 'Conteneurs', how: 'Clic droit sur un onglet → ouvrir dans un conteneur : un second compte sur le même '
        + 'site, sans changer de profil.' },
    ],
  },
  {
    id: 'pages', title: 'Clic droit dans une page',
    intro: 'Des outils qui agissent sur la page en cours.',
    topics: [
      { title: 'Remplir un formulaire', how: 'Clic droit dans un champ → « Remplir : fiche ». Les fiches (nom, e-mail, '
        + 'adresse…) se créent dans Réglages → Formulaires ; jamais de mot de passe ni de carte.' },
      { title: 'Masquer cet élément', how: 'Clic droit sur une bannière ou un bloc gênant : masqué sur toutes les pages '
        + 'du même modèle du site. « Réafficher les éléments masqués » au même endroit.' },
      { title: 'Surveiller cette page', how: 'À la visite suivante, Echo montre en bas de la barre les lignes ajoutées '
        + 'et retirées (un prix, une annonce…).' },
      { title: 'Examiner l’élément', how: 'Les outils de développement, sur l’élément cliqué (F12 pour la page).' },
    ],
  },
  {
    id: 'quotidien', title: 'Au quotidien',
    intro: 'Ce qu’Echo fait tout seul, et comment en profiter.',
    topics: [
      { title: 'Palette d’adresse', how: 'Ctrl+K (ou Ctrl+L) puis tapez : onglets ouverts, favoris et historique '
        + 'apparaissent sous l’adresse ; ↓/↑ puis Entrée. Un onglet déjà ouvert est rejoint, pas dupliqué.' },
      { title: 'Reprise exacte', how: 'Ce que vous aviez tapé et la position des vidéos reviennent après une relance '
        + 'ou un onglet réveillé (jamais les mots de passe).' },
      { title: 'Routines', how: 'Quand vous ouvrez souvent les mêmes sites à la suite, Echo propose d’en faire une '
        + 'routine ; Bibliothèque → Routines pour les rouvrir d’un clic.', action: { label: 'Bibliothèque', request: page('bibliotheque') } },
      { title: 'Veille des onglets', how: 'Les onglets inactifs dorment pour rendre la mémoire ; clic droit → « Garder '
        + 'éveillé » pour l’éviter.' },
      { title: 'Fichiers et dossiers', how: 'Tapez un chemin (/… ou ~/…) dans l’adresse, Ctrl+O pour choisir un fichier, '
        + 'ou « Ouvrir avec Echo » depuis le gestionnaire de fichiers.' },
      { title: 'Extensions', how: 'Le Chrome Web Store fonctionne : « Ajouter à Echo » sur la fiche d’une extension.',
        action: { label: 'Extensions', request: page('extensions') } },
    ],
  },
  {
    id: 'compte', title: 'Compte Echo',
    intro: 'Vos réglages, favoris, extensions, historique et onglets sur toutes vos machines, chiffrés de bout en bout.',
    topics: [
      { title: 'Synchronisation', how: 'Réglages → Compte : mode temps réel, automatique ou manuel. Une alerte en bas '
        + 'de la barre signale quand la machine n’est pas à jour.', action: { label: 'Réglages', request: page('reglages') } },
      { title: 'Vos données', how: 'Réglages → Compte → Données stockées : ce que le serveur garde, déchiffré ici ; et '
        + '« Supprimer le compte » pour tout effacer.' },
      { title: 'Autres machines', how: 'Bibliothèque → Machines : les onglets ouverts sur vos autres ordinateurs.' },
    ],
  },
]

export const SHORTCUTS: Array<[string, string]> = [
  ['Ctrl+T', 'Nouvel onglet'], ['Ctrl+W', 'Fermer l’onglet'], ['Ctrl+Maj+T', 'Rouvrir l’onglet fermé'],
  ['Ctrl+Tab / Ctrl+Page↓', 'Onglet suivant'], ['Ctrl+Maj+Tab / Ctrl+Page↑', 'Onglet précédent'],
  ['Ctrl+1 … 9', 'Aller à l’onglet n'], ['Ctrl+L / Ctrl+K', 'Adresse et palette'], ['Ctrl+O', 'Ouvrir un fichier'],
  ['Ctrl+D', 'Ajouter aux favoris'], ['Ctrl+H / Ctrl+J', 'Bibliothèque (historique, fichiers)'],
  ['Ctrl+P', 'Imprimer'], ['Ctrl+S', 'Enregistrer la page'], ['Ctrl+U', 'Code source'],
  ['Ctrl+R / F5', 'Recharger'], ['Ctrl+Maj+R', 'Recharger sans cache'], ['Alt+← / Alt+→', 'Précédent / suivant'],
  ['Ctrl+ + / Ctrl+ − / Ctrl+0', 'Zoom'], ['F11', 'Quitter le plein écran'], ['F12 / Ctrl+Maj+I', 'Outils de développement'],
  ['F1', 'Cette aide'],
]
