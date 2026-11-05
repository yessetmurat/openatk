//! EEPROM layout and the ranges openatk may write.

/// General block: polling rate, stage count, current stage, bhop, button mode.
pub const GENERAL_BLOCK: u16 = 0x00;
/// DPI stage table.
pub const DPI_STAGES: u16 = 0x0c;
/// Button table.
pub const BUTTONS: u16 = 0x60;
/// Shortcut storage.
pub const SHORTCUTS: u16 = 0x100;

const WRITABLE: [(u16, u16); 4] = [(0x00, 0x0a), (0x0c, 0x2c), (0x60, 0x78), (0x100, 0x1c0)];

/// Returns whether `len` bytes starting at `address` lie inside one writable range.
#[must_use]
pub fn is_writable(address: u16, len: usize) -> bool {
    let start = usize::from(address);
    let end = start + len;

    WRITABLE
        .iter()
        .any(|&(low, high)| start >= usize::from(low) && end <= usize::from(high))
}

#[cfg(test)]
mod tests;
