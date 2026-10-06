//! Responsabilite : le terminal de Claude Code — la commande lancee, et les routes
//! `echo://ui/term/*` par lesquelles la page du terminal lit la sortie et envoie la saisie.

use echo_terminal::Terminal;
use parking_lot::Mutex;
use std::sync::OnceLock;
use tracing::{info, warn};

const DEFAULT_COLS: u16 = 100;
const DEFAULT_ROWS: u16 = 30;
/// La page du terminal, ouverte comme un onglet.
pub const PAGE: &str = "echo://ui/terminal.html";

const SESSION: &str = "claude-navigateur";

static TERMINAL: OnceLock<Mutex<Option<Terminal>>> = OnceLock::new();

/// La commande du terminal : `ECHO_TERM_CMD`, sinon Claude Code dans la session tmux `claude-navigateur`
/// du serveur tmux de Quart, pour qu'elle soit la meme depuis le navigateur et depuis Quart.
fn command() -> Vec<String> {
    if let Ok(custom) = std::env::var("ECHO_TERM_CMD") {
        return custom.split_whitespace().map(str::to_string).collect();
    }
    let home = std::env::var("HOME").unwrap_or_else(|_| "/".into());
    let conf = format!("{home}/.local/share/ccremote/code/poste/session/tmux.conf");
    let mut argv: Vec<String> = vec!["tmux".into(), "-L".into(), "claude".into()];
    if std::path::Path::new(&conf).is_file() {
        argv.extend(["-f".into(), conf]);
    }
    argv.extend(["-u", "new-session", "-A", "-s", SESSION, "-c", &home, "--", "claude"].map(String::from));
    argv
}

fn query_u64(query: &str, key: &str) -> Option<u64> {
    query.split('&').find_map(|pair| pair.strip_prefix(key)?.strip_prefix('=')?.parse().ok())
}

fn query_size(query: &str) -> (u16, u16) {
    let cols = query_u64(query, "cols").and_then(|v| u16::try_from(v).ok()).unwrap_or(DEFAULT_COLS);
    let rows = query_u64(query, "rows").and_then(|v| u16::try_from(v).ok()).unwrap_or(DEFAULT_ROWS);
    (cols.clamp(20, 500), rows.clamp(5, 200))
}

/// Le terminal en cours, lance au premier besoin ou relance s'il est termine.
fn current(size: (u16, u16), restart_if_closed: bool) -> Option<Terminal> {
    let mut slot = TERMINAL.get_or_init(|| Mutex::new(None)).lock();
    let dead = slot.as_ref().is_some_and(|term| term.read_since(u64::MAX).closed);
    if slot.is_none() || (dead && restart_if_closed) {
        let argv = command();
        match Terminal::spawn(&argv, size.0, size.1) {
            Ok(term) => {
                info!(commande = %argv.join(" "), "terminal lance");
                *slot = Some(term);
            }
            Err(error) => {
                warn!(%error, "terminal non lance");
                return None;
            }
        }
    }
    slot.clone()
}

/// Traite une route du terminal. La reponse est binaire : 8 octets (position suivante, grand-boutiste),
/// 1 octet (1 si la commande est terminee), puis la sortie.
pub fn handle(route: &str, body: &[u8]) -> Vec<u8> {
    let (name, query) = route.split_once('?').unwrap_or((route, ""));
    match name {
        "read" => {
            let Some(term) = current(query_size(query), false) else { return Vec::new() };
            let chunk = term.read_since(query_u64(query, "since").unwrap_or(0));
            let mut out = Vec::with_capacity(chunk.data.len() + 9);
            out.extend_from_slice(&chunk.next.to_be_bytes());
            out.push(u8::from(chunk.closed));
            out.extend_from_slice(&chunk.data);
            out
        }
        "write" => {
            if let Some(term) = current(query_size(query), true) {
                term.write(body);
            }
            Vec::new()
        }
        "resize" => {
            let (cols, rows) = query_size(query);
            if let Some(term) = current((cols, rows), false) {
                term.resize(cols, rows);
            }
            Vec::new()
        }
        _ => Vec::new(),
    }
}
