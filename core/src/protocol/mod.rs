//! Byte-level protocol of the mouse: frames, commands, addresses and checks.

pub mod address;
pub mod check;
mod command;
mod frame;
mod protocol_error;

pub use command::Command;
pub use frame::{Frame, REPORT_LEN, ReplyStatus};
pub use protocol_error::ProtocolError;
