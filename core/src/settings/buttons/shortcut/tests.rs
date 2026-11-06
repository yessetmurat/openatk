use super::*;

#[test]
fn single_key_a_packs_like_atk_hub() {
    let shortcut = Shortcut::from_keys(&[ShortcutKey::General(0x04)]).expect("one key");

    assert_eq!(
        shortcut.to_frames(),
        vec![[0x02, 0x81, 0x04, 0x00, 0x41, 0x04, 0x00, 0x89, 0x00, 0x00]]
    );
}

#[test]
fn modifier_entry_uses_type_0() {
    let shortcut = Shortcut::from_keys(&[ShortcutKey::Modifier(0x01)]).expect("one key");

    assert_eq!(
        shortcut.entries,
        vec![[0x80, 0x01, 0x00], [0x40, 0x01, 0x00]]
    );
}

#[test]
fn ctrl_c_packs_like_atk_hub() {
    let keys = [ShortcutKey::Modifier(1), ShortcutKey::General(6)];

    let shortcut = Shortcut::from_keys(&keys).expect("ctrl c");

    assert_eq!(
        shortcut.to_frames(),
        vec![
            [0x04, 0x80, 0x01, 0x00, 0x81, 0x06, 0x00, 0x40, 0x01, 0x00],
            [0x41, 0x06, 0x00, 0xc1, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00],
        ]
    );
}

#[test]
fn ctrl_shift_c_fills_both_frames() {
    let keys = [
        ShortcutKey::Modifier(1),
        ShortcutKey::Modifier(2),
        ShortcutKey::General(6),
    ];

    let shortcut = Shortcut::from_keys(&keys).expect("ctrl shift c");

    assert_eq!(
        shortcut.to_frames(),
        vec![
            [0x06, 0x80, 0x01, 0x00, 0x80, 0x02, 0x00, 0x81, 0x06, 0x00],
            [0x40, 0x01, 0x00, 0x40, 0x02, 0x00, 0x41, 0x06, 0x00, 0xfb],
        ]
    );
}

#[test]
fn rejects_other_key_lists() {
    let m = ShortcutKey::Modifier;
    let g = ShortcutKey::General;
    let cases: [&[ShortcutKey]; 4] = [&[], &[m(1), m(2), m(4), g(6)], &[g(6), g(7)], &[m(1), m(2)]];

    for keys in cases {
        assert!(matches!(
            Shortcut::from_keys(keys),
            Err(ProtocolError::InvalidValue(_))
        ));
    }
}

#[test]
fn round_trips_through_bytes() {
    let m = ShortcutKey::Modifier;
    let g = ShortcutKey::General;
    let cases: [&[ShortcutKey]; 4] = [&[g(4)], &[m(1)], &[m(1), g(6)], &[m(1), m(2), g(6)]];

    for keys in cases {
        let shortcut = Shortcut::from_keys(keys).expect("valid keys");
        let mut bytes = [0u8; 20];
        for (index, frame) in shortcut.to_frames().iter().enumerate() {
            bytes[index * 10..index * 10 + 10].copy_from_slice(frame);
        }

        let read = Shortcut::from_bytes(&bytes).expect("valid check");

        assert_eq!(read.as_ref(), Some(&shortcut));
        assert_eq!(read.and_then(|value| value.keys()), Some(keys.to_vec()));
    }
}

#[test]
fn from_bytes_count_0_and_7_is_none() {
    for count in [0u8, 7] {
        let mut bytes = [0u8; 20];
        bytes[0] = count;

        assert_eq!(Shortcut::from_bytes(&bytes), Ok(None));
    }
}

#[test]
fn from_bytes_rejects_bad_check() {
    let mut bytes = [0u8; 20];
    bytes[..7].copy_from_slice(&[0x02, 0x81, 0x04, 0x00, 0x41, 0x04, 0x00]);
    bytes[7] = 0x00;

    assert_eq!(
        Shortcut::from_bytes(&bytes),
        Err(ProtocolError::BadCheck { offset: 7 })
    );
}

#[test]
fn media_key_entry_has_no_editable_keys() {
    let entries = vec![[0x82, 0x01, 0x00], [0x42, 0x01, 0x00]];
    let mut payload = vec![2u8];
    payload.extend(entries.iter().flatten());
    payload.push(check::sum(&payload));
    let mut bytes = [0u8; 20];
    bytes[..payload.len()].copy_from_slice(&payload);

    let shortcut = Shortcut::from_bytes(&bytes)
        .expect("valid check")
        .expect("present");

    assert_eq!(shortcut.keys(), None);
    assert_eq!(shortcut.entries, entries);
}
