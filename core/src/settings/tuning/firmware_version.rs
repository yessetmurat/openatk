//! The firmware version the mouse reports.

use std::fmt;

/// A firmware version, ordered by major byte, then minor byte.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct FirmwareVersion {
    /// First data byte of the `GetMouseVersion` reply.
    pub major: u8,
    /// Second data byte of the `GetMouseVersion` reply.
    pub minor: u8,
}

/// The bytes are shown as hex digits, as ATK HUB shows them: `03 10` is 3.10, not 3.16.
impl fmt::Display for FirmwareVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:x}.{:02x}", self.major, self.minor)
    }
}

#[cfg(test)]
mod tests;
