//! The object the app holds: it owns the service and reports to a listener.

use std::sync::Arc;

use openatk_core::device::{MouseEvent, MouseService};

use crate::values::{Command, SnapshotValue};

/// What the app is told about the mouse.
#[derive(Debug, Clone, PartialEq, uniffi::Enum)]
pub enum MouseUpdate {
    /// No mouse can be reached.
    NotConnected,
    /// Every setting, as read from the mouse.
    Snapshot {
        /// The settings.
        snapshot: Box<SnapshotValue>,
    },
    /// The battery state.
    Battery {
        /// Charge, 0 to 100.
        percent: u8,
        /// Whether the mouse is charging.
        charging: bool,
    },
    /// A command or a background operation failed.
    CommandFailed {
        /// Human-readable cause.
        message: String,
    },
}

/// Receives updates. It is called from the service thread and must not panic.
#[uniffi::export(with_foreign)]
pub trait MouseListener: Send + Sync {
    /// Called for every update.
    fn on_event(&self, event: MouseUpdate);
}

/// Handle on the mouse service.
#[derive(uniffi::Object)]
pub struct MouseClient {
    service: MouseService,
    listener: Arc<dyn MouseListener>,
}

#[uniffi::export]
impl MouseClient {
    /// Starts the service. It connects at once and reports to `listener`.
    #[uniffi::constructor]
    #[must_use]
    pub fn new(listener: Arc<dyn MouseListener>) -> Arc<Self> {
        let forward = Arc::clone(&listener);
        let service = MouseService::start(move |event| forward.on_event(MouseUpdate::from(event)));

        Arc::new(Self { service, listener })
    }

    /// Queues a command. A value the core would refuse is reported as `CommandFailed`
    /// and nothing is sent.
    pub fn send(&self, command: Command) {
        match command.into_core() {
            Ok(command) => self.service.send(command),
            Err(error) => self.listener.on_event(MouseUpdate::CommandFailed {
                message: error.to_string(),
            }),
        }
    }
}

impl From<MouseEvent> for MouseUpdate {
    fn from(event: MouseEvent) -> Self {
        match event {
            MouseEvent::NotConnected => Self::NotConnected,
            MouseEvent::Snapshot(snapshot) => Self::Snapshot {
                snapshot: Box::new(SnapshotValue::from(&snapshot)),
            },
            MouseEvent::Battery(battery) => Self::Battery {
                percent: battery.percent,
                charging: battery.charging,
            },
            MouseEvent::CommandFailed(error) => Self::CommandFailed {
                message: error.to_string(),
            },
        }
    }
}
