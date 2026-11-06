//! One serial command at a time: send, wait for the matching reply, retry on silence.

use std::time::{Duration, Instant};

use crate::device::{Inbox, Link, MouseError, ReportWait};
use crate::protocol::{Command, Frame, ReplyStatus};

/// Reply timeout and retry count of a command.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Timing {
    /// How long to wait for a reply after each send.
    pub reply_timeout: Duration,
    /// Extra sends after the first one when no reply arrives.
    pub retries: u8,
}

impl Default for Timing {
    fn default() -> Timing {
        Timing {
            reply_timeout: Duration::from_secs(3),
            retries: 3,
        }
    }
}

/// Serial command session over a [`Link`] and an [`Inbox`].
#[derive(Debug)]
pub struct Session<L, C> {
    link: L,
    inbox: Inbox<C>,
    timing: Timing,
    status_flags: Vec<u8>,
}

impl<L: Link, C> Session<L, C> {
    /// Builds a session.
    #[must_use]
    pub fn new(link: L, inbox: Inbox<C>, timing: Timing) -> Session<L, C> {
        Session {
            link,
            inbox,
            timing,
            status_flags: Vec::new(),
        }
    }

    /// Gives access to the inbox, for the thread that waits on it between exchanges.
    pub fn inbox_mut(&mut self) -> &mut Inbox<C> {
        &mut self.inbox
    }

    /// Consumes the session and returns its inbox with the deferred inputs intact.
    #[must_use]
    pub fn into_inbox(self) -> Inbox<C> {
        self.inbox
    }

    /// Returns and clears the status flags of `0x0a` reports seen during exchanges.
    pub fn take_status_flags(&mut self) -> Vec<u8> {
        std::mem::take(&mut self.status_flags)
    }

    /// Sends `request` and waits for the matching reply. Silence is retried up to
    /// `retries` times. Reports that do not match are skipped. Nothing is sent on a link
    /// already known to be closed.
    ///
    /// # Errors
    /// [`MouseError::Timeout`] when every attempt is silent, [`MouseError::NotConnected`]
    /// when the link closes, [`MouseError::Rejected`] for a failure status, and the error
    /// of [`Link::send`] when sending fails.
    pub fn exchange(&mut self, request: &Frame) -> Result<Frame, MouseError> {
        if self.inbox.link_closed() {
            return Err(MouseError::NotConnected);
        }

        let report = request.to_report();

        for _ in 0..=self.timing.retries {
            self.link.send(&report)?;

            let deadline = Instant::now() + self.timing.reply_timeout;

            if let Some(reply) = self.wait_for_reply(request, deadline)? {
                return match reply.status() {
                    ReplyStatus::Success => Ok(reply),
                    status => Err(MouseError::Rejected(status)),
                };
            }
        }

        Err(MouseError::Timeout)
    }

    /// Reads reports until the matching reply, or `None` at the deadline.
    fn wait_for_reply(
        &mut self,
        request: &Frame,
        deadline: Instant,
    ) -> Result<Option<Frame>, MouseError> {
        loop {
            let bytes = match self.inbox.next_report(deadline) {
                None => return Ok(None),
                Some(ReportWait::Closed) => return Err(MouseError::NotConnected),
                Some(ReportWait::Report(bytes)) => bytes,
            };

            let Ok(frame) = Frame::parse(&bytes) else {
                continue;
            };

            if frame.command_code() == Command::ReportMouseStatus.code() {
                self.status_flags.push(frame.payload()[0]);
            } else if matches_request(request, &frame) {
                return Ok(Some(frame));
            }
        }
    }
}

fn matches_request(request: &Frame, reply: &Frame) -> bool {
    let code = request.command_code();
    let is_eeprom = code == Command::GetEeprom.code() || code == Command::SetEeprom.code();

    // A failure reply is the answer even when the mouse does not echo the address.
    let address_matches =
        reply.status() != ReplyStatus::Success || reply.address() == request.address();

    reply.command_code() == code && (!is_eeprom || address_matches)
}

#[cfg(test)]
mod tests;
