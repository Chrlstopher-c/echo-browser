//! Responsabilite : le moteur de recherche (choisi dans les Reglages) et la page d'accueil.

/// Page ouverte a chaque nouvel onglet.
pub const HOME: &str = "echo://ui/nouvel-onglet.html";

/// Un moteur : identifiant du reglage `search.engine`, nom affiche, modele de recherche et de suggestions.
pub struct Engine {
    pub id: &'static str,
    pub name: &'static str,
    query: &'static str,
    suggest: &'static str,
}

/// Moteurs proposes. `{q}` recoit les termes encodes. Les suggestions repondent au format OpenSearch
/// (`["termes", ["suggestion", …]]`).
pub const ENGINES: &[Engine] = &[
    Engine { id: "google", name: "Google", query: "https://www.google.com/search?q={q}",
        suggest: "https://suggestqueries.google.com/complete/search?client=firefox&q={q}" },
    Engine { id: "duckduckgo", name: "DuckDuckGo", query: "https://duckduckgo.com/?q={q}",
        suggest: "https://duckduckgo.com/ac/?type=list&q={q}" },
    Engine { id: "qwant", name: "Qwant", query: "https://www.qwant.com/?q={q}",
        suggest: "https://api.qwant.com/api/suggest/?client=opensearch&q={q}" },
    Engine { id: "ecosia", name: "Ecosia", query: "https://www.ecosia.org/search?q={q}",
        suggest: "https://ac.ecosia.org/autocomplete?type=list&q={q}" },
    Engine { id: "bing", name: "Bing", query: "https://www.bing.com/search?q={q}",
        suggest: "https://api.bing.com/osjson.aspx?query={q}" },
    Engine { id: "startpage", name: "Startpage", query: "https://www.startpage.com/do/search?q={q}",
        suggest: "https://www.startpage.com/osuggestions?q={q}" },
    Engine { id: "brave", name: "Brave Search", query: "https://search.brave.com/search?q={q}",
        suggest: "https://search.brave.com/api/suggest?q={q}" },
];

/// Moteur choisi ; Google si le reglage est absent ou inconnu (anciennes valeurs comprises).
pub fn engine() -> &'static Engine {
    use echo_library::settings::Value;
    let chosen = crate::session::with(|s| echo_library::settings::get(&s.library, "search.engine")).flatten();
    let id = match chosen {
        Some(Value::Text(id)) => id,
        _ => String::new(),
    };
    ENGINES.iter().find(|e| e.id == id).unwrap_or(&ENGINES[0])
}

/// Construit l'adresse de recherche pour des termes saisis par l'utilisateur.
pub fn query_url(terms: &str) -> String {
    engine().query.replace("{q}", &encode(terms))
}

/// Adresse des suggestions du moteur choisi.
pub fn suggest_url(terms: &str) -> String {
    engine().suggest.replace("{q}", &encode(terms))
}

/// Lit une reponse OpenSearch : `["termes", ["a", "b"]]`.
pub fn parse_suggestions(body: &str) -> Vec<String> {
    let Ok(serde_json::Value::Array(parts)) = serde_json::from_str::<serde_json::Value>(body) else {
        return Vec::new();
    };
    let Some(serde_json::Value::Array(items)) = parts.get(1) else { return Vec::new() };
    items.iter().filter_map(|v| v.as_str()).filter(|s| !s.is_empty()).take(6).map(str::to_string).collect()
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn suggestions_opensearch() {
        assert_eq!(parse_suggestions(r#"["met",["meteo","meteo paris"]]"#), vec!["meteo", "meteo paris"]);
        assert!(parse_suggestions("pas du json").is_empty());
    }

    #[test]
    fn moteur_par_defaut() {
        assert!(query_url("a b").starts_with("https://www.google.com/search?q=a+b"));
    }
}
