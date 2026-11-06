//! The table of supported mouse models.

use crate::device::DevicePath;
use crate::settings::{Button, FirmwareVersion, PollingRate, Sensor};

const CABLE_RATES: &[PollingRate] = &[
    PollingRate::Hz125,
    PollingRate::Hz250,
    PollingRate::Hz500,
    PollingRate::Hz1000,
];
const DONGLE_RATES: &[PollingRate] = &[
    PollingRate::Hz125,
    PollingRate::Hz250,
    PollingRate::Hz500,
    PollingRate::Hz1000,
    PollingRate::Hz2000,
    PollingRate::Hz4000,
    PollingRate::Hz8000,
];
const BUTTONS: &[Button] = &[
    Button::Left,
    Button::Right,
    Button::Middle,
    Button::Side1,
    Button::Side2,
];
const DEBOUNCE_MS: &[u8] = &[0, 1, 2, 4, 8, 15, 20];
const SLEEP_SECONDS: &[u16] = &[30, 60, 120, 180, 300, 1200, 1500, 1800];

static MODELS: [Model; 1] = [Model {
    name: "ATK F1 EXTREME",
    cid: 2,
    mid: 50,
    sensor: Sensor::Paw3950Ultra,
    cable_polling_rates: CABLE_RATES,
    dongle_polling_rates: DONGLE_RATES,
    buttons: BUTTONS,
    angle_min_firmware: Some(FirmwareVersion { major: 3, minor: 0 }),
    debounce_ms: DEBOUNCE_MS,
    sleep_seconds: SLEEP_SECONDS,
}];

/// A supported mouse model.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Model {
    /// Marketing name.
    pub name: &'static str,
    /// Customer id reported by the mouse.
    pub cid: u8,
    /// Model id reported by the mouse.
    pub mid: u8,
    /// Sensor of the mouse.
    pub sensor: Sensor,
    /// Polling rates offered over the cable.
    pub cable_polling_rates: &'static [PollingRate],
    /// Polling rates offered through the dongle.
    pub dongle_polling_rates: &'static [PollingRate],
    /// Programmable buttons.
    pub buttons: &'static [Button],
    /// Lowest firmware with the sensor angle. `None`: the model has no angle at all.
    pub angle_min_firmware: Option<FirmwareVersion>,
    /// Debounce times offered, in milliseconds.
    pub debounce_ms: &'static [u8],
    /// Sleep times offered, in seconds.
    pub sleep_seconds: &'static [u16],
}

impl Model {
    /// Returns the polling rates offered over `path`.
    #[must_use]
    pub fn polling_rates(&self, path: DevicePath) -> &'static [PollingRate] {
        match path {
            DevicePath::Cable => self.cable_polling_rates,
            DevicePath::Dongle => self.dongle_polling_rates,
        }
    }

    /// Returns whether the sensor angle may be used on `firmware`. An unknown firmware
    /// has no angle.
    #[must_use]
    pub fn supports_angle(&self, firmware: Option<FirmwareVersion>) -> bool {
        self.angle_min_firmware
            .zip(firmware)
            .is_some_and(|(minimum, version)| version >= minimum)
    }

    /// Returns the model with this identity, or `None` when it is not supported.
    #[must_use]
    pub fn find(cid: u8, mid: u8) -> Option<&'static Model> {
        MODELS
            .iter()
            .find(|model| model.cid == cid && model.mid == mid)
    }
}

#[cfg(test)]
mod tests;
