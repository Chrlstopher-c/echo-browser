//! Responsabilite : glisser le traitement du bouclier dans le flux HTML de la page,
//! avant que le moteur de rendu n'en voie la moindre ligne.
//!
//! C'est le seul moment reellement sur. Injecte par appel de fonction apres le debut
//! du chargement, le traitement arrive alors que sept scripts du site sont deja poses
//! — mesure sur YouTube le 2026-09-10. Place dans le document, il s'execute en premier.

// Les macros `wrap_*` de CEF exigent les traits `Impl*` et `Wrap*` dans la portee : import global impose.
use cef::*;
use std::cell::RefCell;
use std::collections::VecDeque;
use std::rc::Rc as StdRc;
use tracing::debug;

/// Au-dela de cette quantite lue sans avoir trouve ou inserer, on renonce a chercher
/// une balise d'accueil et on insere au plus tot.
const SEARCH_LIMIT: usize = 96 * 1024;

/// Quantite au-dela de laquelle on renonce a trouver un jeton et on insere sans.
/// Une page qui n'en utilise pas accepte les scripts en ligne de toute facon.
const NONCE_PATIENCE: usize = 12 * 1024;

/// Etat du filtre, partage entre les clones que CEF fabrique.
#[derive(Default)]
struct State {
    /// Octets prets a sortir, dans l'ordre.
    outbox: VecDeque<u8>,
    /// Octets lus en attendant de trouver ou inserer.
    scanned: Vec<u8>,
    done: bool,
}

/// Cree le filtre pour une page donnee.
pub fn new_filter(script: String) -> ResponseFilter {
    HtmlInjector::new(script, StdRc::new(RefCell::new(State::default())))
}

wrap_response_filter! {
    struct HtmlInjector {
        script: String,
        state: StdRc<RefCell<State>>,
    }

    impl ResponseFilter {
        fn init_filter(&self) -> i32 {
            1
        }

        fn filter(
            &self,
            data_in: Option<&mut Vec<u8>>,
            data_in_read: Option<&mut usize>,
            data_out: Option<&mut Vec<u8>>,
            data_out_written: Option<&mut usize>,
        ) -> ResponseFilterStatus {
            let incoming: Vec<u8> = data_in.map(std::mem::take).unwrap_or_default();
            if let Some(read) = data_in_read {
                *read = incoming.len();
            }
            let last_chunk = incoming.is_empty();

            self.absorb(incoming, last_chunk);

            let Some(out) = data_out else {
                return ResponseFilterStatus::ERROR;
            };
            let written = self.drain_into(out);
            if let Some(count) = data_out_written {
                *count = written;
            }

            let empty = self.state.borrow().outbox.is_empty();
            if last_chunk && empty {
                ResponseFilterStatus::DONE
            } else {
                ResponseFilterStatus::NEED_MORE_DATA
            }
        }
    }
}

impl HtmlInjector {
    /// Range les octets recus, en inserant le traitement des qu'un point d'accueil apparait.
    fn absorb(&self, incoming: Vec<u8>, last_chunk: bool) {
        let mut state = self.state.borrow_mut();
        if state.done {
            state.outbox.extend(incoming);
            return;
        }
        state.scanned.extend_from_slice(&incoming);

        // Rien ne s'affiche tant que le flux est retenu : on part des qu'on a de quoi
        // travailler — une balise d'accueil et le jeton de la page — sans attendre le
        // plafond. Le jeton apparait des les premiers kilo-octets.
        let reached_limit = state.scanned.len() >= SEARCH_LIMIT || last_chunk;
        let ready = insertion_point(&state.scanned).is_some()
            && (page_nonce(&state.scanned).is_some() || state.scanned.len() >= NONCE_PATIENCE);
        if !ready && !reached_limit {
            return;
        }
        let Some(_) = insertion_point(&state.scanned).or(reached_limit.then_some(0)) else {
            return;
        };

        // La page peut porter sa politique de securite dans une balise du document, que
        // les en-tetes ne montrent pas. Elle refuse alors le traitement. On la desamorce
        // en la renommant : la balise reste, le navigateur ne la reconnait plus.
        let mut scanned = std::mem::take(&mut state.scanned);
        let disarmed = disarm_meta_policy(&mut scanned);
        let at = insertion_point(&scanned).unwrap_or(0);

        // Le document autorise ses propres scripts par un jeton a usage unique. En le
        // reprenant, le traitement est admis sans toucher a la politique de la page.
        let nonce = page_nonce(&scanned);
        let attribute = nonce.as_deref().map(|n| format!(" nonce=\"{n}\"")).unwrap_or_default();
        let payload = format!(
            "<script{attribute}>{}</script>",
            self.script
        );
        debug!(
            position = at,
            octets = payload.len(),
            retenu = scanned.len(),
            jeton = nonce.is_some(),
            politiques_desamorcees = disarmed,
            "traitement insere dans le document"
        );
        state.outbox.extend(&scanned[..at]);
        state.outbox.extend(payload.as_bytes());
        state.outbox.extend(&scanned[at..]);
        state.done = true;
    }

    /// Verse ce qui tient dans le tampon de sortie et renvoie le nombre d'octets ecrits.
    fn drain_into(&self, out: &mut Vec<u8>) -> usize {
        let capacity = out.capacity().max(out.len());
        out.clear();
        let mut state = self.state.borrow_mut();
        let count = capacity.min(state.outbox.len());
        out.extend(state.outbox.drain(..count));
        count
    }
}

/// Renomme les balises de politique de securite du document. Renvoie combien ont ete
/// touchees. Le remplacement garde la meme longueur pour ne pas decaler le document.
fn disarm_meta_policy(buffer: &mut [u8]) -> usize {
    const NEEDLE: &[u8] = b"content-security-policy";
    const REPLACEMENT: &[u8] = b"x-desamorce-par-echo---";
    debug_assert_eq!(NEEDLE.len(), REPLACEMENT.len());

    let lower: Vec<u8> = buffer.iter().map(u8::to_ascii_lowercase).collect();
    let mut count = 0;
    let mut from = 0;
    while let Some(found) = find(&lower[from..], NEEDLE) {
        let at = from + found;
        buffer[at..at + NEEDLE.len()].copy_from_slice(REPLACEMENT);
        count += 1;
        from = at + NEEDLE.len();
    }
    count
}

/// Recupere le jeton que la page attache a ses propres scripts, s'il y en a un.
fn page_nonce(buffer: &[u8]) -> Option<String> {
    let text = String::from_utf8_lossy(buffer);
    let lower = text.to_ascii_lowercase();
    let at = lower.find("nonce=\"")? + "nonce=\"".len();
    let rest = &text[at..];
    let end = rest.find('"')?;
    let value = &rest[..end];
    let acceptable = !value.is_empty()
        && value.len() <= 128
        && value.chars().all(|c| c.is_ascii_alphanumeric() || "+/=_-".contains(c));
    acceptable.then(|| value.to_string())
}

fn find(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    haystack.windows(needle.len()).position(|window| window == needle)
}

/// Ou poser le traitement : juste apres l'ouverture de `<head>`, sinon apres `<html>`.
/// Jamais avant le doctype, qui basculerait la page en mode de compatibilite.
fn insertion_point(buffer: &[u8]) -> Option<usize> {
    let text = String::from_utf8_lossy(buffer).to_ascii_lowercase();
    for tag in ["<head", "<html"] {
        if let Some(start) = text.find(tag) {
            if let Some(end) = text[start..].find('>') {
                return Some(start + end + 1);
            }
        }
    }
    None
}
