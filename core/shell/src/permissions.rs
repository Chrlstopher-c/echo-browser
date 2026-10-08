//! Responsabilite : les demandes de permission des sites (camera, micro, position, notifications,
//! presse-papiers) — decision retenue si elle existe, question a l'utilisateur sinon.

use cef::*;
use echo_contract::CoreEvent;
use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use tracing::{info, warn};

/// Comment la reponse repart vers Chromium.
enum Reply {
    Prompt { prompt_id: u64, callback: PermissionPromptCallback },
    Media { requested: u32, callback: MediaAccessCallback },
}

struct Pending {
    origin: String,
    kinds: Vec<&'static str>,
    reply: Reply,
}

thread_local! {
    static PENDING: RefCell<HashMap<u64, Pending>> = RefCell::new(HashMap::new());
    static NEXT_ID: Cell<u64> = const { Cell::new(1) };
}

fn bits(types: PermissionRequestTypes) -> u32 {
    *types.as_ref() as u32
}

fn media_bits(types: MediaAccessPermissionTypes) -> u32 {
    *types.as_ref() as u32
}

/// Les permissions que l'on sait poser a l'utilisateur ; toute autre demande suit le comportement par defaut.
fn prompt_kinds(requested: u32) -> Option<Vec<&'static str>> {
    let known = [
        ("camera", bits(PermissionRequestTypes::CAMERA_STREAM)),
        ("microphone", bits(PermissionRequestTypes::MIC_STREAM)),
        ("position", bits(PermissionRequestTypes::GEOLOCATION)),
        ("notifications", bits(PermissionRequestTypes::NOTIFICATIONS)),
        ("presse-papiers", bits(PermissionRequestTypes::CLIPBOARD)),
    ];
    let mask: u32 = known.iter().fold(0, |all, (_, bit)| all | bit);
    if requested == 0 || requested & !mask != 0 {
        return None;
    }
    Some(known.iter().filter(|(_, bit)| requested & bit != 0).map(|(name, _)| *name).collect())
}

fn media_kinds(requested: u32) -> Option<Vec<&'static str>> {
    let audio = media_bits(MediaAccessPermissionTypes::DEVICE_AUDIO_CAPTURE);
    let video = media_bits(MediaAccessPermissionTypes::DEVICE_VIDEO_CAPTURE);
    if requested == 0 || requested & !(audio | video) != 0 {
        return None;
    }
    let mut kinds = Vec::new();
    if requested & video != 0 {
        kinds.push("camera");
    }
    if requested & audio != 0 {
        kinds.push("microphone");
    }
    Some(kinds)
}

/// La decision deja retenue pour toutes ces permissions : `Some(true)` si elles sont toutes accordees,
/// `Some(false)` si l'une est refusee, `None` s'il en manque une.
fn stored(origin: &str, kinds: &[&str]) -> Option<bool> {
    let decisions: Vec<Option<bool>> = crate::session::with(|s| {
        kinds.iter().map(|kind| echo_library::permissions::get(&s.library, origin, kind)).collect()
    })?;
    if decisions.contains(&Some(false)) {
        return Some(false);
    }
    decisions.iter().all(|d| *d == Some(true)).then_some(true)
}

fn resolve(reply: Reply, allow: bool) {
    match reply {
        Reply::Prompt { callback, .. } => {
            callback.cont(if allow { PermissionRequestResult::ACCEPT } else { PermissionRequestResult::DENY });
        }
        Reply::Media { requested, callback } => callback.cont(if allow { requested } else { 0 }),
    }
}

fn request(origin: String, kinds: Vec<&'static str>, reply: Reply) {
    if let Some(allow) = stored(&origin, &kinds) {
        info!(%origin, ?kinds, allow, "permission deja decidee");
        journal_decision(&origin, &kinds, allow, true);
        return resolve(reply, allow);
    }
    let id = NEXT_ID.with(|next| {
        let id = next.get();
        next.set(id + 1);
        id
    });
    info!(%origin, ?kinds, id, "permission demandee a l'utilisateur");
    crate::bridge::publish(&CoreEvent::PermissionRequested {
        id,
        origin: origin.clone(),
        kinds: kinds.iter().map(|k| k.to_string()).collect(),
    });
    reveal_sidebar(true);
    PENDING.with(|pending| pending.borrow_mut().insert(id, Pending { origin, kinds, reply }));
}

/// Au journal du site : chaque permission demandee et ce qui a ete decide.
fn journal_decision(origin: &str, kinds: &[&str], allow: bool, remembered: bool) {
    let verdict = match (allow, remembered) {
        (true, false) => "autorisée",
        (false, false) => "refusée",
        (true, true) => "autorisée (décision gardée)",
        (false, true) => "refusée (décision gardée)",
    };
    for kind in kinds {
        crate::network::journal(origin, "permission", &format!("{kind} : {verdict}"));
    }
}

/// Une barre repliee cacherait la question, et la page attendrait sans fin.
fn reveal_sidebar(reveal: bool) {
    let chrome = crate::session::with(|s| s.chrome.clone()).flatten();
    crate::window::reveal_chrome(reveal, chrome.as_ref());
}

/// La reponse de l'utilisateur.
pub fn answer(id: u64, allow: bool, remember: bool) {
    let Some(pending) = PENDING.with(|pending| pending.borrow_mut().remove(&id)) else {
        return warn!(id, "reponse a une permission inconnue");
    };
    if remember {
        crate::session::with(|s| {
            for kind in &pending.kinds {
                echo_library::permissions::set(&s.library, &pending.origin, kind, allow);
            }
        });
    }
    journal_decision(&pending.origin, &pending.kinds, allow, false);
    resolve(pending.reply, allow);
    crate::bridge::publish(&CoreEvent::PermissionResolved { id });
    if remember {
        crate::bridge::publish_permissions();
    }
    reveal_sidebar(false);
}

wrap_permission_handler! {
    pub struct EchoPermissions {
        marker: (),
    }

    impl PermissionHandler {
        fn on_request_media_access_permission(
            &self,
            _browser: Option<&mut Browser>,
            _frame: Option<&mut Frame>,
            requesting_origin: Option<&CefString>,
            requested_permissions: u32,
            callback: Option<&mut MediaAccessCallback>,
        ) -> ::std::os::raw::c_int {
            let (Some(origin), Some(callback), Some(kinds)) =
                (requesting_origin, callback, media_kinds(requested_permissions))
            else {
                return 0;
            };
            let reply = Reply::Media { requested: requested_permissions, callback: callback.clone() };
            request(origin.to_string(), kinds, reply);
            1
        }

        fn on_show_permission_prompt(
            &self,
            _browser: Option<&mut Browser>,
            prompt_id: u64,
            requesting_origin: Option<&CefString>,
            requested_permissions: u32,
            callback: Option<&mut PermissionPromptCallback>,
        ) -> ::std::os::raw::c_int {
            let (Some(origin), Some(callback), Some(kinds)) =
                (requesting_origin, callback, prompt_kinds(requested_permissions))
            else {
                return 0;
            };
            request(origin.to_string(), kinds, Reply::Prompt { prompt_id, callback: callback.clone() });
            1
        }

        fn on_dismiss_permission_prompt(
            &self,
            _browser: Option<&mut Browser>,
            prompt_id: u64,
            _result: PermissionRequestResult,
        ) {
            let gone = PENDING.with(|pending| {
                let mut pending = pending.borrow_mut();
                let id = pending
                    .iter()
                    .find(|(_, p)| matches!(p.reply, Reply::Prompt { prompt_id: pid, .. } if pid == prompt_id))
                    .map(|(id, _)| *id)?;
                pending.remove(&id);
                Some(id)
            });
            if let Some(id) = gone {
                crate::bridge::publish(&CoreEvent::PermissionResolved { id });
                reveal_sidebar(false);
            }
        }
    }
}
