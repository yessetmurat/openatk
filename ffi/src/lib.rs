//! Foreign interface of openatk: a thin uniffi layer over `openatk-core` for the macOS app.

mod mouse_client;
mod values;

pub use mouse_client::{MouseClient, MouseListener, MouseUpdate};
pub use values::{
    Action, ButtonValue, Command, MouseButton, SensorValue, ShortcutKeyValue, SnapshotValue,
    StageValue, nearest_valid_dpi,
};

uniffi::setup_scaffolding!();
