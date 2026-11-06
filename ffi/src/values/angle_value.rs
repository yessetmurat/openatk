//! The sensor angle.

use openatk_core::device;

/// The sensor angle. A field is absent when its value cannot be read.
#[derive(Debug, Clone, Copy, PartialEq, Eq, uniffi::Record)]
pub struct AngleValue {
    /// Angle in degrees.
    pub degrees: Option<i8>,
    /// Whether the angle is applied.
    pub enabled: Option<bool>,
}

impl From<device::AngleValue> for AngleValue {
    fn from(angle: device::AngleValue) -> Self {
        Self {
            degrees: angle.degrees,
            enabled: angle.enabled,
        }
    }
}
