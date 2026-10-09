//! Responsabilite : lire les favoris et l'historique d'un autre navigateur installe (famille Chrome, famille
//! Firefox) pour les reprendre dans Echo. Lecture seule : les bases sont copiees avant d'etre ouvertes, le
//! navigateur d'origine peut tourner.

use rusqlite::{Connection, OpenFlags};
use std::path::{Path, PathBuf};

/// Un navigateur trouve sur la machine.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Source {
    pub id: String,
    pub name: String,
    kind: Kind,
    dir: PathBuf,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Kind {
    Chromium,
    Firefox,
}

/// Ce qu'une source apporte : (adresse, titre) pour les favoris, (adresse, titre, date en secondes) pour l'historique.
#[derive(Debug, Default)]
pub struct Harvest {
    pub bookmarks: Vec<(String, String)>,
    pub history: Vec<(String, String, i64)>,
}

/// Au plus tant de visites reprises : les plus recentes.
const HISTORY_LIMIT: usize = 5000;

const CHROMIUM_FAMILY: &[(&str, &str, &str)] = &[
    ("chrome", "Google Chrome", ".config/google-chrome"),
    ("chromium", "Chromium", ".config/chromium"),
    ("brave", "Brave", ".config/BraveSoftware/Brave-Browser"),
    ("edge", "Microsoft Edge", ".config/microsoft-edge"),
    ("vivaldi", "Vivaldi", ".config/vivaldi"),
];

const FIREFOX_FAMILY: &[(&str, &str, &str)] =
    &[("firefox", "Firefox", ".mozilla/firefox"), ("zen", "Zen", ".zen")];

/// Les navigateurs presents dans `home`, avec un profil lisible.
pub fn sources(home: &Path) -> Vec<Source> {
    let chromium = CHROMIUM_FAMILY.iter().filter_map(|(id, name, rel)| {
        let dir = home.join(rel).join("Default");
        (dir.join("Bookmarks").is_file() || dir.join("History").is_file())
            .then(|| Source { id: (*id).into(), name: (*name).into(), kind: Kind::Chromium, dir })
    });
    let firefox = FIREFOX_FAMILY.iter().filter_map(|(id, name, rel)| {
        let dir = firefox_profile(&home.join(rel))?;
        Some(Source { id: (*id).into(), name: (*name).into(), kind: Kind::Firefox, dir })
    });
    chromium.chain(firefox).collect()
}

/// Le profil le plus fourni (le plus gros places.sqlite) d'une installation Firefox.
fn firefox_profile(root: &Path) -> Option<PathBuf> {
    std::fs::read_dir(root)
        .ok()?
        .flatten()
        .map(|e| e.path())
        .filter_map(|p| std::fs::metadata(p.join("places.sqlite")).ok().map(|m| (m.len(), p)))
        .max_by_key(|(size, _)| *size)
        .map(|(_, p)| p)
}

/// Lit favoris et historique d'une source. `scratch` : dossier ou copier les bases avant lecture.
pub fn harvest(source: &Source, scratch: &Path) -> Harvest {
    match source.kind {
        Kind::Chromium => Harvest {
            bookmarks: chromium_bookmarks(&source.dir.join("Bookmarks")),
            history: copy_and_query(&source.dir.join("History"), scratch, CHROMIUM_HISTORY, chromium_time),
        },
        Kind::Firefox => {
            let places = source.dir.join("places.sqlite");
            let bookmarks = copy_and_query(&places, scratch, FIREFOX_BOOKMARKS, |_| 0)
                .into_iter()
                .map(|(url, title, _)| (url, title))
                .collect();
            Harvest { bookmarks, history: copy_and_query(&places, scratch, FIREFOX_HISTORY, firefox_time) }
        }
    }
}

const CHROMIUM_HISTORY: &str =
    "SELECT url, title, last_visit_time FROM urls WHERE last_visit_time > 0 ORDER BY last_visit_time DESC LIMIT ?1";
const FIREFOX_HISTORY: &str = "SELECT url, COALESCE(title, ''), last_visit_date FROM moz_places \
     WHERE last_visit_date IS NOT NULL ORDER BY last_visit_date DESC LIMIT ?1";
const FIREFOX_BOOKMARKS: &str = "SELECT p.url, COALESCE(b.title, ''), 0 FROM moz_bookmarks b \
     JOIN moz_places p ON b.fk = p.id WHERE b.type = 1 AND p.url LIKE 'http%' ORDER BY b.position LIMIT ?1";

/// Chromium compte en microsecondes depuis 1601.
fn chromium_time(raw: i64) -> i64 {
    raw / 1_000_000 - 11_644_473_600
}

/// Firefox compte en microsecondes depuis 1970.
fn firefox_time(raw: i64) -> i64 {
    raw / 1_000_000
}

/// Copie la base (et son journal) puis l'interroge : l'originale reste intacte et peut etre verrouillee.
fn copy_and_query(db: &Path, scratch: &Path, sql: &str, time: fn(i64) -> i64) -> Vec<(String, String, i64)> {
    let copy = scratch.join(format!("import-{}", db.file_name().map(|n| n.to_string_lossy()).unwrap_or_default()));
    if std::fs::copy(db, &copy).is_err() {
        return Vec::new();
    }
    let wal = PathBuf::from(format!("{}-wal", db.display()));
    if wal.is_file() {
        let _ = std::fs::copy(&wal, PathBuf::from(format!("{}-wal", copy.display())));
    }
    let rows = Connection::open_with_flags(&copy, OpenFlags::SQLITE_OPEN_READ_WRITE)
        .and_then(|conn| {
            let mut statement = conn.prepare(sql)?;
            let rows = statement.query_map([HISTORY_LIMIT as i64], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, String>(1).unwrap_or_default(), time(row.get(2)?)))
            })?;
            rows.collect::<rusqlite::Result<Vec<_>>>()
        })
        .unwrap_or_default();
    let _ = std::fs::remove_file(&copy);
    let _ = std::fs::remove_file(format!("{}-wal", copy.display()));
    rows
}

/// Les favoris Chromium : un JSON dont les dossiers (`roots`) contiennent des `children` de type `url`.
fn chromium_bookmarks(path: &Path) -> Vec<(String, String)> {
    let Ok(text) = std::fs::read_to_string(path) else { return Vec::new() };
    let Ok(json) = serde_json::from_str::<serde_json::Value>(&text) else { return Vec::new() };
    let mut found = Vec::new();
    if let Some(roots) = json.get("roots").and_then(|r| r.as_object()) {
        for root in roots.values() {
            collect(root, &mut found);
        }
    }
    found
}

fn collect(node: &serde_json::Value, out: &mut Vec<(String, String)>) {
    if node.get("type").and_then(|t| t.as_str()) == Some("url") {
        if let Some(url) = node.get("url").and_then(|u| u.as_str()).filter(|u| u.starts_with("http")) {
            let title = node.get("name").and_then(|n| n.as_str()).unwrap_or_default();
            out.push((url.to_string(), title.to_string()));
        }
    }
    for child in node.get("children").and_then(|c| c.as_array()).into_iter().flatten() {
        collect(child, out);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("echo-import-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn chrome_favoris_et_historique() {
        let home = temp("chrome");
        let profile = home.join(".config/google-chrome/Default");
        std::fs::create_dir_all(&profile).unwrap();
        std::fs::write(profile.join("Bookmarks"), r#"{"roots":{"bookmark_bar":{"children":[
            {"type":"url","name":"Exemple","url":"https://example.com/"},
            {"type":"folder","children":[{"type":"url","name":"Rust","url":"https://rust-lang.org/"}]},
            {"type":"url","name":"js","url":"javascript:alert(1)"}]}}}"#).unwrap();
        let db = Connection::open(profile.join("History")).unwrap();
        db.execute_batch("CREATE TABLE urls (url TEXT, title TEXT, last_visit_time INTEGER);
            INSERT INTO urls VALUES ('https://example.com/', 'Exemple', 13300000000000000);").unwrap();
        drop(db);
        let found = sources(&home);
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].name, "Google Chrome");
        let harvest = harvest(&found[0], &home);
        assert_eq!(harvest.bookmarks.len(), 2);
        assert_eq!(harvest.history.len(), 1);
        assert_eq!(harvest.history[0].2, 13_300_000_000 - 11_644_473_600);
        let _ = std::fs::remove_dir_all(&home);
    }

    #[test]
    fn firefox_favoris_et_historique() {
        let home = temp("firefox");
        let profile = home.join(".mozilla/firefox/abcd.default-release");
        std::fs::create_dir_all(&profile).unwrap();
        let db = Connection::open(profile.join("places.sqlite")).unwrap();
        db.execute_batch("CREATE TABLE moz_places (id INTEGER, url TEXT, title TEXT, last_visit_date INTEGER);
            CREATE TABLE moz_bookmarks (fk INTEGER, title TEXT, type INTEGER, position INTEGER);
            INSERT INTO moz_places VALUES (1, 'https://mozilla.org/', 'Mozilla', 1700000000000000);
            INSERT INTO moz_bookmarks VALUES (1, 'Mozilla', 1, 0);").unwrap();
        drop(db);
        let found = sources(&home);
        assert_eq!(found[0].id, "firefox");
        let harvest = harvest(&found[0], &home);
        assert_eq!(harvest.bookmarks, vec![("https://mozilla.org/".to_string(), "Mozilla".to_string())]);
        assert_eq!(harvest.history[0].2, 1_700_000_000);
        let _ = std::fs::remove_dir_all(&home);
    }
}
