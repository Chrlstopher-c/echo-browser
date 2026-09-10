//! Responsabilite : ce qu'une extension propose dans la barre — sa fenetre et son icone.
//!
//! Une extension moderne vit dans sa fenetre : celle de Proton Pass *est* le gestionnaire
//! de mots de passe. La lister sans pouvoir l'ouvrir la rend decorative. Le manifeste dit
//! ou trouver cette fenetre (`action.default_popup`) et quelle icone la represente.

use serde_json::Value;

/// Ce qu'une extension expose a la barre d'outils.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Action {
    /// Chemin de la fenetre dans le paquet, quand l'extension en declare une.
    pub popup: Option<String>,
    /// Chemin de l'icone dans le paquet, la plus grande disponible.
    pub icon: Option<String>,
    /// Chemin de sa page de reglages, quand elle en propose une.
    pub options: Option<String>,
}

impl Action {
    /// Lit l'action d'un manifeste deja analyse.
    ///
    /// Manifeste v3 dit `action`, v2 dit `browser_action` ou `page_action` — les trois
    /// existent encore dans les paquets installes. L'icone se cherche d'abord dans
    /// l'action, puis dans les icones generales du paquet.
    pub fn from_manifest(manifest: &Value) -> Self {
        let action = ["action", "browser_action", "page_action"]
            .iter()
            .find_map(|key| manifest.get(*key));

        let popup = action
            .and_then(|a| a.get("default_popup"))
            .and_then(Value::as_str)
            .map(trim_path);

        let icon = action
            .and_then(|a| a.get("default_icon"))
            .and_then(largest_icon)
            .or_else(|| manifest.get("icons").and_then(largest_icon));

        // `options_ui.page` est la forme moderne, `options_page` celle des vieux paquets.
        let options = manifest
            .get("options_ui")
            .and_then(|ui| ui.get("page"))
            .or_else(|| manifest.get("options_page"))
            .and_then(Value::as_str)
            .map(trim_path);

        Self { popup, icon, options }
    }
}

/// Une icone se declare soit en chemin unique, soit en table taille → chemin.
/// On prend la plus grande : elle sera reduite proprement, l'inverse baverait.
fn largest_icon(value: &Value) -> Option<String> {
    if let Some(path) = value.as_str() {
        return Some(trim_path(path));
    }
    let table = value.as_object()?;
    table
        .iter()
        .filter_map(|(size, path)| Some((size.parse::<u32>().ok()?, path.as_str()?)))
        .max_by_key(|(size, _)| *size)
        .map(|(_, path)| trim_path(path))
}

/// Les paquets ecrivent tantot `popup.html`, tantot `/popup.html` : la barre oblique de
/// tete doublerait celle de l'adresse construite ensuite.
fn trim_path(path: &str) -> String {
    path.trim_start_matches('/').to_string()
}

/// Adresse interne d'une ressource d'extension.
pub fn resource_url(id: &str, path: &str) -> String {
    format!("chrome-extension://{id}/{}", path.trim_start_matches('/'))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lit_une_action_de_manifeste_v3() {
        let manifest = serde_json::json!({
            "manifest_version": 3,
            "action": { "default_popup": "popup.html",
                        "default_icon": { "16": "i16.png", "48": "i48.png" } }
        });
        let action = Action::from_manifest(&manifest);
        assert_eq!(action.popup.as_deref(), Some("popup.html"));
        assert_eq!(action.icon.as_deref(), Some("i48.png"), "la plus grande icone");
    }

    #[test]
    fn retombe_sur_les_icones_du_paquet_et_accepte_le_v2() {
        let manifest = serde_json::json!({
            "manifest_version": 2,
            "browser_action": { "default_popup": "/ui/popup/index.html" },
            "icons": { "128": "logo.png" }
        });
        let action = Action::from_manifest(&manifest);
        assert_eq!(action.popup.as_deref(), Some("ui/popup/index.html"), "barre de tete retiree");
        assert_eq!(action.icon.as_deref(), Some("logo.png"));
    }

    #[test]
    fn lit_la_page_de_reglages_sous_ses_deux_formes() {
        let moderne = serde_json::json!({ "options_ui": { "page": "options.html" } });
        assert_eq!(Action::from_manifest(&moderne).options.as_deref(), Some("options.html"));
        let ancienne = serde_json::json!({ "options_page": "/reglages.html" });
        assert_eq!(Action::from_manifest(&ancienne).options.as_deref(), Some("reglages.html"));
    }

    #[test]
    fn une_extension_sans_action_n_en_declare_aucune() {
        let action = Action::from_manifest(&serde_json::json!({ "manifest_version": 3 }));
        assert_eq!(action, Action::default());
    }

    #[test]
    fn l_adresse_interne_ne_double_jamais_la_barre() {
        assert_eq!(resource_url("abc", "/popup.html"), "chrome-extension://abc/popup.html");
    }
}
