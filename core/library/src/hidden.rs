//! Responsabilite : les elements que l'utilisateur a masques, retenus par gabarit de page (empreinte de la structure,
//! pas de l'adresse) : un element masque sur un article l'est sur tous les articles du meme gabarit, du meme site.

use rusqlite::params;

use crate::{now, Library};

pub fn add(library: &Library, fingerprint: &str, selector: &str, site: &str) -> bool {
    library
        .with(|db| {
            db.execute(
                "INSERT OR IGNORE INTO hidden_elements (fingerprint, selector, site, created) VALUES (?1, ?2, ?3, ?4)",
                params![fingerprint, selector, site, now()],
            )
        })
        .is_some()
}

/// Selecteurs masques pour un gabarit, sur ce site seulement : une page d'un autre site qui annoncerait le meme
/// gabarit n'apprend rien de ce que l'utilisateur a masque ailleurs.
pub fn selectors(library: &Library, fingerprint: &str, site: &str) -> Vec<String> {
    library
        .with(|db| {
            let mut statement =
                db.prepare("SELECT selector FROM hidden_elements WHERE fingerprint = ?1 AND site = ?2")?;
            let rows = statement.query_map(params![fingerprint, site], |row| row.get(0))?;
            rows.collect()
        })
        .unwrap_or_default()
}

/// Reaffiche tout ce qui etait masque sur ce gabarit, sur ce site.
pub fn clear(library: &Library, fingerprint: &str, site: &str) -> bool {
    library
        .with(|db| {
            db.execute("DELETE FROM hidden_elements WHERE fingerprint = ?1 AND site = ?2", params![fingerprint, site])
        })
        .is_some()
}
