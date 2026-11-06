use std::sync::mpsc;
use std::time::{Duration, Instant};

use super::*;

#[test]
fn next_report_times_out_and_keeps_other_inputs() {
    let (sender, receiver) = mpsc::channel();
    let mut inbox = Inbox::new(receiver);
    sender.send(Input::Command(7u8)).unwrap();

    let wait = inbox.next_report(Instant::now() + Duration::from_millis(20));

    assert_eq!(wait, None);
    assert_eq!(inbox.next(), Some(Input::Command(7)));
}

#[test]
fn next_returns_none_when_channel_closed_and_empty() {
    let (sender, receiver) = mpsc::channel::<Input<u8>>();
    let mut inbox = Inbox::new(receiver);
    drop(sender);

    assert_eq!(inbox.next(), None);
}

#[test]
fn next_report_keeps_link_closed_for_next() {
    let (sender, receiver) = mpsc::channel();
    let mut inbox = Inbox::new(receiver);
    sender.send(Input::<u8>::Command(7)).unwrap();
    sender.send(Input::LinkClosed).unwrap();
    drop(sender);

    let wait = inbox.next_report(Instant::now() + Duration::from_millis(20));

    assert_eq!(wait, Some(ReportWait::Closed));
    assert_eq!(inbox.next(), Some(Input::Command(7)));
    assert_eq!(inbox.next(), Some(Input::LinkClosed));
}

#[test]
fn next_report_returns_closed_at_once_after_link_closed() {
    let (sender, receiver) = mpsc::channel();
    let mut inbox = Inbox::new(receiver);
    sender.send(Input::<u8>::LinkClosed).unwrap();
    inbox.next_report(Instant::now() + Duration::from_millis(20));

    let wait = inbox.next_report(Instant::now() + Duration::from_millis(20));

    assert_eq!(wait, Some(ReportWait::Closed));
}

#[test]
fn next_until_gives_deferred_inputs_first() {
    let (sender, receiver) = mpsc::channel();
    let mut inbox = Inbox::new(receiver);
    sender.send(Input::Command(7u8)).unwrap();
    inbox.next_report(Instant::now() + Duration::from_millis(20));
    sender.send(Input::Command(8)).unwrap();

    let first = inbox.next_until(Instant::now());
    let second = inbox.next_until(Instant::now());

    assert_eq!(first, Ok(Input::Command(7)));
    assert_eq!(second, Ok(Input::Command(8)));
}

#[test]
fn next_until_times_out_at_the_deadline() {
    let (_sender, receiver) = mpsc::channel::<Input<u8>>();
    let mut inbox = Inbox::new(receiver);

    let result = inbox.next_until(Instant::now() + Duration::from_millis(20));

    assert_eq!(result, Err(RecvTimeoutError::Timeout));
}

#[test]
fn next_until_tells_a_closed_channel_from_a_timeout() {
    let (sender, receiver) = mpsc::channel::<Input<u8>>();
    let mut inbox = Inbox::new(receiver);
    drop(sender);

    let result = inbox.next_until(Instant::now() + Duration::from_secs(5));

    assert_eq!(result, Err(RecvTimeoutError::Disconnected));
}
