//! Typed settings of the mouse: codecs for values stored in EEPROM blocks.

mod button;
mod button_action;
mod dpi_stage;
mod general_block;
mod polling_rate;
mod sensor;
mod shortcut;

pub use button::Button;
pub use button_action::{ButtonAction, DpiAction, MouseAction, WheelAction};
pub use dpi_stage::DpiStage;
pub use general_block::GeneralBlock;
pub use polling_rate::PollingRate;
pub use sensor::Sensor;
pub use shortcut::{Shortcut, ShortcutKey};
