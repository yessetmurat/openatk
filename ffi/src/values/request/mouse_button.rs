//! A physical button of the mouse.

use openatk_core::settings::Button;

/// A physical button.
#[derive(Debug, Clone, Copy, PartialEq, Eq, uniffi::Enum)]
pub enum MouseButton {
    /// Left button.
    Left,
    /// Right button.
    Right,
    /// Wheel button.
    Middle,
    /// First side button.
    Side1,
    /// Second side button.
    Side2,
}

impl From<Button> for MouseButton {
    fn from(button: Button) -> Self {
        match button {
            Button::Left => Self::Left,
            Button::Right => Self::Right,
            Button::Middle => Self::Middle,
            Button::Side1 => Self::Side1,
            Button::Side2 => Self::Side2,
        }
    }
}

impl From<MouseButton> for Button {
    fn from(button: MouseButton) -> Self {
        match button {
            MouseButton::Left => Self::Left,
            MouseButton::Right => Self::Right,
            MouseButton::Middle => Self::Middle,
            MouseButton::Side1 => Self::Side1,
            MouseButton::Side2 => Self::Side2,
        }
    }
}
