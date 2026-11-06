//! The sensor of the mouse.

use openatk_core::settings::Sensor;

/// The sensor of the mouse.
#[derive(Debug, Clone, Copy, PartialEq, Eq, uniffi::Enum)]
pub enum SensorValue {
    /// PixArt PAW3950 Ultra.
    Paw3950Ultra,
}

impl From<Sensor> for SensorValue {
    fn from(sensor: Sensor) -> Self {
        match sensor {
            Sensor::Paw3950Ultra => Self::Paw3950Ultra,
        }
    }
}

impl From<SensorValue> for Sensor {
    fn from(sensor: SensorValue) -> Self {
        match sensor {
            SensorValue::Paw3950Ultra => Self::Paw3950Ultra,
        }
    }
}
