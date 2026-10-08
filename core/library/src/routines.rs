//! Responsabilite : les routines — suites de sites que l'utilisateur ouvre souvent dans le meme ordre. Chaque suite
//! recente (2 a 4 sites distincts) est comptee par son empreinte ; une suite qui revient assez est proposee une fois,
//! et devient une routine si l'utilisateur l'accepte (elle rouvre ces pages d'un geste).

use rusqlite::{params, OptionalExtension};

use crate::{now, Library};

/// Une meme suite n'est comptee qu'une fois par fenetre (on ne compte pas un aller-retour dans la meme seance).
pub const DEFAULT_GAP_S: i64 = 600;

#[derive(Debug, Clone, PartialEq)]
pub struct Proposal {
    pub fingerprint: String,
    pub sites: Vec<String>,
    pub urls: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Routine {
    pub id: i64,
    pub name: String,
    pub urls: Vec<String>,
}

/// Seuil de proposition : une suite de 2 sites doit revenir plus souvent qu'une suite plus longue (plus banale).
fn threshold(len: usize) -> i64 {
    if len == 2 { 4 } else { 3 }
}

/// Compte les suites qui se terminent par la derniere visite de `recent` ((site, adresse), la plus ancienne d'abord)
/// et rend une proposition si l'une d'elles vient d'atteindre son seuil (la plus longue d'abord).
pub fn observe(library: &Library, recent: &[(String, String)], gap_s: i64) -> Option<Proposal> {
    let stamp = now();
    let mut ready = None;
    let mut covered = false;
    for len in (2..=4usize).rev() {
        let Some(tail) = recent.len().checked_sub(len).map(|start| &recent[start..]) else { continue };
        let sites: Vec<String> = tail.iter().map(|(s, _)| s.clone()).collect();
        if (1..sites.len()).any(|i| sites[..i].contains(&sites[i])) {
            continue;
        }
        let urls: Vec<String> = tail.iter().map(|(_, u)| u.clone()).collect();
        let fingerprint = sites.join(" > ");
        let (n, state) = count(library, &fingerprint, &urls, stamp, gap_s)?;
        // Une suite plus longue deja proposee (ou refusee, ou adoptee) couvre ses morceaux : pas de doublon.
        if state != "watch" {
            covered = true;
        }
        if ready.is_none() && !covered && n >= threshold(len) && mark_proposed(library, &fingerprint) {
            ready = Some(Proposal { fingerprint, sites, urls });
            covered = true;
        }
    }
    ready
}

fn count(library: &Library, fingerprint: &str, urls: &[String], stamp: i64, gap_s: i64) -> Option<(i64, String)> {
    let urls = serde_json::to_string(urls).ok()?;
    library.with(|db| {
        db.execute(
            "INSERT INTO sequences (fingerprint, urls, n, last, state) VALUES (?1, ?2, 1, ?3, 'watch')
             ON CONFLICT (fingerprint) DO UPDATE SET n = n + 1, last = ?3, urls = ?2 WHERE last <= ?3 - ?4",
            params![fingerprint, urls, stamp, gap_s],
        )?;
        db.query_row("SELECT n, state FROM sequences WHERE fingerprint = ?1", params![fingerprint], |row| {
            Ok((row.get(0)?, row.get(1)?))
        })
    })
}

fn mark_proposed(library: &Library, fingerprint: &str) -> bool {
    library
        .with(|db| {
            db.execute("UPDATE sequences SET state = 'proposed' WHERE fingerprint = ?1 AND state = 'watch'",
                params![fingerprint])
        })
        .is_some_and(|changed| changed > 0)
}

/// L'utilisateur ne veut pas de cette routine : elle ne sera plus proposee.
pub fn dismiss(library: &Library, fingerprint: &str) -> bool {
    library
        .with(|db| db.execute("UPDATE sequences SET state = 'dismissed' WHERE fingerprint = ?1", params![fingerprint]))
        .is_some()
}

/// Cree la routine d'une suite proposee.
pub fn adopt(library: &Library, fingerprint: &str, name: &str) -> Option<i64> {
    library.with(|db| {
        let urls: Option<String> = db
            .query_row("SELECT urls FROM sequences WHERE fingerprint = ?1", params![fingerprint], |row| row.get(0))
            .optional()?;
        let Some(urls) = urls else { return Ok(None) };
        db.execute("UPDATE sequences SET state = 'adopted' WHERE fingerprint = ?1", params![fingerprint])?;
        db.execute("INSERT INTO routines (name, urls, created) VALUES (?1, ?2, ?3)", params![name, urls, now()])?;
        Ok(Some(db.last_insert_rowid()))
    })?
}

pub fn list(library: &Library) -> Vec<Routine> {
    library
        .with(|db| {
            let mut statement = db.prepare("SELECT id, name, urls FROM routines ORDER BY created, id")?;
            let rows = statement.query_map([], |row| {
                let urls: String = row.get(2)?;
                Ok(Routine { id: row.get(0)?, name: row.get(1)?, urls: serde_json::from_str(&urls).unwrap_or_default() })
            })?;
            rows.collect()
        })
        .unwrap_or_default()
}

pub fn remove(library: &Library, id: i64) -> bool {
    library.with(|db| db.execute("DELETE FROM routines WHERE id = ?1", params![id])).is_some()
}
