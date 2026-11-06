//! The 2-byte lift-off EEPROM item.

use std::ops::RangeInclusive;

use crate::protocol::{ProtocolError, check};

const VALUE: usize = 0;
/// The stored byte is 1 for 0.7 mm, so tenths of a millimetre are the byte plus 6.
const BYTE_TO_TENTHS: u8 = 6;

/// The lift-off height, kept as raw bytes so that a bad check survives until it is replaced.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LiftOff {
    raw: [u8; 2],
}

impl LiftOff {
    /// The heights ATK HUB offers, in tenths of a millimetre.
    pub const TENTHS_RANGE: RangeInclusive<u8> = 7..=17;

    /// Wraps bytes read from the mouse. Nothing is validated here: see the getter.
    #[must_use]
    pub fn from_raw(raw: [u8; 2]) -> Self {
        Self { raw }
    }

    /// Returns the height in tenths of a millimetre, also outside 7..=17, or `None` for a
    /// bad check or a byte that does not fit.
    #[must_use]
    pub fn millimetres_tenths(&self) -> Option<u8> {
        check::verified(&self.raw, VALUE)
            .ok()?
            .checked_add(BYTE_TO_TENTHS)
    }

    /// Returns the item with another height and its check byte.
    ///
    /// # Errors
    ///
    /// Returns `InvalidValue` when `tenths` is outside 7..=17 or the stored height cannot be
    /// read.
    pub fn with_tenths(mut self, tenths: u8) -> Result<Self, ProtocolError> {
        if !Self::TENTHS_RANGE.contains(&tenths) {
            return Err(ProtocolError::InvalidValue("lift-off height"));
        }
        if self.millimetres_tenths().is_none() {
            return Err(ProtocolError::InvalidValue("stored lift-off height"));
        }

        check::replace(&mut self.raw, VALUE, tenths - BYTE_TO_TENTHS)?;

        Ok(self)
    }

    /// Returns the raw bytes to write back.
    #[must_use]
    pub fn as_bytes(&self) -> &[u8; 2] {
        &self.raw
    }
}

#[cfg(test)]
mod tests;
