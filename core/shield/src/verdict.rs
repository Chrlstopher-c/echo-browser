//! Responsabilite : ce qu'il faut faire d'une requete ou d'une page, exprime sans dependre du moteur.

use serde::{Deserialize, Serialize};

/// Decision prise sur une requete reseau.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Verdict {
    /// Laisser passer.
    Allow,
    /// Bloquer, en citant la regle responsable quand elle est connue.
    Block { rule: Option<String> },
    /// Servir une ressource de remplacement inoffensive a la place de la ressource distante.
    Redirect { resource: String },
}

impl Verdict {
    pub fn is_block(&self) -> bool {
        matches!(self, Verdict::Block { .. })
    }
}

/// Ce qu'il faut injecter dans une page pour completer le blocage reseau.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PageTreatment {
    /// Selecteurs CSS a masquer.
    pub hide_selectors: Vec<String>,
    /// Filtres procéduraux encodes en JSON (`:has-text`, `:upward`, actions de style).
    pub procedural_actions: Vec<String>,
    /// Code des scriptlets a executer avant le script de la page.
    pub injected_script: String,
}

impl PageTreatment {
    pub fn is_empty(&self) -> bool {
        self.hide_selectors.is_empty()
            && self.procedural_actions.is_empty()
            && self.injected_script.is_empty()
    }

    /// Feuille de style prete a injecter, ou `None` s'il n'y a rien a masquer.
    pub fn hiding_stylesheet(&self) -> Option<String> {
        if self.hide_selectors.is_empty() {
            return None;
        }
        Some(format!(
            "{} {{ display: none !important; }}",
            self.hide_selectors.join(",\n")
        ))
    }
}
