//! Responsabilite : arret demande par le systeme (SIGTERM a la fermeture de session ou par `stop.sh`, SIGINT, SIGHUP).
//! Chromium s'arretait seul, sans qu'Echo enregistre ses onglets : les derniers changements (dossier, epinglage…
//! enregistres par lot toutes les 1,2 s) etaient perdus. Notre gestionnaire passe devant : il ne fait qu'ecrire le
//! numero du signal dans un tube (seul geste sur dans un gestionnaire), un fil le lit, le fil de l'interface
//! enregistre, puis le gestionnaire de Chromium est remis et le signal relance : Chromium finit comme avant (etat
//! local, profils).

use std::sync::atomic::{AtomicI32, Ordering};

use cef::*;
use parking_lot::Mutex;
use tracing::{info, warn};

const SIGNALS: [libc::c_int; 3] = [libc::SIGTERM, libc::SIGINT, libc::SIGHUP];

static WRITE_END: AtomicI32 = AtomicI32::new(-1);
/// Gestionnaires d'origine (Chromium), remis avant de relancer le signal.
static PREVIOUS: Mutex<Vec<(libc::c_int, libc::sigaction)>> = Mutex::new(Vec::new());

extern "C" fn on_signal(signal: libc::c_int) {
    let fd = WRITE_END.load(Ordering::Relaxed);
    if fd >= 0 {
        let byte = u8::try_from(signal).unwrap_or(0);
        // SAFETY: write(2) est sur dans un gestionnaire de signal ; l'octet vit sur la pile.
        unsafe { libc::write(fd, std::ptr::addr_of!(byte).cast(), 1) };
    }
}

/// A appeler une fois, au demarrage du navigateur (apres Chromium, dont il garde le gestionnaire pour la fin).
pub fn watch() {
    let mut fds = [0 as libc::c_int; 2];
    // SAFETY: pipe(2) remplit les deux descripteurs du tableau fourni.
    if unsafe { libc::pipe(fds.as_mut_ptr()) } != 0 {
        warn!("signaux d'arret non suivis : un arret par le systeme perdra les derniers changements");
        return;
    }
    WRITE_END.store(fds[1], Ordering::Relaxed);
    for signal in SIGNALS {
        // SAFETY: structures remises a zero puis remplies ; le gestionnaire n'appelle que write(2).
        unsafe {
            let mut action: libc::sigaction = std::mem::zeroed();
            action.sa_sigaction = on_signal as extern "C" fn(libc::c_int) as libc::sighandler_t;
            libc::sigemptyset(&mut action.sa_mask);
            let mut previous: libc::sigaction = std::mem::zeroed();
            libc::sigaction(signal, &action, &mut previous);
            PREVIOUS.lock().push((signal, previous));
        }
    }
    let read_end = fds[0];
    std::thread::spawn(move || {
        let mut byte = 0u8;
        // SAFETY: lecture bloquante d'un octet dans un tampon de la pile.
        if unsafe { libc::read(read_end, std::ptr::addr_of_mut!(byte).cast(), 1) } == 1 {
            info!(signal = byte, "arret demande par le systeme : session enregistree avant de quitter");
            let mut task = QuitTask::new(libc::c_int::from(byte));
            post_task(ThreadId::UI, Some(&mut task));
        }
    });
}

wrap_task! {
    struct QuitTask {
        signal: libc::c_int,
    }

    impl Task {
        fn execute(&self) {
            crate::window::CLOSING.store(true, Ordering::Relaxed);
            crate::persist::flush();
            let previous = PREVIOUS.lock().iter().find(|(s, _)| *s == self.signal).map(|(_, a)| *a);
            // SAFETY: on remet l'action d'origine telle que sigaction(2) l'avait rendue, puis on relance le signal.
            unsafe {
                match previous {
                    Some(action) => libc::sigaction(self.signal, &action, std::ptr::null_mut()),
                    None => libc::signal(self.signal, libc::SIG_DFL) as libc::c_int,
                };
                libc::raise(self.signal);
            }
        }
    }
}
