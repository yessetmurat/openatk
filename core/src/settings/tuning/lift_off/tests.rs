use super::*;

#[test]
fn decodes_mouse_bytes() {
    let lift_off = LiftOff::from_raw([0x01, 0x54]);

    assert_eq!(lift_off.millimetres_tenths(), Some(7));
}

#[test]
fn encodes_the_ends_of_the_range() {
    let lift_off = LiftOff::from_raw([0x01, 0x54]);

    let lowest = lift_off.with_tenths(7).expect("in range");
    let highest = lift_off.with_tenths(17).expect("in range");

    assert_eq!(lowest.as_bytes(), &[0x01, 0x54]);
    assert_eq!(highest.as_bytes(), &[0x0b, 0x4a]);
    assert_eq!(highest.millimetres_tenths(), Some(17));
}

#[test]
fn keeps_neighbours_exactly() {
    let raw = [0x05, 0x99];

    let lift_off = LiftOff::from_raw(raw);

    assert_eq!(lift_off.as_bytes(), &raw);
}

#[test]
fn rejects_values_outside_the_list() {
    let lift_off = LiftOff::from_raw([0x01, 0x54]);

    assert!(lift_off.with_tenths(6).is_err());
    assert!(lift_off.with_tenths(18).is_err());
}

#[test]
fn off_list_byte_shows_as_itself_and_can_be_replaced() {
    let lift_off = LiftOff::from_raw([0x0c, 0x49]);

    assert_eq!(lift_off.millimetres_tenths(), Some(18));
    assert_eq!(
        lift_off.with_tenths(10).expect("listed").as_bytes(),
        &[0x04, 0x51]
    );
}

#[test]
fn bad_check_reads_as_unknown_and_refuses_a_write() {
    let lift_off = LiftOff::from_raw([0x01, 0x55]);

    assert_eq!(lift_off.millimetres_tenths(), None);
    assert!(lift_off.with_tenths(10).is_err());
}

#[test]
fn byte_too_large_for_tenths_reads_as_unknown_and_refuses_a_write() {
    let lift_off = LiftOff::from_raw([0xfa, check::single(0xfa)]);

    assert_eq!(lift_off.millimetres_tenths(), None);
    assert!(lift_off.with_tenths(10).is_err());
}
