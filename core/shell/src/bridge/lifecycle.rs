//! Responsabilite : la vie des onglets — ouverture (dans un conteneur ou non), selection, veille, reveil,
//! purge memoire et fermeture. Chaque appel a Chromium se fait hors de l'acces a l'etat : la creation
//! d'une vue et son rattachement declenchent des rappels qui veulent lire cet etat.

use super::{dismiss_overlays, publish_shield, publish_tabs};
use cef::*;
use echo_contract::TabId;
use tracing::warn;

use crate::session;

pub fn open_tab(url: &str) {
    open_tab_in(url, None);
}

/// Ouvre un onglet dans le meme conteneur que l'onglet actif : un lien suit le compte de la page d'ou il vient.
pub fn open_tab_like_active(url: &str) {
    let container = session::with(|s| s.tabs.active().and_then(|tab| tab.container.clone())).flatten();
    open_tab_in(url, container.as_deref());
}

/// Ouvre un onglet dans le conteneur donne (`None` : contexte commun). Rend l'onglet, sauf si l'ouverture est
/// repoussee (profil du conteneur pas encore pret).
pub fn open_tab_in(url: &str, container: Option<&str>) -> Option<TabId> {
    // Sans conteneur explicite, l'onglet prend les comptes du profil affiche.
    let profile = session::with(|s| crate::profiles::container_for(&s.tabs.space())).flatten();
    let container = container.or(profile.as_deref());
    if wait_for_container(container, || {
        let (url, container) = (url.to_string(), container.map(str::to_string));
        move || {
            open_tab_in(&url, container.as_deref());
            publish_tabs();
        }
    }) {
        return None;
    }
    dismiss_overlays();
    let (mut client, host) = session::with(|s| (s.client.clone(), s.tabs.host()))?;
    let Some(view) = crate::window::create_view(client.as_mut(), url, 0, container) else {
        warn!(%url, "vue d'onglet non creee");
        return None;
    };
    if let Some(host) = host {
        let mut child = View::from(&view);
        host.add_child_view(Some(&mut child));
    }
    session::with(|s| {
        let id = s.tabs.adopt(view, url);
        if let Some(tab) = s.tabs.get_mut(id) {
            tab.container = container.map(str::to_string);
        }
        id
    })
}

/// Une page ouvre une fenetre (`window.open`, lien `target=_blank`) : l'adresse s'ouvre dans un onglet normal du
/// conteneur de la page, rattache a elle (retour a sa fermeture, connexion Google finie). La fenetre native n'est
/// pas creee : dans CEF, la pause des service workers d'extensions (voir `extension_tabs/workers.rs`) la figerait
/// sur about:blank. Revers assume (choix de Chris, 08/10) : la page n'a pas de `window.opener` vers qui ecrire.
/// Faux si la fenetre n'est pas pour nous (adresse vide, page qui n'est pas un onglet) : Chromium la cree alors.
pub fn page_opens_window(browser_id: i32, url: &str, background: bool) -> bool {
    let web = ["http://", "https://", "file://"].iter().any(|scheme| url.starts_with(scheme));
    let opener = session::with(|s| s.tabs.by_browser(browser_id).map(|t| (t.id, t.container.clone()))).flatten();
    let (true, Some((opener, container))) = (web, opener) else { return false };
    let url = url.to_string();
    crate::containers::later(move || {
        let Some(id) = open_tab_in(&url, container.as_deref()) else { return };
        session::with(|s| {
            if let Some(tab) = s.tabs.get_mut(id) {
                tab.opener = Some(opener);
            }
            s.tabs.place_after_opener(id, opener);
            s.tabs.refresh_visibility();
        });
        if background {
            publish_tabs();
        } else {
            select_tab(id);
        }
    });
    true
}

/// Tant que le profil du conteneur n'est pas pret, repousse `job` de quelques instants. Vrai si repousse.
fn wait_for_container<J: FnOnce() + Send + 'static>(container: Option<&str>, job: impl FnOnce() -> J) -> bool {
    let Some(id) = container else { return false };
    if crate::containers::is_ready(id) {
        return false;
    }
    crate::containers::later(job());
    true
}

/// Rouvre l'onglet dans un autre conteneur : sa page repart d'une session neuve, l'ancien onglet se ferme.
pub(super) fn move_to_container(id: TabId, container: Option<String>) {
    if wait_for_container(container.as_deref(), || {
        let container = container.clone();
        move || move_to_container(id, container)
    }) {
        return;
    }
    let url = session::with(|s| s.tabs.wake_url(id)).flatten();
    let Some(url) = url else { return };
    let (index, was_active) = session::with(|s| (s.tabs.index_of(id), s.tabs.active_id() == Some(id))).unwrap_or_default();
    let Some(new) = open_tab_in(&url, container.as_deref()) else { return };
    // Le nouvel onglet prend la place, l'epingle et le dossier de l'ancien.
    session::with(|s| {
        let kept = s.tabs.get_mut(id).map(|t| (t.pinned, t.folder.clone(), t.keep_awake));
        if let (Some((pinned, folder, awake)), Some(tab)) = (kept, s.tabs.get_mut(new)) {
            tab.pinned = pinned;
            tab.folder = folder;
            tab.keep_awake = awake;
        }
        if let Some(index) = index {
            s.tabs.move_to(new, index);
        }
    });
    if was_active {
        select_tab(new);
    }
    close_tab(id);
    publish_tabs();
}

/// Ouvre le terminal de Claude Code, ou revient a l'onglet qui le porte deja.
pub fn open_terminal() {
    match session::with(|s| s.tabs.find_by_url(crate::terminal::PAGE)).flatten() {
        Some(id) => select_tab(id),
        None => {
            open_tab(crate::terminal::PAGE);
            publish_tabs();
        }
    }
}

/// Endort un onglet a la demande de l'utilisateur.
pub fn sleep_tab(id: TabId) {
    if let Some(detached) = session::with(|s| s.tabs.put_to_sleep(id)).flatten() {
        detached.dispose();
        publish_tabs();
    }
}

/// Active un onglet, en le reveillant d'abord s'il dort.
pub fn select_tab(id: TabId) {
    dismiss_overlays();
    if session::with(|s| s.tabs.active_id()).flatten() != Some(id) {
        crate::devtools::undock();
        // La barre de recherche se ferme avec le changement d'onglet : ses surlignages partent avec elle.
        crate::find::stop_clearing();
    }
    if session::with(|s| s.tabs.is_asleep(id)).unwrap_or(false) {
        wake_tab(id);
    }
    session::with(|s| s.tabs.select(id));
    publish_tabs();
    publish_shield();
}

/// Reveille un onglet endormi sans l'afficher : la page charge pendant que la souris approche du clic.
pub fn warm_tab(id: TabId) {
    if !session::with(|s| s.tabs.is_asleep(id)).unwrap_or(false) {
        return;
    }
    wake_tab(id);
    session::with(|s| {
        s.tabs.refresh_visibility();
        if let Some(tab) = s.tabs.get_mut(id) {
            tab.last_active = std::time::Instant::now();
        }
    });
    publish_tabs();
}

/// Recree le navigateur d'un onglet endormi et recharge sa page.
fn wake_tab(id: TabId) {
    let container = session::with(|s| s.tabs.container_of(id)).flatten();
    if wait_for_container(container.as_deref(), || move || wake_tab(id)) {
        return;
    }
    let Some((mut client, host, url, container)) = session::with(|s| {
        Some((s.client.clone(), s.tabs.host(), s.tabs.wake_url(id)?, s.tabs.container_of(id)))
    })
    .flatten() else {
        return;
    };
    let Some(view) = crate::window::create_view(client.as_mut(), &url, 0, container.as_deref()) else {
        warn!(id, %url, "vue de reveil non creee");
        return;
    };
    if let Some(host) = host {
        host.add_child_view(Some(&mut View::from(&view)));
    }
    session::with(|s| s.tabs.wake_with(id, view));
    crate::page_state::arm(id);
}

/// Les sites dont l'onglet ne doit jamais dormir (reglage `tabs.neverSleep`, separes par des virgules).
fn never_sleep_hosts() -> Vec<String> {
    use echo_library::settings::Value;
    let settings = session::with(|s| echo_library::settings::all(&s.library)).unwrap_or_default();
    match settings.into_iter().find(|(key, _)| key == "tabs.neverSleep") {
        Some((_, Value::Text(list))) => {
            list.split(',').map(|h| h.trim().to_lowercase()).filter(|h| !h.is_empty()).collect()
        }
        _ => Vec::new(),
    }
}

/// Endort les onglets inactifs depuis `idle`. Renvoie leur nombre.
pub fn sleep_idle_tabs(idle: std::time::Duration) -> usize {
    let never = never_sleep_hosts();
    let ids = session::with(|s| s.tabs.sleep_candidates(idle, &never)).unwrap_or_default();
    let mut slept = 0;
    for id in ids {
        if let Some(detached) = session::with(|s| s.tabs.put_to_sleep(id)).flatten() {
            detached.dispose();
            slept += 1;
        }
    }
    if slept > 0 {
        publish_tabs();
    }
    slept
}

/// Allege les pages d'arriere-plan inactives depuis `idle` (pression memoire simulee : caches vides, ramasse-miettes).
/// Jamais `Memory.forciblyPurgeJavaScriptMemory` : il detruit le contexte JavaScript, la page reste affichee mais morte
/// (editeurs inertes, collage casse) jusqu'au rechargement. Renvoie le nombre de pages allegees.
pub fn trim_idle_tabs(idle: std::time::Duration) -> usize {
    use cef::{ImplBrowser, ImplBrowserHost};
    let targets = session::with(|s| s.tabs.take_trim_targets(idle)).unwrap_or_default();
    let message = serde_json::json!({
        "id": 1, "method": "Memory.simulatePressureNotification", "params": {"level": "critical"}
    })
    .to_string();
    for browser in &targets {
        if let Some(host) = browser.host() {
            host.send_dev_tools_message(Some(message.as_bytes()));
        }
    }
    targets.len()
}

/// Ferme un onglet. Comme pour l'ouverture, les appels a Chromium se font hors de
/// l'acces a l'etat, sinon la fermeture fige le navigateur.
pub fn close_tab(id: TabId) {
    super::navigation::remember_closed(id);
    let Some(detached) = session::with(|s| s.tabs.detach(id)) else { return };
    let remaining = detached.remaining;
    detached.dispose();
    if remaining == 0 {
        quit_message_loop();
        return;
    }
    session::with(|s| s.tabs.refresh_visibility());
    // Dernier onglet du profil ferme : le profil garde une page d'accueil plutot qu'un ecran vide.
    if session::with(|s| s.tabs.active_id()).flatten().is_none() {
        open_tab(crate::search::HOME);
    }
    publish_tabs();
    publish_shield();
}

/// Range dans les onglets une fenetre ouverte par une page. Faux si la scene n'existe pas encore.
pub fn adopt_popup(view: BrowserView) -> bool {
    let Some((host, container, opener)) = session::with(|s| {
        let container = s.tabs.active().and_then(|tab| tab.container.clone());
        Some((s.tabs.host()?, container, s.tabs.active_id()))
    })
    .flatten() else {
        return false;
    };
    host.add_child_view(Some(&mut View::from(&view)));
    session::with(|s| {
        let id = s.tabs.adopt(view, "about:blank");
        if let Some(tab) = s.tabs.get_mut(id) {
            tab.container = container;
            tab.opener = opener;
        }
        s.tabs.refresh_visibility();
    });
    publish_tabs();
    true
}

/// Une page demande sa propre fermeture (`window.close()`) : on ferme son onglet, pas la fenetre.
/// Vrai si le navigateur est celui d'un onglet (le traitement est alors differe d'un tour de boucle).
pub fn page_asks_close(browser_id: i32) -> bool {
    // Les outils de developpement ancres : leur fermeture retire leur vue, jamais la fenetre.
    if crate::devtools::owns(browser_id) {
        crate::containers::later(crate::devtools::undock);
        return true;
    }
    if crate::window::CLOSING.load(std::sync::atomic::Ordering::Relaxed) {
        return false;
    }
    let Some(id) = session::with(|s| s.tabs.by_browser(browser_id).map(|tab| tab.id)).flatten() else {
        return false;
    };
    crate::containers::later(move || close_popup(id, false));
    true
}

/// Ferme un onglet ouvert par une page et revient sur celle-ci ; `reload` la recharge (connexion faite).
fn close_popup(id: TabId, reload: bool) {
    let opener = session::with(|s| s.tabs.get_mut(id).and_then(|tab| tab.opener)).flatten();
    close_tab(id);
    let Some(opener) = opener.filter(|o| session::with(|s| s.tabs.exists(*o)).unwrap_or(false)) else { return };
    super::select_tab(opener);
    if reload {
        let browser = session::with(|s| s.tabs.get_mut(opener).and_then(|tab| tab.browser())).flatten();
        if let Some(browser) = browser {
            browser.reload();
        }
    }
}

/// La popup de connexion Google n'a pas pu rendre la main a sa page (lien coupe par le site, en-tete COOP) :
/// la connexion est faite, mais la popup resterait blanche. On la ferme et on recharge la page d'origine.
pub fn finish_orphan_signin(browser_id: i32) {
    let Some(id) = session::with(|s| s.tabs.by_browser(browser_id).filter(|t| t.opener.is_some()).map(|t| t.id)).flatten()
    else {
        return;
    };
    tracing::info!(id, "popup de connexion orpheline : fermee, page d'origine rechargee");
    crate::containers::later(move || close_popup(id, true));
}
