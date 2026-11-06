use super::*;

#[test]
fn encodes_stage_vectors() {
    let cases = [
        (800, 800, [0x4f, 0x4f, 0x00, 0xb7]),
        (10000, 10000, [0xe7, 0xe7, 0xcc, 0xbb]),
        (10050, 10050, [0x00, 0x00, 0x22, 0x33]),
        (30000, 30000, [0x8f, 0x8f, 0x66, 0xd1]),
        (30100, 30100, [0x64, 0x64, 0x33, 0x5a]),
        (42000, 42000, [0xdb, 0xdb, 0x33, 0x6c]),
        (800, 30100, [0x4f, 0x64, 0x30, 0x72]),
    ];

    for (x, y, bytes) in cases {
        let stage = DpiStage { x, y };

        assert_eq!(Sensor::Paw3950Ultra.encode_stage(stage), Ok(bytes));
        assert_eq!(Sensor::Paw3950Ultra.decode_stage(bytes), Ok(stage));
    }
}

#[test]
fn rejects_dpi_off_grid() {
    for value in [5, 15, 10010, 30050, 42100] {
        let stage = DpiStage { x: value, y: 800 };

        assert!(matches!(
            Sensor::Paw3950Ultra.encode_stage(stage),
            Err(ProtocolError::InvalidValue(_))
        ));
        assert!(!Sensor::Paw3950Ultra.is_valid_dpi(value));
    }
}

#[test]
fn nearest_valid_dpi_snaps() {
    let cases = [
        (14, 10),
        (10030, 10050),
        (30040, 30000),
        (50000, 42000),
        (0, 10),
    ];

    for (value, expected) in cases {
        assert_eq!(Sensor::Paw3950Ultra.nearest_valid_dpi(value), expected);
    }
}

#[test]
fn decode_rejects_bad_check() {
    assert_eq!(
        Sensor::Paw3950Ultra.decode_stage([0x4f, 0x4f, 0x00, 0x00]),
        Err(ProtocolError::BadCheck { offset: 3 })
    );
}

#[test]
fn dpi_range_is_10_to_42000() {
    assert_eq!(Sensor::Paw3950Ultra.dpi_range(), 10..=42000);
}
