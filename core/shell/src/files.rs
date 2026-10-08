//! Responsabilite : ouvrir des fichiers et des dossiers locaux dans le navigateur — Ctrl+O (dialogue du systeme,
//! plusieurs fichiers), et les chemins tapes dans la barre d'adresse (`/…`, `~/…`) changes en adresses `file://`.

use tracing::info;

/// Adresse `file://` d'un chemin absolu ou relatif au dossier personnel (`~`), ou `None` si ce n'en est pas un.
pub fn url_of_path(input: &str) -> Option<String> {
    let home = std::env::var("HOME").unwrap_or_default();
    let path = if input == "~" {
        home
    } else if let Some(rest) = input.strip_prefix("~/") {
        format!("{home}/{rest}")
    } else if input.starts_with('/') {
        input.to_string()
    } else {
        return None;
    };
    Some(format!("file://{}", encode_path(&path)))
}

/// Encode un chemin pour une adresse : tout sauf lettres, chiffres, `/ - _ . ~` est mis en %XX (UTF-8).
fn encode_path(path: &str) -> String {
    path.bytes()
        .map(|b| match b {
            b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'/' | b'-' | b'_' | b'.' | b'~' => (b as char).to_string(),
            _ => format!("%{b:02X}"),
        })
        .collect()
}

/// Ctrl+O : selecteur de fichiers du systeme (portail du bureau, donc le selecteur choisi par l'utilisateur), dans un
/// fil a part ; chaque fichier choisi s'ouvre dans un nouvel onglet. Le dialogue integre de CEF faisait tomber le
/// navigateur (mesure le 08/10).
pub fn open_dialog() {
    info!("dialogue d'ouverture demande");
    let home = std::env::var("HOME").unwrap_or_default();
    std::thread::spawn(move || {
        let chosen = rfd::FileDialog::new().set_title("Ouvrir un fichier").set_directory(&home).pick_files();
        let urls: Vec<String> = chosen
            .unwrap_or_default()
            .iter()
            .filter_map(|path| url_of_path(&path.to_string_lossy()))
            .collect();
        info!(fichiers = urls.len(), "fichiers choisis");
        crate::containers::later(move || {
            for url in &urls {
                crate::bridge::open_tab(url);
            }
            crate::bridge::publish_tabs();
        });
    });
}

#[cfg(test)]
mod tests {
    use super::url_of_path;

    #[test]
    fn chemins_vers_adresses() {
        assert_eq!(url_of_path("/tmp/a b.pdf").as_deref(), Some("file:///tmp/a%20b.pdf"));
        assert!(url_of_path("~/Téléchargements").unwrap().starts_with("file:///"));
        assert!(url_of_path("~/Téléchargements").unwrap().ends_with("/T%C3%A9l%C3%A9chargements"));
        assert_eq!(url_of_path("exemple.fr"), None);
    }
}
