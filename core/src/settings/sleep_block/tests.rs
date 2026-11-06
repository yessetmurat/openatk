use super::*;

const MOUSE_BYTES: [u8; 6] = [0x00, 0x55, 0x06, 0x4f, 0x00, 0x55];

#[test]
fn decodes_mouse_bytes() {
    let block = SleepBlock::from_raw(MOUSE_BYTES);

    assert_eq!(block.sensor_sleep_enabled(), Some(false));
    assert_eq!(block.sensor_sleep_byte(), Some(6));
    assert_eq!(block.as_bytes(), &MOUSE_BYTES);
}

#[test]
fn with_sleep_sets_enabled_and_the_sleep_byte() {
    let block = SleepBlock::from_raw(MOUSE_BYTES);

    let changed = block.with_sleep(180).expect("readable fields");

    assert_eq!(changed.as_bytes(), &[0x01, 0x54, 180, 0xa1, 0x00, 0x55]);
}

#[test]
fn keeps_neighbours_exactly() {
    let raw = [0x00, 0x55, 0x06, 0x4f, 0x07, 0x99];

    let changed = SleepBlock::from_raw(raw).with_sleep(3).expect("own pairs");

    assert_eq!(changed.as_bytes(), &[0x01, 0x54, 3, 0x52, 0x07, 0x99]);
}

#[test]
fn rejects_values_outside_the_list() {
    let block = SleepBlock::from_raw([0x02, 0x53, 0x06, 0x4f, 0x00, 0x55]);

    assert_eq!(block.sensor_sleep_enabled(), None);
    assert!(block.with_sleep(3).is_err());
}

#[test]
fn bad_check_reads_as_unknown_and_refuses_a_write() {
    let bad_enabled = SleepBlock::from_raw([0x00, 0x99, 0x06, 0x4f, 0x00, 0x55]);
    let bad_sleep = SleepBlock::from_raw([0x00, 0x55, 0x06, 0x99, 0x00, 0x55]);

    assert_eq!(bad_enabled.sensor_sleep_enabled(), None);
    assert_eq!(bad_enabled.sensor_sleep_byte(), Some(6));
    assert_eq!(bad_sleep.sensor_sleep_byte(), None);
    assert!(bad_enabled.with_sleep(3).is_err());
    assert!(bad_sleep.with_sleep(3).is_err());
}
