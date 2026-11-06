//! A button with its action and shortcut keys.

use crate::values::{Action, MouseButton, ShortcutKeyValue};

/// A button with its action and, for an editable shortcut, its keys.
#[derive(Debug, Clone, PartialEq, Eq, uniffi::Record)]
pub struct ButtonValue {
    /// The button.
    pub button: MouseButton,
    /// Its action.
    pub action: Action,
    /// The keys of the stored shortcut, when it has an editable shape.
    pub keys: Option<Vec<ShortcutKeyValue>>,
}
