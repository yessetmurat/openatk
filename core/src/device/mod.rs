//! Device layer: the serial command session over a HID link and the thread that owns it.

mod control;
mod service;
mod transport;

pub use control::{AngleValue, Battery, Model, Mouse, MouseSnapshot};
pub use service::{MouseCommand, MouseEvent, MouseService};
pub use transport::{
    DevicePath, HidLink, Inbox, Input, Link, MouseError, ReportWait, Session, Timing,
};
