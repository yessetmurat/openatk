//! Error type of the device layer.

use std::error::Error;
use std::fmt;

use crate::protocol::{ProtocolError, ReplyStatus};

/// A failure while talking to the mouse.
#[derive(Debug, Clone, PartialEq, Eq)]
#[must_use]
pub enum MouseError {
    /// The mouse or its dongle is not connected.
    NotConnected,
    /// The mouse did not answer in time, retries included.
    Timeout,
    /// The mouse answered with a failure status.
    Rejected(ReplyStatus),
    /// The HID link reported an error.
    Link(String),
    /// A frame or value was rejected by the protocol layer.
    Protocol(ProtocolError),
    /// The bytes read back after a write differ from the bytes written.
    ReadBackMismatch {
        /// EEPROM address of the write.
        address: u16,
        /// Bytes that were written.
        written: Vec<u8>,
        /// Bytes read back.
        actual: Vec<u8>,
    },
    /// The far distance mode read back differs from the mode written.
    FarDistanceMismatch {
        /// Mode that was written.
        written: u8,
        /// Mode read back.
        actual: u8,
    },
    /// The active profile read back differs from the profile written.
    ProfileMismatch {
        /// Profile that was written.
        written: u8,
        /// Profile read back.
        actual: u8,
    },
    /// The mouse model is not supported.
    UnsupportedModel {
        /// Customer id reported by the mouse.
        cid: u8,
        /// Model id reported by the mouse.
        mid: u8,
    },
}

impl fmt::Display for MouseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotConnected => write!(f, "the mouse is not connected"),
            Self::Timeout => write!(f, "the mouse did not answer"),
            Self::Rejected(status) => write!(f, "the mouse rejected the command: {status:?}"),
            Self::Link(message) => write!(f, "link error: {message}"),
            Self::Protocol(error) => write!(f, "protocol error: {error}"),
            Self::ReadBackMismatch { address, .. } => {
                write!(f, "read-back mismatch at EEPROM address {address:#06x}")
            }
            Self::FarDistanceMismatch { written, actual } => {
                write!(f, "far distance mode is {actual} after writing {written}")
            }
            Self::ProfileMismatch { written, actual } => {
                write!(f, "profile is {actual} after writing {written}")
            }
            Self::UnsupportedModel { cid, mid } => {
                write!(
                    f,
                    "unsupported mouse model (cid {cid:#04x}, mid {mid:#04x})"
                )
            }
        }
    }
}

impl Error for MouseError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Protocol(error) => Some(error),
            _ => None,
        }
    }
}

impl From<ProtocolError> for MouseError {
    fn from(error: ProtocolError) -> Self {
        Self::Protocol(error)
    }
}
