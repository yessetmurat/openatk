//! The thread that owns the connection to the mouse. It handles one input at a time, so
//! commands, status reports and USB changes never overlap on the link.

mod connector;
mod hid_connector;
mod mouse_command;
mod mouse_event;
mod mouse_service;
mod polling;
mod state;
mod worker;

pub use mouse_command::MouseCommand;
pub use mouse_event::MouseEvent;
pub use mouse_service::MouseService;
