//! The 10-byte general EEPROM block (polling rate, stage count, current stage).

use crate::protocol::{ProtocolError, check};
use crate::settings::PollingRate;

const POLLING: usize = 0;
const STAGE_COUNT: usize = 2;
const CURRENT_STAGE: usize = 4;
const STAGE_COUNT_RANGE: std::ops::RangeInclusive<u8> = 1..=8;

/// The general block, kept as raw bytes so that unknown fields survive a rewrite.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GeneralBlock {
    raw: [u8; 10],
}

impl GeneralBlock {
    /// Parses a block read from the mouse.
    ///
    /// Only the polling rate, stage count and current stage are validated, each with its
    /// check byte. Bytes 6-9 are kept as they are.
    ///
    /// # Errors
    ///
    /// Returns `BadCheck` for a wrong check byte and `InvalidValue` for an unknown
    /// polling code, a stage count outside 1..=8 or a current stage not below the count.
    pub fn from_raw(raw: [u8; 10]) -> Result<Self, ProtocolError> {
        for offset in [POLLING, STAGE_COUNT, CURRENT_STAGE] {
            if raw[offset + 1] != check::single(raw[offset]) {
                return Err(ProtocolError::BadCheck { offset: offset + 1 });
            }
        }

        if PollingRate::from_code(raw[POLLING]).is_none() {
            return Err(ProtocolError::InvalidValue("polling rate code"));
        }
        if !STAGE_COUNT_RANGE.contains(&raw[STAGE_COUNT]) {
            return Err(ProtocolError::InvalidValue("stage count"));
        }
        if raw[CURRENT_STAGE] >= raw[STAGE_COUNT] {
            return Err(ProtocolError::InvalidValue("current stage"));
        }

        Ok(Self { raw })
    }

    /// Returns the polling rate.
    ///
    /// # Panics
    ///
    /// Never in practice: the code was validated when the block was built.
    #[must_use]
    pub fn polling(&self) -> PollingRate {
        PollingRate::from_code(self.raw[POLLING]).expect("polling code validated at construction")
    }

    /// Returns the number of DPI stages.
    #[must_use]
    pub fn stage_count(&self) -> u8 {
        self.raw[STAGE_COUNT]
    }

    /// Returns the index of the current stage, counted from 0.
    #[must_use]
    pub fn current_stage(&self) -> u8 {
        self.raw[CURRENT_STAGE]
    }

    /// Returns the block with another polling rate and its check byte.
    #[must_use]
    pub fn with_polling(mut self, rate: PollingRate) -> Self {
        self.set(POLLING, rate.code());

        self
    }

    /// Returns the block with another stage count and its check byte.
    ///
    /// Only the count and its check change. A count at or below the current stage is not
    /// rejected here: moving the current stage is the caller's job.
    ///
    /// # Errors
    ///
    /// Returns `InvalidValue` when `count` is outside 1..=8.
    pub fn with_stage_count(mut self, count: u8) -> Result<Self, ProtocolError> {
        if !STAGE_COUNT_RANGE.contains(&count) {
            return Err(ProtocolError::InvalidValue("stage count"));
        }
        self.set(STAGE_COUNT, count);

        Ok(self)
    }

    /// Returns the block with another current stage and its check byte.
    ///
    /// # Errors
    ///
    /// Returns `InvalidValue` when `stage` is not below the current stage count.
    pub fn with_current_stage(mut self, stage: u8) -> Result<Self, ProtocolError> {
        if stage >= self.stage_count() {
            return Err(ProtocolError::InvalidValue("current stage"));
        }
        self.set(CURRENT_STAGE, stage);

        Ok(self)
    }

    /// Returns the raw bytes to write back.
    #[must_use]
    pub fn as_bytes(&self) -> &[u8; 10] {
        &self.raw
    }

    fn set(&mut self, offset: usize, value: u8) {
        self.raw[offset] = value;
        self.raw[offset + 1] = check::single(value);
    }
}

#[cfg(test)]
mod tests;
