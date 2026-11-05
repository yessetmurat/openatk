//! Check bytes used by the mouse EEPROM blocks.

/// Returns the check byte of one value: `0x55 - value`, wrapping.
#[must_use]
pub fn single(value: u8) -> u8 {
    0x55u8.wrapping_sub(value)
}

/// Returns the check byte of a run of bytes: `0x55 - sum`, masked to a byte.
#[must_use]
pub fn sum(items: &[u8]) -> u8 {
    let total = items.iter().fold(0u8, |acc, &item| acc.wrapping_add(item));

    0x55u8.wrapping_sub(total)
}

#[cfg(test)]
mod tests;
