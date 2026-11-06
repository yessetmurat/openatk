use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, Mutex, MutexGuard};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

use super::*;
use crate::device::{Battery, Model, MouseSnapshot};
use crate::protocol::{Command, Frame, REPORT_LEN, ReplyStatus, address, check};
use crate::settings::{ButtonAction, DpiStage, MouseAction, PollingRate, Sensor};

const FAST: Timing = Timing {
    reply_timeout: Duration::from_millis(20),
    retries: 0,
};
const EVENT_WAIT: Duration = Duration::from_secs(5);
const TICK: Duration = Duration::from_millis(5);
const NEVER: Duration = Duration::from_secs(3600);
const LINK_TICKS: PollIntervals = PollIntervals {
    link: TICK,
    battery: NEVER,
};
const BATTERY_TICKS: PollIntervals = PollIntervals {
    link: NEVER,
    battery: TICK,
};
const ALL_TICKS: PollIntervals = PollIntervals {
    link: TICK,
    battery: TICK,
};
const ONLINE_CODE: u8 = 0x03;
const BATTERY_CODE: u8 = 0x04;
const IDENTITY_CODE: u8 = 0x10;
const REFUSED: &str = "refused";
const BATTERY: Battery = Battery {
    percent: 80,
    charging: false,
};

/// The hardware behind the fake connector, as the dongle behaves on the bench: while the
/// mouse is offline only `GetWirelessMouseOnline` gets an answer.
struct Device {
    present: Option<DevicePath>,
    eeprom: [u8; 0x200],
    cid: u8,
    online: bool,
    silent: bool,
    mouse_silent: bool,
    reject_writes: bool,
    close_on_next_send: bool,
    status_on_next_send: Option<u8>,
    wake_after_next_send: bool,
    fail_find: bool,
    fail_open: bool,
    sent: Vec<[u8; REPORT_LEN]>,
    sent_at: Vec<Instant>,
    opens: Vec<(DevicePath, usize)>,
    live_links: usize,
    far_distance: u8,
}

impl Device {
    fn absent() -> Device {
        let mut device = Device {
            present: None,
            eeprom: [0; 0x200],
            cid: 2,
            online: true,
            silent: false,
            mouse_silent: false,
            reject_writes: false,
            close_on_next_send: false,
            status_on_next_send: None,
            wake_after_next_send: false,
            fail_find: false,
            fail_open: false,
            sent: Vec::new(),
            sent_at: Vec::new(),
            opens: Vec::new(),
            live_links: 0,
            far_distance: 0,
        };

        // Polling 1000 Hz, one stage, current stage 0, then unknown bytes 6-9.
        device.put(
            0,
            &[
                1,
                check::single(1),
                1,
                check::single(1),
                0,
                check::single(0),
            ],
        );
        device.put(6, &[0x07, 0x11, 0x22, 0x33]);

        let stage = DpiStage { x: 800, y: 800 };
        device.put(0x0c, &[0xff; 32]);
        device.put(
            0x0c,
            &Sensor::Paw3950Ultra
                .encode_stage(stage)
                .expect("grid value"),
        );

        let click = ButtonAction::Mouse(MouseAction::Left)
            .encode()
            .expect("editable");
        let model = Model::find(2, 50).expect("known model");
        for button in model.buttons {
            device.put(button.address(), &click);
        }

        // The bytes read from the real mouse.
        device.put(address::LIFT_OFF, &[0x01, 0x54]);
        device.put(
            address::SENSOR_BLOCK,
            &[0x00, 0x55, 0x00, 0x55, 0x01, 0x54, 0x00, 0x55, 0x00, 0x55],
        );
        device.put(address::SLEEP_BLOCK, &[0x00, 0x55, 0x06, 0x4f, 0x00, 0x55]);
        device.put(address::ANGLE_BLOCK, &[0x00, 0x55, 0x00, 0x55]);

        device
    }

    fn on(path: DevicePath) -> Device {
        Device {
            present: Some(path),
            ..Device::absent()
        }
    }

    fn put(&mut self, address: u16, bytes: &[u8]) {
        let start = usize::from(address);
        self.eeprom[start..start + bytes.len()].copy_from_slice(bytes);
    }

    fn sends_of(&self, code: u8) -> usize {
        self.sent.iter().filter(|report| report[1] == code).count()
    }

    fn send_times_of(&self, code: u8) -> Vec<Instant> {
        self.sent
            .iter()
            .zip(&self.sent_at)
            .filter(|(report, _)| report[1] == code)
            .map(|(_, time)| *time)
            .collect()
    }

    fn writes(&self) -> usize {
        let is_write = |report: &&[u8; REPORT_LEN]| report[1] == Command::SetEeprom.code();

        self.sent.iter().filter(is_write).count()
    }

    fn answer(&mut self, request: &Frame) -> Option<(u8, Vec<u8>)> {
        let report = request.to_report();
        let start = usize::from(request.address());
        let len = usize::from(report[5]);

        match request.command_code() {
            _ if self.silent => None,
            0x03 => Some((0, vec![u8::from(self.online)])),
            0x1d => Some((0, vec![0x01, 0x0a])),
            _ if !self.online || self.mouse_silent => None,
            0x04 => Some((0, vec![BATTERY.percent, 0])),
            0x0e => Some((0, vec![0])),
            0x10 => Some((0, vec![self.cid, 50, 0x21])),
            0x12 => Some((0, vec![0x03, 0x01])),
            0x16 if self.reject_writes => Some((1, Vec::new())),
            0x16 => {
                self.far_distance = request.payload()[0];

                Some((0, vec![self.far_distance]))
            }
            0x17 => Some((0, vec![self.far_distance])),
            0x08 => Some((0, self.eeprom[start..start + len].to_vec())),
            0x07 if self.reject_writes => Some((1, Vec::new())),
            0x07 => {
                let data = report[6..6 + len].to_vec();
                self.eeprom[start..start + len].copy_from_slice(&data);

                Some((0, data))
            }
            _ => None,
        }
    }
}

type SharedDevice = Arc<Mutex<Device>>;

struct FakeLink {
    sender: Sender<Input<MouseCommand>>,
    device: SharedDevice,
}

impl Link for FakeLink {
    fn send(&mut self, report: &[u8; REPORT_LEN]) -> Result<(), MouseError> {
        let mut device = self.device.lock().expect("device lock");
        device.sent.push(*report);
        device.sent_at.push(Instant::now());

        if std::mem::take(&mut device.close_on_next_send) {
            self.sender.send(Input::LinkClosed).expect("inbox alive");

            return Ok(());
        }

        if let Some(flags) = device.status_on_next_send.take() {
            self.sender.send(status_report(flags)).expect("inbox alive");
        }

        let request = Frame::parse(report).expect("openatk sends valid frames");

        if let Some((status, data)) = device.answer(&request) {
            let bytes = reply(request.command_code(), status, request.address(), &data);
            self.sender.send(Input::Report(bytes)).expect("inbox alive");
        }

        if std::mem::take(&mut device.wake_after_next_send) {
            device.online = true;
        }

        Ok(())
    }
}

impl Drop for FakeLink {
    fn drop(&mut self) {
        // A failed assertion can poison the lock: a panic here would abort the whole run.
        if let Ok(mut device) = self.device.lock() {
            device.live_links -= 1;
        }
    }
}

struct FakeConnector {
    sender: Sender<Input<MouseCommand>>,
    device: SharedDevice,
}

impl Connector for FakeConnector {
    type Link = FakeLink;

    fn find(&mut self) -> Result<Option<DevicePath>, MouseError> {
        let device = self.device.lock().expect("device lock");

        if device.fail_find {
            return Err(refused());
        }

        Ok(device.present)
    }

    fn open(&mut self, path: DevicePath) -> Result<FakeLink, MouseError> {
        let mut device = self.device.lock().expect("device lock");

        if device.fail_open {
            return Err(refused());
        }

        let live_links = device.live_links;
        device.opens.push((path, live_links));
        device.live_links += 1;

        Ok(FakeLink {
            sender: self.sender.clone(),
            device: Arc::clone(&self.device),
        })
    }
}

/// The loop running on its own thread, driven one input at a time.
struct Rig {
    sender: Sender<Input<MouseCommand>>,
    events: Receiver<MouseEvent>,
    device: SharedDevice,
    thread: JoinHandle<()>,
}

impl Rig {
    fn start(device: Device) -> Rig {
        Rig::with_intervals(device, PollIntervals::default())
    }

    fn with_intervals(device: Device, intervals: PollIntervals) -> Rig {
        let device = Arc::new(Mutex::new(device));
        let (sender, receiver) = mpsc::channel();
        let (event_sender, events) = mpsc::channel();
        let connector = FakeConnector {
            sender: sender.clone(),
            device: Arc::clone(&device),
        };

        let thread = thread::spawn(move || {
            let on_event = move |event| event_sender.send(event).expect("test alive");

            run(connector, Inbox::new(receiver), FAST, intervals, on_event);
        });

        Rig {
            sender,
            events,
            device,
            thread,
        }
    }

    fn send(&self, input: Input<MouseCommand>) {
        self.sender.send(input).expect("loop alive");
    }

    fn command(&self, command: MouseCommand) {
        self.send(Input::Command(command));
    }

    fn device(&self) -> MutexGuard<'_, Device> {
        self.device.lock().expect("device lock")
    }

    fn next_event(&self) -> MouseEvent {
        self.events.recv_timeout(EVENT_WAIT).expect("an event")
    }

    fn next_snapshot(&self) -> MouseSnapshot {
        match self.next_event() {
            MouseEvent::Snapshot(snapshot) => snapshot,
            other => panic!("expected a snapshot, got {other:?}"),
        }
    }

    fn set_polling(&self, on: bool) {
        self.command(MouseCommand::SetBackgroundPolling(on));
    }

    /// Waits until `code` has gone out `count` more times.
    fn wait_for_sends(&self, code: u8, count: usize) {
        let wanted = self.device().sends_of(code) + count;

        self.wait_for_total_sends(code, wanted);
    }

    /// Waits until `code` has gone out `wanted` times in all.
    fn wait_for_total_sends(&self, code: u8, wanted: usize) {
        let deadline = Instant::now() + EVENT_WAIT;

        while self.device().sends_of(code) < wanted {
            assert!(Instant::now() < deadline, "no tick sent {code:#04x}");
            thread::sleep(TICK);
        }
    }

    fn assert_no_event_for(&self, duration: Duration) {
        let event = self.events.recv_timeout(duration);

        assert_eq!(event, Err(RecvTimeoutError::Timeout));
    }

    /// Stops the loop and returns the events nobody has read yet.
    fn finish(self) -> Vec<MouseEvent> {
        self.send(Input::Stop);
        self.thread.join().expect("loop ends without a panic");

        self.events.try_iter().collect()
    }
}

fn reply(code: u8, status: u8, address: u16, data: &[u8]) -> Vec<u8> {
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

fn refused() -> MouseError {
    MouseError::Link(REFUSED.to_owned())
}

fn status_report(flags: u8) -> Input<MouseCommand> {
    Input::Report(reply(Command::ReportMouseStatus.code(), 0, 0, &[flags]))
}

#[test]
fn event_flags_trigger_the_right_reads() {
    let rig = Rig::start(Device::on(DevicePath::Dongle));
    rig.next_snapshot();

    rig.send(status_report(0x40));
    assert_eq!(rig.next_event(), MouseEvent::Battery(BATTERY));

    for flag in [0x01, 0x02, 0x04] {
        rig.send(status_report(flag));
        rig.next_snapshot();
    }

    rig.send(status_report(0x08));
    assert_eq!(rig.finish(), []);
}

#[test]
fn malformed_report_is_ignored() {
    let rig = Rig::start(Device::on(DevicePath::Dongle));
    rig.next_snapshot();
    let mut bad_checksum = reply(Command::ReportMouseStatus.code(), 0, 0, &[0x40]);
    bad_checksum[16] ^= 0xff;

    rig.send(Input::Report(vec![1, 2, 3]));
    rig.send(Input::Report(bad_checksum));

    assert_eq!(rig.finish(), []);
}

#[test]
fn status_flags_seen_during_an_exchange_are_drained() {
    let rig = Rig::start(Device::on(DevicePath::Dongle));
    rig.next_snapshot();
    rig.device().status_on_next_send = Some(0x40);

    rig.command(MouseCommand::Refresh);

    rig.next_snapshot();
    assert_eq!(rig.next_event(), MouseEvent::Battery(BATTERY));
    assert_eq!(rig.finish(), []);
}

#[test]
fn successful_write_sends_the_new_snapshot() {
    let rig = Rig::start(Device::on(DevicePath::Dongle));
    assert_eq!(rig.next_snapshot().polling, PollingRate::Hz1000);

    rig.command(MouseCommand::SetPolling(PollingRate::Hz500));

    assert_eq!(rig.next_snapshot().polling, PollingRate::Hz500);
    assert_eq!(rig.finish(), []);
}

#[test]
fn sensor_commands_end_in_a_snapshot_that_shows_them() {
    let rig = Rig::start(Device::on(DevicePath::Dongle));
    rig.next_snapshot();

    rig.command(MouseCommand::SetLiftOff(12));
    assert_eq!(rig.next_snapshot().lift_off_tenths, Some(12));
    rig.command(MouseCommand::SetDebounce(4));
    assert_eq!(rig.next_snapshot().debounce_ms, Some(4));
    rig.command(MouseCommand::SetMotionSync(true));
    assert_eq!(rig.next_snapshot().motion_sync, Some(true));
    rig.command(MouseCommand::SetLinearCorrection(true));
    assert_eq!(rig.next_snapshot().linear_correction, Some(true));
    rig.command(MouseCommand::SetRippleControl(true));
    assert_eq!(rig.next_snapshot().ripple_control, Some(true));
    rig.command(MouseCommand::SetSleep(120));
    assert_eq!(rig.next_snapshot().sleep_seconds, Some(120));
    rig.command(MouseCommand::SetAngle(-5));
    let angle = rig.next_snapshot().angle.expect("firmware 3.01");
    assert_eq!(angle.degrees, Some(-5));
    rig.command(MouseCommand::SetAngleEnabled(true));
    let angle = rig.next_snapshot().angle.expect("firmware 3.01");
    assert_eq!(angle.enabled, Some(true));
    rig.command(MouseCommand::SetFarDistance(true));
    assert_eq!(rig.next_snapshot().far_distance, Some(true));

    assert_eq!(rig.finish(), []);
}

#[test]
fn failed_sensor_write_sends_failure_then_real_snapshot() {
    let rig = Rig::start(Device::on(DevicePath::Dongle));
    rig.next_snapshot();
    rig.device().reject_writes = true;

    for command in [
        MouseCommand::SetLiftOff(12),
        MouseCommand::SetDebounce(4),
        MouseCommand::SetMotionSync(true),
        MouseCommand::SetLinearCorrection(true),
        MouseCommand::SetRippleControl(true),
        MouseCommand::SetSleep(120),
        MouseCommand::SetAngle(-5),
        MouseCommand::SetAngleEnabled(true),
        MouseCommand::SetFarDistance(true),
    ] {
        rig.command(command);

        assert_eq!(
            rig.next_event(),
            MouseEvent::CommandFailed(MouseError::Rejected(ReplyStatus::Fail))
        );
        assert_eq!(rig.next_snapshot().lift_off_tenths, Some(7));
    }

    assert_eq!(rig.finish(), []);
}

#[test]
fn failed_write_sends_failure_then_real_snapshot() {
    let rig = Rig::start(Device::on(DevicePath::Dongle));
    rig.next_snapshot();
    rig.device().reject_writes = true;

    rig.command(MouseCommand::SetPolling(PollingRate::Hz500));

    assert_eq!(
        rig.next_event(),
        MouseEvent::CommandFailed(MouseError::Rejected(ReplyStatus::Fail))
    );
    assert_eq!(rig.next_snapshot().polling, PollingRate::Hz1000);
    assert_eq!(rig.finish(), []);
}

#[test]
fn failed_snapshot_after_write_moves_to_linked() {
    let rig = Rig::start(Device::on(DevicePath::Dongle));
    rig.next_snapshot();
    rig.device().online = false;
    let sent_before = rig.device().sent.len();

    rig.command(MouseCommand::SetPolling(PollingRate::Hz500));

    assert_eq!(
        rig.next_event(),
        MouseEvent::CommandFailed(MouseError::Timeout)
    );
    assert_eq!(rig.next_event(), MouseEvent::NotConnected);
    // Only the write's own first exchange went out: no snapshot read to the silent mouse.
    assert_eq!(rig.device().sent.len(), sent_before + 1);
    assert_eq!(rig.device().writes(), 0);

    rig.device().online = true;
    rig.send(status_report(0x08));

    assert_eq!(rig.next_snapshot().polling, PollingRate::Hz1000);
    assert_eq!(rig.device().opens, [(DevicePath::Dongle, 0)]);
    assert_eq!(rig.finish(), []);
}

#[test]
fn link_closed_sends_not_connected() {
    let rig = Rig::start(Device::on(DevicePath::Dongle));
    rig.next_snapshot();
    rig.device().present = None;

    rig.send(Input::LinkClosed);

    assert_eq!(rig.next_event(), MouseEvent::NotConnected);
    assert_eq!(rig.device().live_links, 0);
    assert_eq!(rig.finish(), []);
}

#[test]
fn link_closed_connects_the_other_path() {
    let rig = Rig::start(Device::on(DevicePath::Cable));
    rig.next_snapshot();
    rig.device().present = Some(DevicePath::Dongle);

    rig.send(Input::LinkClosed);

    assert_eq!(rig.next_event(), MouseEvent::NotConnected);
    rig.next_snapshot();
    assert_eq!(
        rig.device().opens,
        [(DevicePath::Cable, 0), (DevicePath::Dongle, 0)]
    );
    assert_eq!(rig.finish(), []);
}

#[test]
fn link_closed_during_an_exchange_is_not_taken_for_an_offline_mouse() {
    let rig = Rig::start(Device::on(DevicePath::Dongle));
    rig.next_snapshot();
    {
        let mut device = rig.device();
        device.present = None;
        device.close_on_next_send = true;
    }

    rig.command(MouseCommand::Refresh);

    assert_eq!(rig.next_event(), MouseEvent::NotConnected);
    assert_eq!(rig.next_event(), MouseEvent::NotConnected);
    assert_eq!(rig.device().live_links, 0);

    rig.device().present = Some(DevicePath::Dongle);
    rig.send(Input::Hotplug);

    rig.next_snapshot();
    assert_eq!(rig.finish(), []);
}

#[test]
fn commands_while_disconnected_fail_without_sending() {
    let rig = Rig::start(Device::absent());
    assert_eq!(rig.next_event(), MouseEvent::NotConnected);

    rig.command(MouseCommand::SetPolling(PollingRate::Hz500));
    assert_eq!(
        rig.next_event(),
        MouseEvent::CommandFailed(MouseError::NotConnected)
    );

    rig.command(MouseCommand::Refresh);
    assert_eq!(rig.next_event(), MouseEvent::NotConnected);

    assert!(rig.device().sent.is_empty());
    assert!(rig.device().opens.is_empty());
    assert_eq!(rig.finish(), []);
}

#[test]
fn hotplug_reconnects() {
    let rig = Rig::start(Device::absent());
    assert_eq!(rig.next_event(), MouseEvent::NotConnected);
    rig.device().present = Some(DevicePath::Dongle);

    rig.send(Input::Hotplug);

    rig.next_snapshot();
    assert_eq!(rig.finish(), []);
}

#[test]
fn hotplug_without_a_change_does_nothing() {
    let rig = Rig::start(Device::absent());
    assert_eq!(rig.next_event(), MouseEvent::NotConnected);

    rig.send(Input::Hotplug);
    rig.device().present = Some(DevicePath::Dongle);
    rig.send(Input::Hotplug);
    rig.next_snapshot();
    rig.send(Input::Hotplug);

    let opens = rig.device().opens.clone();
    assert_eq!(rig.finish(), []);
    assert_eq!(opens, [(DevicePath::Dongle, 0)]);
}

#[test]
fn hotplug_keeps_the_cable_until_its_link_closes() {
    let rig = Rig::start(Device::on(DevicePath::Cable));
    rig.next_snapshot();
    rig.device().present = Some(DevicePath::Dongle);

    rig.send(Input::Hotplug);

    let device = Arc::clone(&rig.device);
    assert_eq!(rig.finish(), []);
    assert_eq!(
        device.lock().expect("device lock").opens,
        [(DevicePath::Cable, 0)]
    );
}

#[test]
fn better_path_replaces_the_link() {
    let rig = Rig::start(Device::on(DevicePath::Dongle));
    assert_eq!(rig.next_snapshot().polling_rates.len(), 7);
    rig.device().present = Some(DevicePath::Cable);

    rig.send(Input::Hotplug);

    assert_eq!(rig.next_snapshot().polling_rates.len(), 4);
    assert_eq!(
        rig.device().opens,
        [(DevicePath::Dongle, 0), (DevicePath::Cable, 1)]
    );
    assert_eq!(rig.device().live_links, 1);
    assert_eq!(rig.finish(), []);
}

#[test]
fn failed_open_of_a_better_path_keeps_the_link() {
    let rig = Rig::start(Device::on(DevicePath::Dongle));
    rig.next_snapshot();
    {
        let mut device = rig.device();
        device.present = Some(DevicePath::Cable);
        device.fail_open = true;
    }

    rig.send(Input::Hotplug);

    assert_eq!(rig.next_event(), MouseEvent::CommandFailed(refused()));

    rig.command(MouseCommand::Refresh);

    assert_eq!(rig.next_snapshot().polling_rates.len(), 7);
    assert_eq!(rig.device().live_links, 1);
    assert_eq!(rig.finish(), []);
}

#[test]
fn failed_open_reports_and_waits_for_the_next_change() {
    let mut device = Device::on(DevicePath::Dongle);
    device.fail_open = true;
    let rig = Rig::start(device);

    assert_eq!(rig.next_event(), MouseEvent::CommandFailed(refused()));
    assert_eq!(rig.next_event(), MouseEvent::NotConnected);

    rig.device().fail_open = false;
    rig.send(Input::Hotplug);

    rig.next_snapshot();
    assert_eq!(rig.finish(), []);
}

#[test]
fn failed_find_reports_and_stays_disconnected() {
    let mut device = Device::on(DevicePath::Dongle);
    device.fail_find = true;
    let rig = Rig::start(device);

    assert_eq!(rig.next_event(), MouseEvent::CommandFailed(refused()));
    assert_eq!(rig.next_event(), MouseEvent::NotConnected);
    assert!(rig.device().opens.is_empty());

    rig.device().fail_find = false;
    rig.command(MouseCommand::Refresh);

    rig.next_snapshot();
    assert_eq!(rig.finish(), []);
}

#[test]
fn better_path_also_replaces_a_linked_dongle() {
    let mut device = Device::on(DevicePath::Dongle);
    device.online = false;
    let rig = Rig::start(device);
    assert_eq!(rig.next_event(), MouseEvent::NotConnected);
    {
        let mut device = rig.device();
        device.present = Some(DevicePath::Cable);
        device.online = true;
    }

    rig.send(Input::Hotplug);

    assert_eq!(rig.next_snapshot().polling_rates.len(), 4);
    assert_eq!(rig.finish(), []);
}

#[test]
fn linked_state_identifies_on_status_report() {
    let mut device = Device::on(DevicePath::Dongle);
    device.online = false;
    let rig = Rig::start(device);
    assert_eq!(rig.next_event(), MouseEvent::NotConnected);
    rig.device().online = true;

    rig.send(status_report(0x08));

    rig.next_snapshot();
    assert_eq!(rig.device().opens, [(DevicePath::Dongle, 0)]);
    assert_eq!(rig.finish(), []);
}

#[test]
fn status_report_during_identify_identifies_again() {
    let mut device = Device::on(DevicePath::Dongle);
    device.online = false;
    let rig = Rig::start(device);
    assert_eq!(rig.next_event(), MouseEvent::NotConnected);
    rig.device().status_on_next_send = Some(0x08);

    rig.send(status_report(0x08));

    assert_eq!(rig.next_event(), MouseEvent::NotConnected);
    assert_eq!(rig.device().sent.len(), 3);
    assert_eq!(rig.finish(), []);
}

#[test]
fn status_report_during_a_failed_read_identifies_again() {
    let rig = Rig::start(Device::on(DevicePath::Dongle));
    rig.next_snapshot();
    {
        let mut device = rig.device();
        device.online = false;
        device.status_on_next_send = Some(0x08);
        device.wake_after_next_send = true;
    }

    rig.command(MouseCommand::Refresh);

    rig.next_snapshot();
    assert_eq!(rig.device().opens, [(DevicePath::Dongle, 0)]);
    assert_eq!(rig.finish(), []);
}

#[test]
fn refresh_in_linked_state_identifies() {
    let mut device = Device::on(DevicePath::Dongle);
    device.online = false;
    let rig = Rig::start(device);
    assert_eq!(rig.next_event(), MouseEvent::NotConnected);

    rig.command(MouseCommand::Refresh);
    assert_eq!(rig.next_event(), MouseEvent::NotConnected);

    rig.device().online = true;
    rig.command(MouseCommand::Refresh);

    rig.next_snapshot();
    assert_eq!(rig.device().opens, [(DevicePath::Dongle, 0)]);
    assert_eq!(rig.finish(), []);
}

#[test]
fn commands_in_linked_state_fail_without_sending() {
    let mut device = Device::on(DevicePath::Dongle);
    device.online = false;
    let rig = Rig::start(device);
    assert_eq!(rig.next_event(), MouseEvent::NotConnected);
    let sent_before = rig.device().sent.len();

    rig.command(MouseCommand::SetPolling(PollingRate::Hz500));

    assert_eq!(
        rig.next_event(),
        MouseEvent::CommandFailed(MouseError::NotConnected)
    );
    assert_eq!(rig.device().sent.len(), sent_before);
    assert_eq!(rig.finish(), []);
}

#[test]
fn unsupported_model_blocks_writes() {
    let mut device = Device::on(DevicePath::Dongle);
    device.cid = 9;
    let rig = Rig::start(device);
    let unsupported = MouseError::UnsupportedModel { cid: 9, mid: 50 };
    assert_eq!(
        rig.next_event(),
        MouseEvent::CommandFailed(unsupported.clone())
    );

    rig.send(status_report(0x01));
    rig.command(MouseCommand::SetPolling(PollingRate::Hz500));

    assert_eq!(
        rig.next_event(),
        MouseEvent::CommandFailed(MouseError::NotConnected)
    );
    assert_eq!(rig.device().writes(), 0);
    assert_eq!(rig.finish(), []);
}

#[test]
fn stop_ends_the_loop() {
    let rig = Rig::start(Device::on(DevicePath::Dongle));
    rig.next_snapshot();

    let remaining = rig.finish();

    assert_eq!(remaining, []);
}

#[test]
fn polling_off_sends_nothing_when_idle() {
    let rig = Rig::with_intervals(Device::on(DevicePath::Dongle), ALL_TICKS);
    rig.next_snapshot();
    let sent_before = rig.device().sent.len();

    thread::sleep(TICK * 8);

    assert_eq!(rig.device().sent.len(), sent_before);
    assert_eq!(rig.finish(), []);
}

#[test]
fn set_background_polling_is_not_a_mouse_command() {
    let rig = Rig::start(Device::on(DevicePath::Dongle));
    rig.next_snapshot();
    let sent_before = rig.device().sent.len();

    rig.set_polling(true);
    rig.set_polling(false);

    let device = Arc::clone(&rig.device);
    assert_eq!(rig.finish(), []);
    assert_eq!(device.lock().expect("device lock").sent.len(), sent_before);
}

#[test]
fn set_background_polling_works_while_disconnected() {
    let rig = Rig::with_intervals(Device::absent(), LINK_TICKS);
    assert_eq!(rig.next_event(), MouseEvent::NotConnected);

    rig.set_polling(true);
    rig.device().present = Some(DevicePath::Dongle);
    rig.send(Input::Hotplug);
    rig.next_snapshot();
    rig.device().online = false;

    assert_eq!(rig.next_event(), MouseEvent::NotConnected);
    assert_eq!(rig.finish(), []);
}

#[test]
fn linked_becomes_active_when_mouse_returns() {
    let mut device = Device::on(DevicePath::Dongle);
    device.online = false;
    let rig = Rig::with_intervals(device, LINK_TICKS);
    assert_eq!(rig.next_event(), MouseEvent::NotConnected);
    rig.set_polling(true);
    rig.wait_for_sends(ONLINE_CODE, 2);

    rig.device().online = true;

    rig.next_snapshot();
    assert_eq!(rig.device().opens, [(DevicePath::Dongle, 0)]);
    assert_eq!(rig.finish(), []);
}

#[test]
fn active_becomes_linked_when_mouse_leaves() {
    let rig = Rig::with_intervals(Device::on(DevicePath::Dongle), LINK_TICKS);
    rig.next_snapshot();
    rig.set_polling(true);
    rig.wait_for_sends(ONLINE_CODE, 1);

    rig.device().online = false;

    assert_eq!(rig.next_event(), MouseEvent::NotConnected);
    rig.wait_for_sends(ONLINE_CODE, 2);
    assert_eq!(rig.device().opens, [(DevicePath::Dongle, 0)]);
    assert_eq!(rig.finish(), []);
}

#[test]
fn link_tick_timeout_only_moves_the_state() {
    let rig = Rig::with_intervals(Device::on(DevicePath::Dongle), LINK_TICKS);
    rig.next_snapshot();
    rig.set_polling(true);

    rig.device().silent = true;

    assert_eq!(rig.next_event(), MouseEvent::NotConnected);
    rig.wait_for_sends(ONLINE_CODE, 2);
    assert_eq!(rig.finish(), []);
}

#[test]
fn status_report_during_a_link_tick_identifies() {
    let mut device = Device::on(DevicePath::Dongle);
    device.mouse_silent = true;
    let rig = Rig::with_intervals(device, LINK_TICKS);
    assert_eq!(rig.next_event(), MouseEvent::NotConnected);
    rig.device().status_on_next_send = Some(0x40);

    rig.set_polling(true);

    rig.wait_for_total_sends(IDENTITY_CODE, 2);
    assert_eq!(rig.finish(), []);
}

#[test]
fn battery_tick_emits_battery() {
    let rig = Rig::with_intervals(Device::on(DevicePath::Dongle), BATTERY_TICKS);
    rig.next_snapshot();

    rig.set_polling(true);

    assert_eq!(rig.next_event(), MouseEvent::Battery(BATTERY));
    assert_eq!(rig.next_event(), MouseEvent::Battery(BATTERY));
    let remaining = rig.finish();
    assert!(
        remaining
            .iter()
            .all(|event| *event == MouseEvent::Battery(BATTERY))
    );
}

#[test]
fn battery_tick_timeout_only_moves_the_state() {
    let rig = Rig::with_intervals(Device::on(DevicePath::Dongle), BATTERY_TICKS);
    rig.next_snapshot();
    rig.set_polling(true);

    rig.device().mouse_silent = true;

    let mut event = rig.next_event();
    while event == MouseEvent::Battery(BATTERY) {
        event = rig.next_event();
    }
    assert_eq!(event, MouseEvent::NotConnected);
    assert_eq!(rig.finish(), []);
}

#[test]
fn battery_tick_never_runs_when_offline() {
    let mut device = Device::on(DevicePath::Dongle);
    device.online = false;
    let rig = Rig::with_intervals(device, ALL_TICKS);
    assert_eq!(rig.next_event(), MouseEvent::NotConnected);

    rig.set_polling(true);
    rig.wait_for_sends(ONLINE_CODE, 3);

    assert_eq!(rig.device().sends_of(BATTERY_CODE), 0);
    assert_eq!(rig.finish(), []);
}

#[test]
fn deadline_that_passed_while_disconnected_does_not_fire_on_connect() {
    let intervals = PollIntervals {
        link: NEVER,
        battery: Duration::from_millis(200),
    };
    let rig = Rig::with_intervals(Device::absent(), intervals);
    assert_eq!(rig.next_event(), MouseEvent::NotConnected);
    rig.set_polling(true);
    thread::sleep(intervals.battery + TICK);
    rig.device().present = Some(DevicePath::Dongle);

    rig.send(Input::Hotplug);

    rig.next_snapshot();
    rig.assert_no_event_for(intervals.battery / 4);
    rig.finish();
}

#[test]
fn cable_path_has_no_online_check() {
    let rig = Rig::with_intervals(Device::on(DevicePath::Cable), ALL_TICKS);
    rig.next_snapshot();

    rig.set_polling(true);

    assert_eq!(rig.next_event(), MouseEvent::Battery(BATTERY));
    assert_eq!(rig.next_event(), MouseEvent::Battery(BATTERY));
    assert_eq!(rig.device().sends_of(ONLINE_CODE), 0);
    rig.finish();
}

#[test]
fn turning_polling_off_stops_ticks() {
    let rig = Rig::with_intervals(Device::on(DevicePath::Dongle), LINK_TICKS);
    rig.next_snapshot();
    rig.set_polling(true);
    rig.wait_for_sends(ONLINE_CODE, 1);

    rig.set_polling(false);
    rig.command(MouseCommand::Refresh);
    rig.next_snapshot();
    let online_asks = rig.device().sends_of(ONLINE_CODE);

    thread::sleep(TICK * 8);

    assert_eq!(rig.device().sends_of(ONLINE_CODE), online_asks);
    assert_eq!(rig.finish(), []);
}

#[test]
fn commands_still_run_between_ticks() {
    let intervals = PollIntervals {
        link: TICK * 4,
        battery: NEVER,
    };
    let rig = Rig::with_intervals(Device::on(DevicePath::Dongle), intervals);
    rig.next_snapshot();
    rig.set_polling(true);
    let wanted = rig.device().sends_of(ONLINE_CODE) + 3;
    let deadline = Instant::now() + EVENT_WAIT;

    while rig.device().sends_of(ONLINE_CODE) < wanted {
        assert!(Instant::now() < deadline, "the commands starved the ticks");

        rig.command(MouseCommand::SetPolling(PollingRate::Hz500));

        assert_eq!(rig.next_snapshot().polling, PollingRate::Hz500);
        thread::sleep(TICK);
    }

    assert_eq!(rig.finish(), []);
}

#[test]
fn online_but_silent_mouse_is_retried_at_the_slow_interval() {
    let slow = Duration::from_millis(300);
    let intervals = PollIntervals {
        link: TICK,
        battery: slow,
    };
    let mut device = Device::on(DevicePath::Dongle);
    device.mouse_silent = true;
    let rig = Rig::with_intervals(device, intervals);
    assert_eq!(rig.next_event(), MouseEvent::NotConnected);
    let silent_since = Instant::now();

    rig.set_polling(true);
    rig.wait_for_sends(ONLINE_CODE, 5);

    assert_eq!(rig.device().sends_of(IDENTITY_CODE), 1);

    rig.wait_for_sends(IDENTITY_CODE, 1);

    assert!(silent_since.elapsed() >= slow / 2);
    assert_eq!(rig.finish(), []);
}

#[test]
fn online_but_silent_emits_not_connected_once() {
    let intervals = PollIntervals {
        link: TICK,
        battery: Duration::ZERO,
    };
    let mut device = Device::on(DevicePath::Dongle);
    device.mouse_silent = true;
    let rig = Rig::with_intervals(device, intervals);
    assert_eq!(rig.next_event(), MouseEvent::NotConnected);

    rig.set_polling(true);
    rig.wait_for_sends(IDENTITY_CODE, 3);

    assert_eq!(rig.finish(), []);
}

#[test]
fn byte_going_0_to_1_identifies_at_once() {
    let mut device = Device::on(DevicePath::Dongle);
    device.mouse_silent = true;
    let rig = Rig::with_intervals(device, LINK_TICKS);
    assert_eq!(rig.next_event(), MouseEvent::NotConnected);
    rig.set_polling(true);

    rig.device().online = false;
    rig.wait_for_sends(ONLINE_CODE, 2);
    {
        let mut device = rig.device();
        device.online = true;
        device.mouse_silent = false;
    }

    rig.next_snapshot();
    assert_eq!(rig.finish(), []);
}

#[test]
fn next_tick_counts_from_the_end_of_the_work() {
    let link = FAST.reply_timeout / 2;
    let intervals = PollIntervals {
        link,
        battery: Duration::ZERO,
    };
    let mut device = Device::on(DevicePath::Dongle);
    device.mouse_silent = true;
    let rig = Rig::with_intervals(device, intervals);
    assert_eq!(rig.next_event(), MouseEvent::NotConnected);

    rig.set_polling(true);
    rig.wait_for_sends(IDENTITY_CODE, 3);

    let times = rig.device().send_times_of(IDENTITY_CODE);
    let gaps: Vec<Duration> = times.windows(2).map(|pair| pair[1] - pair[0]).collect();
    assert!(gaps.iter().all(|gap| *gap >= FAST.reply_timeout + link));
    rig.finish();
}

#[test]
fn battery_tick_checks_the_link_first() {
    let rig = Rig::with_intervals(Device::on(DevicePath::Dongle), BATTERY_TICKS);
    rig.next_snapshot();
    let battery_reads = rig.device().sends_of(BATTERY_CODE);
    rig.device().online = false;

    rig.set_polling(true);

    assert_eq!(rig.next_event(), MouseEvent::NotConnected);
    assert_eq!(rig.device().sends_of(BATTERY_CODE), battery_reads);
    assert_eq!(rig.finish(), []);
}

#[test]
fn unsupported_model_sends_no_radio_traffic_on_ticks() {
    let mut device = Device::on(DevicePath::Dongle);
    device.cid = 9;
    let rig = Rig::with_intervals(device, LINK_TICKS);
    let unsupported = MouseError::UnsupportedModel { cid: 9, mid: 50 };
    assert_eq!(
        rig.next_event(),
        MouseEvent::CommandFailed(unsupported.clone())
    );

    rig.set_polling(true);
    rig.wait_for_sends(ONLINE_CODE, 3);

    assert_eq!(rig.device().sends_of(IDENTITY_CODE), 1);

    rig.command(MouseCommand::Refresh);

    assert_eq!(rig.next_event(), MouseEvent::CommandFailed(unsupported));
    assert_eq!(rig.device().sends_of(IDENTITY_CODE), 2);

    rig.send(status_report(0x40));
    rig.wait_for_sends(IDENTITY_CODE, 1);

    assert_eq!(rig.finish(), []);
}
