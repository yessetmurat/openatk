//! How the mouse is reached: over its cable or through the dongle.

/// The USB path to the mouse.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DevicePath {
    /// Wired connection.
    Cable,
    /// 2.4 GHz dongle.
    Dongle,
}

/// USB vendor id of the mouse and its dongle.
pub const VENDOR_ID: u16 = 0x373b;
/// HID usage page of the vendor command collection.
pub const USAGE_PAGE: u16 = 0xff02;
/// HID usage of the vendor command collection.
pub const USAGE: u16 = 0x02;

impl DevicePath {
    /// USB product id of this path.
    #[must_use]
    pub fn product_id(self) -> u16 {
        match self {
            Self::Cable => 0x1045,
            Self::Dongle => 0x1031,
        }
    }
}

/// Picks the path to use from `(product_id, usage_page, usage)` of the HID interfaces
/// found. The cable wins over the dongle; only the vendor command collection counts.
#[must_use]
pub fn choose(candidates: &[(u16, u16, u16)]) -> Option<DevicePath> {
    let has = |path: DevicePath| {
        candidates
            .iter()
            .any(|&candidate| candidate == (path.product_id(), USAGE_PAGE, USAGE))
    };

    [DevicePath::Cable, DevicePath::Dongle]
        .into_iter()
        .find(|&path| has(path))
}

#[cfg(test)]
mod tests;
