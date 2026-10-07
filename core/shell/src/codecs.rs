//! Honnetete des codecs : avec le decodeur libre (release publique), l'AAC est annonce par le moteur mais
//! jamais decodable (aucun chemin materiel). On le retire des annonces pour que les sites basculent sur Opus.

use std::sync::OnceLock;

const AAC_MARK: &[u8] = b"AAC decoder\0";

const SHIM: &str = "(()=>{const no=t=>/mp4a|aac/i.test(String(t));\
const cp=HTMLMediaElement.prototype.canPlayType;\
HTMLMediaElement.prototype.canPlayType=function(t){return no(t)?'':cp.call(this,t)};\
if(self.MediaSource){const s=MediaSource.isTypeSupported.bind(MediaSource);MediaSource.isTypeSupported=t=>!no(t)&&s(t)}\
const mc=navigator.mediaCapabilities;if(mc){const d=mc.decodingInfo.bind(mc);\
mc.decodingInfo=c=>no(c&&c.audio&&c.audio.contentType)\
?Promise.resolve({supported:false,smooth:false,powerEfficient:false}):d(c)}})()";

/// Script a injecter dans les pages quand le decodeur charge ne sait pas lire l'AAC ; `None` sinon.
pub fn shim() -> Option<&'static str> {
    static LIBRE: OnceLock<bool> = OnceLock::new();
    LIBRE.get_or_init(detect_libre).then_some(SHIM)
}

fn detect_libre() -> bool {
    let Some(dir) = libcef_dir() else { return false };
    match std::fs::read(dir.join("libffmpeg.so")) {
        Ok(bytes) => {
            let libre = !bytes.windows(AAC_MARK.len()).any(|w| w == AAC_MARK);
            tracing::info!(libre, "decodeur video detecte");
            libre
        }
        Err(err) => {
            tracing::debug!(%err, "pas de libffmpeg.so separe (decodeur integre au moteur)");
            false
        }
    }
}

fn libcef_dir() -> Option<std::path::PathBuf> {
    let maps = std::fs::read_to_string("/proc/self/maps").ok()?;
    let path = maps.lines().filter_map(|l| l.split_whitespace().nth(5)).find(|p| p.ends_with("/libcef.so"))?;
    std::path::Path::new(path).parent().map(|p| p.to_path_buf())
}
