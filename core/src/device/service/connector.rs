//! The seam between the loop and the HID service.

use crate::device::{DevicePath, Link, MouseError};

/// Finds the mouse over HID and opens links to it.
pub(super) trait Connector {
    /// The link this connector opens.
    type Link: Link;

    /// Returns the best path available now.
    fn find(&mut self) -> Result<Option<DevicePath>, MouseError>;

    /// Opens a link on `path`.
    fn open(&mut self, path: DevicePath) -> Result<Self::Link, MouseError>;
}
