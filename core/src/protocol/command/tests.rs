use super::*;

#[test]
fn upgrade_codes_do_not_parse_to_commands() {
    for code in [1, 2, 9, 13, 90, 91] {
        assert_eq!(Command::from_code(code), None);
    }
}

#[test]
fn codes_round_trip() {
    let command = Command::from_code(0x10).expect("known code");

    assert_eq!(command, Command::GetMouseCidMid);
    assert_eq!(command.code(), 0x10);
}

#[test]
fn sensor_codes_round_trip() {
    for (code, command) in [
        (0x12, Command::GetMouseVersion),
        (0x16, Command::SetFarDistanceMode),
        (0x17, Command::GetFarDistanceMode),
    ] {
        assert_eq!(Command::from_code(code), Some(command));
        assert_eq!(command.code(), code);
    }
}
