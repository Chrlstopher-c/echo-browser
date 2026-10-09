//! Responsabilite : les suggestions de la page « nouvel onglet » — onglets ouverts, favoris et
//! historique correspondant a ce que l'utilisateur tape.

use serde_json::{json, Value};

const PER_SECTION: usize = 6;
/// Tuiles du nouvel onglet.
const TOP_SITES: usize = 8;

fn matches(text: &str, needle: &str) -> bool {
    needle.is_empty() || text.to_lowercase().contains(needle)
}

/// A appeler sur le thread interface.
pub fn suggest(query: &str) -> Value {
    let needle = query.trim().to_lowercase();
    let found = crate::session::with(|s| {
        // Seulement le profil affiche : ses onglets, ses favoris, son historique.
        let space = s.tabs.space();
        let tabs: Vec<Value> = s
            .tabs
            .snapshot()
            .iter()
            .filter(|t| t.space == space || (t.space.is_empty() && space == crate::profiles::DEFAULT))
            .filter(|t| !t.url.starts_with("echo://") && (matches(&t.title, &needle) || matches(&t.url, &needle)))
            .take(PER_SECTION)
            .map(|t| json!({"id": t.id, "title": t.title, "url": t.url}))
            .collect();
        let bookmarks: Vec<Value> = echo_library::bookmarks::list_in(&s.library, &space)
            .into_iter()
            .filter(|b| matches(&b.title, &needle) || matches(&b.url, &needle))
            .take(PER_SECTION)
            .map(|b| json!({"title": b.title, "url": b.url}))
            .collect();
        let mut seen = std::collections::HashSet::new();
        let history: Vec<Value> = echo_library::history::search(&s.library, query, &space)
            .0
            .into_iter()
            .filter(|e| seen.insert(e.url.trim_end_matches('/').to_string()))
            .take(PER_SECTION)
            .map(|e| json!({"title": e.title, "url": e.url}))
            .collect();
        let top = top_sites(&s.library, &space);
        json!({"ok": true, "tabs": tabs, "bookmarks": bookmarks, "history": history, "top": top})
    });
    found.unwrap_or_else(|| json!({"ok": false, "error": "navigateur occupe"}))
}

/// Les suggestions de la barre d'adresse, a plat (onglets, favoris, historique), sans doublon d'adresse.
pub fn for_address(query: &str) -> Vec<echo_contract::SuggestionView> {
    let found = suggest(query);
    let mut seen = std::collections::HashSet::new();
    let mut items = search_row(query).into_iter().collect::<Vec<_>>();
    for (section, kind) in [("tabs", "tab"), ("bookmarks", "bookmark"), ("history", "history")] {
        for entry in found[section].as_array().into_iter().flatten() {
            let url = entry["url"].as_str().unwrap_or_default().to_string();
            if url.is_empty() || !seen.insert(url.trim_end_matches('/').to_string()) {
                continue;
            }
            items.push(echo_contract::SuggestionView {
                kind: kind.to_string(),
                title: entry["title"].as_str().unwrap_or_default().to_string(),
                url,
                tab: entry["id"].as_u64().and_then(|id| u32::try_from(id).ok()),
            });
        }
    }
    items
}

/// Une ligne de recherche : `kind` « search » pour les termes tapes, « query » pour une suggestion du moteur.
fn search_item(kind: &str, terms: &str) -> echo_contract::SuggestionView {
    echo_contract::SuggestionView {
        kind: kind.to_string(),
        title: terms.to_string(),
        url: crate::search::query_url(terms),
        tab: None,
    }
}

/// « Rechercher … sur le moteur » en tete, sauf si la saisie est deja une adresse.
fn search_row(query: &str) -> Option<echo_contract::SuggestionView> {
    let terms = query.trim();
    let is_address = crate::bridge::normalize(terms) != crate::search::query_url(terms);
    (!terms.is_empty() && !is_address).then(|| search_item("search", terms))
}

/// Suggestions du moteur, si le reglage les autorise : demandees hors du thread interface, puis la liste complete
/// est republiee (la barre ignore une reponse arrivee apres une nouvelle frappe).
pub fn fetch_engine(query: String) {
    use echo_library::settings::Value;
    let allowed = crate::session::with(|s| echo_library::settings::get(&s.library, "search.suggest")).flatten();
    if matches!(allowed, Some(Value::Flag(false))) || search_row(&query).is_none() || active_is_private() {
        return;
    }
    let url = crate::search::suggest_url(query.trim());
    std::thread::spawn(move || {
        let agent: ureq::Agent = ureq::Agent::config_builder()
            .timeout_global(Some(std::time::Duration::from_millis(1500)))
            .build()
            .into();
        let body = match agent.get(&url).call().and_then(|mut r| r.body_mut().read_to_string()) {
            Ok(body) => body,
            Err(error) => {
                tracing::debug!(%error, "suggestions du moteur indisponibles");
                return;
            }
        };
        let found = crate::search::parse_suggestions(&body);
        if found.is_empty() {
            return;
        }
        crate::containers::later(move || {
            let mut items = for_address(&query);
            let typed = query.trim().to_lowercase();
            let extra = found.iter().filter(|t| t.to_lowercase() != typed).take(4).map(|t| search_item("query", t));
            let at = usize::from(items.first().is_some_and(|i| i.kind == "search"));
            items.splice(at..at, extra);
            crate::bridge::publish(&echo_contract::CoreEvent::Suggestions { query, items });
        });
    });
}

/// Sites les plus frequentes du profil, un par hote : les tuiles du nouvel onglet.
fn top_sites(library: &echo_library::Library, space: &str) -> Vec<Value> {
    let mut frequent = echo_library::history::search(library, "", space).0;
    frequent.sort_by(|a, b| b.visits.cmp(&a.visits));
    let mut hosts = std::collections::HashSet::new();
    frequent
        .into_iter()
        .filter(|e| hosts.insert(e.url.split('/').nth(2).unwrap_or_default().to_string()))
        .take(TOP_SITES)
        .map(|e| json!({"title": e.title, "url": e.url, "visits": e.visits}))
        .collect()
}

/// Le navigateur est-il un onglet de navigation privee ?
pub fn is_private_browser(browser_id: i32) -> bool {
    crate::session::with(|s| s.tabs.by_browser(browser_id).map(|t| crate::containers::is_private(t.container.as_deref())))
        .flatten()
        .unwrap_or(false)
}

/// Le nouvel onglet prive ne montre rien de l'historique ni des favoris (comme Chrome et Firefox).
pub fn private_page() -> Value {
    json!({"ok": true, "private": true, "tabs": [], "bookmarks": [], "history": [], "top": []})
}

/// L'onglet actif est-il prive ? Alors la frappe n'est pas envoyee au moteur.
fn active_is_private() -> bool {
    crate::session::with(|s| s.tabs.active().map(|t| crate::containers::is_private(t.container.as_deref())))
        .flatten()
        .unwrap_or(false)
}
