//! The 10-byte sensor EEPROM block: five value and check pairs.

use crate::protocol::{ProtocolError, check};

const DEBOUNCE: usize = 0;
const MOTION_SYNC: usize = 2;
const LED_OFF: usize = 4;
const LINEAR_CORRECTION: usize = 6;
const RIPPLE_CONTROL: usize = 8;

/// The sensor block, kept as raw bytes so that a bad pair survives a write to another one.
///
/// Getters return `None` when their own pair cannot be read. A `with_` method refuses in
/// that case, and otherwise changes only its own pair.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SensorBlock {
    raw: [u8; 10],
}

impl SensorBlock {
    /// Wraps bytes read from the mouse. Nothing is validated here: see the getters.
    #[must_use]
    pub fn from_raw(raw: [u8; 10]) -> Self {
        Self { raw }
    }

    /// Returns the debounce time in milliseconds.
    #[must_use]
    pub fn debounce_ms(&self) -> Option<u8> {
        check::verified(&self.raw, DEBOUNCE).ok()
    }

    /// Returns whether motion sync is on.
    #[must_use]
    pub fn motion_sync(&self) -> Option<bool> {
        check::verified_flag(&self.raw, MOTION_SYNC).ok()
    }

    /// Returns the LED off time, in units of 10 seconds.
    #[must_use]
    pub fn led_off_byte(&self) -> Option<u8> {
        check::verified(&self.raw, LED_OFF).ok()
    }

    /// Returns whether linear correction is on.
    #[must_use]
    pub fn linear_correction(&self) -> Option<bool> {
        check::verified_flag(&self.raw, LINEAR_CORRECTION).ok()
    }

    /// Returns whether ripple control is on.
    #[must_use]
    pub fn ripple_control(&self) -> Option<bool> {
        check::verified_flag(&self.raw, RIPPLE_CONTROL).ok()
    }

    /// Returns the block with another debounce time. The caller checks the model's list.
    ///
    /// # Errors
    ///
    /// Returns `BadCheck` when the stored pair is corrupt.
    pub fn with_debounce_ms(mut self, ms: u8) -> Result<Self, ProtocolError> {
        check::replace(&mut self.raw, DEBOUNCE, ms)?;

        Ok(self)
    }

    /// Returns the block with motion sync on or off.
    ///
    /// # Errors
    ///
    /// Returns `BadCheck` when the stored pair is corrupt and `InvalidValue` when it holds
    /// neither 0 nor 1.
    pub fn with_motion_sync(mut self, on: bool) -> Result<Self, ProtocolError> {
        check::replace_flag(&mut self.raw, MOTION_SYNC, on)?;

        Ok(self)
    }

    /// Returns the block with another LED off time. The caller checks the model's list.
    ///
    /// # Errors
    ///
    /// Returns `BadCheck` when the stored pair is corrupt.
    pub fn with_led_off_byte(mut self, byte: u8) -> Result<Self, ProtocolError> {
        check::replace(&mut self.raw, LED_OFF, byte)?;

        Ok(self)
    }

    /// Returns the block with linear correction on or off.
    ///
    /// # Errors
    ///
    /// Returns `BadCheck` when the stored pair is corrupt and `InvalidValue` when it holds
    /// neither 0 nor 1.
    pub fn with_linear_correction(mut self, on: bool) -> Result<Self, ProtocolError> {
        check::replace_flag(&mut self.raw, LINEAR_CORRECTION, on)?;

        Ok(self)
    }

    /// Returns the block with ripple control on or off.
    ///
    /// # Errors
    ///
    /// Returns `BadCheck` when the stored pair is corrupt and `InvalidValue` when it holds
    /// neither 0 nor 1.
    pub fn with_ripple_control(mut self, on: bool) -> Result<Self, ProtocolError> {
        check::replace_flag(&mut self.raw, RIPPLE_CONTROL, on)?;

        Ok(self)
    }

    /// Returns the raw bytes to write back.
    #[must_use]
    pub fn as_bytes(&self) -> &[u8; 10] {
        &self.raw
    }
}

#[cfg(test)]
mod tests;
