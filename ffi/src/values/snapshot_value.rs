//! Every setting read from the mouse.

use openatk_core::device::MouseSnapshot;
use openatk_core::settings::Sensor;

use crate::values::{
    Action, AngleValue, ButtonValue, MouseButton, SensorValue, ShortcutKeyValue, StageValue,
};

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
    /// Firmware version as ATK HUB shows it, absent when the mouse does not tell.
    pub firmware: Option<String>,
    /// Lift-off height in tenths of a millimetre, absent when it cannot be read.
    pub lift_off_tenths: Option<u8>,
    /// Lowest lift-off height on offer, in tenths of a millimetre.
    pub lift_off_min_tenths: u8,
    /// Highest lift-off height on offer, in tenths of a millimetre.
    pub lift_off_max_tenths: u8,
    /// Debounce time in milliseconds, absent when it cannot be read.
    pub debounce_ms: Option<u8>,
    /// Debounce times on offer, in milliseconds.
    pub debounce_options_ms: Vec<u8>,
    /// Whether motion sync is on, absent when it cannot be read.
    pub motion_sync: Option<bool>,
    /// Whether linear correction is on, absent when it cannot be read.
    pub linear_correction: Option<bool>,
    /// Whether ripple control is on, absent when it cannot be read.
    pub ripple_control: Option<bool>,
    /// Sleep time in seconds, absent when it cannot be read.
    pub sleep_seconds: Option<u16>,
    /// Sleep times on offer, in seconds.
    pub sleep_options_seconds: Vec<u16>,
    /// Sensor angle, absent when this firmware has none or the mouse refuses it.
    pub angle: Option<AngleValue>,
    /// Lowest angle on offer, in degrees.
    pub angle_min: i8,
    /// Highest angle on offer, in degrees.
    pub angle_max: i8,
    /// Whether far distance mode is on, absent when it cannot be read.
    pub far_distance: Option<bool>,
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
        let lift_off_range = snapshot.lift_off_range();
        let angle_range = snapshot.angle_range();
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
            firmware: snapshot.firmware.map(|version| version.to_string()),
            lift_off_tenths: snapshot.lift_off_tenths,
            lift_off_min_tenths: *lift_off_range.start(),
            lift_off_max_tenths: *lift_off_range.end(),
            debounce_ms: snapshot.debounce_ms,
            debounce_options_ms: snapshot.debounce_options_ms.clone(),
            motion_sync: snapshot.motion_sync,
            linear_correction: snapshot.linear_correction,
            ripple_control: snapshot.ripple_control,
            sleep_seconds: snapshot.sleep_seconds,
            sleep_options_seconds: snapshot.sleep_options_seconds.clone(),
            angle: snapshot.angle.map(AngleValue::from),
            angle_min: *angle_range.start(),
            angle_max: *angle_range.end(),
            far_distance: snapshot.far_distance,
        }
    }
}

#[cfg(test)]
mod tests;
