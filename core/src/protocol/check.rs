//! Check bytes used by the mouse EEPROM blocks.

use crate::protocol::ProtocolError;

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

/// Returns the value at `offset` of a block of value and check pairs.
pub(crate) fn verified(raw: &[u8], offset: usize) -> Result<u8, ProtocolError> {
    if raw[offset + 1] != single(raw[offset]) {
        return Err(ProtocolError::BadCheck { offset: offset + 1 });
    }

    Ok(raw[offset])
}

/// Returns the on or off value at `offset` of a block of value and check pairs.
pub(crate) fn verified_flag(raw: &[u8], offset: usize) -> Result<bool, ProtocolError> {
    flag(verified(raw, offset)?).ok_or(ProtocolError::InvalidValue("neither 0 nor 1"))
}

/// Replaces the pair at `offset`, refusing a pair whose check is bad.
pub(crate) fn replace(raw: &mut [u8], offset: usize, value: u8) -> Result<(), ProtocolError> {
    verified(raw, offset)?;

    raw[offset] = value;
    raw[offset + 1] = single(value);

    Ok(())
}

/// Replaces the on or off pair at `offset`, refusing a pair that cannot be read.
pub(crate) fn replace_flag(raw: &mut [u8], offset: usize, on: bool) -> Result<(), ProtocolError> {
    verified_flag(raw, offset)?;

    replace(raw, offset, u8::from(on))
}

/// Reads an on or off byte: only 0 and 1 have a meaning.
#[must_use]
pub(crate) fn flag(byte: u8) -> Option<bool> {
    match byte {
        0 => Some(false),
        1 => Some(true),
        _ => None,
    }
}

#[cfg(test)]
mod tests;
