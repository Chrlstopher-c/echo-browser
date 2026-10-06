//! Responsabilite : un pseudo-terminal qui survit a la page qui l'affiche.
//!
//! La sortie est gardee dans un tampon borne et numerotee par octet : une page qui se
//! recharge (onglet reveille) redemande depuis zero et rejoue l'ecran, une page vivante ne
//! demande que la suite.

use anyhow::{Context, Result};
use parking_lot::Mutex;
use portable_pty::{native_pty_system, CommandBuilder, MasterPty, PtySize};
use std::io::{Read, Write};
use std::sync::Arc;
use tracing::{debug, warn};

/// Octets de sortie gardes pour rejouer l'ecran.
const RING_CAPACITY: usize = 512 * 1024;

#[derive(Default)]
struct Ring {
    /// Numero du premier octet encore garde.
    base: u64,
    data: Vec<u8>,
    closed: bool,
}

impl Ring {
    fn push(&mut self, bytes: &[u8]) {
        self.data.extend_from_slice(bytes);
        if self.data.len() > RING_CAPACITY {
            let drop = self.data.len() - RING_CAPACITY;
            self.data.drain(..drop);
            self.base += drop as u64;
        }
    }

    fn next_seq(&self) -> u64 {
        self.base + self.data.len() as u64
    }
}

struct Inner {
    master: Mutex<Box<dyn MasterPty + Send>>,
    writer: Mutex<Box<dyn Write + Send>>,
    ring: Mutex<Ring>,
}

/// Un pseudo-terminal en cours. Se clone librement : tous les clones parlent au meme.
#[derive(Clone)]
pub struct Terminal {
    inner: Arc<Inner>,
}

/// Ce qu'une lecture rend : la suite de la sortie et la position a redemander.
pub struct Chunk {
    pub next: u64,
    pub data: Vec<u8>,
    /// Vrai quand la commande est terminee et que tout a ete lu.
    pub closed: bool,
}

impl Terminal {
    /// Lance `argv` dans un pseudo-terminal de `cols` x `rows` caracteres.
    pub fn spawn(argv: &[String], cols: u16, rows: u16) -> Result<Self> {
        let (program, args) = argv.split_first().context("commande vide")?;
        let pair = native_pty_system()
            .openpty(PtySize { rows, cols, pixel_width: 0, pixel_height: 0 })
            .context("ouverture du pseudo-terminal")?;
        let mut command = CommandBuilder::new(program);
        command.args(args);
        command.env("TERM", "xterm-256color");
        command.env("COLORTERM", "truecolor");
        let mut child = pair.slave.spawn_command(command).with_context(|| format!("lancement de {program}"))?;
        drop(pair.slave);
        let mut reader = pair.master.try_clone_reader().context("lecture du pseudo-terminal")?;
        let writer = pair.master.take_writer().context("ecriture du pseudo-terminal")?;
        let inner = Arc::new(Inner {
            master: Mutex::new(pair.master),
            writer: Mutex::new(writer),
            ring: Mutex::new(Ring::default()),
        });
        let sink = Arc::clone(&inner);
        std::thread::spawn(move || {
            let mut buffer = [0u8; 8192];
            loop {
                match reader.read(&mut buffer) {
                    Ok(0) | Err(_) => break,
                    Ok(count) => sink.ring.lock().push(&buffer[..count]),
                }
            }
            if let Err(error) = child.wait() {
                warn!(%error, "attente de la commande du terminal");
            }
            sink.ring.lock().closed = true;
            debug!("terminal ferme");
        });
        Ok(Self { inner })
    }

    /// La sortie depuis l'octet `since`. Un numero trop ancien est ramene au plus ancien garde.
    pub fn read_since(&self, since: u64) -> Chunk {
        let ring = self.inner.ring.lock();
        let from = since.clamp(ring.base, ring.next_seq());
        let offset = (from - ring.base) as usize;
        Chunk { next: ring.next_seq(), data: ring.data[offset..].to_vec(), closed: ring.closed }
    }

    /// Envoie la saisie de l'utilisateur a la commande.
    pub fn write(&self, bytes: &[u8]) {
        let mut writer = self.inner.writer.lock();
        if let Err(error) = writer.write_all(bytes).and_then(|()| writer.flush()) {
            warn!(%error, "ecriture au terminal impossible");
        }
    }

    pub fn resize(&self, cols: u16, rows: u16) {
        let size = PtySize { rows, cols, pixel_width: 0, pixel_height: 0 };
        if let Err(error) = self.inner.master.lock().resize(size) {
            warn!(%error, "redimensionnement du terminal impossible");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{Duration, Instant};

    fn wait_for(term: &Terminal, needle: &str) -> bool {
        let deadline = Instant::now() + Duration::from_secs(5);
        while Instant::now() < deadline {
            if String::from_utf8_lossy(&term.read_since(0).data).contains(needle) {
                return true;
            }
            std::thread::sleep(Duration::from_millis(20));
        }
        false
    }

    #[test]
    fn rend_la_sortie_de_la_commande() {
        let term = Terminal::spawn(&["sh".into(), "-c".into(), "echo bonjour".into()], 80, 24).unwrap();
        assert!(wait_for(&term, "bonjour"));
    }

    #[test]
    fn relaie_la_saisie() {
        let term = Terminal::spawn(&["cat".into()], 80, 24).unwrap();
        term.write(b"ping\n");
        assert!(wait_for(&term, "ping"));
    }

    #[test]
    fn reprend_apres_la_position_donnee() {
        let term = Terminal::spawn(&["sh".into(), "-c".into(), "echo un; sleep 0.3; echo deux".into()], 80, 24).unwrap();
        assert!(wait_for(&term, "un"));
        let first = term.read_since(0);
        assert!(wait_for(&term, "deux"));
        let rest = term.read_since(first.next);
        let text = String::from_utf8_lossy(&rest.data);
        assert!(text.contains("deux") && !text.contains("un\r"));
    }

    #[test]
    fn une_commande_inexistante_est_une_erreur() {
        assert!(Terminal::spawn(&["commande-qui-nexiste-pas-xyz".into()], 80, 24).is_err());
    }
}
