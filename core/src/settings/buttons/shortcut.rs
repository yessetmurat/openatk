//! Keyboard shortcut stored for a button, and its EEPROM frames.

use crate::protocol::ProtocolError;
use crate::protocol::check;

const TYPE_MODIFIER: u8 = 0;
const TYPE_GENERAL: u8 = 1;
const KEY_DOWN: u8 = 0x80;
const KEY_UP: u8 = 0x40;
const MAX_ENTRIES: u8 = 6;
const FRAME_LEN: usize = 10;

/// A key of a shortcut.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShortcutKey {
    /// A modifier, as a mask for HID usages 224-231 (1, 2, 4 ... 128).
    Modifier(u8),
    /// A general key, as a HID usage.
    General(u8),
}

impl ShortcutKey {
    fn entry(self, state: u8) -> [u8; 3] {
        match self {
            Self::Modifier(mask) => [state | TYPE_MODIFIER, mask, 0],
            Self::General(usage) => [state | TYPE_GENERAL, usage, 0],
        }
    }

    fn from_entry(entry: [u8; 3]) -> Option<Self> {
        match entry[0] & 0x07 {
            TYPE_MODIFIER => Some(Self::Modifier(entry[1])),
            TYPE_GENERAL => Some(Self::General(entry[1])),
            _ => None,
        }
    }
}

/// A shortcut as stored in the mouse: raw 3-byte entries, kept untouched when unknown.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Shortcut {
    entries: Vec<[u8; 3]>,
}

impl Shortcut {
    /// Builds a shortcut from one key of either kind, or one or two modifiers followed by one
    /// general key.
    ///
    /// # Errors
    ///
    /// Returns `InvalidValue` for any other list of keys.
    pub fn from_keys(keys: &[ShortcutKey]) -> Result<Self, ProtocolError> {
        let valid = match keys {
            [_] => true,
            [rest @ .., ShortcutKey::General(_)] => {
                matches!(rest.len(), 1 | 2)
                    && rest
                        .iter()
                        .all(|key| matches!(key, ShortcutKey::Modifier(_)))
            }
            _ => false,
        };
        if !valid {
            return Err(ProtocolError::InvalidValue("shortcut keys"));
        }

        let downs = keys.iter().map(|key| key.entry(KEY_DOWN));
        let ups = keys.iter().map(|key| key.entry(KEY_UP));

        Ok(Self {
            entries: downs.chain(ups).collect(),
        })
    }

    /// Returns the keys when the entries have a shape `from_keys` builds, otherwise `None`.
    #[must_use]
    pub fn keys(&self) -> Option<Vec<ShortcutKey>> {
        let half = self.entries.len() / 2;
        let keys = self.entries[..half]
            .iter()
            .map(|&entry| ShortcutKey::from_entry(entry))
            .collect::<Option<Vec<_>>>()?;

        let rebuilt = Self::from_keys(&keys).ok()?;

        (rebuilt == *self).then_some(keys)
    }

    /// Returns the frames to write: one when the payload fits in 9 bytes, otherwise two.
    #[must_use]
    pub fn to_frames(&self) -> Vec<[u8; 10]> {
        let count = u8::try_from(self.entries.len()).expect("at most 6 entries");
        let mut stored = vec![count];
        stored.extend(self.entries.iter().flatten());
        stored.push(check::sum(&stored));

        let mut first = [0u8; FRAME_LEN];
        let mut second = [0u8; FRAME_LEN];
        for (index, &byte) in stored.iter().enumerate() {
            if index < FRAME_LEN {
                first[index] = byte;
            } else {
                second[index - FRAME_LEN] = byte;
            }
        }

        // Payload (entries and check) of at most 9 bytes fits in frame 1 beside the count.
        if stored.len() <= FRAME_LEN {
            vec![first]
        } else {
            vec![first, second]
        }
    }

    /// Reads a shortcut from frame 1 and frame 2 joined. `None` means no shortcut is set.
    ///
    /// # Errors
    ///
    /// Returns `BadCheck` when the stored check byte does not match.
    pub fn from_bytes(bytes: &[u8; 20]) -> Result<Option<Self>, ProtocolError> {
        let count = bytes[0];
        if count == 0 || count > MAX_ENTRIES {
            return Ok(None);
        }

        let end = 1 + 3 * usize::from(count);
        if bytes[end] != check::sum(&bytes[..end]) {
            return Err(ProtocolError::BadCheck { offset: end });
        }

        let (chunks, _) = bytes[1..end].as_chunks::<3>();
        let entries = chunks.to_vec();

        Ok(Some(Self { entries }))
    }
}

#[cfg(test)]
mod tests;
