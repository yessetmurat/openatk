//! Turns HID service changes into inputs of the service thread, without polling.
//!
//! The events come from `async-hid`, which reports HID services, because `nusb` reports the
//! USB device about 29 ms before its HID collections exist, so the lookup after the event
//! finds nothing.

use std::sync::mpsc::Sender;
use std::thread;

use async_hid::HidBackend;
use futures_lite::StreamExt;
use futures_lite::future;

use crate::device::{Input, MouseError};

/// Starts a thread that sends [`Input::Hotplug`] for every HID service change. The thread
/// ends at the first change after the receiver is gone.
///
/// # Errors
/// [`MouseError::Link`] when the system refuses to watch HID services.
pub(crate) fn spawn<C: Send + 'static>(sender: Sender<Input<C>>) -> Result<(), MouseError> {
    let mut watch = HidBackend::default()
        .watch()
        .map_err(|error| MouseError::Link(error.to_string()))?;

    thread::spawn(move || {
        future::block_on(async {
            while watch.next().await.is_some() {
                if sender.send(Input::Hotplug).is_err() {
                    return;
                }
            }
        });
    });

    Ok(())
}
