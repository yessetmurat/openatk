//! Polling rate of the mouse and its EEPROM code.

/// A polling rate the mouse supports.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PollingRate {
    /// 125 Hz.
    Hz125,
    /// 250 Hz.
    Hz250,
    /// 500 Hz.
    Hz500,
    /// 1000 Hz.
    Hz1000,
    /// 2000 Hz.
    Hz2000,
    /// 4000 Hz.
    Hz4000,
    /// 8000 Hz.
    Hz8000,
}

impl PollingRate {
    /// Returns the rate with this frequency in hertz, or `None` for an unknown frequency.
    #[must_use]
    pub fn from_hz(hz: u32) -> Option<Self> {
        match hz {
            125 => Some(Self::Hz125),
            250 => Some(Self::Hz250),
            500 => Some(Self::Hz500),
            1000 => Some(Self::Hz1000),
            2000 => Some(Self::Hz2000),
            4000 => Some(Self::Hz4000),
            8000 => Some(Self::Hz8000),
            _ => None,
        }
    }

    /// Returns the rate stored under this EEPROM code, or `None` for an unknown code.
    #[must_use]
    pub fn from_code(code: u8) -> Option<Self> {
        match code {
            8 => Some(Self::Hz125),
            4 => Some(Self::Hz250),
            2 => Some(Self::Hz500),
            1 => Some(Self::Hz1000),
            16 => Some(Self::Hz2000),
            32 => Some(Self::Hz4000),
            64 => Some(Self::Hz8000),
            _ => None,
        }
    }

    /// Returns the EEPROM code of this rate.
    #[must_use]
    pub fn code(self) -> u8 {
        match self {
            Self::Hz125 => 8,
            Self::Hz250 => 4,
            Self::Hz500 => 2,
            Self::Hz1000 => 1,
            Self::Hz2000 => 16,
            Self::Hz4000 => 32,
            Self::Hz8000 => 64,
        }
    }

    /// Returns the rate in hertz.
    #[must_use]
    pub fn hz(self) -> u32 {
        match self {
            Self::Hz125 => 125,
            Self::Hz250 => 250,
            Self::Hz500 => 500,
            Self::Hz1000 => 1000,
            Self::Hz2000 => 2000,
            Self::Hz4000 => 4000,
            Self::Hz8000 => 8000,
        }
    }
}

#[cfg(test)]
mod tests;
