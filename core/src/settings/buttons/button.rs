//! Physical buttons of the mouse and their EEPROM addresses.

/// A programmable button.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Button {
    /// Left click.
    Left,
    /// Right click.
    Right,
    /// Wheel click.
    Middle,
    /// First side button.
    Side1,
    /// Second side button.
    Side2,
}

impl Button {
    /// Returns the position of this button in the EEPROM tables.
    fn index(self) -> u16 {
        match self {
            Self::Left => 0,
            Self::Right => 1,
            Self::Middle => 2,
            Self::Side1 => 3,
            Self::Side2 => 4,
        }
    }

    /// Returns the EEPROM address of the 4-byte action of this button.
    #[must_use]
    pub fn address(self) -> u16 {
        0x60 + 4 * self.index()
    }

    /// Returns the EEPROM address of the shortcut entries of this button.
    #[must_use]
    pub fn shortcut_address(self) -> u16 {
        0x100 + 32 * self.index()
    }
}
