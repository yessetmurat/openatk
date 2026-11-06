//! Values read from the mouse: the snapshot and the buttons, stages and sensor it holds.

mod angle_value;
mod button_value;
mod sensor_value;
mod snapshot_value;
mod stage_value;

pub use angle_value::AngleValue;
pub use button_value::ButtonValue;
pub use sensor_value::SensorValue;
pub use snapshot_value::{SnapshotValue, nearest_valid_dpi};
pub use stage_value::StageValue;
