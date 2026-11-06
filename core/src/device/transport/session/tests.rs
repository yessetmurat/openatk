use std::cell::Cell;
use std::collections::VecDeque;
use std::rc::Rc;
use std::sync::mpsc::{self, Sender};

use super::*;
use crate::device::Input;
use crate::protocol::{REPORT_LEN, check};

const SHORT: Timing = Timing {
    reply_timeout: Duration::from_millis(20),
    retries: 3,
};
const LONG: Timing = Timing {
    reply_timeout: Duration::from_secs(5),
    retries: 3,
};

type Script = VecDeque<Vec<Input<u8>>>;

/// Fake link: each send counts and pushes the next scripted batch into the channel.
struct FakeLink {
    sender: Sender<Input<u8>>,
    script: Script,
    sends: Rc<Cell<usize>>,
}

impl Link for FakeLink {
    fn send(&mut self, _report: &[u8; REPORT_LEN]) -> Result<(), MouseError> {
        self.sends.set(self.sends.get() + 1);

        for input in self.script.pop_front().unwrap_or_default() {
            self.sender.send(input).unwrap();
        }

        Ok(())
    }
}

fn session(
    script: Vec<Vec<Input<u8>>>,
    timing: Timing,
) -> (Session<FakeLink, u8>, Rc<Cell<usize>>) {
    let (sender, receiver) = mpsc::channel();
    let sends = Rc::new(Cell::new(0));
    let link = FakeLink {
        sender,
        script: script.into(),
        sends: Rc::clone(&sends),
    };

    (Session::new(link, Inbox::new(receiver), timing), sends)
}

fn reply_bytes(code: u8, status: u8, address: u16, data: &[u8]) -> Vec<u8> {
    let mut report = vec![0u8; REPORT_LEN];
    report[0] = 0x08;
    report[1] = code;
    report[2] = status;
    report[3..5].copy_from_slice(&address.to_be_bytes());
    report[5] = data.len() as u8;
    report[6..6 + data.len()].copy_from_slice(data);
    report[16] = check::sum(&report[..16]);

    report
}

fn reply(code: u8, status: u8, address: u16, data: &[u8]) -> Input<u8> {
    Input::Report(reply_bytes(code, status, address, data))
}

fn battery_request() -> Frame {
    Frame::request(Command::GetBatteryLevel)
}

#[test]
fn exchange_returns_matching_reply() {
    let (mut session, sends) = session(vec![vec![reply(0x04, 0, 0, &[80, 1])]], LONG);

    let frame = session.exchange(&battery_request()).unwrap();

    assert_eq!(frame.data(), &[80, 1]);
    assert_eq!(sends.get(), 1);
}

#[test]
fn retries_three_times_then_times_out() {
    let (mut session, sends) = session(vec![], SHORT);

    let result = session.exchange(&battery_request());

    assert_eq!(result, Err(MouseError::Timeout));
    assert_eq!(sends.get(), 4);
}

#[test]
fn second_attempt_succeeds() {
    let script = vec![vec![], vec![reply(0x04, 0, 0, &[50, 0])]];
    let timing = Timing {
        reply_timeout: Duration::from_millis(50),
        retries: 3,
    };
    let (mut session, sends) = session(script, timing);

    let frame = session.exchange(&battery_request()).unwrap();

    assert_eq!(frame.data(), &[50, 0]);
    assert_eq!(sends.get(), 2);
}

#[test]
fn reply_with_other_code_is_skipped() {
    let script = vec![vec![reply(0x03, 0, 0, &[1]), reply(0x04, 0, 0, &[70, 0])]];
    let (mut session, _) = session(script, LONG);

    let frame = session.exchange(&battery_request()).unwrap();

    assert_eq!(frame.command_code(), 0x04);
}

#[test]
fn corrupt_reply_is_treated_as_no_reply() {
    let mut corrupt = reply_bytes(0x04, 0, 0, &[70, 0]);
    corrupt[16] ^= 0xff;
    let (mut session, sends) = session(vec![vec![Input::Report(corrupt)]], SHORT);

    let result = session.exchange(&battery_request());

    assert_eq!(result, Err(MouseError::Timeout));
    assert_eq!(sends.get(), 4);
}

#[test]
fn eeprom_reply_for_other_address_is_skipped() {
    let script = vec![vec![
        reply(0x08, 0, 0x00, &[1, 2]),
        reply(0x08, 0, 0x60, &[9, 9]),
    ]];
    let (mut session, _) = session(script, LONG);
    let request = Frame::get_eeprom(0x60, 2).unwrap();

    let frame = session.exchange(&request).unwrap();

    assert_eq!(frame.address(), 0x60);
    assert_eq!(frame.data(), &[9, 9]);
}

#[test]
fn failure_status_is_rejected_without_retry() {
    let (mut session, sends) = session(vec![vec![reply(0x04, 2, 0, &[])]], LONG);

    let result = session.exchange(&battery_request());

    assert_eq!(result, Err(MouseError::Rejected(ReplyStatus::EepromError)));
    assert_eq!(sends.get(), 1);
}

#[test]
fn status_event_during_exchange_is_kept() {
    let script = vec![vec![reply(0x0a, 0, 0, &[0x40]), reply(0x04, 0, 0, &[1, 0])]];
    let (mut session, _) = session(script, LONG);

    session.exchange(&battery_request()).unwrap();

    assert_eq!(session.take_status_flags(), vec![0x40]);
    assert_eq!(session.take_status_flags(), Vec::<u8>::new());
}

#[test]
fn command_during_exchange_stays_queued() {
    let script = vec![vec![Input::Command(5), reply(0x04, 0, 0, &[1, 0])]];
    let (mut session, _) = session(script, LONG);

    session.exchange(&battery_request()).unwrap();

    assert_eq!(session.inbox_mut().next(), Some(Input::Command(5)));
}

#[test]
fn link_closed_during_exchange_gives_not_connected() {
    let (mut session, sends) = session(vec![vec![Input::LinkClosed]], LONG);

    let result = session.exchange(&battery_request());

    assert_eq!(result, Err(MouseError::NotConnected));
    assert_eq!(sends.get(), 1);
}

#[test]
fn exchange_after_link_closed_sends_nothing() {
    let (mut session, sends) = session(vec![vec![Input::LinkClosed]], SHORT);
    let first = session.exchange(&battery_request());

    let second = session.exchange(&battery_request());

    assert_eq!(first, Err(MouseError::NotConnected));
    assert_eq!(second, Err(MouseError::NotConnected));
    assert_eq!(sends.get(), 1);
}

#[test]
fn report_for_upgrade_code_is_ignored() {
    let script = vec![vec![reply(91, 0, 0, &[0x40]), reply(0x04, 0, 0, &[1, 0])]];
    let (mut session, _) = session(script, LONG);

    session.exchange(&battery_request()).unwrap();

    assert_eq!(session.take_status_flags(), Vec::<u8>::new());
}

#[test]
fn into_inbox_keeps_deferred_inputs() {
    let script = vec![vec![Input::Command(5), reply(0x04, 0, 0, &[1, 0])]];
    let (mut session, _) = session(script, LONG);
    session.exchange(&battery_request()).unwrap();

    let mut inbox = session.into_inbox();

    assert_eq!(inbox.next(), Some(Input::Command(5)));
}

#[test]
fn status_event_with_zero_length_keeps_flags() {
    let script = vec![vec![reply(0x0a, 0, 0, &[]), reply(0x04, 0, 0, &[1, 0])]];
    let mut zero_length = script;
    if let Input::Report(bytes) = &mut zero_length[0][0] {
        bytes[6] = 0x40;
        bytes[16] = check::sum(&bytes[..16]);
    }
    let (mut session, _) = session(zero_length, LONG);

    session.exchange(&battery_request()).unwrap();

    assert_eq!(session.take_status_flags(), vec![0x40]);
}

#[test]
fn rejected_eeprom_reply_without_address_is_rejected() {
    let (mut session, sends) = session(vec![vec![reply(0x07, 2, 0, &[])]], LONG);
    let request = Frame::set_eeprom(0x60, &[1]).unwrap();

    let result = session.exchange(&request);

    assert_eq!(result, Err(MouseError::Rejected(ReplyStatus::EepromError)));
    assert_eq!(sends.get(), 1);
}

#[test]
fn set_eeprom_reply_for_other_address_is_skipped() {
    let script = vec![vec![reply(0x07, 0, 0x00, &[1]), reply(0x07, 0, 0x60, &[1])]];
    let (mut session, _) = session(script, LONG);
    let request = Frame::set_eeprom(0x60, &[1]).unwrap();

    let frame = session.exchange(&request).unwrap();

    assert_eq!(frame.address(), 0x60);
}
