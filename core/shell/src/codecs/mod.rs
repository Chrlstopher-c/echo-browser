//! Decodeurs video : quel `libffmpeg.so` le moteur a charge, installation a la demande du decodeur complet (H.264/AAC)
//! depuis un tiers pour la release publique, et honnetete des annonces quand seul le decodeur libre est la.

mod loader;
mod pack;

use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::OnceLock;

use echo_contract::{CoreEvent, VideoCodecsStatus, VideoCodecsView};
use parking_lot::Mutex;
use tracing::{info, warn};

pub use loader::adopt_installed;

const AAC_MARK: &[u8] = b"AAC decoder\0";

/// Avec le decodeur libre, l'AAC est annonce par le moteur mais jamais decodable (aucun chemin materiel) : on le retire
/// des annonces pour que les sites basculent sur Opus.
const SHIM: &str = "(()=>{const no=t=>/mp4a|aac/i.test(String(t));\
const cp=HTMLMediaElement.prototype.canPlayType;\
HTMLMediaElement.prototype.canPlayType=function(t){return no(t)?'':cp.call(this,t)};\
if(self.MediaSource){const s=MediaSource.isTypeSupported.bind(MediaSource);MediaSource.isTypeSupported=t=>!no(t)&&s(t)}\
const mc=navigator.mediaCapabilities;if(mc){const d=mc.decodingInfo.bind(mc);\
mc.decodingInfo=c=>no(c&&c.audio&&c.audio.contentType)\
?Promise.resolve({supported:false,smooth:false,powerEfficient:false}):d(c)}})()";

static DOWNLOADING: AtomicBool = AtomicBool::new(false);
static LAST_ERROR: Mutex<Option<String>> = Mutex::new(None);

/// Le decodeur charge par ce processus.
struct Loaded {
    path: PathBuf,
    full: bool,
}

fn loaded() -> Option<&'static Loaded> {
    static LOADED: OnceLock<Option<Loaded>> = OnceLock::new();
    LOADED.get_or_init(detect).as_ref()
}

fn detect() -> Option<Loaded> {
    let maps = std::fs::read_to_string("/proc/self/maps").ok()?;
    let path = maps.lines().filter_map(|l| l.split_whitespace().nth(5)).find(|p| p.ends_with("/libffmpeg.so"))?;
    match std::fs::read(path) {
        Ok(bytes) => {
            let full = bytes.windows(AAC_MARK.len()).any(|w| w == AAC_MARK);
            info!(%path, full, "decodeur video charge");
            Some(Loaded { path: PathBuf::from(path), full })
        }
        Err(err) => {
            warn!(%err, %path, "decodeur video illisible");
            None
        }
    }
}

/// Script a injecter dans les pages quand le decodeur charge ne sait pas lire l'AAC ; `None` sinon.
pub fn shim() -> Option<&'static str> {
    loaded().filter(|l| !l.full).map(|_| SHIM)
}

fn status() -> VideoCodecsStatus {
    let Some(loaded) = loaded() else { return VideoCodecsStatus::Unavailable };
    let from_pack = loaded.path.starts_with(pack::dir());
    match (DOWNLOADING.load(Ordering::SeqCst), loaded.full, from_pack, pack::installed()) {
        (true, ..) => VideoCodecsStatus::Downloading,
        (_, true, true, true) => VideoCodecsStatus::Active,
        (_, true, true, false) => VideoCodecsStatus::PendingRemoval,
        (_, true, false, _) => VideoCodecsStatus::BuiltIn,
        (_, false, _, true) => VideoCodecsStatus::PendingRestart,
        (_, false, _, false) => VideoCodecsStatus::Missing,
    }
}

/// Diffuse l'etat des decodeurs a l'interface.
pub fn publish() {
    let codecs = VideoCodecsView { status: status(), source: pack::SOURCE.to_string(), error: LAST_ERROR.lock().clone() };
    crate::bridge::publish(&CoreEvent::VideoCodecsChanged { codecs });
}

/// Telecharge le decodeur complet hors du thread interface ; actif a la prochaine relance.
pub fn install() {
    if DOWNLOADING.swap(true, Ordering::SeqCst) {
        return;
    }
    *LAST_ERROR.lock() = None;
    publish();
    std::thread::spawn(|| {
        let result = pack::install();
        if let Err(err) = &result {
            warn!(%err, "decodeur complet non installe");
        }
        crate::containers::later(move || {
            DOWNLOADING.store(false, Ordering::SeqCst);
            *LAST_ERROR.lock() = result.err();
            publish();
        });
    });
}

/// Retire le decodeur complet ; le decodeur libre reprend a la prochaine relance.
pub fn remove() {
    *LAST_ERROR.lock() = pack::remove().err();
    publish();
}
