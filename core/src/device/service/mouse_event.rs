//! What the service thread reports to the UI.

use crate::device::{Battery, MouseError, MouseSnapshot};

/// What the service thread tells the UI.
#[derive(Debug, Clone, PartialEq)]
pub enum MouseEvent {
    /// No mouse can be reached.
    NotConnected,
    /// Every setting, as read from the mouse.
    Snapshot(MouseSnapshot),
    /// The battery state, as read from the mouse.
    Battery(Battery),
    /// A command or a background operation failed.
    CommandFailed(MouseError),
}
