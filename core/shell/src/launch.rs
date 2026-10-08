//! Responsabilite : ce que l'on demande a Echo en le lancant — `echo-browser page.html dossier/ https://…` (gestionnaire
//! de fichiers, « Ouvrir avec Echo »). Si Echo tourne deja, les pages lui sont confiees par sa prise de pilotage et ce
//! lancement s'arrete ; sinon elles s'ouvrent au demarrage.

use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixStream;
use std::sync::OnceLock;

static PENDING: OnceLock<Vec<String>> = OnceLock::new();

/// L'adresse d'un argument : une adresse telle quelle, ou un chemin (absolu, `~`, ou relatif au dossier courant).
fn target_of(arg: &str) -> Option<String> {
    if arg.contains("://") || arg.starts_with("about:") {
        return Some(arg.to_string());
    }
    if let Some(url) = crate::files::url_of_path(arg) {
        return Some(url);
    }
    let absolute = std::env::current_dir().ok()?.join(arg);
    absolute.exists().then(|| crate::files::url_of_path(&absolute.to_string_lossy())).flatten()
}

/// Les pages demandees sur la ligne de commande (les options `--…` sont pour Chromium).
pub fn targets() -> Vec<String> {
    std::env::args().skip(1).filter(|a| !a.starts_with('-')).filter_map(|a| target_of(&a)).collect()
}

/// Confie les pages a l'Echo deja lance ; vrai s'il les a prises.
pub fn forward_to_running(targets: &[String]) -> bool {
    let Some(path) = crate::control::socket_path() else { return false };
    let Ok(mut stream) = UnixStream::connect(path) else { return false };
    let Ok(reader) = stream.try_clone() else { return false };
    let mut lines = BufReader::new(reader).lines();
    targets.iter().all(|url| {
        let request = serde_json::json!({ "op": "open", "url": url }).to_string() + "\n";
        stream.write_all(request.as_bytes()).is_ok() && lines.next().is_some_and(|line| line.is_ok())
    })
}

/// Retient les pages a ouvrir au demarrage (Echo n'etait pas lance).
pub fn keep(targets: Vec<String>) {
    let _ = PENDING.set(targets);
}

/// La premiere page a ouvrir, et les suivantes.
pub fn pending() -> (Option<String>, Vec<String>) {
    let all = PENDING.get().cloned().unwrap_or_default();
    let mut iter = all.into_iter();
    (iter.next(), iter.collect())
}

#[cfg(test)]
mod tests {
    use super::target_of;

    #[test]
    fn arguments_vers_adresses() {
        assert_eq!(target_of("https://exemple.fr").as_deref(), Some("https://exemple.fr"));
        assert_eq!(target_of("/tmp").as_deref(), Some("file:///tmp"));
        assert_eq!(target_of("fichier-qui-n-existe-pas.txt"), None);
    }
}
