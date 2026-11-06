//! Requests from the UI to the service thread.

use crate::settings::{Button, ButtonAction, DpiStage, PollingRate, Shortcut};

/// A request from the UI to the service thread.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MouseCommand {
    /// Set the polling rate.
    SetPolling(PollingRate),
    /// Select the DPI stage with this index, counted from 0.
    SetCurrentStage(u8),
    /// Change the number of DPI stages.
    SetStageCount(u8),
    /// Set the DPI of the stage with this index.
    SetStageDpi(u8, DpiStage),
    /// Assign an action to a button.
    SetButton(Button, ButtonAction),
    /// Store a shortcut and assign it to a button.
    SetShortcut(Button, Shortcut),
    /// Set the lift-off height, in tenths of a millimetre.
    SetLiftOff(u8),
    /// Set the debounce time, in milliseconds.
    SetDebounce(u8),
    /// Turn motion sync on or off.
    SetMotionSync(bool),
    /// Turn linear correction on or off.
    SetLinearCorrection(bool),
    /// Turn ripple control on or off.
    SetRippleControl(bool),
    /// Set the sleep time, in seconds.
    SetSleep(u16),
    /// Set the sensor angle, in degrees.
    SetAngle(i8),
    /// Turn the sensor angle on or off.
    SetAngleEnabled(bool),
    /// Turn far distance mode on or off.
    SetFarDistance(bool),
    /// Switch to the profile with this index, counted from 0.
    SetProfile(u8),
    /// Read everything again, connecting first when needed.
    Refresh,
    /// Turn the background polling of link and battery on or off. It starts off. This
    /// stays inside the service: nothing is sent to the mouse and no event answers it.
    SetBackgroundPolling(bool),
}
