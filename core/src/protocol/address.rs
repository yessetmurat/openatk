//! EEPROM layout and the ranges openatk may write.

/// General block: polling rate, stage count, current stage, bhop, button mode.
pub const GENERAL_BLOCK: u16 = 0x00;
/// Lift-off height.
pub const LIFT_OFF: u16 = 0x0a;
/// DPI stage table.
pub const DPI_STAGES: u16 = 0x0c;
/// Button table.
pub const BUTTONS: u16 = 0x60;
/// Sensor block: debounce, motion sync, LED off time, linear correction, ripple control.
pub const SENSOR_BLOCK: u16 = 0xa9;
/// Sensor sleep block: enabled flag, sleep time, sensor model.
pub const SLEEP_BLOCK: u16 = 0xb5;
/// Angle block: angle and switch.
pub const ANGLE_BLOCK: u16 = 0xbd;
/// Shortcut storage.
pub const SHORTCUTS: u16 = 0x100;

const WRITABLE: [(u16, u16); 8] = [
    (0x00, 0x0a),
    (0x0a, 0x0c),
    (0x0c, 0x2c),
    (0x60, 0x78),
    (0xa9, 0xb3),
    (0xb5, 0xbb),
    (0xbd, 0xc1),
    (0x100, 0x1c0),
];

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
