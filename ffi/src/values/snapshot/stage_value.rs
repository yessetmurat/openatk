//! One DPI stage.

/// One DPI stage.
#[derive(Debug, Clone, Copy, PartialEq, Eq, uniffi::Record)]
pub struct StageValue {
    /// DPI on the X axis.
    pub x: u32,
    /// DPI on the Y axis.
    pub y: u32,
}
