//! Typed settings of the mouse: codecs for values stored in EEPROM blocks.

mod angle_block;
mod button;
mod button_action;
mod dpi_stage;
mod firmware_version;
mod general_block;
mod lift_off;
mod polling_rate;
mod sensor;
mod sensor_block;
mod shortcut;
mod sleep_block;

pub use angle_block::AngleBlock;
pub use button::Button;
pub use button_action::{ButtonAction, DpiAction, MouseAction, WheelAction};
pub use dpi_stage::DpiStage;
pub use firmware_version::FirmwareVersion;
pub use general_block::GeneralBlock;
pub use lift_off::LiftOff;
pub use polling_rate::PollingRate;
pub use sensor::Sensor;
pub use sensor_block::SensorBlock;
pub use shortcut::{Shortcut, ShortcutKey};
pub use sleep_block::SleepBlock;
