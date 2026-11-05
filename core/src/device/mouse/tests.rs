use std::cell::RefCell;
use std::rc::Rc;
use std::sync::mpsc::{self, Sender};
use std::time::Duration;

use super::*;
use crate::device::{Input, Timing};
use crate::protocol::{REPORT_LEN, ReplyStatus, check};
use crate::settings::{DpiAction, MouseAction, Sensor, ShortcutKey};

const FAST: Timing = Timing {
    reply_timeout: Duration::from_millis(20),
    retries: 0,
};

struct Fake {
    eeprom: [u8; 0x200],
    log: Vec<[u8; REPORT_LEN]>,
    cid: u8,
    mid: u8,
    online: u8,
    battery: [u8; 2],
    profile: u8,
    drop_writes: bool,
    mute_writes: bool,
    ack_then_silent: bool,
    silent_reads: bool,
    drop_address: Option<u16>,
    reject_writes: bool,
    zero_length: bool,
}

impl Fake {
    fn codes(&self) -> Vec<u8> {
        self.log.iter().map(|report| report[1]).collect()
    }

    fn writes(&self) -> Vec<(u16, Vec<u8>)> {
        self.log
            .iter()
            .filter(|report| report[1] == Command::SetEeprom.code())
            .map(|report| {
                let address = u16::from_be_bytes([report[3], report[4]]);

                (address, report[6..6 + usize::from(report[5])].to_vec())
            })
            .collect()
    }

    fn put(&mut self, address: u16, bytes: &[u8]) {
        let start = usize::from(address);
        self.eeprom[start..start + bytes.len()].copy_from_slice(bytes);
    }

    fn answer(&mut self, request: &Frame) -> Option<(u8, Vec<u8>)> {
        let address = usize::from(request.address());
        let len = usize::from(request.to_report()[5]);

        match request.command_code() {
            0x03 => Some((0, vec![self.online])),
            0x04 => Some((0, self.battery.to_vec())),
            0x0e => Some((0, vec![self.profile])),
            0x10 => Some((0, vec![self.cid, self.mid, 0x21])),
            0x08 if self.silent_reads => None,
            0x08 => Some((0, self.eeprom[address..address + len].to_vec())),
            0x07 if self.mute_writes => None,
            0x07 if self.reject_writes => Some((1, Vec::new())),
            0x07 => {
                self.silent_reads = self.ack_then_silent;

                if !self.drop_writes && self.drop_address != Some(request.address()) {
                    let data = request.to_report()[6..6 + len].to_vec();
                    self.eeprom[address..address + len].copy_from_slice(&data);
                }

                Some((0, request.to_report()[6..6 + len].to_vec()))
            }
            _ => None,
        }
    }
}

struct FakeLink {
    sender: Sender<Input<u8>>,
    fake: Rc<RefCell<Fake>>,
}

impl Link for FakeLink {
    fn send(&mut self, report: &[u8; REPORT_LEN]) -> Result<(), MouseError> {
        let mut fake = self.fake.borrow_mut();
        fake.log.push(*report);
        let request = Frame::parse(report).expect("openatk sends valid frames");

        if let Some((status, data)) = fake.answer(&request) {
            let length = if fake.zero_length { 0 } else { data.len() };
            let mut reply = [0u8; REPORT_LEN];
            reply[0] = 0x08;
            reply[1] = request.command_code();
            reply[2] = status;
            reply[3..5].copy_from_slice(&request.address().to_be_bytes());
            reply[5] = length as u8;
            reply[6..6 + data.len()].copy_from_slice(&data);
            reply[16] = check::sum(&reply[..16]);
            self.sender
                .send(Input::Report(reply.to_vec()))
                .expect("inbox alive");
        }

        Ok(())
    }
}

fn stage(dpi: u32) -> DpiStage {
    DpiStage { x: dpi, y: dpi }
}

fn encoded(dpi: u32) -> [u8; 4] {
    Sensor::Paw3950Ultra
        .encode_stage(stage(dpi))
        .expect("grid value")
}

fn new_fake() -> Fake {
    let mut fake = Fake {
        eeprom: [0; 0x200],
        log: Vec::new(),
        cid: 2,
        mid: 50,
        online: 1,
        battery: [80, 1],
        profile: 1,
        drop_writes: false,
        mute_writes: false,
        ack_then_silent: false,
        silent_reads: false,
        drop_address: None,
        reject_writes: false,
        zero_length: false,
    };

    // Polling 1000 Hz, 3 stages, current stage 1, then unknown bytes 6-9.
    let block = [
        1,
        check::single(1),
        3,
        check::single(3),
        1,
        check::single(1),
    ];
    fake.put(0, &block);
    fake.put(6, &[0x07, 0x11, 0x22, 0x33]);

    // Stages 800, 1600, 3200; the rest was never written.
    fake.put(0x0c, &[0xff; 32]);
    fake.put(0x0c, &encoded(800));
    fake.put(0x10, &encoded(1600));
    fake.put(0x14, &encoded(3200));

    let actions = [
        ButtonAction::Mouse(MouseAction::Left),
        ButtonAction::Mouse(MouseAction::Right),
        ButtonAction::Mouse(MouseAction::Middle),
        ButtonAction::Shortcut,
        ButtonAction::Dpi(DpiAction::Cycle),
    ];
    for (button, action) in model().buttons.iter().zip(actions) {
        fake.put(button.address(), &action.encode().expect("editable"));
    }

    fake
}

fn model() -> &'static Model {
    Model::find(2, 50).expect("known model")
}

type Parts = (Session<FakeLink, u8>, Rc<RefCell<Fake>>, Sender<Input<u8>>);

fn rig_with(fake: Fake) -> Parts {
    let fake = Rc::new(RefCell::new(fake));
    let (sender, receiver) = mpsc::channel();
    let link = FakeLink {
        sender: sender.clone(),
        fake: Rc::clone(&fake),
    };

    (Session::new(link, Inbox::new(receiver), FAST), fake, sender)
}

fn rig(path: DevicePath) -> (Mouse<FakeLink, u8>, Rc<RefCell<Fake>>) {
    let (session, fake, _) = rig_with(new_fake());

    (Mouse::new(session, model(), path), fake)
}

fn ctrl_c() -> Shortcut {
    Shortcut::from_keys(&[ShortcutKey::Modifier(1), ShortcutKey::General(6)]).expect("valid keys")
}

#[test]
fn identify_rejects_unknown_model() {
    let mut fake = new_fake();
    fake.cid = 9;
    let (mut session, _, _) = rig_with(fake);

    let result = Mouse::identify(&mut session, DevicePath::Cable);

    assert_eq!(
        result.unwrap_err(),
        MouseError::UnsupportedModel { cid: 9, mid: 50 }
    );
}

#[test]
fn identify_reports_offline_when_dongle_says_0() {
    let mut fake = new_fake();
    fake.online = 0;
    let (mut session, fake, _) = rig_with(fake);

    let result = Mouse::identify(&mut session, DevicePath::Dongle);

    assert_eq!(result.unwrap_err(), MouseError::NotConnected);
    assert_eq!(fake.borrow().codes(), [0x03]);
}

#[test]
fn identify_skips_online_check_on_cable() {
    let mut fake = new_fake();
    fake.online = 0;
    let (mut session, fake, _) = rig_with(fake);

    let found = Mouse::identify(&mut session, DevicePath::Cable).expect("identified");

    assert_eq!(found.name, "ATK F1 EXTREME");
    assert_eq!(fake.borrow().codes(), [0x10]);
}

#[test]
fn into_inbox_keeps_deferred_inputs() {
    let (session, _, sender) = rig_with(new_fake());
    let mut mouse = Mouse::new(session, model(), DevicePath::Cable);
    sender.send(Input::Command(7)).expect("inbox alive");

    mouse.battery().expect("battery");
    let mut inbox = mouse.into_inbox();

    assert_eq!(inbox.next(), Some(Input::Command(7)));
}

#[test]
fn replies_are_read_by_requested_length() {
    let (mut mouse, fake) = rig(DevicePath::Cable);
    fake.borrow_mut().zero_length = true;

    let bytes = mouse.read_eeprom(0x0c, 4).expect("read");
    let battery = mouse.battery().expect("battery");

    assert_eq!(bytes, encoded(800));
    assert_eq!(
        battery,
        Battery {
            percent: 80,
            charging: true
        }
    );
}

#[test]
fn battery_above_100_is_an_error() {
    let (mut mouse, fake) = rig(DevicePath::Cable);
    fake.borrow_mut().battery = [101, 0];

    let result = mouse.battery();

    assert!(matches!(
        result,
        Err(MouseError::Protocol(ProtocolError::InvalidValue(_)))
    ));
}

#[test]
fn snapshot_reads_all_fields() {
    let (mut mouse, fake) = rig(DevicePath::Dongle);
    let mut frames = ctrl_c().to_frames();
    frames.push([0; 10]);
    fake.borrow_mut()
        .put(Button::Side1.shortcut_address(), &frames[0]);
    fake.borrow_mut()
        .put(Button::Side1.shortcut_address() + 10, &frames[1]);

    let snapshot = mouse.snapshot().expect("snapshot");

    assert_eq!(snapshot.model_name, "ATK F1 EXTREME");
    assert_eq!((snapshot.cid, snapshot.mid, snapshot.profile), (2, 50, 1));
    assert_eq!(
        snapshot.battery,
        Battery {
            percent: 80,
            charging: true
        }
    );
    assert_eq!(snapshot.polling, PollingRate::Hz1000);
    assert_eq!(snapshot.polling_rates.len(), 7);
    assert_eq!(snapshot.current_stage, 1);
    assert_eq!(snapshot.stages, [stage(800), stage(1600), stage(3200)]);
    assert_eq!(snapshot.buttons.len(), 5);
    assert_eq!(snapshot.buttons[3], (Button::Side1, ButtonAction::Shortcut));
    assert_eq!(snapshot.shortcuts, [(Button::Side1, ctrl_c())]);
    assert_eq!(snapshot.nearest_valid_dpi(804), 800);
    assert_eq!(snapshot.dpi_range(), 10..=42000);
    assert!(fake.borrow().writes().is_empty());
}

#[test]
fn snapshot_leaves_out_a_button_without_stored_shortcut() {
    let (mut mouse, _) = rig(DevicePath::Cable);

    let snapshot = mouse.snapshot().expect("snapshot");

    assert!(snapshot.shortcuts.is_empty());
}

#[test]
fn snapshot_tolerates_bad_unused_stage() {
    let (mut mouse, fake) = rig(DevicePath::Cable);
    fake.borrow_mut().put(0x1c, &[1, 2, 3, 4]);

    let snapshot = mouse.snapshot().expect("snapshot");

    assert_eq!(snapshot.stages.len(), 3);
}

#[test]
fn snapshot_fails_on_bad_used_stage() {
    let (mut mouse, fake) = rig(DevicePath::Cable);
    fake.borrow_mut().put(0x10, &[1, 2, 3, 4]);

    let result = mouse.snapshot();

    assert!(matches!(
        result,
        Err(MouseError::Protocol(ProtocolError::BadCheck { .. }))
    ));
}

#[test]
fn set_polling_reads_block_first_and_keeps_neighbours() {
    let (mut mouse, fake) = rig(DevicePath::Dongle);

    mouse.set_polling(PollingRate::Hz8000).expect("set");

    let fake = fake.borrow();
    assert_eq!(fake.codes()[..2], [0x08, 0x07]);
    assert_eq!(fake.eeprom[0..2], [64, check::single(64)]);
    assert_eq!(
        fake.eeprom[2..10],
        [
            3,
            check::single(3),
            1,
            check::single(1),
            7,
            0x11,
            0x22,
            0x33
        ]
    );
}

#[test]
fn set_polling_rejects_rate_the_path_lacks() {
    let (mut mouse, fake) = rig(DevicePath::Cable);

    let result = mouse.set_polling(PollingRate::Hz8000);

    assert!(result.is_err());
    assert!(fake.borrow().log.is_empty());
}

#[test]
fn setter_sends_no_write_when_block_is_malformed() {
    let (mut mouse, fake) = rig(DevicePath::Cable);
    fake.borrow_mut().put(1, &[0]);

    let result = mouse.set_polling(PollingRate::Hz500);

    assert!(result.is_err());
    assert!(fake.borrow().writes().is_empty());
}

#[test]
fn read_back_difference_is_an_error() {
    let (mut mouse, fake) = rig(DevicePath::Cable);
    fake.borrow_mut().drop_writes = true;

    let result = mouse.set_polling(PollingRate::Hz500);

    assert!(matches!(
        result,
        Err(MouseError::ReadBackMismatch { address: 0, .. })
    ));
    assert_eq!(
        mouse.snapshot().expect("snapshot").polling,
        PollingRate::Hz1000
    );
}

#[test]
fn write_timeout_leaves_real_value() {
    let (mut mouse, fake) = rig(DevicePath::Cable);
    fake.borrow_mut().mute_writes = true;

    let result = mouse.set_polling(PollingRate::Hz500);

    assert_eq!(result, Err(MouseError::Timeout));
    assert_eq!(
        mouse.snapshot().expect("snapshot").polling,
        PollingRate::Hz1000
    );
}

#[test]
fn rejected_write_is_not_read_back() {
    let (mut mouse, fake) = rig(DevicePath::Cable);
    fake.borrow_mut().reject_writes = true;

    let result = mouse.set_polling(PollingRate::Hz500);

    assert_eq!(result, Err(MouseError::Rejected(ReplyStatus::Fail)));
    assert_eq!(fake.borrow().codes(), [0x08, 0x07]);
}

#[test]
fn set_current_stage_rejects_index_at_count() {
    let (mut mouse, fake) = rig(DevicePath::Cable);

    let result = mouse.set_current_stage(3);

    assert!(result.is_err());
    assert!(fake.borrow().writes().is_empty());
}

#[test]
fn set_current_stage_writes_block() {
    let (mut mouse, fake) = rig(DevicePath::Cable);

    mouse.set_current_stage(2).expect("set");

    assert_eq!(fake.borrow().eeprom[4..6], [2, check::single(2)]);
}

#[test]
fn set_stage_dpi_keeps_paired_stage() {
    let (mut mouse, fake) = rig(DevicePath::Cable);
    fake.borrow_mut().put(0x18, &encoded(6400));
    mouse.set_stage_count(4).expect("grow");
    fake.borrow_mut().log.clear();

    mouse.set_stage_dpi(3, stage(5000)).expect("set");

    let fake = fake.borrow();
    let mut expected = encoded(3200).to_vec();
    expected.extend(encoded(5000));
    assert_eq!(fake.writes(), [(0x14, expected)]);
}

#[test]
fn set_stage_dpi_rejects_off_grid_value() {
    let (mut mouse, fake) = rig(DevicePath::Cable);

    let result = mouse.set_stage_dpi(0, stage(805));

    assert!(result.is_err());
    assert!(fake.borrow().log.is_empty());
}

#[test]
fn set_stage_dpi_rejects_index_at_count() {
    let (mut mouse, fake) = rig(DevicePath::Cable);

    let result = mouse.set_stage_dpi(3, stage(800));

    assert!(result.is_err());
    assert!(fake.borrow().writes().is_empty());
}

#[test]
fn growing_stage_count_writes_stage_first() {
    let (mut mouse, fake) = rig(DevicePath::Cable);
    assert_eq!(fake.borrow().eeprom[0x18..0x1c], [0xff; 4]);

    mouse.set_stage_count(4).expect("grow");

    let fake = fake.borrow();
    let writes = fake.writes();
    let mut pair = encoded(3200).to_vec();
    pair.extend(encoded(3200));
    assert_eq!(writes.len(), 2);
    assert_eq!(writes[0], (0x14, pair));
    assert_eq!(writes[1].0, 0);
    assert_eq!(fake.eeprom[2..4], [4, check::single(4)]);
}

#[test]
fn growing_keeps_a_new_slot_that_decodes() {
    let (mut mouse, fake) = rig(DevicePath::Cable);
    fake.borrow_mut().put(0x18, &encoded(6400));

    mouse.set_stage_count(4).expect("grow");

    let fake = fake.borrow();
    assert_eq!(fake.writes().len(), 1);
    assert_eq!(fake.eeprom[0x18..0x1c], encoded(6400));
}

#[test]
fn shrinking_count_moves_current_stage_down() {
    let (mut mouse, fake) = rig(DevicePath::Cable);
    mouse.set_current_stage(2).expect("set");
    fake.borrow_mut().log.clear();

    mouse.set_stage_count(2).expect("shrink");

    let fake = fake.borrow();
    assert_eq!(fake.writes().len(), 1);
    assert_eq!(
        fake.eeprom[2..6],
        [2, check::single(2), 1, check::single(1)]
    );
}

#[test]
fn stage_count_outside_range_sends_nothing_to_write() {
    let (mut mouse, fake) = rig(DevicePath::Cable);

    let result = mouse.set_stage_count(9);

    assert!(result.is_err());
    assert!(fake.borrow().writes().is_empty());
}

#[test]
fn set_button_writes_action_after_reading_it() {
    let (mut mouse, fake) = rig(DevicePath::Cable);

    mouse
        .set_button(Button::Side2, ButtonAction::Mouse(MouseAction::Middle))
        .expect("set");

    let expected = ButtonAction::Mouse(MouseAction::Middle)
        .encode()
        .expect("editable");
    assert_eq!(fake.borrow().codes(), [0x08, 0x07, 0x08]);
    assert_eq!(fake.borrow().writes(), [(0x70, expected.to_vec())]);
}

#[test]
fn set_button_refuses_other() {
    let (mut mouse, fake) = rig(DevicePath::Cable);
    let other = ButtonAction::Other {
        class: 9,
        value1: 1,
        value2: 0,
    };

    let result = mouse.set_button(Button::Left, other);

    assert!(result.is_err());
    assert!(fake.borrow().log.is_empty());
}

#[test]
fn set_button_refuses_shortcut() {
    let (mut mouse, fake) = rig(DevicePath::Cable);

    let result = mouse.set_button(Button::Left, ButtonAction::Shortcut);

    assert!(result.is_err());
    assert!(fake.borrow().log.is_empty());
}

#[test]
fn set_shortcut_writes_frames_then_button_class() {
    let (mut mouse, fake) = rig(DevicePath::Cable);
    let frames = ctrl_c().to_frames();

    mouse.set_shortcut(Button::Left, &ctrl_c()).expect("set");

    let base = Button::Left.shortcut_address();
    let shortcut_class = ButtonAction::Shortcut.encode().expect("editable").to_vec();
    assert_eq!(
        fake.borrow().writes(),
        [
            (base, frames[0].to_vec()),
            (base + 10, frames[1].to_vec()),
            (Button::Left.address(), shortcut_class),
        ]
    );
}

fn set_block(fake: &Rc<RefCell<Fake>>, count: u8, current: u8) {
    let block = [
        1,
        check::single(1),
        count,
        check::single(count),
        current,
        check::single(current),
    ];
    fake.borrow_mut().put(0, &block);
}

fn mismatch_address(result: Result<(), MouseError>) -> u16 {
    match result {
        Err(MouseError::ReadBackMismatch { address, .. }) => address,
        other => panic!("expected ReadBackMismatch, got {other:?}"),
    }
}

#[test]
fn set_shortcut_with_bad_button_bytes_writes_nothing() {
    let (mut mouse, fake) = rig(DevicePath::Cable);
    fake.borrow_mut().put(Button::Left.address(), &[1, 1, 0, 0]);

    let result = mouse.set_shortcut(Button::Left, &ctrl_c());

    assert!(matches!(
        result,
        Err(MouseError::Protocol(ProtocolError::BadCheck { .. }))
    ));
    assert!(fake.borrow().writes().is_empty());
}

#[test]
fn set_shortcut_reads_button_before_first_frame() {
    let (mut mouse, fake) = rig(DevicePath::Cable);

    mouse.set_shortcut(Button::Left, &ctrl_c()).expect("set");

    assert_eq!(fake.borrow().codes()[..2], [0x08, 0x07]);
}

#[test]
fn set_button_refuses_bad_check_with_no_write() {
    let (mut mouse, fake) = rig(DevicePath::Cable);
    fake.borrow_mut().put(Button::Left.address(), &[1, 1, 0, 0]);

    let result = mouse.set_button(Button::Left, ButtonAction::Mouse(MouseAction::Right));

    assert!(matches!(
        result,
        Err(MouseError::Protocol(ProtocolError::BadCheck { .. }))
    ));
    assert!(fake.borrow().writes().is_empty());
}

fn uneditable_button(fake: &Rc<RefCell<Fake>>) {
    let other = [9, 1, 0, check::sum(&[9, 1, 0])];
    fake.borrow_mut().put(Button::Left.address(), &other);
}

#[test]
fn set_button_leaves_uneditable_button_untouched() {
    let (mut mouse, fake) = rig(DevicePath::Cable);
    uneditable_button(&fake);

    let result = mouse.set_button(Button::Left, ButtonAction::Mouse(MouseAction::Right));

    assert!(matches!(
        result,
        Err(MouseError::Protocol(ProtocolError::InvalidValue(_)))
    ));
    assert!(fake.borrow().writes().is_empty());
}

#[test]
fn set_shortcut_leaves_uneditable_button_untouched() {
    let (mut mouse, fake) = rig(DevicePath::Cable);
    uneditable_button(&fake);

    let result = mouse.set_shortcut(Button::Left, &ctrl_c());

    assert!(matches!(
        result,
        Err(MouseError::Protocol(ProtocolError::InvalidValue(_)))
    ));
    assert!(fake.borrow().writes().is_empty());
}

#[test]
fn snapshot_leaves_out_shortcut_with_bad_check_and_set_shortcut_recovers() {
    let (mut mouse, fake) = rig(DevicePath::Cable);
    let half_written = [4, 0x80, 1, 0, 0x81, 6, 0, 0x40, 1, 0];
    fake.borrow_mut()
        .put(Button::Side1.shortcut_address(), &half_written);

    let snapshot = mouse.snapshot().expect("snapshot");

    assert_eq!(snapshot.buttons[3], (Button::Side1, ButtonAction::Shortcut));
    assert!(snapshot.shortcuts.is_empty());
    mouse.set_shortcut(Button::Side1, &ctrl_c()).expect("set");
    assert_eq!(mouse.snapshot().expect("snapshot").shortcuts.len(), 1);
}

#[test]
fn silence_after_acknowledged_write_is_a_timeout() {
    let (mut mouse, fake) = rig(DevicePath::Cable);
    fake.borrow_mut().ack_then_silent = true;

    let result = mouse.set_polling(PollingRate::Hz500);

    assert_eq!(result, Err(MouseError::Timeout));
}

#[test]
fn growth_keeps_the_neighbour_of_the_pair() {
    let (mut mouse, fake) = rig(DevicePath::Cable);
    set_block(&fake, 2, 1);
    fake.borrow_mut().put(0x14, &[0xff; 4]);
    fake.borrow_mut().put(0x18, &encoded(6400));

    mouse.set_stage_count(3).expect("grow");

    let fake = fake.borrow();
    assert_eq!(fake.eeprom[0x14..0x18], encoded(1600));
    assert_eq!(fake.eeprom[0x18..0x1c], encoded(6400));
}

#[test]
fn growing_from_one_to_eight_gives_eight_stages() {
    let (mut mouse, fake) = rig(DevicePath::Cable);
    set_block(&fake, 1, 0);
    fake.borrow_mut().put(0x10, &[0xff; 16]);

    mouse.set_stage_count(8).expect("grow");

    let snapshot = mouse.snapshot().expect("snapshot");
    assert_eq!(snapshot.stages, [stage(800); 8]);
}

#[test]
fn growing_from_a_bad_last_stage_sends_no_write() {
    let (mut mouse, fake) = rig(DevicePath::Cable);
    fake.borrow_mut().put(0x14, &[1, 2, 3, 4]);

    let result = mouse.set_stage_count(4);

    assert!(matches!(
        result,
        Err(MouseError::Protocol(ProtocolError::BadCheck { .. }))
    ));
    assert!(fake.borrow().writes().is_empty());
}

#[test]
fn set_stage_dpi_is_verified() {
    let (mut mouse, fake) = rig(DevicePath::Cable);
    fake.borrow_mut().drop_writes = true;

    let result = mouse.set_stage_dpi(0, stage(900));

    assert_eq!(mismatch_address(result), 0x0c);
}

#[test]
fn stage_growth_is_verified() {
    let (mut mouse, fake) = rig(DevicePath::Cable);
    fake.borrow_mut().drop_writes = true;

    let result = mouse.set_stage_count(4);

    assert_eq!(mismatch_address(result), 0x14);
}

#[test]
fn stage_count_block_write_is_verified() {
    let (mut mouse, fake) = rig(DevicePath::Cable);
    fake.borrow_mut().drop_writes = true;

    let result = mouse.set_stage_count(2);

    assert_eq!(mismatch_address(result), 0);
}

#[test]
fn set_shortcut_verifies_each_frame_and_the_button() {
    let base = Button::Left.shortcut_address();

    for dropped in [base, base + 10, Button::Left.address()] {
        let (mut mouse, fake) = rig(DevicePath::Cable);
        fake.borrow_mut().drop_address = Some(dropped);

        let result = mouse.set_shortcut(Button::Left, &ctrl_c());

        assert_eq!(mismatch_address(result), dropped);
    }
}

#[test]
fn shrinking_keeps_current_stage_zero() {
    let (mut mouse, fake) = rig(DevicePath::Cable);
    set_block(&fake, 3, 0);

    mouse.set_stage_count(2).expect("shrink");

    assert_eq!(fake.borrow().eeprom[4..6], [0, check::single(0)]);
}

#[test]
fn online_byte_other_than_1_is_not_connected() {
    let mut fake = new_fake();
    fake.online = 2;
    let (mut session, _, _) = rig_with(fake);

    let result = Mouse::identify(&mut session, DevicePath::Dongle);

    assert_eq!(result.unwrap_err(), MouseError::NotConnected);
}

#[test]
fn shortcuts_are_read_only_for_shortcut_buttons() {
    let (mut mouse, fake) = rig(DevicePath::Cable);
    let side1 = ButtonAction::Mouse(MouseAction::Side1)
        .encode()
        .expect("editable");
    fake.borrow_mut().put(Button::Side1.address(), &side1);

    mouse.snapshot().expect("snapshot");

    let fake = fake.borrow();
    let reads_shortcuts = fake
        .log
        .iter()
        .any(|report| report[1] == 0x08 && report[3] >= 0x01);
    assert!(!reads_shortcuts);
}
