//! Responsabilite : ouvrir un paquet d'extension Chrome et l'etaler sur le disque.
//!
//! Un paquet CRX est une entete signee suivie d'une archive ZIP ordinaire. On ne
//! verifie pas la signature : le paquet vient d'etre telecharge sur le catalogue
//! officiel, en HTTPS, et n'a pas transite ailleurs.

use std::io::Cursor;
use std::path::Path;

#[derive(Debug, thiserror::Error)]
pub enum CrxError {
    #[error("ce n'est pas un paquet d'extension")]
    NotACrx,
    #[error("version de paquet non geree : {0}")]
    UnsupportedVersion(u32),
    #[error("paquet tronque")]
    Truncated,
    #[error("archive illisible : {0}")]
    Archive(#[from] zip::result::ZipError),
    #[error("acces disque : {0}")]
    Io(#[from] std::io::Error),
}

const MAGIC: &[u8; 4] = b"Cr24";

/// Etale le contenu du paquet dans `target`, qui est vide ou recree.
/// Renvoie le nombre de fichiers ecrits.
pub fn unpack(package: &[u8], target: &Path) -> Result<usize, CrxError> {
    let archive_offset = zip_offset(package)?;
    if target.exists() {
        std::fs::remove_dir_all(target)?;
    }
    std::fs::create_dir_all(target)?;

    let mut archive = zip::ZipArchive::new(Cursor::new(&package[archive_offset..]))?;
    let mut written = 0usize;
    for index in 0..archive.len() {
        let mut entry = archive.by_index(index)?;
        // `enclosed_name` refuse les chemins qui sortiraient du dossier cible.
        let Some(relative) = entry.enclosed_name() else { continue };
        let path = target.join(relative);
        if entry.is_dir() {
            std::fs::create_dir_all(&path)?;
            continue;
        }
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let mut file = std::fs::File::create(&path)?;
        std::io::copy(&mut entry, &mut file)?;
        written += 1;
    }
    Ok(written)
}

/// Position de l'archive ZIP a l'interieur du paquet.
fn zip_offset(package: &[u8]) -> Result<usize, CrxError> {
    if package.len() < 12 || &package[..4] != MAGIC {
        return Err(CrxError::NotACrx);
    }
    let version = u32::from_le_bytes(package[4..8].try_into().map_err(|_| CrxError::Truncated)?);
    if version != 3 {
        return Err(CrxError::UnsupportedVersion(version));
    }
    let header_len =
        u32::from_le_bytes(package[8..12].try_into().map_err(|_| CrxError::Truncated)?) as usize;
    let offset = 12 + header_len;
    if offset >= package.len() {
        return Err(CrxError::Truncated);
    }
    Ok(offset)
}
