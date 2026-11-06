//! The outgoing half of the connection to the mouse.

use crate::device::MouseError;
use crate::protocol::REPORT_LEN;

/// Sends reports to the mouse. Incoming reports arrive through an [`Inbox`](crate::device::Inbox).
pub trait Link {
    /// Writes one report to the mouse.
    ///
    /// # Errors
    /// Returns [`MouseError::Link`] or [`MouseError::NotConnected`] when the write fails.
    fn send(&mut self, report: &[u8; REPORT_LEN]) -> Result<(), MouseError>;
}
