//! Plain values that cross the foreign boundary, and their conversions to and from `core`.

mod request;
mod snapshot;

pub use request::{Action, Command, MouseButton, ShortcutKeyValue};
pub use snapshot::{
    AngleValue, ButtonValue, SensorValue, SnapshotValue, StageValue, nearest_valid_dpi,
};
