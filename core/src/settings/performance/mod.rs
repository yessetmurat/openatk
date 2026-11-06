//! Polling rate, DPI stages and the sensor models that encode them.

mod dpi_stage;
mod general_block;
mod polling_rate;
mod sensor;

pub use dpi_stage::DpiStage;
pub use general_block::GeneralBlock;
pub use polling_rate::PollingRate;
pub use sensor::Sensor;
