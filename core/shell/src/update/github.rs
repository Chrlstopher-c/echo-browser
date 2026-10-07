//! La derniere release publiee : version, archive Linux et son empreinte. API publique de GitHub, sans cle ; un faux
//! serveur peut la remplacer pour les essais (`ECHO_UPDATE_URL`, meme forme de reponse).

use serde_json::Value;

const ARCHIVE_SUFFIX: &str = "-linux-x64.tar.xz";
const MAX_JSON_BYTES: u64 = 2 * 1024 * 1024;

/// Ce qu'il faut pour installer une release.
#[derive(Debug, Clone)]
pub struct Release {
    pub version: String,
    pub archive_url: String,
    /// Empreinte SHA-256 donnee par GitHub (`digest`), quand il la fournit.
    pub digest: Option<String>,
    /// Adresse du fichier `.sha256` publie avec l'archive.
    pub sha_url: Option<String>,
}

pub fn latest_url(repo: &str) -> String {
    std::env::var("ECHO_UPDATE_URL").unwrap_or_else(|_| format!("https://api.github.com/repos/{repo}/releases/latest"))
}

pub fn latest(repo: &str) -> Result<Release, String> {
    let response = ureq::get(&latest_url(repo))
        .header("Accept", "application/vnd.github+json")
        .header("User-Agent", "echo-browser")
        .call()
        .map_err(|e| format!("verification : {e}"))?;
    let body = response.into_body().with_config().limit(MAX_JSON_BYTES).read_to_vec().map_err(|e| e.to_string())?;
    let json: Value = serde_json::from_slice(&body).map_err(|e| format!("reponse illisible : {e}"))?;
    parse(&json)
}

fn parse(json: &Value) -> Result<Release, String> {
    let tag = json["tag_name"].as_str().ok_or("release sans version")?;
    let assets = json["assets"].as_array().ok_or("release sans fichiers")?;
    let archive = assets
        .iter()
        .find(|a| a["name"].as_str().is_some_and(|n| n.ends_with(ARCHIVE_SUFFIX)))
        .ok_or("release sans archive Linux")?;
    let sha_name = format!("{}.sha256", archive["name"].as_str().unwrap_or_default());
    let sha_url = assets
        .iter()
        .find(|a| a["name"].as_str() == Some(sha_name.as_str()))
        .and_then(|a| a["browser_download_url"].as_str())
        .map(str::to_string);
    Ok(Release {
        version: tag.trim_start_matches('v').to_string(),
        archive_url: archive["browser_download_url"].as_str().ok_or("archive sans adresse")?.to_string(),
        digest: archive["digest"].as_str().map(|d| d.trim_start_matches("sha256:").to_lowercase()),
        sha_url,
    })
}

/// Vrai si `candidate` est une version plus recente que `current` (a.b.c, numerique).
pub fn is_newer(candidate: &str, current: &str) -> bool {
    let parts = |v: &str| -> Vec<u64> { v.split('.').map(|p| p.parse().unwrap_or(0)).collect() };
    parts(candidate) > parts(current)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compare_les_versions_numeriquement() {
        assert!(is_newer("0.4.2", "0.4.1"));
        assert!(is_newer("0.10.0", "0.9.9"));
        assert!(!is_newer("0.4.1", "0.4.1"));
        assert!(!is_newer("0.3.9", "0.4.0"));
    }

    #[test]
    fn lit_une_release_github() {
        let json = serde_json::json!({
            "tag_name": "v0.4.2",
            "assets": [
                {"name": "echo-browser-0.4.2-linux-x64.tar.xz", "browser_download_url": "https://x/a.tar.xz",
                 "digest": "sha256:ABCD"},
                {"name": "echo-browser-0.4.2-linux-x64.tar.xz.sha256", "browser_download_url": "https://x/a.sha256"}
            ]
        });
        let release = parse(&json).unwrap();
        assert_eq!(release.version, "0.4.2");
        assert_eq!(release.digest.as_deref(), Some("abcd"));
        assert_eq!(release.sha_url.as_deref(), Some("https://x/a.sha256"));
    }
}
