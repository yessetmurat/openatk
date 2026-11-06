use super::*;

const MOUSE_BYTES: [u8; 4] = [0x00, 0x55, 0x00, 0x55];

#[test]
fn decodes_mouse_bytes() {
    let block = AngleBlock::from_raw(MOUSE_BYTES);

    assert_eq!(block.angle(), Some(0));
    assert_eq!(block.enabled(), Some(false));
    assert_eq!(block.as_bytes(), &MOUSE_BYTES);
}

#[test]
fn angle_round_trips() {
    let block = AngleBlock::from_raw(MOUSE_BYTES);

    for degrees in [-30, -1, 0, 1, 30] {
        let changed = block.with_angle(degrees).expect("in range");
        let reread = AngleBlock::from_raw(*changed.as_bytes());

        assert_eq!(reread.angle(), Some(degrees));
    }
}

#[test]
fn stores_a_negative_angle_as_twos_complement() {
    let block = AngleBlock::from_raw(MOUSE_BYTES);

    let changed = block.with_angle(-30).expect("in range");

    assert_eq!(changed.as_bytes(), &[0xe2, 0x73, 0x00, 0x55]);
}

#[test]
fn keeps_neighbours_exactly() {
    let bad_switch = [0x00, 0x55, 0x01, 0x99];
    let bad_angle = [0x05, 0x99, 0x00, 0x55];

    let angle = AngleBlock::from_raw(bad_switch).with_angle(-1);
    let switch = AngleBlock::from_raw(bad_angle).with_enabled(true);

    assert_eq!(
        angle.expect("own pair").as_bytes(),
        &[0xff, 0x56, 0x01, 0x99]
    );
    assert_eq!(
        switch.expect("own pair").as_bytes(),
        &[0x05, 0x99, 0x01, 0x54]
    );
}

#[test]
fn rejects_values_outside_the_list() {
    let block = AngleBlock::from_raw(MOUSE_BYTES);

    assert!(block.with_angle(31).is_err());
    assert!(block.with_angle(-31).is_err());
}

#[test]
fn bad_check_reads_as_unknown_and_refuses_a_write() {
    let block = AngleBlock::from_raw([0x00, 0x99, 0x00, 0x99]);

    assert_eq!(block.angle(), None);
    assert_eq!(block.enabled(), None);
    assert!(block.with_angle(1).is_err());
    assert!(block.with_enabled(true).is_err());
}
