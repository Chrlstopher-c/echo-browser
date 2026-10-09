//! Responsabilite : les demandes qui touchent la bibliotheque, et ce qu'elle renvoie.

use crate::session;
use echo_contract::{BookmarkView, CoreEvent, DownloadState, DownloadView, HistoryEntryView};
use echo_contract::{SettingValue, SettingView};
use echo_library::{bookmarks, downloads, history, settings};
use tracing::warn;

/// Diffuse les favoris.
pub fn publish_bookmarks() {
    // Les favoris du profil affiche seulement : un profil est une identite.
    let Some(list) = session::with(|s| bookmarks::list_in(&s.library, &s.tabs.space())) else { return };
    let bookmarks = list
        .into_iter()
        .map(|entry| BookmarkView {
            url: entry.url,
            title: entry.title,
            favicon: entry.favicon,
            added_at: entry.added_at,
        })
        .collect();
    super::publish(&CoreEvent::BookmarksChanged { bookmarks });
}

/// Diffuse les permissions retenues.
pub fn publish_permissions() {
    let Some(list) = session::with(|s| echo_library::permissions::list(&s.library)) else { return };
    let grants = list
        .into_iter()
        .map(|(origin, kind, allow)| echo_contract::PermissionGrantView { origin, kind, allow })
        .collect();
    super::publish(&CoreEvent::PermissionsChanged { grants });
}

/// Diffuse l'historique, filtre par `terms`.
pub fn publish_history(terms: &str) {
    let Some((found, total)) = session::with(|s| history::search(&s.library, terms, &s.tabs.space())) else {
        return;
    };
    let entries = found
        .into_iter()
        .map(|entry| HistoryEntryView {
            url: entry.url,
            title: entry.title,
            favicon: entry.favicon,
            visited_at: entry.visited_at,
            visits: entry.visits,
        })
        .collect();
    super::publish(&CoreEvent::HistoryChanged { entries, total });
}

/// Diffuse les telechargements.
pub fn publish_downloads() {
    let Some(list) = session::with(|s| downloads::list(&s.library)) else { return };
    let downloads = list.into_iter().map(to_view).collect();
    super::publish(&CoreEvent::DownloadsChanged { downloads });
}

fn to_view(entry: downloads::Download) -> DownloadView {
    DownloadView {
        id: entry.id,
        file_name: entry.file_name,
        url: entry.url,
        path: entry.path,
        received: entry.received,
        total: entry.total,
        state: match entry.state {
            downloads::State::Running => DownloadState::Running,
            downloads::State::Paused => DownloadState::Paused,
            downloads::State::Complete => DownloadState::Complete,
            downloads::State::Cancelled => DownloadState::Cancelled,
            downloads::State::Failed => DownloadState::Failed,
        },
        started_at: entry.started_at,
    }
}

/// Diffuse les reglages.
pub fn publish_settings() {
    let Some(all) = session::with(|s| settings::all(&s.library)) else { return };
    let mut settings: Vec<SettingView> =
        all.into_iter().map(|(key, value)| SettingView { key, value: to_contract(value) }).collect();
    // Lecture seule (jamais enregistree ni synchronisee) : les profils d'avant la liste `profiles.list` qui ont deja
    // des donnees. Une installation neuve n'en a aucun et ne montre qu'un profil.
    settings.push(SettingView {
        key: "profiles.legacy".to_string(),
        value: SettingValue::Text(crate::profiles::legacy_with_data().join(",")),
    });
    settings.push(SettingView {
        key: "system.claudeCode".to_string(),
        value: SettingValue::Flag(crate::terminal::available()),
    });
    super::publish(&CoreEvent::SettingsChanged { settings });
}

fn to_contract(value: settings::Value) -> SettingValue {
    match value {
        settings::Value::Flag(on) => SettingValue::Flag(on),
        settings::Value::Text(text) => SettingValue::Text(text),
        settings::Value::Number(number) => SettingValue::Number(number),
    }
}

fn from_contract(value: &SettingValue) -> settings::Value {
    match value {
        SettingValue::Flag(on) => settings::Value::Flag(*on),
        SettingValue::Text(text) => settings::Value::Text(text.clone()),
        SettingValue::Number(number) => settings::Value::Number(*number),
    }
}

/// Met en favori la page de l'onglet donne.
/// Une page du web ou un fichier local, pas une page interne.
pub fn is_web(url: &str) -> bool {
    ["http://", "https://", "file://"].iter().any(|scheme| url.starts_with(scheme))
}

pub fn add_bookmark(id: echo_contract::TabId) {
    let entry = session::with(|s| {
        let tab = s.tabs.get_mut(id)?;
        Some((tab.url.clone(), tab.title.clone(), crate::tabs::Tabs::space_of_tab(tab)))
    })
    .flatten();
    let Some((url, title, space)) = entry else { return };
    // Les pages d'Echo (reglages, bibliotheque, aide) ne sont pas des sites : rien a mettre en favori.
    if url.is_empty() || !is_web(&url) {
        return;
    }
    session::with(|s| bookmarks::add(&s.library, &url, &title, None, &space));
    publish_bookmarks();
    super::publish(&CoreEvent::Notice {
        level: echo_contract::NoticeLevel::Info,
        message: format!("Ajouté aux favoris : {title}"),
        actions: vec![
            echo_contract::NoticeAction { label: "Retirer".into(), request: echo_contract::UiRequest::RemoveBookmark { url } },
            echo_contract::NoticeAction {
                label: "Voir les favoris".into(),
                request: echo_contract::UiRequest::OpenPage { page: "bibliotheque".into() },
            },
        ],
    });
}

pub fn remove_bookmark(url: &str) {
    session::with(|s| bookmarks::remove(&s.library, url));
    publish_bookmarks();
}

pub fn move_bookmark(url: &str, to: usize) {
    session::with(|s| bookmarks::move_to(&s.library, url, to, &s.tabs.space()));
    publish_bookmarks();
}

pub fn remove_history_entry(url: &str, visited_at: i64) {
    session::with(|s| history::remove(&s.library, url, visited_at));
    publish_history("");
}

pub fn clear_history() {
    session::with(|s| history::clear(&s.library));
    publish_history("");
}

/// Pages dont le chargement a echoue (certificat, reseau) : leur page d'erreur n'est pas une visite.
static FAILED: parking_lot::Mutex<Vec<String>> = parking_lot::Mutex::new(Vec::new());

pub fn note_failed_load(url: &str) {
    let mut failed = FAILED.lock();
    failed.push(url.to_string());
    if failed.len() > 20 {
        failed.remove(0);
    }
}

/// Enregistre une visite. Appele a chaque page arrivee a son terme.
pub fn record_visit(url: &str, title: &str, space: &str) {
    let failed = {
        let mut list = FAILED.lock();
        list.iter().position(|u| u == url).map(|at| list.remove(at)).is_some()
    };
    if failed || !is_web(url) {
        return;
    }
    if session::with(|s| history::record(&s.library, url, title, None, space)) == Some(true) {
        crate::account::schedule::touch_soft();
        crate::routines::visited(url);
        crate::signals::note_visit(url);
    }
}

/// Applique un reglage, ou previent l'interface s'il est refuse.
pub fn update_setting(key: &str, value: &SettingValue) {
    let outcome = session::with(|s| settings::set(&s.library, key, &from_contract(value)));
    match outcome {
        Some(Err(reason)) => {
            warn!(%reason, "reglage refuse");
            super::publish(&CoreEvent::notice(echo_contract::NoticeLevel::Error, reason));
        }
        _ => {
            // Le meme interrupteur existe dans le bouclier et dans les reglages : les deux doivent agir.
            if let (Some(Ok(())), "shield.enabled", SettingValue::Flag(on)) = (&outcome, key, value) {
                session::with(|s| s.shield.set_enabled(*on));
                super::publish_shield();
            }
            publish_settings();
        }
    }
}

/// Ouvre le fichier telecharge, ou le dossier qui le contient.
pub fn open_download(id: echo_contract::DownloadId, reveal: bool) {
    let path = session::with(|s| {
        downloads::list(&s.library).into_iter().find(|entry| entry.id == id).and_then(|e| e.path)
    })
    .flatten();
    let Some(path) = path else {
        warn!(id, "telechargement sans fichier sur disque");
        return;
    };
    let target = if reveal {
        std::path::Path::new(&path).parent().map(|p| p.to_string_lossy().to_string())
    } else {
        Some(path)
    };
    let Some(target) = target else { return };
    // `xdg-open` est le point d'entree standard du bureau : c'est lui qui sait quelle
    // application ouvre quel type de fichier.
    match std::process::Command::new("xdg-open").arg(&target).spawn() {
        Ok(_) => (),
        Err(err) => warn!(%err, %target, "ouverture impossible"),
    }
}

pub fn forget_download(id: echo_contract::DownloadId) {
    session::with(|s| downloads::forget(&s.library, id));
    publish_downloads();
}
