//! A request from the app, and its conversion to the core command.

use std::error::Error;
use std::fmt;

use openatk_core::device::MouseCommand;
use openatk_core::settings::{DpiStage, PollingRate, Shortcut, ShortcutKey};

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
}

impl fmt::Display for CommandError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnknownPollingRate(hz) => write!(f, "unknown polling rate: {hz} Hz"),
            Self::ReadOnlyAction => write!(f, "this button action cannot be set"),
            Self::InvalidShortcut(reason) => write!(f, "invalid shortcut: {reason}"),
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
                let action = action.editable().ok_or(CommandError::ReadOnlyAction)?;

                MouseCommand::SetButton(button.into(), action)
            }
            Self::SetShortcut { button, keys } => {
                let keys: Vec<ShortcutKey> = keys.into_iter().map(ShortcutKey::from).collect();
                let shortcut = Shortcut::from_keys(&keys)
                    .map_err(|error| CommandError::InvalidShortcut(error.to_string()))?;

                MouseCommand::SetShortcut(button.into(), shortcut)
            }
            Self::SetBackgroundPolling { enabled } => MouseCommand::SetBackgroundPolling(enabled),
            Self::Refresh => MouseCommand::Refresh,
        })
    }
}

#[cfg(test)]
mod tests;
