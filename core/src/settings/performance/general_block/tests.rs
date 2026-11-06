use super::*;

fn raw_with(count: u8, current: u8) -> [u8; 10] {
    [
        0x01,
        0x54,
        count,
        check::single(count),
        current,
        check::single(current),
        0xaa,
        0xbb,
        0xcc,
        0xdd,
    ]
}

#[test]
fn changing_polling_keeps_invalid_neighbour_bytes() {
    let raw = [0x01, 0x54, 0x04, 0x51, 0x00, 0x55, 0xaa, 0xbb, 0xcc, 0xdd];

    let block = GeneralBlock::from_raw(raw).expect("checked fields are valid");
    let changed = block.with_polling(PollingRate::Hz8000);

    assert_eq!(changed.as_bytes()[..2], [0x40, 0x15]);
    assert_eq!(changed.as_bytes()[2..], raw[2..]);
    assert_eq!(changed.polling(), PollingRate::Hz8000);
}

#[test]
fn from_raw_rejects_bad_polling_check() {
    let mut raw = raw_with(4, 0);
    raw[1] = 0;

    assert_eq!(
        GeneralBlock::from_raw(raw),
        Err(ProtocolError::BadCheck { offset: 1 })
    );
}

#[test]
fn from_raw_rejects_unknown_polling_code() {
    let mut raw = raw_with(4, 0);
    raw[0] = 3;
    raw[1] = check::single(3);

    assert!(matches!(
        GeneralBlock::from_raw(raw),
        Err(ProtocolError::InvalidValue(_))
    ));
}

#[test]
fn from_raw_rejects_stage_count_0_and_9() {
    for count in [0, 9] {
        assert!(matches!(
            GeneralBlock::from_raw(raw_with(count, 0)),
            Err(ProtocolError::InvalidValue(_))
        ));
    }
}

#[test]
fn from_raw_rejects_current_stage_not_below_count() {
    assert!(matches!(
        GeneralBlock::from_raw(raw_with(4, 4)),
        Err(ProtocolError::InvalidValue(_))
    ));
}

#[test]
fn from_raw_rejects_bad_count_check() {
    let mut raw = raw_with(4, 0);
    raw[3] = 0;

    assert_eq!(
        GeneralBlock::from_raw(raw),
        Err(ProtocolError::BadCheck { offset: 3 })
    );
}

#[test]
fn setters_touch_only_their_field_and_check() {
    let block = GeneralBlock::from_raw(raw_with(4, 1)).expect("valid");

    let counted = block.with_stage_count(6).expect("in range");
    let moved = block.with_current_stage(3).expect("below count");

    assert_eq!(counted.stage_count(), 6);
    assert_eq!(counted.as_bytes()[3], check::single(6));
    assert_eq!(counted.current_stage(), 1);
    assert_eq!(moved.current_stage(), 3);
    assert_eq!(moved.as_bytes()[5], check::single(3));
    assert_eq!(moved.stage_count(), 4);
}

#[test]
fn setters_reject_out_of_range() {
    let block = GeneralBlock::from_raw(raw_with(4, 1)).expect("valid");

    assert!(block.with_stage_count(0).is_err());
    assert!(block.with_stage_count(9).is_err());
    assert!(block.with_current_stage(4).is_err());
    assert!(block.with_stage_count(1).is_ok());
}
