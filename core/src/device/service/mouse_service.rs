//! The handle the app holds on the service thread.

use std::sync::mpsc::{self, Sender};
use std::thread;

use crate::device::hotplug;
use crate::device::service::hid_connector::HidConnector;
use crate::device::service::polling::PollIntervals;
use crate::device::service::worker;
use crate::device::{Inbox, Input, MouseCommand, MouseEvent, Timing};

/// Handle to the thread that owns the connection. Dropping it asks the thread to stop
/// and does not wait: an exchange with a silent mouse can take 12 s.
#[derive(Debug)]
pub struct MouseService {
    sender: Sender<Input<MouseCommand>>,
}

impl MouseService {
    /// Starts the thread, which connects at once. `on_event` runs on that thread and must
    /// not panic: a panic ends the thread, and later commands get no answer.
    ///
    /// When USB changes cannot be watched, the service runs without hotplug and reports
    /// that once as [`MouseEvent::CommandFailed`].
    #[must_use]
    pub fn start(on_event: impl Fn(MouseEvent) + Send + 'static) -> Self {
        let (sender, receiver) = mpsc::channel();
        let watching = hotplug::spawn(sender.clone());
        let connector = sender.clone();

        thread::spawn(move || {
            if let Err(error) = watching {
                on_event(MouseEvent::CommandFailed(error));
            }

            let connector = HidConnector::new(connector);

            worker::run(
                connector,
                Inbox::new(receiver),
                Timing::default(),
                PollIntervals::default(),
                on_event,
            );
        });

        Self { sender }
    }

    /// Queues a command. Its outcome arrives as events.
    pub fn send(&self, command: MouseCommand) {
        // This fails only when the thread died, and then nothing can answer the command.
        let _ = self.sender.send(Input::Command(command));
    }
}

impl Drop for MouseService {
    fn drop(&mut self) {
        let _ = self.sender.send(Input::Stop);
    }
}
