//! Responsabilite : la prise de pilotage locale — un outil (MCP de Claude, script) y liste,
//! lit, ouvre et ferme des onglets. Une prise Unix en 0600 : aucune page web ne peut l'atteindre.

use cef::*;
use parking_lot::Mutex;
use serde_json::{json, Value};
use std::collections::VecDeque;
use std::io::{BufRead, BufReader, Write};
use std::os::unix::fs::PermissionsExt;
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::PathBuf;
use std::sync::mpsc::{self, Sender};
use std::time::Duration;
use tracing::{info, warn};

const REPLY_TIMEOUT: Duration = Duration::from_secs(10);
/// Au-dela, le texte d'une page est coupe : un outil n'en a pas besoin de plus.
const MAX_TEXT_CHARS: usize = 200_000;

struct Job {
    request: Value,
    reply: Sender<Value>,
}

static QUEUE: Mutex<VecDeque<Job>> = Mutex::new(VecDeque::new());

/// Chemin de la prise, dans le dossier d'execution de l'utilisateur.
pub fn socket_path() -> Option<PathBuf> {
    let runtime = std::env::var_os("XDG_RUNTIME_DIR")?;
    Some(PathBuf::from(runtime).join("echo-browser").join("control.sock"))
}

/// Ouvre la prise, sauf `ECHO_CONTROL=0`.
pub fn start() {
    if std::env::var("ECHO_CONTROL").is_ok_and(|v| v == "0") {
        return;
    }
    match bind() {
        Ok(listener) => {
            info!("prise de pilotage ouverte");
            std::thread::spawn(move || serve(listener));
        }
        Err(error) => warn!(%error, "prise de pilotage non ouverte"),
    }
}

fn bind() -> std::io::Result<UnixListener> {
    let path = socket_path().ok_or_else(|| std::io::Error::other("XDG_RUNTIME_DIR absent"))?;
    let dir = path.parent().ok_or_else(|| std::io::Error::other("chemin sans dossier"))?;
    std::fs::create_dir_all(dir)?;
    std::fs::set_permissions(dir, std::fs::Permissions::from_mode(0o700))?;
    if UnixStream::connect(&path).is_ok() {
        return Err(std::io::Error::other("un autre navigateur tient deja la prise"));
    }
    let _ = std::fs::remove_file(&path);
    let listener = UnixListener::bind(&path)?;
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600))?;
    Ok(listener)
}

fn serve(listener: UnixListener) {
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                std::thread::spawn(move || converse(stream));
            }
            Err(error) => warn!(%error, "connexion de pilotage refusee"),
        }
    }
}

/// Une demande JSON par ligne, une reponse JSON par ligne.
fn converse(stream: UnixStream) {
    let Ok(clone) = stream.try_clone() else { return };
    for line in BufReader::new(clone).lines() {
        let Ok(line) = line else { return };
        let reply = match serde_json::from_str::<Value>(&line) {
            Ok(request) => call_ui(request),
            Err(error) => json!({"ok": false, "error": format!("json illisible : {error}")}),
        };
        if writeln!(&stream, "{reply}").is_err() {
            return;
        }
    }
}

fn call_ui(request: Value) -> Value {
    let (reply, wait) = mpsc::channel();
    QUEUE.lock().push_back(Job { request, reply });
    let mut task = DrainTask::new(());
    post_task(ThreadId::UI, Some(&mut task));
    wait.recv_timeout(REPLY_TIMEOUT).unwrap_or_else(|_| json!({"ok": false, "error": "delai depasse"}))
}

wrap_task! {
    struct DrainTask {
        marker: (),
    }

    impl Task {
        fn execute(&self) {
            while let Some(job) = QUEUE.lock().pop_front() {
                run(job);
            }
        }
    }
}

fn fail(reply: &Sender<Value>, error: &str) {
    let _ = reply.send(json!({"ok": false, "error": error}));
}

fn tab_id(request: &Value) -> Option<u32> {
    request.get("id").and_then(Value::as_u64).and_then(|id| u32::try_from(id).ok())
}

fn run(job: Job) {
    let Job { request, reply } = job;
    match request.get("op").and_then(Value::as_str) {
        Some("tabs") => {
            let _ = reply.send(tabs());
        }
        Some("read") => read(&request, reply),
        Some("open") => open(&request, &reply),
        Some("navigate") => navigate(&request, &reply),
        Some("activate") => with_id(&request, &reply, crate::bridge::select_tab),
        Some("close") => with_id(&request, &reply, crate::bridge::close_tab),
        Some("sleep") => with_id(&request, &reply, crate::bridge::sleep_tab),
        _ => fail(&reply, "operation inconnue : tabs, read, open, navigate, activate, sleep, close"),
    }
}

fn tabs() -> Value {
    let Some((views, active)) = crate::session::with(|s| (s.tabs.snapshot(), s.tabs.active_id())) else {
        return json!({"ok": false, "error": "navigateur occupe"});
    };
    let list: Vec<Value> = views
        .iter()
        .map(|t| {
            json!({"id": t.id, "title": t.title, "url": t.url, "active": active == Some(t.id),
                   "asleep": t.asleep, "loading": t.loading})
        })
        .collect();
    json!({"ok": true, "tabs": list})
}

fn open(request: &Value, reply: &Sender<Value>) {
    let Some(url) = request.get("url").and_then(Value::as_str) else { return fail(reply, "url manquante") };
    crate::bridge::open_tab(&crate::bridge::normalize(url));
    crate::bridge::publish_tabs();
    let id = crate::session::with(|s| s.tabs.active_id()).flatten();
    let _ = reply.send(json!({"ok": true, "id": id}));
}

fn navigate(request: &Value, reply: &Sender<Value>) {
    let (Some(id), Some(url)) = (tab_id(request), request.get("url").and_then(Value::as_str)) else {
        return fail(reply, "id et url requis");
    };
    crate::bridge::submit(json!({"kind": "navigate", "id": id, "input": url}).to_string().as_bytes());
    let _ = reply.send(json!({"ok": true}));
}

fn with_id(request: &Value, reply: &Sender<Value>, action: fn(u32)) {
    match tab_id(request) {
        Some(id) if !crate::session::with(|s| s.tabs.exists(id)).unwrap_or(false) => fail(reply, "onglet inconnu"),
        Some(id) => {
            action(id);
            let _ = reply.send(json!({"ok": true}));
        }
        None => fail(reply, "id requis"),
    }
}

/// Le texte visible d'un onglet (l'actif par defaut). L'onglet endormi n'a pas de page a lire.
fn read(request: &Value, reply: Sender<Value>) {
    let target = tab_id(request).or_else(|| crate::session::with(|s| s.tabs.active_id()).flatten());
    let Some(id) = target else { return fail(&reply, "aucun onglet") };
    let found = crate::session::with(|s| {
        let tab = s.tabs.get_mut(id)?;
        Some((tab.browser(), json!({"id": id, "title": tab.title, "url": tab.url, "loading": tab.loading})))
    })
    .flatten();
    let Some((browser, meta)) = found else { return fail(&reply, "onglet inconnu") };
    let Some(frame) = browser.and_then(|b| b.main_frame()) else {
        return fail(&reply, "onglet endormi : l'activer d'abord");
    };
    let mut visitor = TextVisitor::new(reply, meta);
    frame.text(Some(&mut visitor));
}

wrap_string_visitor! {
    struct TextVisitor {
        reply: Sender<Value>,
        meta: Value,
    }

    impl CefStringVisitor {
        fn visit(&self, string: Option<&CefString>) {
            let text = string.map(CefString::to_string).unwrap_or_default();
            let text: String = text.chars().take(MAX_TEXT_CHARS).collect();
            let mut out = self.meta.clone();
            out["ok"] = json!(true);
            out["text"] = json!(text);
            let _ = self.reply.send(out);
        }
    }
}
