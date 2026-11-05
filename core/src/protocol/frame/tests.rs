use super::*;

#[test]
fn identity_request_matches_hardware_capture() {
    let report = Frame::request(Command::GetMouseCidMid).to_report();

    assert_eq!(report[..2], [0x08, 0x10]);
    assert_eq!(report[16], 0x3d);
}

#[test]
fn identity_reply_from_hardware_parses() {
    let report = [
        0x08, 0x10, 0, 0, 0, 0x03, 0x02, 0x32, 0x21, 0, 0, 0, 0, 0, 0, 0, 0xe5,
    ];

    let frame = Frame::parse(&report).expect("valid capture");

    assert_eq!(frame.status(), ReplyStatus::Success);
    assert_eq!(frame.data(), [0x02, 0x32, 0x21]);
}

#[test]
fn parse_rejects_wrong_length() {
    assert_eq!(Frame::parse(&[0x08; 16]), Err(ProtocolError::BadLength));
    assert_eq!(Frame::parse(&[0x08; 65]), Err(ProtocolError::BadLength));
}

#[test]
fn parse_rejects_wrong_report_id() {
    let mut report = Frame::request(Command::GetBatteryLevel).to_report();
    report[0] = 0x09;

    assert_eq!(Frame::parse(&report), Err(ProtocolError::BadReportId));
}

#[test]
fn parse_rejects_bad_checksum() {
    let mut report = Frame::request(Command::GetBatteryLevel).to_report();
    report[16] ^= 0xff;

    assert_eq!(Frame::parse(&report), Err(ProtocolError::BadChecksum));
}

#[test]
fn parse_rejects_valid_length_above_ten() {
    let mut report = Frame::request(Command::GetBatteryLevel).to_report();
    report[5] = 11;
    report[16] = check::sum(&report[..16]);

    assert_eq!(Frame::parse(&report), Err(ProtocolError::BadLength));
}

#[test]
fn parse_keeps_unknown_command_codes() {
    let mut report = Frame::request(Command::GetBatteryLevel).to_report();
    report[1] = 90;
    report[16] = check::sum(&report[..16]);

    let frame = Frame::parse(&report).expect("checksum is valid");

    assert_eq!(frame.command_code(), 90);
}

#[test]
fn address_is_big_endian() {
    let report = Frame::get_eeprom(0x0160, 10)
        .expect("valid read")
        .to_report();

    assert_eq!(report[3..5], [0x01, 0x60]);
}

#[test]
fn set_eeprom_rejects_unlisted_address() {
    for address in [0xa0, 0x300, 0x1b30] {
        assert_eq!(
            Frame::set_eeprom(address, &[1]),
            Err(ProtocolError::AddressNotWritable(address))
        );
    }
}

#[test]
fn set_eeprom_rejects_data_over_10_bytes() {
    assert_eq!(
        Frame::set_eeprom(0x100, &[0; 11]),
        Err(ProtocolError::DataTooLong)
    );
}

#[test]
fn set_eeprom_round_trips_through_parse() {
    let report = Frame::set_eeprom(0x0c, &[1, 2, 3])
        .expect("writable")
        .to_report();

    let frame = Frame::parse(&report).expect("own report parses");

    assert_eq!(frame.command_code(), Command::SetEeprom.code());
    assert_eq!(frame.address(), 0x0c);
    assert_eq!(frame.data(), [1, 2, 3]);
}

#[test]
fn payload_ignores_valid_length() {
    let mut report = [0u8; REPORT_LEN];
    report[0] = 0x08;
    report[1] = 0x0a;
    report[5] = 0;
    report[6] = 0x40;
    report[7] = 0x02;
    report[16] = check::sum(&report[..16]);

    let frame = Frame::parse(&report).unwrap();

    assert!(frame.data().is_empty());
    assert_eq!(frame.payload()[0], 0x40);
    assert_eq!(frame.payload()[1], 0x02);
}
