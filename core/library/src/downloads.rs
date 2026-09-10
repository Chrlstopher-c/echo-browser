//! Responsabilite : les telechargements, en cours et passes.

use crate::{now, Library};
use rusqlite::params;

pub type DownloadId = u32;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum State {
    Running,
    Paused,
    Complete,
    Cancelled,
    Failed,
}

impl State {
    fn encode(self) -> &'static str {
        match self {
            State::Running => "running",
            State::Paused => "paused",
            State::Complete => "complete",
            State::Cancelled => "cancelled",
            State::Failed => "failed",
        }
    }

    fn decode(raw: &str) -> Self {
        match raw {
            "running" => State::Running,
            "paused" => State::Paused,
            "complete" => State::Complete,
            "cancelled" => State::Cancelled,
            _ => State::Failed,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Download {
    pub id: DownloadId,
    pub file_name: String,
    pub url: String,
    pub path: Option<String>,
    pub received: u64,
    pub total: Option<u64>,
    pub state: State,
    pub started_at: i64,
}

/// Enregistre ou met a jour un telechargement.
pub fn upsert(library: &Library, download: &Download) -> bool {
    library
        .with(|db| {
            db.execute(
                "INSERT INTO downloads (id, file_name, url, path, received, total, state, started_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
                 ON CONFLICT(id) DO UPDATE SET
                    file_name = ?2, path = ?4, received = ?5, total = ?6, state = ?7",
                params![
                    download.id,
                    download.file_name,
                    download.url,
                    download.path,
                    download.received as i64,
                    download.total.map(|t| t as i64),
                    download.state.encode(),
                    download.started_at,
                ],
            )
        })
        .is_some()
}

/// Les telechargements, du plus recent au plus ancien.
pub fn list(library: &Library) -> Vec<Download> {
    library
        .with(|db| {
            let mut statement = db.prepare(
                "SELECT id, file_name, url, path, received, total, state, started_at
                 FROM downloads ORDER BY started_at DESC LIMIT 200",
            )?;
            let rows = statement.query_map([], |row| {
                Ok(Download {
                    id: row.get::<_, i64>(0)? as DownloadId,
                    file_name: row.get(1)?,
                    url: row.get(2)?,
                    path: row.get(3)?,
                    received: row.get::<_, i64>(4)? as u64,
                    total: row.get::<_, Option<i64>>(5)?.map(|t| t as u64),
                    state: State::decode(&row.get::<_, String>(6)?),
                    started_at: row.get(7)?,
                })
            })?;
            rows.collect()
        })
        .unwrap_or_default()
}

/// Retire une entree de la liste. Le fichier reste sur le disque.
pub fn forget(library: &Library, id: DownloadId) -> bool {
    library.with(|db| db.execute("DELETE FROM downloads WHERE id = ?1", params![id])).is_some()
}

/// Cree l'entree d'un telechargement qui demarre.
pub fn started(id: DownloadId, file_name: &str, url: &str, total: Option<u64>) -> Download {
    Download {
        id,
        file_name: file_name.to_string(),
        url: url.to_string(),
        path: None,
        received: 0,
        total,
        state: State::Running,
        started_at: now(),
    }
}
