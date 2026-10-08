//! Responsabilite : les raccourcis clavier du navigateur, interceptes avant la page.
//!
//! Une page web peut capter les memes combinaisons : elles sont donc traitees en amont,
//! puis consommees, pour qu'un site ne puisse pas confisquer Ctrl+T ou Ctrl+W.

// Les macros `wrap_*` de CEF exigent les traits `Impl*` et `Wrap*` dans la portee : import global impose.
use cef::*;
use tracing::debug;

/// Codes de touches Windows, seule table que Chromium expose de facon portable. Les lettres, elles, sont lues dans le
/// caractere produit par la disposition du clavier (voir `letter`) : en AZERTY, le code suit souvent la position US,
/// et Ctrl+W devenait Ctrl+Z.
mod key {
    pub const TAB: i32 = 0x09;
    pub const ESCAPE: i32 = 0x1B;
    pub const PAGE_UP: i32 = 0x21;
    pub const PAGE_DOWN: i32 = 0x22;
    pub const LEFT: i32 = 0x25;
    pub const RIGHT: i32 = 0x27;
    pub const DIGIT_0: i32 = 0x30;
    pub const DIGIT_1: i32 = 0x31;
    pub const DIGIT_9: i32 = 0x39;
    pub const NUMPAD_0: i32 = 0x60;
    pub const ADD: i32 = 0x6B;
    pub const SUBTRACT: i32 = 0x6D;
    pub const F1: i32 = 0x70;
    pub const F5: i32 = 0x74;
    pub const F11: i32 = 0x7A;
    pub const F12: i32 = 0x7B;
    pub const OEM_PLUS: i32 = 0xBB;
    pub const OEM_MINUS: i32 = 0xBD;
}

const MOD_SHIFT: u32 = 1 << 1;
const MOD_CTRL: u32 = 1 << 2;
const MOD_ALT: u32 = 1 << 3;

/// Ce que le navigateur doit faire d'une combinaison reconnue.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    NewTab,
    CloseTab,
    ReopenTab,
    NextTab,
    PreviousTab,
    SelectTab(usize),
    FocusAddress,
    Reload { bypass_cache: bool },
    ToggleFullscreen,
    ToggleDevTools,
    /// Referme ce qui est pose au-dessus de la page : fenetre d'extension, menu.
    DismissOverlay,
    OpenFile,
    Bookmark,
    Library,
    Print,
    SavePage,
    ViewSource,
    Back,
    Forward,
    Zoom(ZoomStep),
    Help,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ZoomStep {
    In,
    Out,
    Reset,
}

/// La lettre tapee : le caractere sans modificateur s'il en est une (un caractere de controle Ctrl+lettre, 1 a 26,
/// compte aussi), sinon la lettre du code de touche.
fn letter(code: i32, unmodified: u16) -> Option<char> {
    let control = (1..=26).contains(&unmodified).then(|| char::from(b'a' + (unmodified - 1) as u8));
    char::from_u32(u32::from(unmodified))
        .filter(char::is_ascii_alphabetic)
        .map(|c| c.to_ascii_lowercase())
        .or(control)
        .or_else(|| (0x41..=0x5A).contains(&code).then(|| char::from(b'a' + (code - 0x41) as u8)))
}

/// Traduit une frappe en action, ou `None` si elle appartient a la page.
pub fn resolve(code: i32, unmodified: u16, modifiers: u32) -> Option<Action> {
    let ctrl = modifiers & MOD_CTRL != 0;
    let shift = modifiers & MOD_SHIFT != 0;
    let alt = modifiers & MOD_ALT != 0;
    if let Some(action) = zoom(code, unmodified, ctrl && !alt) {
        return Some(action);
    }
    if let (true, false, Some(l)) = (ctrl, alt, letter(code, unmodified)) {
        return ctrl_letter(l, shift);
    }
    match (ctrl, shift, alt, code) {
        (true, false, false, key::TAB) | (true, false, false, key::PAGE_DOWN) => Some(Action::NextTab),
        (true, true, false, key::TAB) | (true, false, false, key::PAGE_UP) => Some(Action::PreviousTab),
        (false, false, true, key::LEFT) => Some(Action::Back),
        (false, false, true, key::RIGHT) => Some(Action::Forward),
        (false, false, false, key::F1) => Some(Action::Help),
        (false, false, false, key::F5) => Some(Action::Reload { bypass_cache: false }),
        (false, false, false, key::F12) => Some(Action::ToggleDevTools),
        (false, false, false, key::F11) => Some(Action::ToggleFullscreen),
        (false, false, false, key::ESCAPE) => Some(Action::DismissOverlay),
        (true, false, false, code) if (key::DIGIT_1..=key::DIGIT_9).contains(&code) => {
            Some(Action::SelectTab((code - key::DIGIT_1) as usize))
        }
        _ => None,
    }
}

fn ctrl_letter(letter: char, shift: bool) -> Option<Action> {
    Some(match (letter, shift) {
        ('t', false) => Action::NewTab,
        ('t', true) => Action::ReopenTab,
        ('w', false) => Action::CloseTab,
        ('l', false) => Action::FocusAddress,
        ('r', false) => Action::Reload { bypass_cache: false },
        ('r', true) => Action::Reload { bypass_cache: true },
        ('i' | 'j', true) => Action::ToggleDevTools,
        ('o', false) => Action::OpenFile,
        ('d', false) => Action::Bookmark,
        ('h' | 'j', false) => Action::Library,
        ('p', false) => Action::Print,
        ('s', false) => Action::SavePage,
        ('u', false) => Action::ViewSource,
        _ => return None,
    })
}

/// Ctrl + / Ctrl - / Ctrl 0, par le caractere (AZERTY : « - » est sur le 6) ou par les touches du pave.
fn zoom(code: i32, unmodified: u16, ctrl: bool) -> Option<Action> {
    if !ctrl {
        return None;
    }
    let ch = char::from_u32(u32::from(unmodified));
    match (ch, code) {
        (Some('+' | '='), _) | (_, key::OEM_PLUS | key::ADD) => Some(Action::Zoom(ZoomStep::In)),
        (Some('-'), _) | (_, key::OEM_MINUS | key::SUBTRACT) => Some(Action::Zoom(ZoomStep::Out)),
        (Some('0' | 'à'), _) | (_, key::NUMPAD_0) => Some(Action::Zoom(ZoomStep::Reset)),
        (_, key::DIGIT_0) if ch.is_none_or(|c| c == '\0') => Some(Action::Zoom(ZoomStep::Reset)),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const CTRL: u32 = MOD_CTRL;

    #[test]
    fn azerty_la_lettre_tapee_compte_pas_la_position() {
        // AZERTY : la touche W est a la place du Z americain ; le code peut dire Z, le caractere dit w.
        assert_eq!(resolve(0x5A, u16::from(b'w'), CTRL), Some(Action::CloseTab));
        assert_eq!(resolve(0x57, u16::from(b'z'), CTRL), None, "Ctrl+Z reste a la page (annuler)");
        assert_eq!(resolve(0x4F, u16::from(b'o'), CTRL), Some(Action::OpenFile));
        assert_eq!(resolve(0x54, 0, CTRL | MOD_SHIFT), Some(Action::ReopenTab), "sans caractere : le code");
        assert_eq!(resolve(0x57, 23, CTRL), Some(Action::CloseTab), "caractere de controle Ctrl+W");
    }

    #[test]
    fn zoom_et_onglets() {
        assert_eq!(resolve(0x36, u16::from(b'-'), CTRL), Some(Action::Zoom(ZoomStep::Out)), "AZERTY : - sur le 6");
        assert_eq!(resolve(key::OEM_PLUS, u16::from(b'='), CTRL), Some(Action::Zoom(ZoomStep::In)));
        assert_eq!(resolve(0x32, u16::from('é' as u32 as u16), CTRL), Some(Action::SelectTab(1)));
        assert_eq!(resolve(key::PAGE_DOWN, 0, CTRL), Some(Action::NextTab));
        assert_eq!(resolve(key::LEFT, 0, MOD_ALT), Some(Action::Back));
        assert_eq!(resolve(0x41, u16::from(b'a'), 0), None, "une lettre seule appartient a la page");
    }
}

wrap_keyboard_handler! {
    pub struct BrowserShortcuts {
        marker: (),
    }

    impl KeyboardHandler {
        fn on_pre_key_event(
            &self,
            _browser: Option<&mut Browser>,
            event: Option<&KeyEvent>,
            _os_event: Option<&mut sys::XEvent>,
            _is_keyboard_shortcut: Option<&mut i32>,
        ) -> i32 {
            let Some(event) = event else { return 0 };
            if event.type_ != KeyEventType::RAWKEYDOWN {
                return 0;
            }
            let Some(action) = resolve(event.windows_key_code, event.unmodified_character, event.modifiers) else {
                debug!(code = event.windows_key_code, ch = event.unmodified_character, mods = event.modifiers, "touche");
                return 0;
            };
            // Echap appartient a la page tant que rien n'est pose au-dessus d'elle :
            // le confisquer casserait la fermeture des fenetres des sites.
            let rien_au_dessus =
                crate::overlay::open_popup_id().is_none() && !crate::overlay::menu_open();
            if action == Action::DismissOverlay && rien_au_dessus {
                return 0;
            }
            debug!(?action, "raccourci");
            crate::bridge::perform(action);
            1
        }
    }
}
