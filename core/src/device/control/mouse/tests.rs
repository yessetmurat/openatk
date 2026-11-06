use std::cell::RefCell;
use std::rc::Rc;
use std::sync::mpsc::{self, Sender};
use std::time::Duration;

use super::*;
use crate::device::{AngleValue, Input, Timing};
use crate::protocol::{REPORT_LEN, ReplyStatus, check};
use crate::settings::{DpiAction, FirmwareVersion, MouseAction, Sensor, ShortcutKey};

const FAST: Timing = Timing {
    reply_timeout: Duration::from_millis(20),
    retries: 0,
};

const LIFT_OFF_BYTES: [u8; 2] = [0x01, 0x54];
const SENSOR_BYTES: [u8; 10] = [0x00, 0x55, 0x00, 0x55, 0x01, 0x54, 0x00, 0x55, 0x00, 0x55];
const SLEEP_BYTES: [u8; 6] = [0x00, 0x55, 0x06, 0x4f, 0x00, 0x55];
const ANGLE_BYTES: [u8; 4] = [0x00, 0x55, 0x00, 0x55];

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
    firmware: Option<[u8; 2]>,
    dongle_firmware: Option<[u8; 2]>,
    silent_dongle_version: bool,
    silent_mouse_version: bool,
    far_distance: Option<u8>,
    unavailable_status: u8,
    reject_read_at: Option<u16>,
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

    fn at(&self, address: u16, len: usize) -> &[u8] {
        &self.eeprom[usize::from(address)..usize::from(address) + len]
    }

    fn read_addresses(&self) -> Vec<u16> {
        self.log
            .iter()
            .filter(|report| report[1] == Command::GetEeprom.code())
            .map(|report| u16::from_be_bytes([report[3], report[4]]))
            .collect()
    }

    fn answer(&mut self, request: &Frame) -> Option<(u8, Vec<u8>)> {
        let address = usize::from(request.address());
        let len = usize::from(request.to_report()[5]);

        match request.command_code() {
            0x03 => Some((0, vec![self.online])),
            0x04 => Some((0, self.battery.to_vec())),
            0x0e => Some((0, vec![self.profile])),
            0x0f => {
                if !self.drop_writes {
                    self.profile = request.payload()[0];
                }

                Some((0, vec![request.payload()[0]]))
            }
            0x10 => Some((0, vec![self.cid, self.mid, 0x21])),
            0x12 if self.silent_mouse_version => None,
            0x12 => Some(match self.firmware {
                Some(version) => (0, version.to_vec()),
                None => (self.unavailable_status, Vec::new()),
            }),
            0x1d if self.silent_dongle_version => None,
            0x1d => Some(match self.dongle_firmware {
                Some(version) => (0, version.to_vec()),
                None => (self.unavailable_status, Vec::new()),
            }),
            0x16 => {
                let mode = request.payload()[0];

                if !self.drop_writes {
                    self.far_distance = Some(mode);
                }

                Some((0, vec![mode]))
            }
            0x17 => Some(match self.far_distance {
                Some(mode) => (0, vec![mode]),
                None => (self.unavailable_status, Vec::new()),
            }),
            0x08 if self.silent_reads => None,
            0x08 if self.reject_read_at == Some(request.address()) => Some((1, Vec::new())),
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
        firmware: Some([0x03, 0x01]),
        dongle_firmware: Some([0x01, 0x0a]),
        silent_dongle_version: false,
        silent_mouse_version: false,
        far_distance: Some(0),
        unavailable_status: 255,
        reject_read_at: None,
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

    // The bytes read from the real mouse.
    fake.put(address::LIFT_OFF, &LIFT_OFF_BYTES);
    fake.put(address::SENSOR_BLOCK, &SENSOR_BYTES);
    fake.put(address::SLEEP_BLOCK, &SLEEP_BYTES);
    fake.put(address::ANGLE_BLOCK, &ANGLE_BYTES);

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
    assert!(!snapshot.on_cable);
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
fn snapshot_on_the_cable_path_is_on_cable() {
    let (mut mouse, _) = rig(DevicePath::Cable);

    let snapshot = mouse.snapshot().expect("snapshot");

    assert!(snapshot.on_cable);
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

#[test]
fn snapshot_reads_the_sensor_settings_of_the_real_mouse() {
    let (mut mouse, _) = rig(DevicePath::Cable);

    let snapshot = mouse.snapshot().expect("snapshot");

    assert_eq!(
        snapshot.firmware,
        Some(FirmwareVersion { major: 3, minor: 1 })
    );
    assert_eq!(snapshot.lift_off_tenths, Some(7));
    assert_eq!(snapshot.debounce_ms, Some(0));
    assert_eq!(snapshot.motion_sync, Some(false));
    assert_eq!(snapshot.linear_correction, Some(false));
    assert_eq!(snapshot.ripple_control, Some(false));
    assert_eq!(snapshot.sleep_seconds, Some(10));
    assert_eq!(
        snapshot.angle,
        Some(AngleValue {
            degrees: Some(0),
            enabled: Some(false),
        })
    );
    assert_eq!(snapshot.far_distance, Some(false));
}

#[test]
fn snapshot_carries_the_lists_and_ranges_of_the_model() {
    let (mut mouse, _) = rig(DevicePath::Cable);

    let snapshot = mouse.snapshot().expect("snapshot");

    assert_eq!(snapshot.debounce_options_ms, model().debounce_ms);
    assert_eq!(snapshot.sleep_options_seconds, model().sleep_seconds);
    assert_eq!(snapshot.lift_off_range(), 7..=17);
    assert_eq!(snapshot.angle_range(), -30..=30);
}

#[test]
fn snapshot_tolerates_a_rejected_block() {
    for address in [
        address::LIFT_OFF,
        address::SENSOR_BLOCK,
        address::ANGLE_BLOCK,
    ] {
        let (mut mouse, fake) = rig(DevicePath::Cable);
        fake.borrow_mut().reject_read_at = Some(address);

        let snapshot = mouse.snapshot().expect("snapshot");

        assert_eq!(
            snapshot.lift_off_tenths.is_none(),
            address == address::LIFT_OFF
        );
        assert_eq!(
            snapshot.debounce_ms.is_none(),
            address == address::SENSOR_BLOCK
        );
        assert_eq!(
            snapshot.sleep_seconds.is_none(),
            address == address::SENSOR_BLOCK
        );
        assert_eq!(snapshot.angle.is_none(), address == address::ANGLE_BLOCK);
    }
}

#[test]
fn one_bad_pair_reads_as_unknown_and_keeps_the_snapshot() {
    let (mut mouse, fake) = rig(DevicePath::Cable);
    fake.borrow_mut()
        .put(address::SENSOR_BLOCK + 2, &[0x00, 0x99]);
    fake.borrow_mut()
        .put(address::ANGLE_BLOCK + 2, &[0x00, 0x99]);

    let snapshot = mouse.snapshot().expect("snapshot");

    assert_eq!(snapshot.motion_sync, None);
    assert_eq!(snapshot.debounce_ms, Some(0));
    assert_eq!(
        snapshot.angle,
        Some(AngleValue {
            degrees: Some(0),
            enabled: None,
        })
    );
}

#[test]
fn setter_refuses_its_own_bad_pair_with_no_write() {
    let (mut mouse, fake) = rig(DevicePath::Cable);
    fake.borrow_mut()
        .put(address::SENSOR_BLOCK + 2, &[0x00, 0x99]);

    let result = mouse.set_motion_sync(true);

    assert!(matches!(
        result,
        Err(MouseError::Protocol(ProtocolError::BadCheck { .. }))
    ));
    assert!(fake.borrow().writes().is_empty());
}

#[test]
fn bad_neighbour_does_not_block_a_write_and_stays_as_it_is() {
    let (mut mouse, fake) = rig(DevicePath::Cable);
    fake.borrow_mut()
        .put(address::SENSOR_BLOCK + 2, &[0x00, 0x99]);

    mouse.set_debounce(4).expect("own pair is readable");

    let fake = fake.borrow();
    assert_eq!(
        fake.at(address::SENSOR_BLOCK, 10),
        [0x04, 0x51, 0x00, 0x99, 0x01, 0x54, 0x00, 0x55, 0x00, 0x55]
    );
}

#[test]
fn set_lift_off_writes_the_byte_and_reads_it_back() {
    let (mut mouse, fake) = rig(DevicePath::Cable);

    mouse.set_lift_off(17).expect("set");

    let fake = fake.borrow();
    assert_eq!(fake.codes(), [0x08, 0x07, 0x08]);
    assert_eq!(fake.writes(), [(address::LIFT_OFF, vec![0x0b, 0x4a])]);
}

#[test]
fn set_lift_off_rejects_a_height_outside_the_range() {
    let (mut mouse, fake) = rig(DevicePath::Cable);

    assert!(mouse.set_lift_off(6).is_err());
    assert!(mouse.set_lift_off(18).is_err());

    assert!(fake.borrow().writes().is_empty());
}

#[test]
fn set_debounce_rejects_values_outside_the_list() {
    let (mut mouse, fake) = rig(DevicePath::Cable);

    let result = mouse.set_debounce(3);

    assert_eq!(
        result,
        Err(ProtocolError::InvalidValue("debounce time").into())
    );
    assert!(fake.borrow().log.is_empty());
}

#[test]
fn toggles_change_only_their_pair() {
    let (mut mouse, fake) = rig(DevicePath::Cable);

    mouse.set_linear_correction(true).expect("set");
    mouse.set_ripple_control(true).expect("set");

    assert_eq!(
        fake.borrow().at(address::SENSOR_BLOCK, 10),
        [0x00, 0x55, 0x00, 0x55, 0x01, 0x54, 0x01, 0x54, 0x01, 0x54]
    );
}

#[test]
fn off_list_led_byte_survives_a_motion_sync_write() {
    let (mut mouse, fake) = rig(DevicePath::Cable);

    mouse.set_motion_sync(true).expect("set");

    let snapshot = mouse.snapshot().expect("snapshot");
    assert_eq!(snapshot.sleep_seconds, Some(10));
    assert_eq!(snapshot.motion_sync, Some(true));
    assert_eq!(
        fake.borrow().at(address::SENSOR_BLOCK, 10),
        [0x00, 0x55, 0x01, 0x54, 0x01, 0x54, 0x00, 0x55, 0x00, 0x55]
    );
}

#[test]
fn sensor_block_write_is_verified() {
    let (mut mouse, fake) = rig(DevicePath::Cable);
    fake.borrow_mut().drop_address = Some(address::SENSOR_BLOCK);

    let result = mouse.set_motion_sync(true);

    assert!(matches!(
        result,
        Err(MouseError::ReadBackMismatch { address: 0xa9, .. })
    ));
}

#[test]
fn sleep_writes_both_blocks_and_sets_enabled() {
    let (mut mouse, fake) = rig(DevicePath::Cable);

    mouse.set_sleep(1800).expect("set");

    let fake = fake.borrow();
    assert_eq!(
        fake.writes(),
        [
            (
                address::SENSOR_BLOCK,
                vec![0x00, 0x55, 0x00, 0x55, 180, 0xa1, 0x00, 0x55, 0x00, 0x55]
            ),
            (
                address::SLEEP_BLOCK,
                vec![0x01, 0x54, 180, 0xa1, 0x00, 0x55]
            ),
        ]
    );
}

#[test]
fn sleep_keeps_sensor_model() {
    let (mut mouse, fake) = rig(DevicePath::Cable);
    fake.borrow_mut()
        .put(address::SLEEP_BLOCK + 4, &[0x07, 0x99]);

    mouse.set_sleep(60).expect("set");

    assert_eq!(
        fake.borrow().at(address::SLEEP_BLOCK, 6),
        [0x01, 0x54, 6, 0x4f, 0x07, 0x99]
    );
}

#[test]
fn sleep_refused_by_either_block_writes_nothing() {
    for bad_pair in [address::SENSOR_BLOCK + 4, address::SLEEP_BLOCK + 2] {
        let (mut mouse, fake) = rig(DevicePath::Cable);
        fake.borrow_mut().put(bad_pair, &[0x06, 0x99]);

        let result = mouse.set_sleep(60);

        assert!(result.is_err());
        assert!(fake.borrow().writes().is_empty());
    }
}

#[test]
fn partial_sleep_write_is_reported_and_a_second_set_heals_it() {
    let (mut mouse, fake) = rig(DevicePath::Cable);
    fake.borrow_mut().drop_address = Some(address::SLEEP_BLOCK);

    let result = mouse.set_sleep(60);

    assert_eq!(
        result,
        Err(MouseError::ReadBackMismatch {
            address: address::SLEEP_BLOCK,
            written: vec![0x01, 0x54, 6, 0x4f, 0x00, 0x55],
            actual: SLEEP_BYTES.to_vec(),
        })
    );

    // Only the LED off half landed, and the snapshot shows only that half.
    let snapshot = mouse.snapshot().expect("snapshot");
    assert_eq!(snapshot.sleep_seconds, Some(60));
    assert_eq!(fake.borrow().at(address::SLEEP_BLOCK, 6), SLEEP_BYTES);

    fake.borrow_mut().drop_address = None;
    fake.borrow_mut().log.clear();

    mouse.set_sleep(60).expect("the same value again");

    let fake = fake.borrow();
    let written: Vec<u16> = fake.writes().iter().map(|(address, _)| *address).collect();
    assert_eq!(written, [address::SENSOR_BLOCK, address::SLEEP_BLOCK]);
    assert_eq!(
        fake.at(address::SLEEP_BLOCK, 6),
        [0x01, 0x54, 6, 0x4f, 0x00, 0x55]
    );
}

#[test]
fn set_sleep_rejects_values_outside_the_list() {
    let (mut mouse, fake) = rig(DevicePath::Cable);

    let result = mouse.set_sleep(10);

    assert_eq!(
        result,
        Err(ProtocolError::InvalidValue("sleep time").into())
    );
    assert!(fake.borrow().log.is_empty());
}

#[test]
fn below_minimum_firmware_has_no_angle() {
    let (mut mouse, fake) = rig(DevicePath::Cable);
    fake.borrow_mut().firmware = Some([0x02, 0x99]);

    let snapshot = mouse.snapshot().expect("snapshot");

    assert_eq!(snapshot.angle, None);
    assert!(
        !fake
            .borrow()
            .read_addresses()
            .contains(&address::ANGLE_BLOCK)
    );
}

#[test]
fn not_supported_firmware_is_absent_and_has_no_angle() {
    let (mut mouse, fake) = rig(DevicePath::Cable);
    fake.borrow_mut().firmware = None;

    let snapshot = mouse.snapshot().expect("snapshot");

    assert_eq!(snapshot.firmware, None);
    assert_eq!(snapshot.angle, None);
}

#[test]
fn any_rejection_of_firmware_or_far_distance_leaves_the_field_absent() {
    let (mut mouse, fake) = rig(DevicePath::Cable);
    fake.borrow_mut().firmware = None;
    fake.borrow_mut().far_distance = None;
    fake.borrow_mut().unavailable_status = 1;

    let snapshot = mouse.snapshot().expect("snapshot");

    assert_eq!(snapshot.firmware, None);
    assert_eq!(snapshot.far_distance, None);
    assert_eq!(snapshot.angle, None);
}

#[test]
fn set_angle_keeps_the_switch() {
    let (mut mouse, fake) = rig(DevicePath::Cable);
    fake.borrow_mut()
        .put(address::ANGLE_BLOCK + 2, &[0x01, 0x99]);

    mouse.set_angle(-30).expect("set");

    let fake = fake.borrow();
    assert_eq!(fake.codes(), [0x12, 0x08, 0x07, 0x08]);
    assert_eq!(fake.at(address::ANGLE_BLOCK, 4), [0xe2, 0x73, 0x01, 0x99]);
}

#[test]
fn set_angle_enabled_keeps_the_angle() {
    let (mut mouse, fake) = rig(DevicePath::Cable);
    fake.borrow_mut().put(address::ANGLE_BLOCK, &[0xe2, 0x99]);

    mouse.set_angle_enabled(true).expect("set");

    assert_eq!(
        fake.borrow().at(address::ANGLE_BLOCK, 4),
        [0xe2, 0x99, 0x01, 0x54]
    );
}

#[test]
fn set_angle_rejects_values_outside_the_range_with_no_write() {
    let (mut mouse, fake) = rig(DevicePath::Cable);

    assert!(mouse.set_angle(31).is_err());
    assert!(mouse.set_angle(-31).is_err());

    assert!(fake.borrow().writes().is_empty());
}

#[test]
fn set_angle_refused_below_minimum_firmware() {
    let (mut mouse, fake) = rig(DevicePath::Cable);
    fake.borrow_mut().firmware = Some([0x02, 0x99]);

    let angle = mouse.set_angle(5);
    let enabled = mouse.set_angle_enabled(true);

    let refused = Err(ProtocolError::InvalidValue("angle on this firmware").into());
    assert_eq!(angle, refused);
    assert_eq!(enabled, refused);
    assert_eq!(fake.borrow().codes(), [0x12, 0x12]);
}

#[test]
fn set_angle_refused_when_the_firmware_is_not_supported() {
    let (mut mouse, fake) = rig(DevicePath::Cable);
    fake.borrow_mut().firmware = None;

    let result = mouse.set_angle(5);

    assert!(result.is_err());
    assert_eq!(fake.borrow().codes(), [0x12]);
}

#[test]
fn far_distance_reads_back_through_get() {
    let (mut mouse, fake) = rig(DevicePath::Cable);

    mouse.set_far_distance(true).expect("set");

    let fake = fake.borrow();
    assert_eq!(fake.codes(), [0x17, 0x16, 0x17]);
    assert_eq!(fake.log[1][5..8], [10, 1, 0]);
    assert_eq!(fake.far_distance, Some(1));
}

#[test]
fn far_distance_mismatch_is_reported() {
    let (mut mouse, fake) = rig(DevicePath::Cable);
    fake.borrow_mut().drop_writes = true;

    let result = mouse.set_far_distance(true);

    let error = result.expect_err("the mode did not change");
    assert_eq!(
        error,
        MouseError::FarDistanceMismatch {
            written: 1,
            actual: 0,
        }
    );
    assert!(!error.to_string().contains("EEPROM"));
    assert!(!error.to_string().contains("0x0000"));
}

#[test]
fn not_supported_far_distance_is_absent() {
    let (mut mouse, fake) = rig(DevicePath::Cable);
    fake.borrow_mut().far_distance = None;

    let snapshot = mouse.snapshot().expect("snapshot");
    let result = mouse.set_far_distance(true);

    assert_eq!(snapshot.far_distance, None);
    assert_eq!(result, Err(MouseError::Rejected(ReplyStatus::NotSupported)));
    assert!(!fake.borrow().codes().contains(&0x16));
}

#[test]
fn unknown_far_distance_mode_is_shown_as_unknown_and_not_rewritten() {
    let (mut mouse, fake) = rig(DevicePath::Cable);
    fake.borrow_mut().far_distance = Some(5);

    let snapshot = mouse.snapshot().expect("snapshot");
    let result = mouse.set_far_distance(true);

    assert_eq!(snapshot.far_distance, None);
    assert!(result.is_err());
    assert!(!fake.borrow().codes().contains(&0x16));
}

#[test]
fn unwritable_neighbour_0xb3_is_refused() {
    let (mut mouse, fake) = rig(DevicePath::Cable);

    let result = mouse.write_eeprom(0xb3, &[0x00, 0x55]);

    assert_eq!(result, Err(ProtocolError::AddressNotWritable(0xb3).into()));
    assert!(fake.borrow().log.is_empty());
}

#[test]
fn dongle_firmware_decodes() {
    let (mut mouse, _) = rig(DevicePath::Dongle);

    let snapshot = mouse.snapshot().expect("snapshot");

    assert_eq!(
        snapshot.dongle_firmware,
        Some(FirmwareVersion {
            major: 1,
            minor: 10
        })
    );
}

#[test]
fn cable_path_reads_no_dongle_version() {
    let (mut mouse, fake) = rig(DevicePath::Cable);

    let snapshot = mouse.snapshot().expect("snapshot");

    assert_eq!(snapshot.dongle_firmware, None);
    assert!(!fake.borrow().codes().contains(&0x1d));
}

#[test]
fn rejected_dongle_version_is_absent() {
    let (mut mouse, fake) = rig(DevicePath::Dongle);
    fake.borrow_mut().dongle_firmware = None;
    fake.borrow_mut().unavailable_status = 1;

    let snapshot = mouse.snapshot().expect("snapshot");

    assert_eq!(snapshot.dongle_firmware, None);
    assert_eq!(
        snapshot.firmware,
        Some(FirmwareVersion { major: 3, minor: 1 })
    );
}

fn dongle_version_requests(fake: &Rc<RefCell<Fake>>) -> usize {
    fake.borrow()
        .codes()
        .iter()
        .filter(|code| **code == 0x1d)
        .count()
}

#[test]
fn unanswered_dongle_version_leaves_the_field_empty() {
    let (mut mouse, fake) = rig(DevicePath::Dongle);
    fake.borrow_mut().silent_dongle_version = true;

    let snapshot = mouse.snapshot().expect("snapshot");

    assert_eq!(snapshot.dongle_firmware, None);
    assert_eq!(
        snapshot.firmware,
        Some(FirmwareVersion { major: 3, minor: 1 })
    );
    assert!(snapshot.far_distance.is_some());
    assert!(snapshot.lift_off_tenths.is_some());
}

#[test]
fn dongle_firmware_is_read_once_per_connection() {
    let (mut mouse, fake) = rig(DevicePath::Dongle);
    let expected = Some(FirmwareVersion {
        major: 1,
        minor: 10,
    });

    let first = mouse.snapshot().expect("first snapshot");
    let second = mouse.snapshot().expect("second snapshot");

    assert_eq!(first.dongle_firmware, expected);
    assert_eq!(second.dongle_firmware, expected);
    assert_eq!(dongle_version_requests(&fake), 1);
}

#[test]
fn silent_dongle_is_not_asked_again_on_this_connection() {
    let (mut mouse, fake) = rig(DevicePath::Dongle);
    fake.borrow_mut().silent_dongle_version = true;
    mouse.snapshot().expect("first snapshot");
    let requests = dongle_version_requests(&fake);

    mouse.snapshot().expect("second snapshot");

    assert_eq!(dongle_version_requests(&fake), requests);
}

#[test]
fn rejected_dongle_version_is_not_asked_again_on_this_connection() {
    let (mut mouse, fake) = rig(DevicePath::Dongle);
    fake.borrow_mut().dongle_firmware = None;
    fake.borrow_mut().unavailable_status = 1;

    mouse.snapshot().expect("first snapshot");
    mouse.snapshot().expect("second snapshot");

    assert_eq!(dongle_version_requests(&fake), 1);
}

#[test]
fn new_connection_reads_the_dongle_version_again() {
    let (mut mouse, fake) = rig(DevicePath::Dongle);
    mouse.snapshot().expect("first snapshot");

    let mut mouse = Mouse::new(mouse.into_session(), model(), DevicePath::Dongle);
    mouse.snapshot().expect("second snapshot");

    assert_eq!(dongle_version_requests(&fake), 2);
}

#[test]
fn snapshot_stopped_before_the_dongle_version_does_not_settle_it() {
    let (mut mouse, fake) = rig(DevicePath::Dongle);
    fake.borrow_mut().silent_reads = true;
    assert_eq!(mouse.snapshot(), Err(MouseError::Timeout));
    fake.borrow_mut().silent_reads = false;

    let snapshot = mouse.snapshot().expect("snapshot");

    assert!(snapshot.dongle_firmware.is_some());
}

#[test]
fn unanswered_mouse_version_is_still_a_timeout() {
    let (session, fake, _) = rig_with(new_fake());
    let mut mouse = Mouse::new(session, model(), DevicePath::Cable);
    fake.borrow_mut().silent_mouse_version = true;

    assert_eq!(mouse.snapshot(), Err(MouseError::Timeout));
}

#[test]
fn set_profile_reads_back() {
    let (mut mouse, fake) = rig(DevicePath::Cable);

    mouse.set_profile(3).expect("set");

    let fake = fake.borrow();
    assert_eq!(fake.codes(), [0x0e, 0x0f, 0x0e]);
    assert_eq!(fake.log[1][5..7], [1, 3]);
    assert_eq!(fake.profile, 3);
}

#[test]
fn set_profile_rejects_index_4_with_no_frame() {
    let (mut mouse, fake) = rig(DevicePath::Cable);

    let result = mouse.set_profile(4);

    assert!(matches!(result, Err(MouseError::Protocol(_))));
    assert!(fake.borrow().log.is_empty());
}

#[test]
fn set_profile_of_the_active_one_writes_nothing() {
    let (mut mouse, fake) = rig(DevicePath::Cable);

    mouse.set_profile(1).expect("already active");

    assert_eq!(fake.borrow().codes(), [0x0e]);
}

#[test]
fn profile_mismatch_is_an_error() {
    let (mut mouse, fake) = rig(DevicePath::Cable);
    fake.borrow_mut().drop_writes = true;

    let result = mouse.set_profile(2);

    assert_eq!(
        result,
        Err(MouseError::ProfileMismatch {
            written: 2,
            actual: 1,
        })
    );
}

#[test]
fn snapshot_holds_the_profile_count() {
    let (mut mouse, _) = rig(DevicePath::Cable);

    assert_eq!(mouse.snapshot().expect("snapshot").profile_count, 4);
}
