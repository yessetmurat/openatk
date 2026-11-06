//! Typed settings of the mouse: codecs for values stored in EEPROM blocks.

mod buttons;
mod performance;
mod tuning;

pub use buttons::{
    Button, ButtonAction, DpiAction, MouseAction, Shortcut, ShortcutKey, WheelAction,
};
pub use performance::{DpiStage, GeneralBlock, PollingRate, Sensor};
pub use tuning::{AngleBlock, FirmwareVersion, LiftOff, SensorBlock, SleepBlock};
