use super::*;

const MOUSE_BYTES: [u8; 10] = [0x00, 0x55, 0x00, 0x55, 0x01, 0x54, 0x00, 0x55, 0x00, 0x55];
const BAD_CHECKS: [u8; 10] = [0x04, 0x99, 0x01, 0x98, 0x03, 0x97, 0x01, 0x96, 0x01, 0x95];

#[test]
fn decodes_mouse_bytes() {
    let block = SensorBlock::from_raw(MOUSE_BYTES);

    assert_eq!(block.debounce_ms(), Some(0));
    assert_eq!(block.motion_sync(), Some(false));
    assert_eq!(block.led_off_byte(), Some(1));
    assert_eq!(block.linear_correction(), Some(false));
    assert_eq!(block.ripple_control(), Some(false));
    assert_eq!(block.as_bytes(), &MOUSE_BYTES);
}

#[test]
fn keeps_neighbours_exactly() {
    let mut debounce = BAD_CHECKS;
    debounce[0..2].copy_from_slice(&[0x00, 0x55]);
    let mut motion_sync = BAD_CHECKS;
    motion_sync[2..4].copy_from_slice(&[0x00, 0x55]);
    let mut led_off = BAD_CHECKS;
    led_off[4..6].copy_from_slice(&[0x00, 0x55]);
    let mut linear = BAD_CHECKS;
    linear[6..8].copy_from_slice(&[0x00, 0x55]);
    let mut ripple = BAD_CHECKS;
    ripple[8..10].copy_from_slice(&[0x00, 0x55]);

    let debounce = SensorBlock::from_raw(debounce).with_debounce_ms(20);
    let motion_sync = SensorBlock::from_raw(motion_sync).with_motion_sync(true);
    let led_off = SensorBlock::from_raw(led_off).with_led_off_byte(180);
    let linear = SensorBlock::from_raw(linear).with_linear_correction(true);
    let ripple = SensorBlock::from_raw(ripple).with_ripple_control(true);

    let expect_only = |offset: usize, pair: [u8; 2]| {
        let mut bytes = BAD_CHECKS;
        bytes[offset..offset + 2].copy_from_slice(&pair);

        bytes
    };
    assert_eq!(
        debounce.expect("own pair").as_bytes(),
        &expect_only(0, [20, 0x41])
    );
    assert_eq!(
        motion_sync.expect("own pair").as_bytes(),
        &expect_only(2, [1, 0x54])
    );
    assert_eq!(
        led_off.expect("own pair").as_bytes(),
        &expect_only(4, [180, 0xa1])
    );
    assert_eq!(
        linear.expect("own pair").as_bytes(),
        &expect_only(6, [1, 0x54])
    );
    assert_eq!(
        ripple.expect("own pair").as_bytes(),
        &expect_only(8, [1, 0x54])
    );
}

#[test]
fn rejects_values_outside_the_list() {
    let mut raw = MOUSE_BYTES;
    raw[2..4].copy_from_slice(&[0x02, 0x53]);

    let block = SensorBlock::from_raw(raw);

    assert_eq!(block.motion_sync(), None);
    assert!(block.with_motion_sync(true).is_err());
    assert_eq!(block.linear_correction(), Some(false));
}

#[test]
fn bad_check_reads_as_unknown_and_refuses_a_write() {
    let block = SensorBlock::from_raw(BAD_CHECKS);

    assert_eq!(block.debounce_ms(), None);
    assert_eq!(block.motion_sync(), None);
    assert_eq!(block.led_off_byte(), None);
    assert_eq!(block.linear_correction(), None);
    assert_eq!(block.ripple_control(), None);
    assert_eq!(
        block.with_debounce_ms(4),
        Err(ProtocolError::BadCheck { offset: 1 })
    );
    assert_eq!(
        block.with_led_off_byte(3),
        Err(ProtocolError::BadCheck { offset: 5 })
    );
    assert!(block.with_motion_sync(true).is_err());
    assert!(block.with_linear_correction(true).is_err());
    assert!(block.with_ripple_control(true).is_err());
}
