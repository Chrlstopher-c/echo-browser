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
/// Ce que le menu doit savoir de la page, au-dela du clic.
#[derive(Debug, Clone, Default)]
pub struct PageFacts {
    /// Des elements sont masques sur ce gabarit.
    pub hidden_here: bool,
    /// La page est surveillee.
    pub watched: bool,
    /// Noms des fiches de formulaire proposees.
    pub forms: Vec<String>,
    /// La page est affichee en mode lecture.
    pub reading: bool,
    /// Noms des conteneurs proposes pour ouvrir un lien.
    pub containers: Vec<String>,
    /// Le bouclier filtre-t-il cette page ? L'entree dit ce qu'elle fera, pas un simple « Bouclier ».
    pub shield_on: bool,
}

pub fn build(click: &Click, facts: PageFacts) -> ContextTarget {
    let mut entries = Vec::new();
    let groups: [(bool, Vec<MenuEntry>); 5] = [
        (!click.link.is_empty(), link_entries(&click.link, &facts.containers)),
        (!click.image.is_empty(), image_entries(&click.image)),
        (!click.media.is_empty(), media_entries()),
        (!click.selection.is_empty(), selection_entries(&click.selection)),
        (click.editable, field_entries(click, &facts.forms)),
    ];
    for (_, group) in groups.into_iter().filter(|(shown, _)| *shown) {
        if !entries.is_empty() {
            entries.push(MenuEntry::separator());
        }
        entries.extend(group);
    }
    if entries.is_empty() {
        entries.extend(page_entries(click, facts.shield_on));
    } else {
        entries.push(MenuEntry::separator());
        entries.push(MenuEntry::new(MenuItemKind::HideElement, "Masquer cet élément"));
        entries.push(MenuEntry::new(MenuItemKind::Inspect, "Examiner l'élément"));
    }
    entries.extend(page_tools(click, &facts));
    ContextTarget { entries, link: click.link.clone(), selection: trim(&click.selection) }
}

/// Outils qui dependent de la page : reafficher ce qui est masque, surveiller ou non.
fn page_tools(click: &Click, facts: &PageFacts) -> Vec<MenuEntry> {
    let mut tools = Vec::new();
    if facts.hidden_here {
        tools.push(MenuEntry::new(MenuItemKind::UnhideElements, "Réafficher les éléments masqués"));
    }
    if click.page.starts_with("http") || click.page.starts_with("file:") {
        tools.push(MenuEntry::new(MenuItemKind::FindInPage, "Rechercher dans la page"));
    }
    if click.page.starts_with("http") && !click.editable {
        tools.push(MenuEntry::new(MenuItemKind::TranslatePage, "Traduire la page en français"));
        let label = if facts.reading { "Quitter la lecture" } else { "Lire en mode lecture" };
        tools.push(MenuEntry::new(MenuItemKind::Reader, label));
        tools.push(if facts.watched {
            MenuEntry::new(MenuItemKind::UnwatchPage, "Ne plus surveiller cette page")
        } else {
            MenuEntry::new(MenuItemKind::WatchPage, "Surveiller cette page")
        });
    }
    tools
}

fn link_entries(link: &str, containers: &[String]) -> Vec<MenuEntry> {
    const KINDS: [MenuItemKind; 3] =
        [MenuItemKind::OpenLinkInContainer1, MenuItemKind::OpenLinkInContainer2, MenuItemKind::OpenLinkInContainer3];
    let mut entries = vec![
        MenuEntry::new(MenuItemKind::OpenLinkInTab, "Ouvrir dans un nouvel onglet"),
        MenuEntry::new(MenuItemKind::OpenLinkInBackground, "Ouvrir en arrière-plan"),
        MenuEntry::new(MenuItemKind::OpenLinkPrivate, "Ouvrir en navigation privée"),
    ];
    let in_container =
        containers.iter().zip(KINDS).map(|(name, kind)| MenuEntry::new(kind, &format!("Ouvrir dans : {name}")));
    entries.extend(in_container);
    entries.push(MenuEntry::new(MenuItemKind::CopyLink, "Copier l'adresse du lien"));
    let save = MenuEntry::new(MenuItemKind::SaveLink, "Enregistrer la cible");
    entries.push(save.disabled_when(link.starts_with("javascript:")));
    entries
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
        MenuEntry::new(MenuItemKind::SearchSelection, &search_label(selection)),
    ];
    if looks_like_url(selection) {
        entries.push(MenuEntry::new(MenuItemKind::OpenSelection, "Ouvrir cette adresse"));
    }
    entries
}

fn field_entries(click: &Click, forms: &[String]) -> Vec<MenuEntry> {
    let mut entries = form_entries(forms);
    entries.extend([
        MenuEntry::new(MenuItemKind::Cut, "Couper").disabled_when(click.selection.is_empty()),
        MenuEntry::new(MenuItemKind::Copy, "Copier").disabled_when(click.selection.is_empty()),
        MenuEntry::new(MenuItemKind::Paste, "Coller"),
        MenuEntry::new(MenuItemKind::PastePlain, "Coller sans mise en forme"),
        MenuEntry::new(MenuItemKind::SelectAll, "Tout sélectionner"),
    ]);
    entries
}

/// « Remplir : <fiche> » pour chaque fiche, ou une entree qui mene a leur creation.
fn form_entries(forms: &[String]) -> Vec<MenuEntry> {
    const KINDS: [MenuItemKind; 3] = [MenuItemKind::FillForm1, MenuItemKind::FillForm2, MenuItemKind::FillForm3];
    let mut entries: Vec<MenuEntry> =
        forms.iter().zip(KINDS).map(|(name, kind)| MenuEntry::new(kind, &format!("Remplir : {name}"))).collect();
    if entries.is_empty() {
        entries.push(MenuEntry::new(MenuItemKind::ManageForms, "Remplir le formulaire…"));
    }
    entries.push(MenuEntry::separator());
    entries
}

fn page_entries(click: &Click, shield_on: bool) -> Vec<MenuEntry> {
    let shield = if shield_on { "Désactiver le bouclier sur ce site" } else { "Réactiver le bouclier sur ce site" };
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
        MenuEntry::new(MenuItemKind::ToggleShield, shield),
        MenuEntry::new(MenuItemKind::ViewSource, "Code source"),
        MenuEntry::new(MenuItemKind::HideElement, "Masquer cet élément"),
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
        let menu = build(&clic(), PageFacts::default());
        let kinds: Vec<_> = menu.entries.iter().map(|e| e.kind).collect();
        assert!(kinds.contains(&MenuItemKind::Reload));
        assert!(kinds.contains(&MenuItemKind::ToggleShield));
        assert!(kinds.contains(&MenuItemKind::Print));
        // Rien a copier ni a ouvrir : aucune entree de lien.
        assert!(!kinds.contains(&MenuItemKind::CopyLink));
    }

    #[test]
    fn un_champ_propose_les_fiches_ou_leur_creation() {
        let mut click = clic();
        click.editable = true;
        let kinds: Vec<_> = build(&click, PageFacts::default()).entries.iter().map(|e| e.kind).collect();
        assert!(kinds.contains(&MenuItemKind::ManageForms));
        let facts = PageFacts { forms: vec!["Perso".into(), "Travail".into()], ..PageFacts::default() };
        let menu = build(&click, facts);
        let labels: Vec<_> = menu.entries.iter().map(|e| e.label.as_str()).collect();
        assert!(labels.contains(&"Remplir : Perso") && labels.contains(&"Remplir : Travail"));
        assert!(!menu.entries.iter().any(|e| e.kind == MenuItemKind::ManageForms || e.kind == MenuItemKind::FillForm3));
    }

    #[test]
    fn precedent_est_grise_quand_il_n_y_a_pas_d_historique() {
        let menu = build(&clic(), PageFacts::default());
        let back = menu.entries.iter().find(|e| e.kind == MenuItemKind::Back).expect("precedent");
        assert!(!back.enabled);
    }

    #[test]
    fn un_lien_propose_de_l_ouvrir_et_de_le_copier() {
        let mut click = clic();
        click.link = "https://exemple.fr/page".to_string();
        let kinds: Vec<_> = build(&click, PageFacts::default()).entries.iter().map(|e| e.kind).collect();
        assert!(kinds.contains(&MenuItemKind::OpenLinkInTab));
        assert!(kinds.contains(&MenuItemKind::CopyLink));
        assert!(kinds.contains(&MenuItemKind::Inspect), "toujours examinable");
    }

    #[test]
    fn une_selection_qui_ressemble_a_une_adresse_s_ouvre() {
        let mut click = clic();
        click.selection = "exemple.fr/page".to_string();
        let kinds: Vec<_> = build(&click, PageFacts::default()).entries.iter().map(|e| e.kind).collect();
        assert!(kinds.contains(&MenuItemKind::OpenSelection));

        click.selection = "deux mots".to_string();
        let kinds: Vec<_> = build(&click, PageFacts::default()).entries.iter().map(|e| e.kind).collect();
        assert!(!kinds.contains(&MenuItemKind::OpenSelection));
        assert!(kinds.contains(&MenuItemKind::SearchSelection));
    }

    #[test]
    fn un_champ_propose_couper_copier_coller() {
        let mut click = clic();
        click.editable = true;
        let entries = build(&click, PageFacts::default()).entries;
        let couper = entries.iter().find(|e| e.kind == MenuItemKind::Cut).expect("couper");
        assert!(!couper.enabled, "rien de selectionne");
        assert!(entries.iter().any(|e| e.kind == MenuItemKind::Paste));
    }

    #[test]
    fn une_selection_longue_est_ecourtee() {
        let long = "a".repeat(200);
        let mut click = clic();
        click.selection = long;
        assert_eq!(build(&click, PageFacts::default()).selection.chars().count(), 65, "64 caracteres et l'ellipse");
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

/// « Rechercher « debut de la selection » sur <moteur> », comme Chrome.
fn search_label(selection: &str) -> String {
    let short: String = selection.split_whitespace().collect::<Vec<_>>().join(" ").chars().take(24).collect();
    let more = if selection.trim().chars().count() > 24 { "…" } else { "" };
    format!("Rechercher « {short}{more} » sur {}", crate::search::engine().name)
}
