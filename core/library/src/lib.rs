//! Bibliotheque — favoris, historique, telechargements et reglages persistes.
//!
//! Une seule base pour les quatre : ce sont les memes donnees du point de vue de
//! l'utilisateur, elles se sauvegardent et s'effacent ensemble.

pub mod bookmarks;
pub mod downloads;
pub mod hidden;
pub mod history;
pub mod journal;
pub mod permissions;
pub mod routines;
pub mod schema;
pub mod settings;

use parking_lot::Mutex;
use rusqlite::Connection;
use std::path::Path;
use tracing::warn;

/// Acces a la bibliotheque, partageable entre les threads du navigateur.
pub struct Library {
    connection: Mutex<Connection>,
}

impl Library {
    /// Ouvre la base, en la creant au besoin.
    pub fn open(data_dir: &Path) -> anyhow::Result<Self> {
        std::fs::create_dir_all(data_dir)?;
        let connection = Connection::open(data_dir.join("library.db"))?;
        schema::prepare(&connection)?;
        Ok(Self { connection: Mutex::new(connection) })
    }

    /// Une base en memoire, pour les essais.
    pub fn in_memory() -> anyhow::Result<Self> {
        let connection = Connection::open_in_memory()?;
        schema::prepare(&connection)?;
        Ok(Self { connection: Mutex::new(connection) })
    }

    /// Emprunte la connexion le temps d'une operation.
    pub fn with<R>(&self, action: impl FnOnce(&Connection) -> rusqlite::Result<R>) -> Option<R> {
        let guard = self.connection.lock();
        match action(&guard) {
            Ok(value) => Some(value),
            Err(err) => {
                warn!(%err, "operation refusee par la bibliotheque");
                None
            }
        }
    }
}

/// Horodatage en secondes, tel que stocke dans la base.
pub fn now() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}
