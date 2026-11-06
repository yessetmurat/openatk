//! One key of a shortcut.

use openatk_core::settings::ShortcutKey;

/// One key of a shortcut.
#[derive(Debug, Clone, Copy, PartialEq, Eq, uniffi::Enum)]
pub enum ShortcutKeyValue {
    /// A modifier key, as a HID modifier mask.
    Modifier {
        /// HID modifier bit mask.
        mask: u8,
    },
    /// An ordinary key, as a HID usage id.
    General {
        /// HID keyboard usage id.
        usage: u8,
    },
}

impl From<ShortcutKey> for ShortcutKeyValue {
    fn from(key: ShortcutKey) -> Self {
        match key {
            ShortcutKey::Modifier(mask) => Self::Modifier { mask },
            ShortcutKey::General(usage) => Self::General { usage },
        }
    }
}

impl From<ShortcutKeyValue> for ShortcutKey {
    fn from(key: ShortcutKeyValue) -> Self {
        match key {
            ShortcutKeyValue::Modifier { mask } => Self::Modifier(mask),
            ShortcutKeyValue::General { usage } => Self::General(usage),
        }
    }
}
