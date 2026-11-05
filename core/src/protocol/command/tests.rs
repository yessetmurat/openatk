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
