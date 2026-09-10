//! Responsabilite : quelles listes de filtres le navigateur souscrit, ou les telecharger, comment les rafraichir.

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};
use tracing::{info, warn};

/// Duree au-dela de laquelle une liste locale est consideree perimee.
pub const REFRESH_AFTER: Duration = Duration::from_secs(24 * 3600);

/// Une souscription a une liste de filtres.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Subscription {
    /// Identifiant stable, sert aussi de nom de fichier.
    pub id: String,
    /// Nom lisible, affiche dans les reglages.
    pub title: String,
    pub url: String,
    /// Une liste desactivee reste telechargee mais n'alimente pas le moteur.
    pub enabled: bool,
}

impl Subscription {
    fn new(id: &str, title: &str, url: &str, enabled: bool) -> Self {
        Self { id: id.into(), title: title.into(), url: url.into(), enabled }
    }
}

/// Les souscriptions actives par defaut — celles d'uBlock Origin, plus la liste francaise.
pub fn default_subscriptions() -> Vec<Subscription> {
    vec![
        Subscription::new("easylist", "EasyList — publicites",
            "https://easylist.to/easylist/easylist.txt", true),
        Subscription::new("easyprivacy", "EasyPrivacy — traqueurs",
            "https://easylist.to/easylist/easyprivacy.txt", true),
        Subscription::new("ubo-filters", "uBlock Origin — filtres",
            "https://raw.githubusercontent.com/uBlockOrigin/uAssets/master/filters/filters.txt", true),
        Subscription::new("ubo-privacy", "uBlock Origin — vie privee",
            "https://raw.githubusercontent.com/uBlockOrigin/uAssets/master/filters/privacy.txt", true),
        Subscription::new("ubo-badware", "uBlock Origin — sites malveillants",
            "https://raw.githubusercontent.com/uBlockOrigin/uAssets/master/filters/badware.txt", true),
        Subscription::new("ubo-quick-fixes", "uBlock Origin — correctifs rapides",
            "https://raw.githubusercontent.com/uBlockOrigin/uAssets/master/filters/quick-fixes.txt", true),
        Subscription::new("ubo-unbreak", "uBlock Origin — anti-casse",
            "https://raw.githubusercontent.com/uBlockOrigin/uAssets/master/filters/unbreak.txt", true),
        Subscription::new("liste-fr", "Liste FR — publicites francophones",
            "https://easylist-downloads.adblockplus.org/liste_fr.txt", true),
        Subscription::new("peter-lowe", "Peter Lowe — regies et traqueurs",
            "https://pgl.yoyo.org/adservers/serverlist.php?hostformat=adblockplus&showintro=0&mimetype=plaintext", true),
        Subscription::new("easylist-cookie", "EasyList Cookie — bandeaux de consentement",
            "https://secure.fanboy.co.nz/fanboy-cookiemonster.txt", false),
        Subscription::new("fanboy-annoyances", "Fanboy — nuisances et surcouches",
            "https://secure.fanboy.co.nz/fanboy-annoyance.txt", false),
    ]
}

/// Emplacement local d'une liste.
pub fn list_path(dir: &Path, id: &str) -> PathBuf {
    dir.join(format!("{id}.txt"))
}

/// Vrai si la copie locale manque ou depasse [`REFRESH_AFTER`].
pub fn needs_refresh(path: &Path) -> bool {
    let Ok(meta) = std::fs::metadata(path) else {
        return true;
    };
    match meta.modified().and_then(|m| SystemTime::now().duration_since(m).map_err(std::io::Error::other)) {
        Ok(age) => age > REFRESH_AFTER,
        Err(err) => {
            warn!(?path, %err, "date de modification illisible, rafraichissement force");
            true
        }
    }
}

/// Telecharge une liste et l'ecrit sur disque. Renvoie le nombre d'octets ecrits.
pub fn fetch_list(sub: &Subscription, dir: &Path) -> anyhow::Result<usize> {
    let body = ureq::get(&sub.url)
        .config().timeout_global(Some(Duration::from_secs(30))).build()
        .call()
        .map_err(|err| anyhow::anyhow!("telechargement de {} echoue : {err}", sub.id))?
        .body_mut()
        .read_to_string()
        .map_err(|err| anyhow::anyhow!("lecture de {} echouee : {err}", sub.id))?;

    if body.len() < 512 {
        anyhow::bail!("liste {} suspecte : {} octets recus", sub.id, body.len());
    }
    std::fs::create_dir_all(dir)?;
    let path = list_path(dir, &sub.id);
    std::fs::write(&path, &body)?;
    info!(id = %sub.id, octets = body.len(), "liste rafraichie");
    Ok(body.len())
}
