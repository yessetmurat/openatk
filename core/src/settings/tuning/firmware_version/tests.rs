use super::*;

#[test]
fn displays_the_mouse_bytes_as_hex() {
    let real = FirmwareVersion { major: 3, minor: 1 };
    let hex_minor = FirmwareVersion {
        major: 3,
        minor: 0x10,
    };

    assert_eq!(real.to_string(), "3.01");
    assert_eq!(hex_minor.to_string(), "3.10");
}

#[test]
fn compares_major_before_minor() {
    let old = FirmwareVersion {
        major: 2,
        minor: 0x99,
    };
    let minimum = FirmwareVersion { major: 3, minor: 0 };
    let newer = FirmwareVersion { major: 3, minor: 1 };

    assert!(old < minimum);
    assert!(newer > minimum);
}
