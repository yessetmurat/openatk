//! Device layer: the serial command session over a HID link and the thread that owns it.

mod device_path;
mod hid_link;
mod hotplug;
mod inbox;
mod link;
mod model;
mod mouse;
mod mouse_error;
mod mouse_snapshot;
mod service;
mod session;

pub use device_path::DevicePath;
pub use hid_link::HidLink;
pub use inbox::{Inbox, Input, ReportWait};
pub use link::Link;
pub use model::Model;
pub use mouse::Mouse;
pub use mouse_error::MouseError;
pub use mouse_snapshot::{AngleValue, Battery, MouseSnapshot};
pub use service::{MouseCommand, MouseEvent, MouseService};
pub use session::{Session, Timing};
