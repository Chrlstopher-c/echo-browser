//! Responsabilite : recuperer une extension depuis le catalogue Chrome.
//!
//! On interroge le meme point de distribution que Chrome lui-meme, en HTTPS.

use std::time::Duration;
use tracing::info;

/// Version de Chromium annoncee au catalogue. A tenir alignee avec le moteur embarque,
/// sinon le catalogue peut refuser de servir certaines extensions.
const CHROME_VERSION: &str = "152.0.7977.54";

const ENDPOINT: &str = "https://clients2.google.com/service/update2/crx";

/// Un identifiant du catalogue : 32 lettres minuscules de a a p.
pub fn is_valid_id(id: &str) -> bool {
    id.len() == 32 && id.bytes().all(|b| (b'a'..=b'p').contains(&b))
}

/// Extrait l'identifiant d'une adresse du catalogue, ou le rend tel quel s'il en est deja un.
pub fn extract_id(input: &str) -> Option<String> {
    let candidate = input.trim();
    if is_valid_id(candidate) {
        return Some(candidate.to_string());
    }
    candidate
        .rsplit(['/', '?', '='])
        .map(str::trim)
        .find(|part| is_valid_id(part))
        .map(str::to_string)
}

/// Telecharge le paquet d'une extension. Renvoie ses octets bruts.
pub fn download(id: &str) -> anyhow::Result<Vec<u8>> {
    anyhow::ensure!(is_valid_id(id), "identifiant d'extension invalide : {id}");
    let url = format!(
        "{ENDPOINT}?response=redirect&os=linux&arch=x64&os_arch=x86_64&nacl_arch=x86-64\
         &prod=chromiumcrx&prodchannel=unknown&prodversion={CHROME_VERSION}\
         &acceptformat=crx2,crx3&x=id%3D{id}%26uc"
    );
    let mut response = ureq::get(&url)
        .config()
        .timeout_global(Some(Duration::from_secs(120)))
        .build()
        .call()?;
    let bytes = response.body_mut().with_config().limit(128 * 1024 * 1024).read_to_vec()?;
    anyhow::ensure!(
        bytes.len() > 1024,
        "paquet suspect pour {id} : {} octets recus",
        bytes.len()
    );
    info!(%id, octets = bytes.len(), "extension telechargee");
    Ok(bytes)
}
