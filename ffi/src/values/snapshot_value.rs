//! Every setting read from the mouse.

use openatk_core::device::MouseSnapshot;
use openatk_core::settings::Sensor;

use crate::values::{Action, ButtonValue, MouseButton, SensorValue, ShortcutKeyValue, StageValue};

/// Every setting read from the mouse.
#[derive(Debug, Clone, PartialEq, Eq, uniffi::Record)]
pub struct SnapshotValue {
    /// Marketing name of the model.
    pub model_name: String,
    /// Active profile.
    pub profile: u8,
    /// Battery charge, 0 to 100.
    pub percent: u8,
    /// Whether the mouse is charging.
    pub charging: bool,
    /// Current polling rate in hertz.
    pub polling_hz: u32,
    /// Polling rates on offer, in hertz.
    pub polling_options_hz: Vec<u32>,
    /// Lowest DPI of the sensor.
    pub dpi_min: u32,
    /// Highest DPI of the sensor.
    pub dpi_max: u32,
    /// Sensor of the mouse.
    pub sensor: SensorValue,
    /// Index of the current DPI stage, counted from 0.
    pub current_stage: u8,
    /// The used DPI stages.
    pub stages: Vec<StageValue>,
    /// Every button with its action.
    pub buttons: Vec<ButtonValue>,
}

/// Returns the DPI closest to `value` that the sensor of `snapshot` accepts.
#[must_use]
#[uniffi::export]
pub fn nearest_valid_dpi(snapshot: &SnapshotValue, value: u32) -> u32 {
    Sensor::from(snapshot.sensor).nearest_valid_dpi(value)
}

impl From<&MouseSnapshot> for SnapshotValue {
    fn from(snapshot: &MouseSnapshot) -> Self {
        let range = snapshot.dpi_range();
        let buttons = snapshot
            .buttons
            .iter()
            .map(|(button, action)| {
                let keys = snapshot
                    .shortcuts
                    .iter()
                    .find(|(shortcut_button, _)| shortcut_button == button)
                    .and_then(|(_, shortcut)| shortcut.keys())
                    .map(|keys| keys.into_iter().map(ShortcutKeyValue::from).collect());

                ButtonValue {
                    button: MouseButton::from(*button),
                    action: Action::from(*action),
                    keys,
                }
            })
            .collect();

        Self {
            model_name: snapshot.model_name.clone(),
            profile: snapshot.profile,
            percent: snapshot.battery.percent,
            charging: snapshot.battery.charging,
            polling_hz: snapshot.polling.hz(),
            polling_options_hz: snapshot
                .polling_rates
                .iter()
                .map(|rate| rate.hz())
                .collect(),
            dpi_min: *range.start(),
            dpi_max: *range.end(),
            sensor: SensorValue::from(snapshot.sensor),
            current_stage: snapshot.current_stage,
            stages: snapshot
                .stages
                .iter()
                .map(|stage| StageValue {
                    x: stage.x,
                    y: stage.y,
                })
                .collect(),
            buttons,
        }
    }
}

#[cfg(test)]
mod tests;
