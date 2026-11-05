//! One DPI stage: a value per axis.

/// The DPI of one stage, per axis.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DpiStage {
    /// DPI of the X axis.
    pub x: u32,
    /// DPI of the Y axis.
    pub y: u32,
}
