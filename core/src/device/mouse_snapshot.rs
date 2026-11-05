//! Everything read from the mouse at one moment.

use std::ops::RangeInclusive;

use crate::settings::{Button, ButtonAction, DpiStage, PollingRate, Sensor, Shortcut};

/// Battery state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Battery {
    /// Charge, 0 to 100.
    pub percent: u8,
    /// Whether the mouse is charging.
    pub charging: bool,
}

/// A full read of the mouse settings.
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
}
