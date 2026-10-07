//! Preparer une release a cote de l'installation : telechargement, empreinte verifiee, extraction dans
//! `<installation>.maj`. Rien ne touche a l'installation en cours ; la bascule se fait au redemarrage (`apply.rs`).

use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use sha2::{Digest, Sha256};

use super::github::Release;

const MAX_ARCHIVE_BYTES: u64 = 600 * 1024 * 1024;

pub fn staged_dir(install: &Path) -> PathBuf {
    sibling(install, "maj")
}

pub fn sibling(install: &Path, suffix: &str) -> PathBuf {
    let name = install.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
    install.with_file_name(format!("{name}.{suffix}"))
}

/// Telecharge, verifie et extrait. Bloquant : hors du fil de l'interface.
pub fn prepare(install: &Path, release: &Release) -> Result<(), String> {
    let expected = expected_digest(release)?;
    let archive = sibling(install, "maj.tar.xz");
    let digest = download(&release.archive_url, &archive)?;
    if digest != expected {
        let _ = std::fs::remove_file(&archive);
        return Err(format!("empreinte inattendue ({digest}) : archive refusee"));
    }
    let result = extract(install, &archive, &release.version);
    let _ = std::fs::remove_file(&archive);
    result
}

/// L'empreinte attendue : celle de GitHub et celle du fichier `.sha256`, qui doivent concorder quand les deux existent.
fn expected_digest(release: &Release) -> Result<String, String> {
    let published = match &release.sha_url {
        Some(url) => Some(read_sha_file(url)?),
        None => None,
    };
    match (release.digest.clone(), published) {
        (Some(a), Some(b)) if a != b => Err("empreintes contradictoires : mise a jour refusee".into()),
        (Some(d), _) | (None, Some(d)) => Ok(d),
        (None, None) => Err("release sans empreinte : mise a jour refusee".into()),
    }
}

fn read_sha_file(url: &str) -> Result<String, String> {
    let response = ureq::get(url).header("User-Agent", "echo-browser").call().map_err(|e| format!("empreinte : {e}"))?;
    let text = response.into_body().with_config().limit(4096).read_to_string().map_err(|e| e.to_string())?;
    let digest = text.split_whitespace().next().unwrap_or_default().to_lowercase();
    if digest.len() != 64 || !digest.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err("fichier d'empreinte illisible".into());
    }
    Ok(digest)
}

/// Telecharge vers `target` en calculant l'empreinte au fil de l'eau. Renvoie l'empreinte.
fn download(url: &str, target: &Path) -> Result<String, String> {
    let response = ureq::get(url).header("User-Agent", "echo-browser").call().map_err(|e| format!("telechargement : {e}"))?;
    let mut body = response.into_body();
    let mut reader = body.with_config().limit(MAX_ARCHIVE_BYTES).reader();
    let mut file = std::fs::File::create(target).map_err(|e| format!("ecriture : {e}"))?;
    let mut hasher = Sha256::new();
    let mut buffer = vec![0u8; 256 * 1024];
    loop {
        let n = reader.read(&mut buffer).map_err(|e| format!("telechargement : {e}"))?;
        if n == 0 {
            break;
        }
        hasher.update(&buffer[..n]);
        file.write_all(&buffer[..n]).map_err(|e| format!("ecriture : {e}"))?;
    }
    Ok(format!("{:x}", hasher.finalize()))
}

/// Extrait l'archive (un dossier racine) et le range en `<installation>.maj`, verifie (binaire, version annoncee).
fn extract(install: &Path, archive: &Path, version: &str) -> Result<(), String> {
    let work = sibling(install, "maj.extraction");
    let _ = std::fs::remove_dir_all(&work);
    std::fs::create_dir_all(&work).map_err(|e| format!("dossier : {e}"))?;
    let status = std::process::Command::new("tar")
        .arg("-xJf")
        .arg(archive)
        .arg("-C")
        .arg(&work)
        .status()
        .map_err(|e| format!("extraction : {e}"))?;
    if !status.success() {
        let _ = std::fs::remove_dir_all(&work);
        return Err("extraction impossible".into());
    }
    let root = std::fs::read_dir(&work)
        .map_err(|e| e.to_string())?
        .flatten()
        .map(|e| e.path())
        .find(|p| p.is_dir())
        .ok_or("archive vide")?;
    let announced = super::read_release_version(&root);
    if !root.join("echo-browser").is_file() || announced.as_deref() != Some(version) {
        let _ = std::fs::remove_dir_all(&work);
        return Err(format!("archive incomplete ou de version inattendue ({announced:?})"));
    }
    let staged = staged_dir(install);
    let _ = std::fs::remove_dir_all(&staged);
    std::fs::rename(&root, &staged).map_err(|e| format!("mise en place : {e}"))?;
    let _ = std::fs::remove_dir_all(&work);
    Ok(())
}
