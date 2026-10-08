//! Onglets d'Echo dans les service workers des extensions. Par le protocole de debogage du navigateur :
//! - chaque service worker d'extension qui demarre est mis en pause, arrete juste avant son premier script (`chrome`
//!   n'existe qu'a ce moment-la), recoit `worker.js` et la liste des onglets, puis repart ; on s'en detache aussitot,
//!   un debogueur attache l'empecherait de se mettre en veille ;
//! - a chaque changement d'onglets, les service workers eveilles recoivent la nouvelle liste (attache, envoi, detache) ;
//! - une extension qui ecoute les evenements d'onglets et dort est reveillee quand les onglets de son profil changent,
//!   comme Chrome le fait ; a son reveil elle recoit l'etat vu avant la veille puis l'actuel, donc les changements
//!   faits entre-temps (un onglet ferme pendant la veille n'est pas annonce : son identifiant est perdu avec le worker).
//!
//! Seules les extensions qui voient deja les onglets (`tabs` ou acces a tous les sites) sont servies, et chacune ne
//! recoit que les onglets de son profil (contexte Chromium).

use std::collections::{HashMap, HashSet};
use std::net::TcpStream;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::OnceLock;
use std::time::{Duration, Instant};

use parking_lot::Mutex;
use serde_json::{json, Value};
use tracing::{debug, warn};
use tungstenite::stream::MaybeTlsStream;
use tungstenite::{connect, Message, WebSocket};

const WORKER_JS: &str = include_str!("worker.js");
const PUSH_EVERY: Duration = Duration::from_millis(250);
/// Delai apres le premier script d'un service worker pour savoir s'il ecoute les evenements d'onglets.
const PROBE_AFTER: Duration = Duration::from_secs(1);

/// Une extension dans un profil (contexte Chromium) : (extension, contexte).
type Key = (String, String);

/// Ce que le fil de l'interface envoie : les onglets vivants, et les extensions autorisees a les voir.
pub struct Update {
    pub tabs: Vec<Value>,
    pub allowed: Option<HashSet<String>>,
}

static SENDER: OnceLock<Mutex<Sender<Update>>> = OnceLock::new();

/// Transmet l'etat des onglets (appele a chaque publication des onglets).
pub fn send(update: Update) {
    if let Some(sender) = SENDER.get() {
        let _ = sender.lock().send(update);
    }
}

pub fn start(first: Update) {
    let (tx, rx) = channel();
    let _ = tx.send(first);
    if SENDER.set(Mutex::new(tx)).is_err() {
        return;
    }
    std::thread::spawn(move || {
        let mut state = State::default();
        for attempt in 1..=30 {
            std::thread::sleep(Duration::from_millis(if attempt == 1 { 300 } else { 2000 }));
            match run(&rx, &mut state) {
                Ok(()) => return,
                Err(err) => debug!(attempt, %err, "onglets pour les service workers : nouvel essai"),
            }
        }
        warn!("onglets pour les service workers abandonnes : les extensions ne verront pas les onglets d'Echo");
    });
}

#[derive(Default)]
struct State {
    tabs: Vec<Value>,
    allowed: HashSet<String>,
    /// Pages : cible -> (adresse, contexte).
    pages: HashMap<String, (String, String)>,
    /// Onglet d'Echo -> contexte, appris quand son adresse est celle d'une page de ce contexte, puis garde : apres une
    /// navigation, Echo connait la nouvelle adresse avant que le protocole ne l'annonce.
    tab_context: HashMap<i64, String>,
    /// Service workers d'extensions servies : cible -> (extension, contexte).
    workers: HashMap<String, Key>,
    /// Sessions en pause au demarrage : session -> (extension, contexte).
    starting: HashMap<String, Key>,
    /// Derniers onglets transmis a chaque extension de chaque profil (l'etat qu'elle a vu avant de s'endormir).
    delivered: HashMap<Key, Vec<Value>>,
    /// Extensions dont le service worker ecoute les evenements d'onglets : on les reveille.
    listening: HashSet<Key>,
    /// Commande d'envoi d'etat -> extension, pour lire en retour si elle ecoute.
    sent: HashMap<i64, Key>,
    /// Service workers a sonder (ecoute-t-il ?) une fois leur premier script passe : (quand, cible).
    probes: Vec<(Instant, String)>,
    /// Contextes ou reveiller des extensions, en attente d'une page ou s'attacher.
    waking: HashMap<String, Vec<String>>,
    /// Commande -> session a detacher une fois la commande traitee. Se detacher tout de suite perd les ordres encore en
    /// route vers le service worker : un « reprends » perdu le laisse fige avant son premier script.
    detach_after: HashMap<i64, String>,
    dirty: bool,
    /// Contextes ou des service workers servis tournaient deja a la connexion : leurs ecouteurs d'onglets ont ete
    /// enregistres sans nous. On les arrete une fois (ils repartent a la demande, cette fois interceptes).
    stale: HashSet<String>,
    /// Contextes dont l'arret est demande, en attente d'une page ou s'attacher.
    stopping: HashSet<String>,
    connected_at: Option<Instant>,
}

/// Fenetre apres la connexion pendant laquelle un service worker deja lance compte comme « parti sans nous ».
const STARTUP_WINDOW: Duration = Duration::from_millis(1500);

type Socket = WebSocket<MaybeTlsStream<TcpStream>>;

fn run(rx: &Receiver<Update>, state: &mut State) -> anyhow::Result<()> {
    let url = crate::anchor_watch::browser_endpoint().ok_or_else(|| anyhow::anyhow!("point de debogage introuvable"))?;
    let (mut socket, _) = connect(url.as_str())?;
    if let MaybeTlsStream::Plain(stream) = socket.get_ref() {
        stream.set_read_timeout(Some(Duration::from_millis(100)))?;
    }
    let filter = json!([{"type": "service_worker", "exclude": false}, {"exclude": true}]);
    send_cmd(&mut socket, None, "Target.setDiscoverTargets", json!({"discover": true}))?;
    send_cmd(&mut socket, None, "Target.setAutoAttach",
        json!({"autoAttach": true, "waitForDebuggerOnStart": true, "flatten": true, "filter": filter}))?;
    let mut last_push = Instant::now();
    state.connected_at = Some(Instant::now());
    let mut stale_sent = false;
    loop {
        if !stale_sent && state.connected_at.is_some_and(|t| t.elapsed() >= STARTUP_WINDOW) {
            stale_sent = true;
            restart_stale(&mut socket, state)?;
        }
        match socket.read() {
            Ok(Message::Text(text)) => handle(&mut socket, state, &text)?,
            Ok(_) => {}
            Err(tungstenite::Error::Io(e)) if matches!(e.kind(), std::io::ErrorKind::WouldBlock | std::io::ErrorKind::TimedOut) => {}
            Err(err) => return Err(err.into()),
        }
        while let Ok(update) = rx.try_recv() {
            state.tabs = update.tabs;
            if let Some(allowed) = update.allowed {
                state.allowed = allowed;
            }
            state.dirty = true;
        }
        if state.dirty {
            learn_contexts(state);
        }
        if state.dirty && last_push.elapsed() >= PUSH_EVERY {
            state.dirty = false;
            last_push = Instant::now();
            for target in state.workers.keys() {
                send_cmd(&mut socket, None, "Target.attachToTarget", json!({"targetId": target, "flatten": true}))?;
            }
            wake_sleepers(&mut socket, state)?;
        }
        run_probes(&mut socket, state)?;
    }
}

fn send_cmd(socket: &mut Socket, session: Option<&str>, method: &str, params: Value) -> anyhow::Result<i64> {
    static NEXT: std::sync::atomic::AtomicI64 = std::sync::atomic::AtomicI64::new(1);
    let id = NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let mut message = json!({"id": id, "method": method, "params": params});
    if let Some(session) = session {
        message["sessionId"] = json!(session);
    }
    socket.send(Message::text(message.to_string()))?;
    Ok(id)
}

/// Envoie la derniere commande d'une session, puis s'en detache quand elle a ete traitee.
fn finish(socket: &mut Socket, state: &mut State, session: &str, method: &str, params: Value) -> anyhow::Result<()> {
    let id = send_cmd(socket, Some(session), method, params)?;
    state.detach_after.insert(id, session.to_string());
    Ok(())
}

/// L'identifiant d'extension d'une adresse `chrome-extension://<id>/...`.
fn extension_of(url: &str) -> Option<&str> {
    url.strip_prefix("chrome-extension://")?.split('/').next()
}

fn handle(socket: &mut Socket, state: &mut State, text: &str) -> anyhow::Result<()> {
    let msg: Value = serde_json::from_str(text).unwrap_or(Value::Null);
    if let Some(key) = msg["id"].as_i64().and_then(|id| state.sent.remove(&id)) {
        match msg["result"]["result"]["value"].as_bool() {
            Some(true) => state.listening.insert(key),
            Some(false) => state.listening.remove(&key),
            None => false,
        };
    }
    if let Some(session) = msg["id"].as_i64().and_then(|id| state.detach_after.remove(&id)) {
        send_cmd(socket, None, "Target.detachFromTarget", json!({"sessionId": session}))?;
        return Ok(());
    }
    let params = &msg["params"];
    match msg["method"].as_str() {
        Some("Target.targetCreated" | "Target.targetInfoChanged") => track(state, &params["targetInfo"]),
        Some("Target.targetDestroyed") => {
            let target = params["targetId"].as_str().unwrap_or_default();
            state.pages.remove(target);
            state.workers.remove(target);
        }
        Some("Target.attachedToTarget") if params["targetInfo"]["type"] == "page" => stop_workers_in(socket, state, params)?,
        Some("Target.attachedToTarget") => attached(socket, state, params)?,
        Some("Debugger.paused") => {
            let session = msg["sessionId"].as_str().unwrap_or_default().to_string();
            if let Some(key) = state.starting.remove(&session) {
                inject(socket, state, &session, &key, true)?;
                send_cmd(socket, Some(&session), "Debugger.disable", json!({}))?;
                finish(socket, state, &session, "Debugger.resume", json!({}))?;
                if let Some(target) = state.workers.iter().find(|(_, k)| **k == key).map(|(t, _)| t.clone()) {
                    state.probes.push((Instant::now() + PROBE_AFTER, target));
                }
            }
        }
        _ => {}
    }
    Ok(())
}

fn track(state: &mut State, info: &Value) {
    let target = info["targetId"].as_str().unwrap_or_default().to_string();
    let url = info["url"].as_str().unwrap_or_default().to_string();
    let context = info["browserContextId"].as_str().unwrap_or_default().to_string();
    match info["type"].as_str() {
        Some("page") => {
            state.pages.insert(target, (url, context));
            state.dirty = true;
        }
        Some("service_worker") => {
            if let Some(id) = extension_of(&url).filter(|id| served(state, id)) {
                state.workers.insert(target, (id.to_string(), context));
            }
        }
        _ => {}
    }
}

fn served(state: &State, extension: &str) -> bool {
    !super::is_pont(extension) && state.allowed.contains(extension)
}

fn attached(socket: &mut Socket, state: &mut State, params: &Value) -> anyhow::Result<()> {
    let session = params["sessionId"].as_str().unwrap_or_default().to_string();
    let info = &params["targetInfo"];
    let url = info["url"].as_str().unwrap_or_default();
    let context = info["browserContextId"].as_str().unwrap_or_default().to_string();
    let waiting = params["waitingForDebugger"].as_bool().unwrap_or(false);
    debug!(%url, waiting, served = extension_of(url).is_some_and(|id| served(state, id)), "service worker attache");
    if !extension_of(url).is_some_and(|id| served(state, id)) {
        if waiting {
            return finish(socket, state, &session, "Runtime.runIfWaitingForDebugger", json!({}));
        }
        return send_cmd(socket, None, "Target.detachFromTarget", json!({"sessionId": session})).map(drop);
    }
    let key: Key = (extension_of(url).unwrap_or_default().to_string(), context.clone());
    if let Some(target) = info["targetId"].as_str() {
        state.workers.insert(target.to_string(), key.clone());
    }
    if waiting {
        // `chrome` n'existe pas encore : on s'arrete juste avant le premier script, ou il existe.
        state.starting.insert(session.clone(), key);
        send_cmd(socket, Some(&session), "Debugger.enable", json!({}))?;
        send_cmd(socket, Some(&session), "Debugger.setInstrumentationBreakpoint",
            json!({"instrumentation": "beforeScriptExecution"}))?;
        return send_cmd(socket, Some(&session), "Runtime.runIfWaitingForDebugger", json!({})).map(drop);
    }
    if state.connected_at.is_some_and(|t| t.elapsed() < STARTUP_WINDOW) {
        state.stale.insert(context.clone());
    }
    let id = inject(socket, state, &session, &key, false)?;
    state.detach_after.insert(id, session);
    Ok(())
}

/// S'attache a une page de chaque contexte « parti sans nous » pour y arreter les service workers.
fn restart_stale(socket: &mut Socket, state: &mut State) -> anyhow::Result<()> {
    for context in std::mem::take(&mut state.stale) {
        let page = state.pages.iter().find(|(_, (_, ctx))| *ctx == context).map(|(target, _)| target.clone());
        if let Some(page) = page {
            debug!(%context, "service workers lances avant Echo : redemarrage");
            state.stopping.insert(context);
            send_cmd(socket, None, "Target.attachToTarget", json!({"targetId": page, "flatten": true}))?;
        }
    }
    Ok(())
}

/// Page d'un contexte attachee : on y arrete les service workers « partis sans nous », ou on y reveille ceux attendus.
fn stop_workers_in(socket: &mut Socket, state: &mut State, params: &Value) -> anyhow::Result<()> {
    let session = params["sessionId"].as_str().unwrap_or_default().to_string();
    let context = params["targetInfo"]["browserContextId"].as_str().unwrap_or_default();
    if state.stopping.remove(context) {
        send_cmd(socket, Some(&session), "ServiceWorker.enable", json!({}))?;
        return finish(socket, state, &session, "ServiceWorker.stopAllWorkers", json!({}));
    }
    if let Some(extensions) = state.waking.remove(context) {
        send_cmd(socket, Some(&session), "ServiceWorker.enable", json!({}))?;
        let mut last = None;
        for extension in extensions {
            debug!(%extension, %context, "reveil d'une extension pour un changement d'onglets");
            let scope = format!("chrome-extension://{extension}/");
            last = Some(send_cmd(socket, Some(&session), "ServiceWorker.startWorker", json!({"scopeURL": scope}))?);
        }
        if let Some(id) = last {
            state.detach_after.insert(id, session);
        }
        return Ok(());
    }
    send_cmd(socket, None, "Target.detachFromTarget", json!({"sessionId": session})).map(drop)
}

fn learn_contexts(state: &mut State) {
    let live: HashSet<i64> = state.tabs.iter().filter_map(|tab| tab["e"].as_i64()).collect();
    state.tab_context.retain(|tab, _| live.contains(tab));
    for tab in &state.tabs {
        let (Some(id), Some(url)) = (tab["e"].as_i64(), tab["url"].as_str()) else { continue };
        if let Some((_, context)) = state.pages.values().find(|(page, _)| page == url) {
            state.tab_context.insert(id, context.clone());
        }
    }
}

fn tabs_of(state: &State, context: &str) -> Vec<Value> {
    state
        .tabs
        .iter()
        .filter(|tab| tab["e"].as_i64().and_then(|id| state.tab_context.get(&id)).is_some_and(|ctx| ctx == context))
        .cloned()
        .collect()
}

/// Pose `worker.js` (sans effet s'il y est deja) et la liste des onglets du profil de ce service worker. Au demarrage
/// (`starting`), aussi l'etat qu'il avait vu avant de s'endormir. Rend vrai (en retour de commande) s'il ecoute.
fn inject(socket: &mut Socket, state: &mut State, session: &str, key: &Key, starting: bool) -> anyhow::Result<i64> {
    let tabs = tabs_of(state, &key.1);
    let asleep = if starting { state.delivered.get(key).map(|t| json!({ "tabs": t })) } else { None };
    let script = format!(
        "{};self.__echoTabs&&self.__echoTabs({},{});!!(self.__echoListening&&self.__echoListening())",
        WORKER_JS.replace("__PONT__", super::PONT_ID),
        json!({ "tabs": tabs }),
        asleep.unwrap_or(Value::Null),
    );
    state.delivered.insert(key.clone(), tabs);
    let id = send_cmd(socket, Some(session), "Runtime.evaluate", json!({"expression": script, "returnByValue": true}))?;
    if !starting {
        state.sent.insert(id, key.clone());
    }
    Ok(id)
}

/// Sonde les service workers demarres il y a un instant : leur premier script a enregistre (ou non) des ecouteurs.
fn run_probes(socket: &mut Socket, state: &mut State) -> anyhow::Result<()> {
    let now = Instant::now();
    let (due, later): (Vec<_>, Vec<_>) = std::mem::take(&mut state.probes).into_iter().partition(|(at, _)| *at <= now);
    state.probes = later;
    for (_, target) in due.into_iter().filter(|(_, t)| state.workers.contains_key(t)) {
        send_cmd(socket, None, "Target.attachToTarget", json!({"targetId": target, "flatten": true}))?;
    }
    Ok(())
}

/// Reveille les extensions qui ecoutent les onglets, dorment, et dont les onglets du profil ont change.
fn wake_sleepers(socket: &mut Socket, state: &mut State) -> anyhow::Result<()> {
    let awake: HashSet<&Key> = state.workers.values().collect();
    let mut wanted: Vec<Key> = state
        .listening
        .iter()
        .filter(|key| !awake.contains(key) && state.delivered.get(*key) != Some(&tabs_of(state, &key.1)))
        .cloned()
        .collect();
    wanted.retain(|(extension, _)| served(state, extension));
    for (extension, context) in wanted {
        let page = state.pages.iter().find(|(_, (_, ctx))| *ctx == context).map(|(target, _)| target.clone());
        let Some(page) = page else { continue };
        let pending = state.waking.entry(context).or_default();
        if !pending.contains(&extension) {
            pending.push(extension);
            if pending.len() == 1 {
                send_cmd(socket, None, "Target.attachToTarget", json!({"targetId": page, "flatten": true}))?;
            }
        }
    }
    Ok(())
}
