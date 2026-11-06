//! Commands that openatk sends to the mouse.

/// A protocol command. Upgrade and factory-reset commands are deliberately absent.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Command {
    /// Whether the mouse is linked to the dongle.
    GetWirelessMouseOnline = 0x03,
    /// Battery percentage and charging flag.
    GetBatteryLevel = 0x04,
    /// Write EEPROM bytes.
    SetEeprom = 0x07,
    /// Read EEPROM bytes.
    GetEeprom = 0x08,
    /// Change flags pushed by the mouse.
    ReportMouseStatus = 0x0a,
    /// Active profile.
    GetCurrentConfig = 0x0e,
    /// Select the active profile.
    SetCurrentConfig = 0x0f,
    /// Identity: CID, MID and unique id bytes.
    GetMouseCidMid = 0x10,
    /// Firmware version.
    GetMouseVersion = 0x12,
    /// Turn far distance mode on or off.
    SetFarDistanceMode = 0x16,
    /// Whether far distance mode is on.
    GetFarDistanceMode = 0x17,
    /// Dongle firmware version.
    GetDongleVersion = 0x1d,
}

impl Command {
    /// Returns the command for a wire code, or `None` for codes openatk does not use.
    #[must_use]
    pub fn from_code(code: u8) -> Option<Command> {
        match code {
            0x03 => Some(Self::GetWirelessMouseOnline),
            0x04 => Some(Self::GetBatteryLevel),
            0x07 => Some(Self::SetEeprom),
            0x08 => Some(Self::GetEeprom),
            0x0a => Some(Self::ReportMouseStatus),
            0x0e => Some(Self::GetCurrentConfig),
            0x0f => Some(Self::SetCurrentConfig),
            0x10 => Some(Self::GetMouseCidMid),
            0x12 => Some(Self::GetMouseVersion),
            0x16 => Some(Self::SetFarDistanceMode),
            0x17 => Some(Self::GetFarDistanceMode),
            0x1d => Some(Self::GetDongleVersion),
            _ => None,
        }
    }

    /// Returns the wire code of this command.
    #[must_use]
    pub fn code(self) -> u8 {
        self as u8
    }
}

#[cfg(test)]
mod tests;
