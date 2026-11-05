//! The loop of the service thread: one method per input, moving between the states.

use crate::device::service::connector::Connector;
use crate::device::service::state::State;
use crate::device::{
    DevicePath, Inbox, Input, Link, Mouse, MouseCommand, MouseError, MouseEvent, Session, Timing,
};
use crate::protocol::{Command, Frame};

const BATTERY_FLAG: u8 = 0x40;
/// DPI, polling and profile changes.
const SETTINGS_FLAGS: u8 = 0x01 | 0x02 | 0x04;

/// Connects, then handles inputs one by one until [`Input::Stop`].
pub(super) fn run<K: Connector, F: Fn(MouseEvent)>(
    connector: K,
    inbox: Inbox<MouseCommand>,
    timing: Timing,
    on_event: F,
) {
    let mut worker = Worker {
        connector,
        timing,
        on_event,
    };
    let mut state = worker.connect(inbox);

    while let Some(input) = state.inbox_mut().next() {
        state = match input {
            Input::Report(bytes) => worker.on_report(state, &bytes),
            Input::Command(command) => worker.on_command(state, command),
            Input::LinkClosed => worker.on_link_closed(state),
            Input::Hotplug => worker.on_hotplug(state),
            Input::Stop => return,
        };
    }
}

/// What the loop needs besides its state.
struct Worker<K, F> {
    connector: K,
    timing: Timing,
    on_event: F,
}

impl<K: Connector, F: Fn(MouseEvent)> Worker<K, F> {
    fn on_report(&self, state: State<K::Link>, bytes: &[u8]) -> State<K::Link> {
        let status = Frame::parse(bytes)
            .ok()
            .filter(|frame| frame.command_code() == Command::ReportMouseStatus.code());
        let Some(status) = status else {
            return state;
        };

        match state {
            State::Idle(_) => state,
            State::Linked {
                session,
                path,
                unsupported,
            } => self.identify(session, path, unsupported),
            State::Active { mouse, path } => self.settle(mouse, path, status.payload()[0]),
        }
    }

    fn on_command(&mut self, state: State<K::Link>, command: MouseCommand) -> State<K::Link> {
        let refresh = command == MouseCommand::Refresh;

        match state {
            State::Active { mut mouse, path } => {
                if let Err(error) = execute(&mut mouse, command) {
                    self.emit(MouseEvent::CommandFailed(error));
                }

                self.settle(mouse, path, SETTINGS_FLAGS)
            }
            State::Linked { session, path, .. } if refresh => self.identify(session, path, None),
            State::Idle(inbox) if refresh => self.connect(inbox),
            state => {
                self.emit(MouseEvent::CommandFailed(MouseError::NotConnected));

                state
            }
        }
    }

    fn on_link_closed(&mut self, state: State<K::Link>) -> State<K::Link> {
        let inbox = state.into_inbox();

        self.emit(MouseEvent::NotConnected);

        self.try_connect(inbox)
    }

    fn on_hotplug(&mut self, state: State<K::Link>) -> State<K::Link> {
        let found = self.lookup();

        match (state, found) {
            (State::Idle(inbox), Some(path)) => self.open(path, inbox),
            (state, Some(DevicePath::Cable)) if state.path() == Some(DevicePath::Dongle) => {
                self.replace_link(state, DevicePath::Cable)
            }
            (state, _) => state,
        }
    }

    /// Opens `path` first, so that a failed open leaves the current link in place.
    fn replace_link(&mut self, state: State<K::Link>, path: DevicePath) -> State<K::Link> {
        match self.connector.open(path) {
            Ok(link) => {
                let session = Session::new(link, state.into_inbox(), self.timing);

                self.identify(session, path, None)
            }
            Err(error) => {
                self.emit(MouseEvent::CommandFailed(error));

                state
            }
        }
    }

    fn connect(&mut self, inbox: Inbox<MouseCommand>) -> State<K::Link> {
        let state = self.try_connect(inbox);

        if matches!(state, State::Idle(_)) {
            self.emit(MouseEvent::NotConnected);
        }

        state
    }

    fn try_connect(&mut self, inbox: Inbox<MouseCommand>) -> State<K::Link> {
        match self.lookup() {
            Some(path) => self.open(path, inbox),
            None => State::Idle(inbox),
        }
    }

    fn lookup(&mut self) -> Option<DevicePath> {
        self.connector.find().unwrap_or_else(|error| {
            self.emit(MouseEvent::CommandFailed(error));

            None
        })
    }

    fn open(&mut self, path: DevicePath, inbox: Inbox<MouseCommand>) -> State<K::Link> {
        match self.connector.open(path) {
            Ok(link) => self.identify(Session::new(link, inbox, self.timing), path, None),
            Err(error) => {
                self.emit(MouseEvent::CommandFailed(error));

                State::Idle(inbox)
            }
        }
    }

    /// Tries to reach the mouse on an open link, again when a status report arrives
    /// during a failed attempt. `reported` is the unsupported model already reported for
    /// this link, which is not reported twice.
    fn identify(
        &self,
        mut session: Session<K::Link, MouseCommand>,
        path: DevicePath,
        reported: Option<MouseError>,
    ) -> State<K::Link> {
        loop {
            let error = match Mouse::identify(&mut session, path) {
                Ok(model) => {
                    return self.settle(Mouse::new(session, model, path), path, SETTINGS_FLAGS);
                }
                Err(error) => error,
            };

            if session.take_status_flags().is_empty() {
                return self.stay_linked(session, path, reported, error);
            }
        }
    }

    fn stay_linked(
        &self,
        session: Session<K::Link, MouseCommand>,
        path: DevicePath,
        reported: Option<MouseError>,
        error: MouseError,
    ) -> State<K::Link> {
        let unsupported = match error {
            MouseError::UnsupportedModel { .. } => {
                if reported.as_ref() != Some(&error) {
                    self.emit(MouseEvent::CommandFailed(error.clone()));
                }

                Some(error)
            }
            MouseError::NotConnected | MouseError::Timeout => {
                self.emit(MouseEvent::NotConnected);

                None
            }
            other => {
                self.emit(MouseEvent::CommandFailed(other));
                self.emit(MouseEvent::NotConnected);

                None
            }
        };

        State::Linked {
            session,
            path,
            unsupported,
        }
    }

    /// Reads what `flags` name, then again for the flags that arrived during the reads.
    fn settle(
        &self,
        mut mouse: Mouse<K::Link, MouseCommand>,
        path: DevicePath,
        mut flags: u8,
    ) -> State<K::Link> {
        loop {
            flags = mouse
                .session_mut()
                .take_status_flags()
                .into_iter()
                .fold(flags, |all, flag| all | flag);

            if flags & (BATTERY_FLAG | SETTINGS_FLAGS) == 0 {
                return State::Active { mouse, path };
            }

            if let Err(error) = self.read_flagged(&mut mouse, flags) {
                return self.after_failed_read(mouse, path, error);
            }

            flags = 0;
        }
    }

    fn read_flagged(
        &self,
        mouse: &mut Mouse<K::Link, MouseCommand>,
        flags: u8,
    ) -> Result<(), MouseError> {
        if flags & BATTERY_FLAG != 0 {
            self.emit(MouseEvent::Battery(mouse.battery()?));
        }

        if flags & SETTINGS_FLAGS != 0 {
            self.emit(MouseEvent::Snapshot(mouse.snapshot()?));
        }

        Ok(())
    }

    fn after_failed_read(
        &self,
        mouse: Mouse<K::Link, MouseCommand>,
        path: DevicePath,
        error: MouseError,
    ) -> State<K::Link> {
        match error {
            MouseError::Timeout | MouseError::NotConnected => {
                let mut session = mouse.into_session();

                // A status report during the failed read means the mouse may be back.
                if !session.take_status_flags().is_empty() {
                    return self.identify(session, path, None);
                }

                self.emit(MouseEvent::NotConnected);

                State::Linked {
                    session,
                    path,
                    unsupported: None,
                }
            }
            other => {
                self.emit(MouseEvent::CommandFailed(other));

                State::Active { mouse, path }
            }
        }
    }

    fn emit(&self, event: MouseEvent) {
        (self.on_event)(event);
    }
}

fn execute<L: Link>(
    mouse: &mut Mouse<L, MouseCommand>,
    command: MouseCommand,
) -> Result<(), MouseError> {
    match command {
        MouseCommand::SetPolling(rate) => mouse.set_polling(rate),
        MouseCommand::SetCurrentStage(index) => mouse.set_current_stage(index),
        MouseCommand::SetStageCount(count) => mouse.set_stage_count(count),
        MouseCommand::SetStageDpi(index, stage) => mouse.set_stage_dpi(index, stage),
        MouseCommand::SetButton(button, action) => mouse.set_button(button, action),
        MouseCommand::SetShortcut(button, shortcut) => mouse.set_shortcut(button, &shortcut),
        MouseCommand::Refresh => Ok(()),
    }
}

#[cfg(test)]
mod tests;
