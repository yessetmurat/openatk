//! What a button does, and its 4-byte EEPROM encoding.

use crate::protocol::ProtocolError;
use crate::protocol::check;

const CLASS_MOUSE: u8 = 1;
const CLASS_DPI: u8 = 2;
const CLASS_SHORTCUT: u8 = 5;
const CLASS_WHEEL: u8 = 11;

/// A mouse button the action presses, or `Disabled`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MouseAction {
    /// The button does nothing.
    Disabled,
    /// Left click.
    Left,
    /// Right click.
    Right,
    /// Middle click.
    Middle,
    /// First side button.
    Side1,
    /// Second side button.
    Side2,
}

/// A DPI stage action.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DpiAction {
    /// Go to the next stage, wrapping around.
    Cycle,
    /// Go to the next higher stage.
    Plus,
    /// Go to the next lower stage.
    Minus,
}

/// A wheel action.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WheelAction {
    /// Scroll up.
    Up,
    /// Scroll down.
    Down,
}

/// The action assigned to a button.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ButtonAction {
    /// A mouse button press, or disabled.
    Mouse(MouseAction),
    /// A DPI stage change.
    Dpi(DpiAction),
    /// A wheel scroll.
    Wheel(WheelAction),
    /// A keyboard shortcut stored at the button's shortcut address.
    Shortcut,
    /// An action the app does not edit. Shown read-only and never written.
    Other {
        /// Class byte.
        class: u8,
        /// First value byte.
        value1: u8,
        /// Second value byte.
        value2: u8,
    },
}

impl ButtonAction {
    /// Returns the 4 EEPROM bytes of this action, or `None` for `Other`, which is never written.
    #[must_use]
    pub fn encode(self) -> Option<[u8; 4]> {
        let (class, value1) = match self {
            Self::Mouse(action) => (CLASS_MOUSE, mouse_value(action)),
            Self::Dpi(action) => (CLASS_DPI, dpi_value(action)),
            Self::Wheel(action) => (CLASS_WHEEL, wheel_value(action)),
            Self::Shortcut => (CLASS_SHORTCUT, 0),
            Self::Other { .. } => return None,
        };

        Some([class, value1, 0, check::sum(&[class, value1, 0])])
    }

    /// Reads an action from 4 EEPROM bytes. Classes and values the app does not know give
    /// `Other`.
    ///
    /// # Errors
    ///
    /// Returns `BadCheck` when the check byte does not match.
    pub fn decode(bytes: [u8; 4]) -> Result<Self, ProtocolError> {
        let [class, value1, value2, stored] = bytes;
        if stored != check::sum(&bytes[..3]) {
            return Err(ProtocolError::BadCheck { offset: 3 });
        }

        let known = match (class, value1, value2) {
            (CLASS_MOUSE, 0, 0) => Some(Self::Mouse(MouseAction::Disabled)),
            (CLASS_MOUSE, 1, 0) => Some(Self::Mouse(MouseAction::Left)),
            (CLASS_MOUSE, 2, 0) => Some(Self::Mouse(MouseAction::Right)),
            (CLASS_MOUSE, 4, 0) => Some(Self::Mouse(MouseAction::Middle)),
            (CLASS_MOUSE, 8, 0) => Some(Self::Mouse(MouseAction::Side1)),
            (CLASS_MOUSE, 16, 0) => Some(Self::Mouse(MouseAction::Side2)),
            (CLASS_DPI, 1, 0) => Some(Self::Dpi(DpiAction::Cycle)),
            (CLASS_DPI, 2, 0) => Some(Self::Dpi(DpiAction::Plus)),
            (CLASS_DPI, 3, 0) => Some(Self::Dpi(DpiAction::Minus)),
            (CLASS_WHEEL, 1, 0) => Some(Self::Wheel(WheelAction::Up)),
            (CLASS_WHEEL, 2, 0) => Some(Self::Wheel(WheelAction::Down)),
            (CLASS_SHORTCUT, 0, 0) => Some(Self::Shortcut),
            _ => None,
        };

        Ok(known.unwrap_or(Self::Other {
            class,
            value1,
            value2,
        }))
    }

    /// Returns whether the app may write this action.
    #[must_use]
    pub fn is_editable(self) -> bool {
        !matches!(self, Self::Other { .. })
    }
}

fn mouse_value(action: MouseAction) -> u8 {
    match action {
        MouseAction::Disabled => 0,
        MouseAction::Left => 1,
        MouseAction::Right => 2,
        MouseAction::Middle => 4,
        MouseAction::Side1 => 8,
        MouseAction::Side2 => 16,
    }
}

fn dpi_value(action: DpiAction) -> u8 {
    match action {
        DpiAction::Cycle => 1,
        DpiAction::Plus => 2,
        DpiAction::Minus => 3,
    }
}

fn wheel_value(action: WheelAction) -> u8 {
    match action {
        WheelAction::Up => 1,
        WheelAction::Down => 2,
    }
}

#[cfg(test)]
mod tests;
