use super::*;

const CABLE: (u16, u16, u16) = (0x1045, 0xff02, 0x02);
const DONGLE: (u16, u16, u16) = (0x1031, 0xff02, 0x02);

#[test]
fn prefers_cable() {
    assert_eq!(choose(&[DONGLE, CABLE]), Some(DevicePath::Cable));
}

#[test]
fn falls_back_to_dongle() {
    assert_eq!(choose(&[DONGLE]), Some(DevicePath::Dongle));
}

#[test]
fn ignores_other_collections() {
    let other_page = (0x1045, 0xff04, 0x02);
    let other_usage = (0x1045, 0xff02, 0x01);

    assert_eq!(choose(&[other_page, other_usage]), None);
}

#[test]
fn none_when_absent() {
    assert_eq!(choose(&[]), None);
    assert_eq!(choose(&[(0x9999, 0xff02, 0x02)]), None);
}
