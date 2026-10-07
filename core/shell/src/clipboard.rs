//! Responsabilite : mettre une image dans le presse-papiers du bureau, pour la coller ailleurs (Ctrl+V)
//! sans la telecharger. L'image est recuperee, convertie en PNG (le format que toutes les applications
//! acceptent), puis confiee a `wl-copy` (Wayland) ou `xclip` (X11), qui la servent ensuite.

use std::io::Write;
use std::process::{Command, Stdio};
use tracing::warn;

const USER_AGENT: &str = "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/154.0.0.0 Safari/537.36";
const MAX_BYTES: u64 = 40 * 1024 * 1024;

/// Copie l'image a cette adresse ; le travail se fait hors du fil principal, le resultat est annonce.
pub fn copy_image(url: &str) {
    let url = url.to_string();
    std::thread::spawn(move || {
        let message = match fetch_png(&url).and_then(|png| write_clipboard(&png)) {
            Ok(()) => "Image copiée.".to_string(),
            Err(error) => {
                warn!(%url, %error, "copie d'image impossible");
                format!("Copie impossible : {error}")
            }
        };
        crate::containers::later(move || {
            crate::bridge::publish(&echo_contract::CoreEvent::Notice {
                level: echo_contract::NoticeLevel::Info,
                message,
            });
        });
    });
}

fn fetch_png(url: &str) -> Result<Vec<u8>, String> {
    let bytes = if let Some(data) = url.strip_prefix("data:") {
        decode_data_url(data)?
    } else if let Some(path) = url.strip_prefix("file://") {
        std::fs::read(path).map_err(|e| e.to_string())?
    } else {
        let response = ureq::get(url).header("User-Agent", USER_AGENT).call().map_err(|e| e.to_string())?;
        response.into_body().with_config().limit(MAX_BYTES).read_to_vec().map_err(|e| e.to_string())?
    };
    let image = image::load_from_memory(&bytes).map_err(|e| format!("image illisible ({e})"))?;
    let mut png = Vec::new();
    image
        .write_to(&mut std::io::Cursor::new(&mut png), image::ImageFormat::Png)
        .map_err(|e| e.to_string())?;
    Ok(png)
}

fn decode_data_url(data: &str) -> Result<Vec<u8>, String> {
    use base64::Engine;
    let (meta, payload) = data.split_once(',').ok_or("adresse data: invalide")?;
    if meta.ends_with(";base64") {
        base64::engine::general_purpose::STANDARD.decode(payload).map_err(|e| e.to_string())
    } else {
        Ok(payload.as_bytes().to_vec())
    }
}

fn write_clipboard(png: &[u8]) -> Result<(), String> {
    let wayland = std::env::var_os("WAYLAND_DISPLAY").is_some();
    let (program, args): (&str, &[&str]) = if wayland {
        ("wl-copy", &["--type", "image/png"])
    } else {
        ("xclip", &["-selection", "clipboard", "-t", "image/png"])
    };
    let mut child = Command::new(program)
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|e| format!("{program} introuvable ({e})"))?;
    child.stdin.take().ok_or("entree fermee")?.write_all(png).map_err(|e| e.to_string())?;
    // wl-copy rend la main en se detachant pour servir le presse-papiers ; on attend seulement ce retour.
    child.wait().map_err(|e| e.to_string())?;
    Ok(())
}
