//! The schedule of the background polling, a workaround for events the mouse never sends.

use std::time::{Duration, Instant};

use crate::device::DevicePath;
use crate::device::service::state::State;

// Workaround, agreed by the owner: the mouse sends no event for sleep, wake or power off,
// so nothing else can tell the service that it left or returned. Both intervals are
// guesses. The link ask itself costs the mouse nothing: the dongle answers it without
// radio traffic. The battery read does reach the mouse, a small drain on its battery, and
// so does the identify that follows when the ask finds the mouse online, at most once per
// `BATTERY_POLL_INTERVAL` while the mouse stays silent.
const LINK_POLL_INTERVAL: Duration = Duration::from_secs(10);
const BATTERY_POLL_INTERVAL: Duration = Duration::from_secs(60);

/// How often each poll runs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct PollIntervals {
    /// Time between asks whether the mouse is linked to the dongle.
    pub(super) link: Duration,
    /// Time between battery reads.
    pub(super) battery: Duration,
}

impl Default for PollIntervals {
    fn default() -> Self {
        Self {
            link: LINK_POLL_INTERVAL,
            battery: BATTERY_POLL_INTERVAL,
        }
    }
}

/// Which polls are due.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct Due {
    /// The link check is due.
    pub(super) link: bool,
    /// The battery read is due.
    pub(super) battery: bool,
}

#[derive(Debug, Clone, Copy)]
struct Deadlines {
    link: Instant,
    battery: Instant,
}

/// When the next polls run. Off until [`Polling::set`] turns it on.
#[derive(Debug)]
pub(super) struct Polling {
    intervals: PollIntervals,
    deadlines: Option<Deadlines>,
}

impl Polling {
    /// Builds a schedule that is off.
    #[must_use]
    pub(super) fn new(intervals: PollIntervals) -> Self {
        Self {
            intervals,
            deadlines: None,
        }
    }

    /// Turns polling on or off. Turning it on while on keeps the schedule.
    pub(super) fn set(&mut self, on: bool, now: Instant) {
        match (on, self.deadlines) {
            (true, None) => self.deadlines = Some(self.counted_from(now)),
            (false, _) => self.deadlines = None,
            (true, Some(_)) => {}
        }
    }

    /// Counts both intervals from `now`, when polling is on. A new state starts afresh,
    /// so a deadline that passed in the old one does not fire at once.
    pub(super) fn restart(&mut self, now: Instant) {
        if self.deadlines.is_some() {
            self.deadlines = Some(self.counted_from(now));
        }
    }

    /// Returns when the loop must wake for a poll in `state`, or `None` when it can block.
    #[must_use]
    pub(super) fn deadline<L>(&self, state: &State<L>) -> Option<Instant> {
        let deadlines = self.deadlines?;

        match state {
            State::Idle(_) => None,
            State::Linked { path, .. } => match path {
                DevicePath::Dongle => Some(deadlines.link),
                DevicePath::Cable => None,
            },
            State::Active { path, .. } => match path {
                DevicePath::Dongle => Some(deadlines.link.min(deadlines.battery)),
                DevicePath::Cable => Some(deadlines.battery),
            },
        }
    }

    /// Returns what is due at `now`.
    #[must_use]
    pub(super) fn due(&self, now: Instant) -> Due {
        match self.deadlines {
            Some(deadlines) => Due {
                link: now >= deadlines.link,
                battery: now >= deadlines.battery,
            },
            None => Due {
                link: false,
                battery: false,
            },
        }
    }

    /// Schedules the next run of what was `due`. `now` is the end of the work: a tick can
    /// take longer than an interval, and counting from its start would leave no pause.
    pub(super) fn reschedule(&mut self, due: Due, now: Instant) {
        let Some(deadlines) = self.deadlines.as_mut() else {
            return;
        };

        if due.link {
            deadlines.link = now + self.intervals.link;
        }

        if due.battery {
            deadlines.battery = now + self.intervals.battery;
        }
    }

    /// Returns when a mouse that stayed silent while online gets its next identify.
    #[must_use]
    pub(super) fn identify_retry_at(&self, now: Instant) -> Instant {
        now + self.intervals.battery
    }

    fn counted_from(&self, now: Instant) -> Deadlines {
        Deadlines {
            link: now + self.intervals.link,
            battery: now + self.intervals.battery,
        }
    }
}

#[cfg(test)]
mod tests;
