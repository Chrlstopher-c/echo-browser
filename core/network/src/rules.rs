//! Responsabilite : les regles reseau choisies par l'utilisateur, par site — domaines bloques sur ce site, et
//! « isolement strict » (aucune requete tierce). Gardees dans un fichier JSON du dossier de donnees.

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::site;

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Rules {
    /// Site de la page → hotes (ou sites) bloques quand on est sur cette page.
    #[serde(default)]
    pub blocked: BTreeMap<String, BTreeSet<String>>,
    /// Sites isoles : aucune requete vers un autre site.
    #[serde(default)]
    pub strict: BTreeSet<String>,
}

impl Rules {
    pub fn load(path: &Path) -> Self {
        std::fs::read(path).ok().and_then(|raw| serde_json::from_slice(&raw).ok()).unwrap_or_default()
    }

    pub fn save(&self, path: &Path) -> anyhow::Result<()> {
        let tmp = path.with_extension("tmp");
        std::fs::write(&tmp, serde_json::to_vec_pretty(self)?)?;
        std::fs::rename(&tmp, path)?;
        Ok(())
    }

    /// La raison de bloquer `url` emise par la page `page`, si une regle s'applique.
    pub fn verdict(&self, url: &str, page: &str) -> Option<&'static str> {
        let page_site = site::site_of(page)?;
        let host = site::host_of(url)?.to_ascii_lowercase();
        if let Some(hosts) = self.blocked.get(&page_site)
            && (hosts.contains(&host) || hosts.contains(&site::site_of_host(&host)))
        {
            return Some("regle");
        }
        (self.strict.contains(&page_site) && site::is_third_party(url, page)).then_some("isolement")
    }

    /// Bloque (ou debloque) `host` sur le site de `page`.
    pub fn set_blocked(&mut self, page: &str, host: &str, on: bool) {
        let Some(page_site) = site::site_of(page) else { return };
        let hosts = self.blocked.entry(page_site.clone()).or_default();
        if on {
            hosts.insert(host.to_ascii_lowercase());
        } else {
            hosts.remove(&host.to_ascii_lowercase());
            if hosts.is_empty() {
                self.blocked.remove(&page_site);
            }
        }
    }

    pub fn set_strict(&mut self, page: &str, on: bool) {
        let Some(page_site) = site::site_of(page) else { return };
        if on {
            self.strict.insert(page_site);
        } else {
            self.strict.remove(&page_site);
        }
    }

    pub fn blocked_on(&self, page: &str) -> Vec<String> {
        site::site_of(page).and_then(|s| self.blocked.get(&s)).map(|h| h.iter().cloned().collect()).unwrap_or_default()
    }

    pub fn is_strict(&self, page: &str) -> bool {
        site::site_of(page).is_some_and(|s| self.strict.contains(&s))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn domaine_bloque_sur_un_site_seulement() {
        let mut rules = Rules::default();
        rules.set_blocked("https://www.example.com/", "cdn.tracker.net", true);
        assert_eq!(rules.verdict("https://cdn.tracker.net/a.js", "https://example.com/x"), Some("regle"));
        assert_eq!(rules.verdict("https://cdn.tracker.net/a.js", "https://autre.fr/"), None);
        rules.set_blocked("https://www.example.com/", "cdn.tracker.net", false);
        assert!(rules.blocked.is_empty());
    }

    #[test]
    fn isolement_strict_bloque_les_tiers() {
        let mut rules = Rules::default();
        rules.set_strict("https://www.example.com/", true);
        assert_eq!(rules.verdict("https://fonts.googleapis.com/x", "https://www.example.com/"), Some("isolement"));
        assert_eq!(rules.verdict("https://static.example.com/x", "https://www.example.com/"), None);
    }
}
