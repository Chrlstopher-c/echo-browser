//! Responsabilite : le moteur de recherche et la page d'accueil.
//!
//! Rassembles ici pour qu'un changement soit une ligne, et qu'ils deviennent un
//! reglage le jour ou l'interface en proposera un.

/// Page ouverte a chaque nouvel onglet.
pub const HOME: &str = "echo://ui/nouvel-onglet.html";

/// Modele de recherche. `{q}` recoit les termes, deja encodes.
const QUERY_TEMPLATE: &str = "https://www.google.com/search?q={q}";

/// Construit l'adresse de recherche pour des termes saisis par l'utilisateur.
pub fn query_url(terms: &str) -> String {
    QUERY_TEMPLATE.replace("{q}", &encode(terms))
}

/// Encodage des termes pour une chaine de requete.
fn encode(value: &str) -> String {
    value
        .bytes()
        .map(|b| match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                (b as char).to_string()
            }
            b' ' => "+".to_string(),
            other => format!("%{other:02X}"),
        })
        .collect()
}
