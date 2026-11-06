//! The 6-byte sensor sleep EEPROM block: enabled flag, sleep time, sensor model.

use crate::protocol::{ProtocolError, check};

const ENABLED: usize = 0;
const SLEEP: usize = 2;

/// The sensor sleep block, kept as raw bytes: the sensor model pair at offset 4 belongs to
/// another setting and is never touched here.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SleepBlock {
    raw: [u8; 6],
}

impl SleepBlock {
    /// Wraps bytes read from the mouse. Nothing is validated here: see the getters.
    #[must_use]
    pub fn from_raw(raw: [u8; 6]) -> Self {
        Self { raw }
    }

    /// Returns whether the custom sensor sleep time is on, or `None` when its pair cannot
    /// be read.
    #[must_use]
    pub fn sensor_sleep_enabled(&self) -> Option<bool> {
        check::verified_flag(&self.raw, ENABLED).ok()
    }

    /// Returns the sensor sleep time in units of 10 seconds, or `None` for a bad check.
    #[must_use]
    pub fn sensor_sleep_byte(&self) -> Option<u8> {
        check::verified(&self.raw, SLEEP).ok()
    }

    /// Returns the block with the custom sleep time on and set to `byte`. The caller checks
    /// the model's list.
    ///
    /// # Errors
    ///
    /// Returns `BadCheck` when a stored pair of these two is corrupt and `InvalidValue` when
    /// the enabled flag holds neither 0 nor 1.
    pub fn with_sleep(mut self, byte: u8) -> Result<Self, ProtocolError> {
        check::replace_flag(&mut self.raw, ENABLED, true)?;
        check::replace(&mut self.raw, SLEEP, byte)?;

        Ok(self)
    }

    /// Returns the raw bytes to write back.
    #[must_use]
    pub fn as_bytes(&self) -> &[u8; 6] {
        &self.raw
    }
}

#[cfg(test)]
mod tests;
