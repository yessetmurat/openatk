//! How far the connection has come, and what each stage owns.

use crate::device::{DevicePath, Inbox, Link, Mouse, MouseCommand, MouseError, Session};

/// How far the connection has come.
pub(super) enum State<L> {
    /// No USB device.
    Idle(Inbox<MouseCommand>),
    /// The USB device is open but no write is allowed: the mouse is offline, or its
    /// model is not supported. The link stays open to hear the mouse return.
    Linked {
        session: Session<L, MouseCommand>,
        path: DevicePath,
        unsupported: Option<MouseError>,
    },
    /// The mouse is identified and answers.
    Active {
        mouse: Mouse<L, MouseCommand>,
        path: DevicePath,
    },
}

impl<L: Link> State<L> {
    pub(super) fn inbox_mut(&mut self) -> &mut Inbox<MouseCommand> {
        match self {
            Self::Idle(inbox) => inbox,
            Self::Linked { session, .. } => session.inbox_mut(),
            Self::Active { mouse, .. } => mouse.session_mut().inbox_mut(),
        }
    }

    /// Drops the link, if any, and keeps the queued inputs.
    pub(super) fn into_inbox(self) -> Inbox<MouseCommand> {
        match self {
            Self::Idle(inbox) => inbox,
            Self::Linked { session, .. } => session.into_inbox(),
            Self::Active { mouse, .. } => mouse.into_inbox(),
        }
    }

    pub(super) fn path(&self) -> Option<DevicePath> {
        match self {
            Self::Idle(_) => None,
            Self::Linked { path, .. } | Self::Active { path, .. } => Some(*path),
        }
    }
}
