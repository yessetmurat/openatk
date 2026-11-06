//! One 17-byte HID report: report id, command, status, address, length, data, checksum.

use crate::protocol::{Command, ProtocolError, address, check};

/// Length of a full HID report including the report id.
pub const REPORT_LEN: usize = 17;

const REPORT_ID: u8 = 0x08;
const DATA_LEN: usize = 10;

/// Status byte of a reply.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReplyStatus {
    /// The command succeeded.
    Success,
    /// The command failed.
    Fail,
    /// The mouse reported an EEPROM error.
    EepromError,
    /// The mouse does not support the command.
    NotSupported,
    /// Any other status byte.
    Unknown(u8),
}

/// A decoded or to-be-sent frame. The command is kept as a raw byte so frames with
/// commands openatk does not know still parse.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Frame {
    command: u8,
    status: u8,
    address: u16,
    length: u8,
    data: [u8; DATA_LEN],
}

impl Frame {
    /// Builds a request with no address and no data.
    #[must_use]
    pub fn request(command: Command) -> Frame {
        Frame {
            command: command.code(),
            status: 0,
            address: 0,
            length: 0,
            data: [0; DATA_LEN],
        }
    }

    /// Builds an EEPROM read request for `len` bytes at `address`.
    ///
    /// # Errors
    /// Returns [`ProtocolError::BadLength`] when `len` is above 10.
    pub fn get_eeprom(address: u16, len: u8) -> Result<Frame, ProtocolError> {
        if usize::from(len) > DATA_LEN {
            return Err(ProtocolError::BadLength);
        }

        Ok(Frame {
            command: Command::GetEeprom.code(),
            status: 0,
            address,
            length: len,
            data: [0; DATA_LEN],
        })
    }

    /// Builds an EEPROM write request.
    ///
    /// # Errors
    /// Returns [`ProtocolError::DataTooLong`] for more than 10 bytes and
    /// [`ProtocolError::AddressNotWritable`] when the range is not writable.
    pub fn set_eeprom(address: u16, data: &[u8]) -> Result<Frame, ProtocolError> {
        if data.len() > DATA_LEN {
            return Err(ProtocolError::DataTooLong);
        }
        if !address::is_writable(address, data.len()) {
            return Err(ProtocolError::AddressNotWritable(address));
        }

        let mut padded = [0; DATA_LEN];
        padded[..data.len()].copy_from_slice(data);

        Ok(Frame {
            command: Command::SetEeprom.code(),
            status: 0,
            address,
            length: data.len() as u8,
            data: padded,
        })
    }

    /// Builds a far distance mode request. The mode has no check byte.
    #[must_use]
    pub fn set_far_distance(enabled: bool) -> Frame {
        let mut data = [0; DATA_LEN];
        data[0] = u8::from(enabled);

        Frame {
            command: Command::SetFarDistanceMode.code(),
            status: 0,
            address: 0,
            length: DATA_LEN as u8,
            data,
        }
    }

    /// Encodes the frame as a HID report with its checksum.
    #[must_use]
    pub fn to_report(&self) -> [u8; REPORT_LEN] {
        let mut report = [0; REPORT_LEN];
        report[0] = REPORT_ID;
        report[1] = self.command;
        report[2] = self.status;
        report[3..5].copy_from_slice(&self.address.to_be_bytes());
        report[5] = self.length;
        report[6..16].copy_from_slice(&self.data);
        report[16] = check::sum(&report[..16]);

        report
    }

    /// Decodes a HID report.
    ///
    /// # Errors
    /// Returns [`ProtocolError::BadLength`] for a wrong report length or a valid length
    /// above 10, [`ProtocolError::BadReportId`] and [`ProtocolError::BadChecksum`].
    pub fn parse(report: &[u8]) -> Result<Frame, ProtocolError> {
        if report.len() != REPORT_LEN {
            return Err(ProtocolError::BadLength);
        }
        if report[0] != REPORT_ID {
            return Err(ProtocolError::BadReportId);
        }
        if report[16] != check::sum(&report[..16]) {
            return Err(ProtocolError::BadChecksum);
        }
        if usize::from(report[5]) > DATA_LEN {
            return Err(ProtocolError::BadLength);
        }

        let mut data = [0; DATA_LEN];
        data.copy_from_slice(&report[6..16]);

        Ok(Frame {
            command: report[1],
            status: report[2],
            address: u16::from_be_bytes([report[3], report[4]]),
            length: report[5],
            data,
        })
    }

    /// Returns the raw command byte.
    #[must_use]
    pub fn command_code(&self) -> u8 {
        self.command
    }

    /// Returns the reply status.
    #[must_use]
    pub fn status(&self) -> ReplyStatus {
        match self.status {
            0 => ReplyStatus::Success,
            1 => ReplyStatus::Fail,
            2 => ReplyStatus::EepromError,
            255 => ReplyStatus::NotSupported,
            other => ReplyStatus::Unknown(other),
        }
    }

    /// Returns the EEPROM address.
    #[must_use]
    pub fn address(&self) -> u16 {
        self.address
    }

    /// Returns all ten data bytes, whatever the valid-length byte says. The mouse's
    /// ATK HUB reads replies by offset and ignores that byte.
    #[must_use]
    pub fn payload(&self) -> &[u8; DATA_LEN] {
        &self.data
    }

    /// Returns the valid data bytes.
    #[must_use]
    pub fn data(&self) -> &[u8] {
        &self.data[..usize::from(self.length)]
    }
}

#[cfg(test)]
mod tests;
