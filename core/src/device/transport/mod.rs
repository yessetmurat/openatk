//! The connection to the mouse: its paths, the HID link, the inbox and the command session.

mod device_path;
mod hid_link;
pub(super) mod hotplug;
mod inbox;
mod link;
mod mouse_error;
mod session;

pub use device_path::DevicePath;
pub use hid_link::HidLink;
pub use inbox::{Inbox, Input, ReportWait};
pub use link::Link;
pub use mouse_error::MouseError;
pub use session::{Session, Timing};
