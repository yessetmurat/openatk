//! What a button does.

use openatk_core::settings::{ButtonAction, DpiAction, MouseAction, WheelAction};

/// What a button does.
#[derive(Debug, Clone, Copy, PartialEq, Eq, uniffi::Enum)]
pub enum Action {
    /// The button does nothing.
    Disabled,
    /// Acts as the left button.
    MouseLeft,
    /// Acts as the right button.
    MouseRight,
    /// Acts as the wheel button.
    MouseMiddle,
    /// Acts as the first side button.
    MouseSide1,
    /// Acts as the second side button.
    MouseSide2,
    /// Cycles through the DPI stages.
    DpiCycle,
    /// Raises the DPI stage.
    DpiPlus,
    /// Lowers the DPI stage.
    DpiMinus,
    /// Scrolls up.
    WheelUp,
    /// Scrolls down.
    WheelDown,
    /// Sends the stored shortcut.
    Shortcut,
    /// An action this app cannot edit; it keeps its bytes.
    Other {
        /// Action class byte.
        class: u8,
        /// First value byte.
        value1: u8,
        /// Second value byte.
        value2: u8,
    },
}

impl From<ButtonAction> for Action {
    fn from(action: ButtonAction) -> Self {
        match action {
            ButtonAction::Mouse(MouseAction::Disabled) => Self::Disabled,
            ButtonAction::Mouse(MouseAction::Left) => Self::MouseLeft,
            ButtonAction::Mouse(MouseAction::Right) => Self::MouseRight,
            ButtonAction::Mouse(MouseAction::Middle) => Self::MouseMiddle,
            ButtonAction::Mouse(MouseAction::Side1) => Self::MouseSide1,
            ButtonAction::Mouse(MouseAction::Side2) => Self::MouseSide2,
            ButtonAction::Dpi(DpiAction::Cycle) => Self::DpiCycle,
            ButtonAction::Dpi(DpiAction::Plus) => Self::DpiPlus,
            ButtonAction::Dpi(DpiAction::Minus) => Self::DpiMinus,
            ButtonAction::Wheel(WheelAction::Up) => Self::WheelUp,
            ButtonAction::Wheel(WheelAction::Down) => Self::WheelDown,
            ButtonAction::Shortcut => Self::Shortcut,
            ButtonAction::Other {
                class,
                value1,
                value2,
            } => Self::Other {
                class,
                value1,
                value2,
            },
        }
    }
}

impl Action {
    /// Returns the core action, or `None` for `Other`, which cannot be written.
    pub(crate) fn editable(self) -> Option<ButtonAction> {
        Some(match self {
            Self::Disabled => ButtonAction::Mouse(MouseAction::Disabled),
            Self::MouseLeft => ButtonAction::Mouse(MouseAction::Left),
            Self::MouseRight => ButtonAction::Mouse(MouseAction::Right),
            Self::MouseMiddle => ButtonAction::Mouse(MouseAction::Middle),
            Self::MouseSide1 => ButtonAction::Mouse(MouseAction::Side1),
            Self::MouseSide2 => ButtonAction::Mouse(MouseAction::Side2),
            Self::DpiCycle => ButtonAction::Dpi(DpiAction::Cycle),
            Self::DpiPlus => ButtonAction::Dpi(DpiAction::Plus),
            Self::DpiMinus => ButtonAction::Dpi(DpiAction::Minus),
            Self::WheelUp => ButtonAction::Wheel(WheelAction::Up),
            Self::WheelDown => ButtonAction::Wheel(WheelAction::Down),
            Self::Shortcut => ButtonAction::Shortcut,
            Self::Other { .. } => return None,
        })
    }
}

#[cfg(test)]
mod tests;
