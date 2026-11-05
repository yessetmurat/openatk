//! Error type of the protocol layer.

use std::error::Error;
use std::fmt;

/// A frame or value that the protocol layer rejects.
#[derive(Debug, Clone, PartialEq, Eq)]
#[must_use]
pub enum ProtocolError {
    /// The report or the valid-length byte has a wrong length.
    BadLength,
    /// The first byte of the report is not `0x08`.
    BadReportId,
    /// The frame checksum does not match.
    BadChecksum,
    /// More than 10 data bytes were given.
    DataTooLong,
    /// The EEPROM range starting at this address may not be written.
    AddressNotWritable(u16),
    /// A stored check byte at this offset does not match its value.
    BadCheck {
        /// Offset of the offending check byte.
        offset: usize,
    },
    /// A value is outside what the mouse accepts.
    InvalidValue(&'static str),
}

impl fmt::Display for ProtocolError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::BadLength => write!(f, "report has a wrong length"),
            Self::BadReportId => write!(f, "report id is not 0x08"),
            Self::BadChecksum => write!(f, "frame checksum mismatch"),
            Self::DataTooLong => write!(f, "more than 10 data bytes"),
            Self::AddressNotWritable(address) => {
                write!(f, "EEPROM address {address:#06x} is not writable")
            }
            Self::BadCheck { offset } => write!(f, "check byte mismatch at offset {offset}"),
            Self::InvalidValue(what) => write!(f, "invalid value: {what}"),
        }
    }
}

impl Error for ProtocolError {}
