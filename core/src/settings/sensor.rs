//! Sensor models and their DPI encoding.

use std::ops::RangeInclusive;

use crate::protocol::{ProtocolError, check};
use crate::settings::DpiStage;

/// A DPI segment: `start` up to (excluding) `end` in steps of `step`.
struct Segment {
    start: u32,
    end: u32,
    step: u32,
}

const PAW3950_ULTRA_SEGMENTS: [Segment; 3] = [
    Segment {
        start: 10,
        end: 10000,
        step: 10,
    },
    Segment {
        start: 10000,
        end: 30000,
        step: 50,
    },
    Segment {
        start: 30000,
        end: 42001,
        step: 100,
    },
];

/// A sensor model of the mouse.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Sensor {
    /// PixArt PAW3950 in its Ultra variant.
    Paw3950Ultra,
}

impl Sensor {
    /// Returns the DPI range the sensor accepts.
    #[must_use]
    pub fn dpi_range(self) -> RangeInclusive<u32> {
        10..=42000
    }

    /// Returns whether `dpi` is on the sensor's step grid.
    #[must_use]
    pub fn is_valid_dpi(self, dpi: u32) -> bool {
        self.segments()
            .iter()
            .any(|s| (s.start..s.end).contains(&dpi) && (dpi - s.start).is_multiple_of(s.step))
    }

    /// Returns the valid DPI closest to `dpi`, clamped to the range.
    ///
    /// Rounding is done inside the segment that holds the value, measured from the
    /// segment start. A value on a segment boundary belongs to the higher segment.
    #[must_use]
    pub fn nearest_valid_dpi(self, dpi: u32) -> u32 {
        let range = self.dpi_range();
        let dpi = dpi.clamp(*range.start(), *range.end());
        let segment = self
            .segments()
            .iter()
            .rfind(|s| dpi >= s.start)
            .expect("clamped value is inside the first segment or above");
        let steps = (dpi - segment.start + segment.step / 2) / segment.step;

        segment.start + steps * segment.step
    }

    /// Encodes a stage as the 4 bytes `[x, y, extra, check]`.
    ///
    /// # Errors
    ///
    /// Returns `InvalidValue` when an axis is not a valid DPI for this sensor.
    pub fn encode_stage(self, stage: DpiStage) -> Result<[u8; 4], ProtocolError> {
        if !self.is_valid_dpi(stage.x) || !self.is_valid_dpi(stage.y) {
            return Err(ProtocolError::InvalidValue("dpi"));
        }

        let (x_index, x_flags) = encode_axis(stage.x);
        let (y_index, y_flags) = encode_axis(stage.y);
        let extra =
            x_flags | (y_flags << 4) | ((x_index >> 8) as u8) << 2 | ((y_index >> 8) as u8) << 6;
        let low_x = x_index as u8;
        let low_y = y_index as u8;
        let check = check::sum(&[low_x, low_y, extra]);

        Ok([low_x, low_y, extra, check])
    }

    /// Decodes a stage from 4 bytes. The value need not be on the step grid.
    ///
    /// # Errors
    ///
    /// Returns `BadCheck` when the check byte does not match.
    pub fn decode_stage(self, bytes: [u8; 4]) -> Result<DpiStage, ProtocolError> {
        let [x, y, extra, stored_check] = bytes;
        if stored_check != check::sum(&[x, y, extra]) {
            return Err(ProtocolError::BadCheck { offset: 3 });
        }

        let x_index = u32::from(x) | u32::from((extra >> 2) & 0b11) << 8;
        let y_index = u32::from(y) | u32::from((extra >> 6) & 0b11) << 8;

        Ok(DpiStage {
            x: decode_axis(x_index, extra & 0b11),
            y: decode_axis(y_index, (extra >> 4) & 0b11),
        })
    }

    fn segments(self) -> &'static [Segment] {
        match self {
            Self::Paw3950Ultra => &PAW3950_ULTRA_SEGMENTS,
        }
    }
}

/// Returns the 10-bit index and the flags (bit 1 high, bit 0 doubled) of one axis.
fn encode_axis(dpi: u32) -> (u32, u8) {
    if dpi <= 10000 {
        (dpi / 10 - 1, 0)
    } else if dpi <= 30000 {
        ((dpi - 10050) / 50, 0b10)
    } else {
        ((dpi / 2 - 10050) / 50, 0b11)
    }
}

fn decode_axis(index: u32, flags: u8) -> u32 {
    let dpi = if flags & 0b10 != 0 {
        index * 50 + 10050
    } else {
        (index + 1) * 10
    };

    if flags & 0b01 != 0 { dpi * 2 } else { dpi }
}

#[cfg(test)]
mod tests;
