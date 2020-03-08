use crossterm::event::KeyModifiers;

pub const NONE_MODIFIER: KeyModifiers = KeyModifiers::empty();

#[macro_use]
macro_rules! key {
    // Control + Key
    (^$($key:tt)*) => {
        crossterm::event::KeyEvent {
            code: crossterm::event::KeyCode::$($key)*,
            modifiers: crossterm::event::KeyModifiers::CONTROL,
        }
    };
    // Key
    ($key:literal) => {
        crossterm::event::KeyEvent {
            code: crossterm::event::KeyCode::Char($key),
            modifiers: crate::macros::NONE_MODIFIER,
        }
    };
    ($($key:tt)+) => {
        crossterm::event::KeyEvent {
            code: crossterm::event::KeyCode::$($key)+,
            modifiers: crate::macros::NONE_MODIFIER,
        }
    };
}
