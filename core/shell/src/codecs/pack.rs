//! Le decodeur complet tiers : ou il vient, ou il se range, comment il est verifie. Echo ne le distribue pas (H.264 et
//! AAC sont brevetes) : l'utilisateur le telecharge lui-meme, et seul le fichier dont l'empreinte est epinglee est garde.

use std::io::Read;
use std::path::PathBuf;

use sha2::{Digest, Sha256};
use tracing::info;

/// Branche du moteur pour laquelle ce decodeur est construit : a changer avec chaque montee de version de CEF.
pub const CHROMIUM_MAJOR: &str = "154";
pub const SOURCE: &str = "nwjs-ffmpeg-prebuilt 0.117.0 (Chromium 154)";
const ZIP_URL: &str =
    "https://github.com/nwjs-ffmpeg-prebuilt/nwjs-ffmpeg-prebuilt/releases/download/0.117.0/0.117.0-linux-x64.zip";
const LIB_SHA256: &str = "2f8601d1aaed7e5d4c16d849e4f75009917bbdefbc6267bbf1cdd9edbd5a33e0";
const MAX_ZIP_BYTES: u64 = 32 * 1024 * 1024;
const LIB: &str = "libffmpeg.so";
const MARK: &str = "chromium";

pub fn dir() -> PathBuf {
    crate::flags::data_dir().join("codecs")
}

/// Le decodeur est la et construit pour ce moteur-ci.
pub fn installed() -> bool {
    let dir = dir();
    dir.join(LIB).is_file() && std::fs::read_to_string(dir.join(MARK)).is_ok_and(|v| v.trim() == CHROMIUM_MAJOR)
}

/// Telecharge, verifie et range le decodeur. Bloquant : a appeler hors du thread interface.
pub fn install() -> Result<(), String> {
    let zip = download()?;
    let lib = extract(&zip)?;
    let digest = format!("{:x}", Sha256::digest(&lib));
    if digest != LIB_SHA256 {
        return Err(format!("empreinte inattendue ({digest}) : fichier refuse"));
    }
    let dir = dir();
    std::fs::create_dir_all(&dir).map_err(|e| format!("dossier {} : {e}", dir.display()))?;
    // Ecrit a cote puis renomme : le fichier en place peut etre charge par le moteur, l'ecraser le ferait planter.
    let tmp = dir.join(format!("{LIB}.part"));
    std::fs::write(&tmp, &lib).map_err(|e| format!("ecriture : {e}"))?;
    std::fs::rename(&tmp, dir.join(LIB)).map_err(|e| format!("mise en place : {e}"))?;
    std::fs::write(dir.join(MARK), CHROMIUM_MAJOR).map_err(|e| format!("ecriture : {e}"))?;
    info!(source = SOURCE, "decodeur complet installe");
    Ok(())
}

pub fn remove() -> Result<(), String> {
    match std::fs::remove_dir_all(dir()) {
        Ok(()) => Ok(()),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(e) => Err(format!("suppression : {e}")),
    }
}

fn download() -> Result<Vec<u8>, String> {
    let response = ureq::get(ZIP_URL).call().map_err(|e| format!("telechargement : {e}"))?;
    response.into_body().with_config().limit(MAX_ZIP_BYTES).read_to_vec().map_err(|e| format!("telechargement : {e}"))
}

fn extract(zip: &[u8]) -> Result<Vec<u8>, String> {
    let mut archive = zip::ZipArchive::new(std::io::Cursor::new(zip)).map_err(|e| format!("archive : {e}"))?;
    let mut entry = archive.by_name(LIB).map_err(|e| format!("archive sans {LIB} : {e}"))?;
    let mut lib = Vec::with_capacity(usize::try_from(entry.size()).unwrap_or(0));
    entry.read_to_end(&mut lib).map_err(|e| format!("archive : {e}"))?;
    Ok(lib)
}
