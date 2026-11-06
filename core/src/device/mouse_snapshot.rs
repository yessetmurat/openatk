//! Everything read from the mouse at one moment.

use std::ops::RangeInclusive;

use crate::settings::{
    AngleBlock, Button, ButtonAction, DpiStage, FirmwareVersion, LiftOff, PollingRate, Sensor,
    Shortcut,
};

/// Battery state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Battery {
    /// Charge, 0 to 100.
    pub percent: u8,
    /// Whether the mouse is charging.
    pub charging: bool,
}

/// The sensor angle. A field is `None` when its value cannot be read.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AngleValue {
    /// Angle in degrees.
    pub degrees: Option<i8>,
    /// Whether the angle is applied.
    pub enabled: Option<bool>,
}

/// A full read of the mouse settings.
///
/// A sensor field is `None` when the mouse does not offer it or its value cannot be read.
/// A value outside the model's lists shows as it is.
#[derive(Debug, Clone, PartialEq)]
pub struct MouseSnapshot {
    /// Marketing name of the model.
    pub model_name: String,
    /// Customer id.
    pub cid: u8,
    /// Model id.
    pub mid: u8,
    /// Active profile.
    pub profile: u8,
    /// Battery state.
    pub battery: Battery,
    /// Current polling rate.
    pub polling: PollingRate,
    /// Polling rates offered on the current path.
    pub polling_rates: Vec<PollingRate>,
    /// Sensor of the mouse.
    pub sensor: Sensor,
    /// Index of the current DPI stage, counted from 0.
    pub current_stage: u8,
    /// The used DPI stages.
    pub stages: Vec<DpiStage>,
    /// The action of each button.
    pub buttons: Vec<(Button, ButtonAction)>,
    /// The shortcut of each button whose action is a shortcut.
    pub shortcuts: Vec<(Button, Shortcut)>,
    /// Firmware version.
    pub firmware: Option<FirmwareVersion>,
    /// Dongle firmware version, `None` on the cable path or when the dongle refuses it.
    pub dongle_firmware: Option<FirmwareVersion>,
    /// Lift-off height in tenths of a millimetre.
    pub lift_off_tenths: Option<u8>,
    /// Debounce time in milliseconds.
    pub debounce_ms: Option<u8>,
    /// Debounce times on offer, in milliseconds.
    pub debounce_options_ms: Vec<u8>,
    /// Whether motion sync is on.
    pub motion_sync: Option<bool>,
    /// Whether linear correction is on.
    pub linear_correction: Option<bool>,
    /// Whether ripple control is on.
    pub ripple_control: Option<bool>,
    /// Sleep time in seconds: the LED off time, which the sleep control writes and shows.
    pub sleep_seconds: Option<u16>,
    /// Sleep times on offer, in seconds.
    pub sleep_options_seconds: Vec<u16>,
    /// Sensor angle, `None` below the model's minimum firmware.
    pub angle: Option<AngleValue>,
    /// Whether far distance mode is on.
    pub far_distance: Option<bool>,
}

impl MouseSnapshot {
    /// Returns the valid DPI closest to `dpi` for this sensor.
    #[must_use]
    pub fn nearest_valid_dpi(&self, dpi: u32) -> u32 {
        self.sensor.nearest_valid_dpi(dpi)
    }

    /// Returns the DPI range of this sensor.
    #[must_use]
    pub fn dpi_range(&self) -> RangeInclusive<u32> {
        self.sensor.dpi_range()
    }

    /// Returns the lift-off heights on offer, in tenths of a millimetre.
    #[must_use]
    pub fn lift_off_range(&self) -> RangeInclusive<u8> {
        LiftOff::TENTHS_RANGE
    }

    /// Returns the angles on offer, in degrees.
    #[must_use]
    pub fn angle_range(&self) -> RangeInclusive<i8> {
        AngleBlock::ANGLE_RANGE
    }
}
