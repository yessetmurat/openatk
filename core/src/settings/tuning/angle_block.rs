//! The 4-byte sensor angle EEPROM block: angle and switch.

use std::ops::RangeInclusive;

use crate::protocol::{ProtocolError, check};

const ANGLE: usize = 0;
const SWITCH: usize = 2;

/// The angle block, kept as raw bytes so that a bad pair survives a write to the other one.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AngleBlock {
    raw: [u8; 4],
}

impl AngleBlock {
    /// The angles ATK HUB offers, in degrees.
    pub const ANGLE_RANGE: RangeInclusive<i8> = -30..=30;

    /// Wraps bytes read from the mouse. Nothing is validated here: see the getters.
    #[must_use]
    pub fn from_raw(raw: [u8; 4]) -> Self {
        Self { raw }
    }

    /// Returns the angle in degrees, also outside -30..=30, or `None` for a bad check.
    #[must_use]
    pub fn angle(&self) -> Option<i8> {
        let byte = check::verified(&self.raw, ANGLE).ok()?;

        Some(byte as i8)
    }

    /// Returns whether the angle is applied, or `None` when its pair cannot be read.
    #[must_use]
    pub fn enabled(&self) -> Option<bool> {
        check::verified_flag(&self.raw, SWITCH).ok()
    }

    /// Returns the block with another angle and its check byte.
    ///
    /// # Errors
    ///
    /// Returns `InvalidValue` when `degrees` is outside -30..=30 and `BadCheck` when the
    /// stored pair is corrupt.
    pub fn with_angle(mut self, degrees: i8) -> Result<Self, ProtocolError> {
        if !Self::ANGLE_RANGE.contains(&degrees) {
            return Err(ProtocolError::InvalidValue("angle"));
        }

        check::replace(&mut self.raw, ANGLE, degrees as u8)?;

        Ok(self)
    }

    /// Returns the block with the switch on or off and its check byte.
    ///
    /// # Errors
    ///
    /// Returns `BadCheck` when the stored pair is corrupt and `InvalidValue` when it holds
    /// neither 0 nor 1.
    pub fn with_enabled(mut self, on: bool) -> Result<Self, ProtocolError> {
        check::replace_flag(&mut self.raw, SWITCH, on)?;

        Ok(self)
    }

    /// Returns the raw bytes to write back.
    #[must_use]
    pub fn as_bytes(&self) -> &[u8; 4] {
        &self.raw
    }
}

#[cfg(test)]
mod tests;
