//! Responsabilite : les raccourcis clavier du navigateur, interceptes avant la page.
//!
//! Une page web peut capter les memes combinaisons : elles sont donc traitees en amont,
//! puis consommees, pour qu'un site ne puisse pas confisquer Ctrl+T ou Ctrl+W.

// Les macros `wrap_*` de CEF exigent les traits `Impl*` et `Wrap*` dans la portee : import global impose.
use cef::*;
use tracing::debug;

/// Codes de touches Windows, seule table que Chromium expose de facon portable.
mod key {
    pub const TAB: i32 = 0x09;
    pub const ESCAPE: i32 = 0x1B;
    pub const F5: i32 = 0x74;
    pub const F11: i32 = 0x7A;
    pub const L: i32 = 0x4C;
    pub const T: i32 = 0x54;
    pub const W: i32 = 0x57;
    pub const R: i32 = 0x52;
    pub const DIGIT_1: i32 = 0x31;
    pub const DIGIT_9: i32 = 0x39;
}

const MOD_SHIFT: u32 = 1 << 1;
const MOD_CTRL: u32 = 1 << 2;

/// Ce que le navigateur doit faire d'une combinaison reconnue.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    NewTab,
    CloseTab,
    NextTab,
    PreviousTab,
    SelectTab(usize),
    FocusAddress,
    Reload { bypass_cache: bool },
    ToggleFullscreen,
    /// Referme ce qui est pose au-dessus de la page : fenetre d'extension, menu.
    DismissOverlay,
}

/// Traduit une frappe en action, ou `None` si elle appartient a la page.
pub fn resolve(code: i32, modifiers: u32) -> Option<Action> {
    let ctrl = modifiers & MOD_CTRL != 0;
    let shift = modifiers & MOD_SHIFT != 0;

    match (ctrl, shift, code) {
        (true, false, key::T) => Some(Action::NewTab),
        (true, false, key::W) => Some(Action::CloseTab),
        (true, false, key::L) => Some(Action::FocusAddress),
        (true, false, key::TAB) => Some(Action::NextTab),
        (true, true, key::TAB) => Some(Action::PreviousTab),
        (true, false, key::R) => Some(Action::Reload { bypass_cache: false }),
        (true, true, key::R) => Some(Action::Reload { bypass_cache: true }),
        (false, false, key::F5) => Some(Action::Reload { bypass_cache: false }),
        (false, false, key::F11) => Some(Action::ToggleFullscreen),
        (false, false, key::ESCAPE) => Some(Action::DismissOverlay),
        (true, false, code) if (key::DIGIT_1..=key::DIGIT_9).contains(&code) => {
            Some(Action::SelectTab((code - key::DIGIT_1) as usize))
        }
        _ => None,
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
            let Some(action) = resolve(event.windows_key_code, event.modifiers) else {
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
