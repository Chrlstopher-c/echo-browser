//! Responsabilite : ce que le clic droit propose, selon ce qui est sous le curseur.
//!
//! Chromium sert son propre menu — quatre entrees en anglais, sans rapport avec la
//! cible. Un navigateur doit proposer autre chose, dans sa langue, et selon qu'on a
//! clique sur un lien, une image, une selection, un champ ou la page nue.

use echo_contract::{ContextTarget, MenuEntry, MenuItemKind};

/// Ce que Chromium rapporte du clic, ramene a ce dont le menu a besoin.
pub struct Click {
    pub link: String,
    pub image: String,
    /// Adresse de la video ou du son clique.
    pub media: String,
    pub selection: String,
    pub page: String,
    pub editable: bool,
    pub can_go_back: bool,
    pub can_go_forward: bool,
}

/// Construit le menu qui convient a ce clic.
pub fn build(click: &Click) -> ContextTarget {
    let mut entries = Vec::new();
    if !click.link.is_empty() {
        entries.extend(link_entries(&click.link));
    }
    if !click.image.is_empty() {
        if !entries.is_empty() {
            entries.push(MenuEntry::separator());
        }
        entries.extend(image_entries(&click.image));
    }
    if !click.media.is_empty() {
        if !entries.is_empty() {
            entries.push(MenuEntry::separator());
        }
        entries.extend(media_entries());
    }
    if !click.selection.is_empty() {
        if !entries.is_empty() {
            entries.push(MenuEntry::separator());
        }
        entries.extend(selection_entries(&click.selection));
    }
    if click.editable {
        if !entries.is_empty() {
            entries.push(MenuEntry::separator());
        }
        entries.extend(field_entries(click));
    }
    if entries.is_empty() {
        entries.extend(page_entries(click));
    } else {
        entries.push(MenuEntry::separator());
        entries.push(MenuEntry::new(MenuItemKind::Inspect, "Examiner l'élément"));
    }
    ContextTarget { entries, link: click.link.clone(), selection: trim(&click.selection) }
}

fn link_entries(link: &str) -> Vec<MenuEntry> {
    vec![
        MenuEntry::new(MenuItemKind::OpenLinkInTab, "Ouvrir dans un nouvel onglet"),
        MenuEntry::new(MenuItemKind::OpenLinkInBackground, "Ouvrir en arrière-plan"),
        MenuEntry::new(MenuItemKind::CopyLink, "Copier l'adresse du lien"),
        MenuEntry::new(MenuItemKind::SaveLink, "Enregistrer la cible")
            .disabled_when(link.starts_with("javascript:")),
    ]
}

fn media_entries() -> Vec<MenuEntry> {
    vec![
        MenuEntry::new(MenuItemKind::OpenMedia, "Ouvrir la vidéo dans un onglet"),
        MenuEntry::new(MenuItemKind::CopyMediaLink, "Copier l'adresse de la vidéo"),
        MenuEntry::new(MenuItemKind::SaveMedia, "Enregistrer la vidéo"),
    ]
}

fn image_entries(_image: &str) -> Vec<MenuEntry> {
    vec![
        MenuEntry::new(MenuItemKind::CopyImage, "Copier l'image"),
        MenuEntry::new(MenuItemKind::OpenImage, "Ouvrir l'image dans un onglet"),
        MenuEntry::new(MenuItemKind::CopyImageLink, "Copier l'adresse de l'image"),
        MenuEntry::new(MenuItemKind::SaveImage, "Enregistrer l'image"),
    ]
}

fn selection_entries(selection: &str) -> Vec<MenuEntry> {
    let mut entries = vec![
        MenuEntry::new(MenuItemKind::Copy, "Copier"),
        MenuEntry::new(MenuItemKind::SearchSelection, "Rechercher cette sélection"),
    ];
    if looks_like_url(selection) {
        entries.push(MenuEntry::new(MenuItemKind::OpenSelection, "Ouvrir cette adresse"));
    }
    entries
}

fn field_entries(click: &Click) -> Vec<MenuEntry> {
    vec![
        MenuEntry::new(MenuItemKind::Cut, "Couper").disabled_when(click.selection.is_empty()),
        MenuEntry::new(MenuItemKind::Copy, "Copier").disabled_when(click.selection.is_empty()),
        MenuEntry::new(MenuItemKind::Paste, "Coller"),
        MenuEntry::new(MenuItemKind::PastePlain, "Coller sans mise en forme"),
        MenuEntry::new(MenuItemKind::SelectAll, "Tout sélectionner"),
    ]
}

fn page_entries(click: &Click) -> Vec<MenuEntry> {
    vec![
        MenuEntry::new(MenuItemKind::Back, "Précédent").disabled_when(!click.can_go_back),
        MenuEntry::new(MenuItemKind::Forward, "Suivant").disabled_when(!click.can_go_forward),
        MenuEntry::new(MenuItemKind::Reload, "Recharger"),
        MenuEntry::separator(),
        MenuEntry::new(MenuItemKind::CopyPageLink, "Copier l'adresse de la page"),
        MenuEntry::new(MenuItemKind::Bookmark, "Ajouter aux favoris"),
        MenuEntry::new(MenuItemKind::SavePage, "Enregistrer la page"),
        MenuEntry::new(MenuItemKind::Print, "Imprimer"),
        MenuEntry::separator(),
        MenuEntry::new(MenuItemKind::ToggleShield, "Bouclier sur ce site"),
        MenuEntry::new(MenuItemKind::ViewSource, "Code source"),
        MenuEntry::new(MenuItemKind::Inspect, "Examiner l'élément"),
    ]
}

/// Une selection courte et sans espace qui porte un point ou un schema est une adresse.
fn looks_like_url(selection: &str) -> bool {
    let trimmed = selection.trim();
    if trimmed.is_empty() || trimmed.len() > 2048 || trimmed.split_whitespace().count() > 1 {
        return false;
    }
    trimmed.starts_with("http://") || trimmed.starts_with("https://") || trimmed.contains('.')
}

/// Une selection sert d'etiquette : au-dela, elle deborde du menu.
fn trim(selection: &str) -> String {
    let clean = selection.trim();
    if clean.chars().count() <= 64 {
        return clean.to_string();
    }
    clean.chars().take(64).collect::<String>() + "…"
}

#[cfg(test)]
mod tests {
    use super::*;

    fn clic() -> Click {
        Click {
            link: String::new(),
            image: String::new(),
            media: String::new(),
            selection: String::new(),
            page: "https://exemple.fr/".to_string(),
            editable: false,
            can_go_back: false,
            can_go_forward: false,
        }
    }

    #[test]
    fn la_page_nue_propose_la_navigation_et_le_bouclier() {
        let menu = build(&clic());
        let kinds: Vec<_> = menu.entries.iter().map(|e| e.kind).collect();
        assert!(kinds.contains(&MenuItemKind::Reload));
        assert!(kinds.contains(&MenuItemKind::ToggleShield));
        assert!(kinds.contains(&MenuItemKind::Print));
        // Rien a copier ni a ouvrir : aucune entree de lien.
        assert!(!kinds.contains(&MenuItemKind::CopyLink));
    }

    #[test]
    fn precedent_est_grise_quand_il_n_y_a_pas_d_historique() {
        let menu = build(&clic());
        let back = menu.entries.iter().find(|e| e.kind == MenuItemKind::Back).expect("precedent");
        assert!(!back.enabled);
    }

    #[test]
    fn un_lien_propose_de_l_ouvrir_et_de_le_copier() {
        let mut click = clic();
        click.link = "https://exemple.fr/page".to_string();
        let kinds: Vec<_> = build(&click).entries.iter().map(|e| e.kind).collect();
        assert!(kinds.contains(&MenuItemKind::OpenLinkInTab));
        assert!(kinds.contains(&MenuItemKind::CopyLink));
        assert!(kinds.contains(&MenuItemKind::Inspect), "toujours examinable");
    }

    #[test]
    fn une_selection_qui_ressemble_a_une_adresse_s_ouvre() {
        let mut click = clic();
        click.selection = "exemple.fr/page".to_string();
        let kinds: Vec<_> = build(&click).entries.iter().map(|e| e.kind).collect();
        assert!(kinds.contains(&MenuItemKind::OpenSelection));

        click.selection = "deux mots".to_string();
        let kinds: Vec<_> = build(&click).entries.iter().map(|e| e.kind).collect();
        assert!(!kinds.contains(&MenuItemKind::OpenSelection));
        assert!(kinds.contains(&MenuItemKind::SearchSelection));
    }

    #[test]
    fn un_champ_propose_couper_copier_coller() {
        let mut click = clic();
        click.editable = true;
        let entries = build(&click).entries;
        let couper = entries.iter().find(|e| e.kind == MenuItemKind::Cut).expect("couper");
        assert!(!couper.enabled, "rien de selectionne");
        assert!(entries.iter().any(|e| e.kind == MenuItemKind::Paste));
    }

    #[test]
    fn une_selection_longue_est_ecourtee() {
        let long = "a".repeat(200);
        let mut click = clic();
        click.selection = long;
        assert_eq!(build(&click).selection.chars().count(), 65, "64 caracteres et l'ellipse");
    }
}

/// Hauteur d'une entree et d'un separateur, en pixels. Le coeur dimensionne la
/// surimpression : la page ne peut pas se mesurer elle-meme et nous le dire.
const ENTRY_HEIGHT: i32 = 28;
const SEPARATOR_HEIGHT: i32 = 9;
const MENU_PADDING: i32 = 8;
const MENU_WIDTH: i32 = 232;

/// Taille que la surimpression doit prendre pour contenir ce menu.
pub fn size_of(target: &ContextTarget) -> (i32, i32) {
    let height: i32 = target
        .entries
        .iter()
        .map(|entry| if entry.separator { SEPARATOR_HEIGHT } else { ENTRY_HEIGHT })
        .sum();
    (MENU_WIDTH, height + 2 * MENU_PADDING)
}
