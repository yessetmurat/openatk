use super::*;

#[test]
fn check_single_wraps() {
    assert_eq!(single(0x60), 0xf5);
}

#[test]
fn check_sum_masks() {
    assert_eq!(sum(&[0xff, 0xff]), 0x57);
}

#[test]
fn verified_reads_a_value_with_its_check() {
    let raw = [0x00, 0x00, 0xe2, 0x73];

    assert_eq!(verified(&raw, 2), Ok(0xe2));
    assert_eq!(
        verified(&raw, 0),
        Err(ProtocolError::BadCheck { offset: 1 })
    );
}

#[test]
fn verified_flag_accepts_only_0_and_1() {
    let raw = [0x00, 0x55, 0x01, 0x54, 0x02, 0x53];

    assert_eq!(verified_flag(&raw, 0), Ok(false));
    assert_eq!(verified_flag(&raw, 2), Ok(true));
    assert!(verified_flag(&raw, 4).is_err());
}

#[test]
fn replace_touches_only_a_readable_pair() {
    let mut raw = [0x00, 0x55, 0x09, 0x09];

    let refused = replace(&mut raw, 2, 1);
    replace(&mut raw, 0, 0xe2).expect("readable pair");

    assert!(refused.is_err());
    assert_eq!(raw, [0xe2, 0x73, 0x09, 0x09]);
}
