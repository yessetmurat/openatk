//! The real HID connection to the mouse.

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::Sender;
use std::thread;

use hidapi::{DeviceInfo, HidApi, HidDevice};

use crate::device::device_path::{self, USAGE, USAGE_PAGE, VENDOR_ID};
use crate::device::{DevicePath, Input, Link, MouseError};
use crate::protocol::REPORT_LEN;

const READ_BUFFER_LEN: usize = 64;

/// Writer half of the HID connection. The reader half runs on its own thread.
#[derive(Debug)]
pub struct HidLink {
    writer: HidDevice,
    stop: Arc<AtomicBool>,
}

impl HidLink {
    /// Looks for the mouse among the connected HID devices and returns the best path.
    #[must_use]
    pub fn find_path(api: &HidApi) -> Option<DevicePath> {
        let candidates: Vec<(u16, u16, u16)> = api
            .device_list()
            .filter(|info| info.vendor_id() == VENDOR_ID)
            .map(|info| (info.product_id(), info.usage_page(), info.usage()))
            .collect();

        device_path::choose(&candidates)
    }

    /// Opens two shared handles to the device on `path`. A thread reads reports from one
    /// and sends them as [`Input::Report`]; the other is used for writing.
    ///
    /// # Errors
    /// [`MouseError::NotConnected`] when the device is absent, [`MouseError::Link`] when
    /// opening a handle fails.
    pub fn open<C: Send + 'static>(
        api: &HidApi,
        path: DevicePath,
        sender: Sender<Input<C>>,
    ) -> Result<Self, MouseError> {
        #[cfg(target_os = "macos")]
        api.set_open_exclusive(false);

        let info = api
            .device_list()
            .find(|info| matches(info, path))
            .ok_or(MouseError::NotConnected)?;
        let reader = info.open_device(api).map_err(link_error)?;
        let writer = info.open_device(api).map_err(link_error)?;
        let stop = Arc::new(AtomicBool::new(false));

        let reader_stop = Arc::clone(&stop);
        thread::spawn(move || read_loop(&reader, &sender, &reader_stop));

        Ok(Self { writer, stop })
    }
}

impl Link for HidLink {
    fn send(&mut self, report: &[u8; REPORT_LEN]) -> Result<(), MouseError> {
        self.writer.write(report).map_err(link_error)?;

        Ok(())
    }
}

impl Drop for HidLink {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::SeqCst);
    }
}

fn matches(info: &DeviceInfo, path: DevicePath) -> bool {
    info.vendor_id() == VENDOR_ID
        && info.product_id() == path.product_id()
        && info.usage_page() == USAGE_PAGE
        && info.usage() == USAGE
}

fn link_error(error: hidapi::HidError) -> MouseError {
    MouseError::Link(error.to_string())
}

fn read_loop<C>(reader: &HidDevice, sender: &Sender<Input<C>>, stop: &AtomicBool) {
    let mut buffer = [0u8; READ_BUFFER_LEN];

    loop {
        let Ok(len) = reader.read(&mut buffer) else {
            if !stop.load(Ordering::SeqCst) {
                let _ = sender.send(Input::LinkClosed);
            }

            return;
        };

        if len == 0 && !stop.load(Ordering::SeqCst) {
            continue;
        }

        if stop.load(Ordering::SeqCst)
            || sender.send(Input::Report(buffer[..len].to_vec())).is_err()
        {
            return;
        }
    }
}
