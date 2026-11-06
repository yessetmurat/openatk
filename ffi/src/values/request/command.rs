//! A request from the app, and its conversion to the core command.

use std::error::Error;
use std::fmt;

use openatk_core::device::MouseCommand;
use openatk_core::settings::{DpiStage, PollingRate, ProfileAction, Shortcut, ShortcutKey};

use crate::values::{Action, MouseButton, ShortcutKeyValue};

/// A request from the app.
#[derive(Debug, Clone, PartialEq, Eq, uniffi::Enum)]
pub enum Command {
    /// Set the polling rate.
    SetPolling {
        /// Rate in hertz; must be one the mouse knows.
        hz: u32,
    },
    /// Select a DPI stage.
    SetCurrentStage {
        /// Stage index, counted from 0.
        index: u8,
    },
    /// Change the number of DPI stages.
    SetStageCount {
        /// New number of stages.
        count: u8,
    },
    /// Set the DPI of one stage.
    SetStageDpi {
        /// Stage index, counted from 0.
        index: u8,
        /// DPI on the X axis.
        x: u32,
        /// DPI on the Y axis.
        y: u32,
    },
    /// Assign an action to a button.
    SetButton {
        /// The button.
        button: MouseButton,
        /// The action; `Other` cannot be written.
        action: Action,
    },
    /// Store a shortcut and assign it to a button.
    SetShortcut {
        /// The button.
        button: MouseButton,
        /// The keys, in press order.
        keys: Vec<ShortcutKeyValue>,
    },
    /// Set the lift-off height.
    SetLiftOff {
        /// Height in tenths of a millimetre.
        tenths: u8,
    },
    /// Set the debounce time.
    SetDebounce {
        /// Time in milliseconds; must be one the snapshot offers.
        ms: u8,
    },
    /// Turn motion sync on or off.
    SetMotionSync {
        /// Whether motion sync is on.
        enabled: bool,
    },
    /// Turn linear correction on or off.
    SetLinearCorrection {
        /// Whether linear correction is on.
        enabled: bool,
    },
    /// Turn ripple control on or off.
    SetRippleControl {
        /// Whether ripple control is on.
        enabled: bool,
    },
    /// Set the sleep time.
    SetSleep {
        /// Time in seconds; must be one the snapshot offers.
        seconds: u16,
    },
    /// Set the sensor angle.
    SetAngle {
        /// Angle in degrees.
        degrees: i8,
    },
    /// Turn the sensor angle on or off.
    SetAngleEnabled {
        /// Whether the angle is applied.
        enabled: bool,
    },
    /// Turn far distance mode on or off.
    SetFarDistance {
        /// Whether far distance mode is on.
        enabled: bool,
    },
    /// Switch to a profile.
    SetProfile {
        /// Profile index, counted from 0.
        index: u8,
    },
    /// Turn background polling of link and battery on or off.
    SetBackgroundPolling {
        /// Whether polling runs.
        enabled: bool,
    },
    /// Read everything again.
    Refresh,
}

/// Why a command was refused before it reached the mouse.
#[derive(Debug, Clone, PartialEq, Eq)]
#[must_use]
pub enum CommandError {
    /// The polling rate is not one the mouse knows.
    UnknownPollingRate(u32),
    /// The action cannot be written to a button.
    ReadOnlyAction,
    /// The key list is not a valid shortcut.
    InvalidShortcut(String),
    /// The profile index is not one a button can switch to.
    UnknownProfile(u8),
}

impl fmt::Display for CommandError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnknownPollingRate(hz) => write!(f, "unknown polling rate: {hz} Hz"),
            Self::ReadOnlyAction => write!(f, "this button action cannot be set"),
            Self::InvalidShortcut(reason) => write!(f, "invalid shortcut: {reason}"),
            Self::UnknownProfile(index) => write!(f, "unknown profile index: {index}"),
        }
    }
}

impl Error for CommandError {}

impl Command {
    /// Converts to the core command.
    ///
    /// # Errors
    ///
    /// Returns [`CommandError`] for a value the core would refuse.
    pub fn into_core(self) -> Result<MouseCommand, CommandError> {
        Ok(match self {
            Self::SetPolling { hz } => {
                let rate = PollingRate::from_hz(hz).ok_or(CommandError::UnknownPollingRate(hz))?;

                MouseCommand::SetPolling(rate)
            }
            Self::SetCurrentStage { index } => MouseCommand::SetCurrentStage(index),
            Self::SetStageCount { count } => MouseCommand::SetStageCount(count),
            Self::SetStageDpi { index, x, y } => {
                MouseCommand::SetStageDpi(index, DpiStage { x, y })
            }
            Self::SetButton { button, action } => {
                if let Action::Profile { index } = action
                    && index >= ProfileAction::COUNT
                {
                    return Err(CommandError::UnknownProfile(index));
                }

                let action = action.editable().ok_or(CommandError::ReadOnlyAction)?;

                MouseCommand::SetButton(button.into(), action)
            }
            Self::SetShortcut { button, keys } => {
                let keys: Vec<ShortcutKey> = keys.into_iter().map(ShortcutKey::from).collect();
                let shortcut = Shortcut::from_keys(&keys)
                    .map_err(|error| CommandError::InvalidShortcut(error.to_string()))?;

                MouseCommand::SetShortcut(button.into(), shortcut)
            }
            Self::SetLiftOff { tenths } => MouseCommand::SetLiftOff(tenths),
            Self::SetDebounce { ms } => MouseCommand::SetDebounce(ms),
            Self::SetMotionSync { enabled } => MouseCommand::SetMotionSync(enabled),
            Self::SetLinearCorrection { enabled } => MouseCommand::SetLinearCorrection(enabled),
            Self::SetRippleControl { enabled } => MouseCommand::SetRippleControl(enabled),
            Self::SetSleep { seconds } => MouseCommand::SetSleep(seconds),
            Self::SetAngle { degrees } => MouseCommand::SetAngle(degrees),
            Self::SetAngleEnabled { enabled } => MouseCommand::SetAngleEnabled(enabled),
            Self::SetFarDistance { enabled } => MouseCommand::SetFarDistance(enabled),
            Self::SetProfile { index } => MouseCommand::SetProfile(index),
            Self::SetBackgroundPolling { enabled } => MouseCommand::SetBackgroundPolling(enabled),
            Self::Refresh => MouseCommand::Refresh,
        })
    }
}

#[cfg(test)]
mod tests;
