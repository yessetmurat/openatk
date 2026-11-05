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
    /// Read everything again, connecting first when needed.
    Refresh,
}
