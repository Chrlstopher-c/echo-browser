//! Contrat des surimpressions : theme transmis aux surimpressions, menu contextuel des pages, ancres.

use serde::{Deserialize, Serialize};

/// Les quelques couleurs dont une surimpression a besoin pour se fondre dans l'espace.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OverlayTheme {
    pub shell: String,
    pub card: String,
    pub hover: String,
    pub hairline: String,
    pub ink: String,
    pub ink_muted: String,
    pub ink_faint: String,
    /// Lumiere et ombre du relief neumorphique, et teinte de l'espace.
    pub hi: String,
    pub lo: String,
    pub tint: String,
    pub danger: String,
}

impl Default for OverlayTheme {
    fn default() -> Self {
        Self {
            shell: "#222326".to_string(),
            card: "#222326".to_string(),
            hover: "#27282c".to_string(),
            hairline: "#34363b".to_string(),
            ink: "#ebeced".to_string(),
            ink_muted: "#a3a6ae".to_string(),
            ink_faint: "#6c707a".to_string(),
            hi: "rgba(255, 255, 255, 0.075)".to_string(),
            lo: "rgba(0, 0, 0, 0.7)".to_string(),
            tint: "#8f96a3".to_string(),
            danger: "#e5484d".to_string(),
        }
    }
}

/// Ce que propose le clic droit, construit par le coeur selon la cible.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContextTarget {
    pub entries: Vec<MenuEntry>,
    /// Adresse du lien clique, vide s'il n'y en avait pas.
    pub link: String,
    /// Texte selectionne, ecourte pour l'affichage.
    pub selection: String,
}

/// Une entree du menu contextuel.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MenuEntry {
    pub kind: MenuItemKind,
    pub label: String,
    pub enabled: bool,
    /// Vrai pour un simple trait de separation : ni libelle, ni action.
    pub separator: bool,
}

impl MenuEntry {
    pub fn new(kind: MenuItemKind, label: &str) -> Self {
        Self { kind, label: label.to_string(), enabled: true, separator: false }
    }

    pub fn separator() -> Self {
        Self { kind: MenuItemKind::Separator, label: String::new(), enabled: false, separator: true }
    }

    /// Grise l'entree quand la condition est vraie. L'entree reste affichee : une action
    /// qui disparait deplace les autres et se cherche du regard.
    pub fn disabled_when(mut self, condition: bool) -> Self {
        self.enabled = !condition;
        self
    }
}

/// Les actions que le menu contextuel sait declencher.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum MenuItemKind {
    Separator,
    /// Remplir le formulaire avec la premiere, deuxieme ou troisieme fiche.
    FillForm1,
    FillForm2,
    FillForm3,
    /// Aucune fiche : ouvre les reglages pour en creer une.
    ManageForms,
    /// Entrer en mode lecture, ou en sortir.
    Reader,
    OpenLinkInTab,
    OpenLinkInBackground,
    CopyLink,
    SaveLink,
    OpenImage,
    CopyImageLink,
    SaveImage,
    /// Copie l'image elle-meme (pas son adresse) dans le presse-papiers.
    CopyImage,
    OpenMedia,
    CopyMediaLink,
    SaveMedia,
    Copy,
    Cut,
    Paste,
    PastePlain,
    SelectAll,
    SearchSelection,
    OpenSelection,
    Back,
    Forward,
    Reload,
    CopyPageLink,
    Bookmark,
    SavePage,
    Print,
    ToggleShield,
    ViewSource,
    Inspect,
    /// Masquer l'element clique, sur toutes les pages du meme gabarit.
    HideElement,
    /// Reafficher ce qui a ete masque sur le gabarit de cette page.
    UnhideElements,
    /// Surveiller la page : signaler ce qui a change a la prochaine visite.
    WatchPage,
    UnwatchPage,
}

/// Un rectangle de l'interface, en pixels, repere depuis le coin haut-gauche de la
/// fenetre. Sert d'ancre a ce qui s'affiche au-dessus de la page.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AnchorRect {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}
