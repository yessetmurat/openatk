//! The connector for real hardware, the only part of the service that touches `hidapi`.

use std::sync::mpsc::Sender;

use hidapi::HidApi;

use crate::device::service::connector::Connector;
use crate::device::{DevicePath, HidLink, Input, MouseCommand, MouseError};

/// The real connector. Without a HID context every lookup reports the failure, and the
/// loop keeps answering commands.
pub(super) struct HidConnector {
    api: Result<HidApi, MouseError>,
    sender: Sender<Input<MouseCommand>>,
}

impl HidConnector {
    /// Builds the connector. Links it opens deliver their reports to `sender`.
    pub(super) fn new(sender: Sender<Input<MouseCommand>>) -> Self {
        let api = HidApi::new().map_err(|error| MouseError::Link(error.to_string()));

        Self { api, sender }
    }
}

impl Connector for HidConnector {
    type Link = HidLink;

    fn find(&mut self) -> Result<Option<DevicePath>, MouseError> {
        let api = self.api.as_mut().map_err(|error| error.clone())?;

        api.refresh_devices()
            .map_err(|error| MouseError::Link(error.to_string()))?;

        Ok(HidLink::find_path(api))
    }

    fn open(&mut self, path: DevicePath) -> Result<HidLink, MouseError> {
        let api = self.api.as_ref().map_err(Clone::clone)?;

        HidLink::open(api, path, self.sender.clone())
    }
}
