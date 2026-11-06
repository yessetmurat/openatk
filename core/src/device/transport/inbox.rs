//! The single channel that carries every input to the thread that owns the session.

use std::collections::VecDeque;
use std::sync::mpsc::{Receiver, RecvTimeoutError};
use std::time::Instant;

/// One thing that can happen to the device thread.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Input<C> {
    /// A raw report from the mouse.
    Report(Vec<u8>),
    /// The HID link closed.
    LinkClosed,
    /// A HID device was added or removed.
    Hotplug,
    /// A command from the UI.
    Command(C),
    /// Shut the thread down.
    Stop,
}

/// Outcome of waiting for a report.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReportWait {
    /// A raw report arrived.
    Report(Vec<u8>),
    /// The link closed.
    Closed,
}

/// Receiving end of the input channel, with a queue for inputs set aside while waiting
/// for a report.
#[derive(Debug)]
pub struct Inbox<C> {
    receiver: Receiver<Input<C>>,
    deferred: VecDeque<Input<C>>,
}

impl<C> Inbox<C> {
    /// Wraps the receiving end of the channel.
    #[must_use]
    pub fn new(receiver: Receiver<Input<C>>) -> Inbox<C> {
        Inbox {
            receiver,
            deferred: VecDeque::new(),
        }
    }

    /// Returns the next input: deferred ones first, then blocks on the channel.
    /// Returns `None` when the channel is closed and nothing is deferred.
    #[expect(
        clippy::should_implement_trait,
        reason = "blocking receive on a channel, not an iterator"
    )]
    pub fn next(&mut self) -> Option<Input<C>> {
        match self.deferred.pop_front() {
            Some(input) => Some(input),
            None => self.receiver.recv().ok(),
        }
    }

    /// Like [`Inbox::next`], but gives up at `deadline`.
    ///
    /// # Errors
    /// [`RecvTimeoutError::Timeout`] when `deadline` passes, and
    /// [`RecvTimeoutError::Disconnected`] when the channel is closed and nothing is deferred.
    pub(crate) fn next_until(&mut self, deadline: Instant) -> Result<Input<C>, RecvTimeoutError> {
        match self.deferred.pop_front() {
            Some(input) => Ok(input),
            None => self
                .receiver
                .recv_timeout(deadline.saturating_duration_since(Instant::now())),
        }
    }

    /// Returns whether a link close is waiting for [`Inbox::next`].
    #[must_use]
    pub(crate) fn link_closed(&self) -> bool {
        self.deferred
            .iter()
            .any(|input| matches!(input, Input::LinkClosed))
    }

    /// Blocks until a report or a link close arrives, or `deadline` passes (`None`).
    /// Other inputs are kept for [`Inbox::next`], and so is [`Input::LinkClosed`]: the
    /// error of a failed exchange does not tell a lost link from an offline mouse. While
    /// it is kept, this returns at once. A closed channel counts as a closed link.
    pub fn next_report(&mut self, deadline: Instant) -> Option<ReportWait> {
        if self.link_closed() {
            return Some(ReportWait::Closed);
        }

        loop {
            let remaining = deadline.saturating_duration_since(Instant::now());

            match self.receiver.recv_timeout(remaining) {
                Ok(Input::Report(bytes)) => return Some(ReportWait::Report(bytes)),
                Ok(Input::LinkClosed) => {
                    self.deferred.push_back(Input::LinkClosed);

                    return Some(ReportWait::Closed);
                }
                Err(RecvTimeoutError::Disconnected) => return Some(ReportWait::Closed),
                Ok(other) => self.deferred.push_back(other),
                Err(RecvTimeoutError::Timeout) => return None,
            }
        }
    }
}

#[cfg(test)]
mod tests;
