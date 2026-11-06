//! The mouse as typed settings: supported models, reads, verified writes and snapshots.

mod model;
mod mouse;
mod mouse_snapshot;

pub use model::Model;
pub use mouse::Mouse;
pub use mouse_snapshot::{AngleValue, Battery, MouseSnapshot};
