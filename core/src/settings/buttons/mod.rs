//! Physical buttons, their actions and their keyboard shortcuts.

mod button;
mod button_action;
mod shortcut;

pub use button::Button;
pub use button_action::{ButtonAction, DpiAction, MouseAction, WheelAction};
pub use shortcut::{Shortcut, ShortcutKey};
