//! Turns USB changes into inputs of the service thread, without polling.

use std::sync::mpsc::Sender;
use std::thread;

use futures_lite::stream;

use crate::device::{Input, MouseError};

/// Starts a thread that sends [`Input::Hotplug`] for every USB change. The thread ends
/// at the first change after the receiver is gone.
///
/// # Errors
/// [`MouseError::Link`] when the system refuses to watch USB devices.
pub(crate) fn spawn<C: Send + 'static>(sender: Sender<Input<C>>) -> Result<(), MouseError> {
    let watch = nusb::watch_devices().map_err(|error| MouseError::Link(error.to_string()))?;

    thread::spawn(move || {
        for _change in stream::block_on(watch) {
            if sender.send(Input::Hotplug).is_err() {
                return;
            }
        }
    });

    Ok(())
}
