use super::*;
use crate::settings::Button;

#[test]
fn button_vectors() {
    let cases = [
        (ButtonAction::Mouse(MouseAction::Left), [1, 1, 0, 0x53]),
        (ButtonAction::Mouse(MouseAction::Disabled), [1, 0, 0, 0x54]),
        (ButtonAction::Mouse(MouseAction::Side1), [1, 8, 0, 0x4c]),
        (ButtonAction::Dpi(DpiAction::Cycle), [2, 1, 0, 0x52]),
        (ButtonAction::Wheel(WheelAction::Up), [11, 1, 0, 0x49]),
        (ButtonAction::Shortcut, [5, 0, 0, 0x50]),
    ];

    for (action, bytes) in cases {
        assert_eq!(action.encode(), Some(bytes));
        assert_eq!(ButtonAction::decode(bytes), Ok(action));
    }
}

#[test]
fn every_editable_action_round_trips() {
    let mouse = [
        MouseAction::Disabled,
        MouseAction::Left,
        MouseAction::Right,
        MouseAction::Middle,
        MouseAction::Side1,
        MouseAction::Side2,
    ];
    let mut actions: Vec<ButtonAction> = mouse.into_iter().map(ButtonAction::Mouse).collect();
    actions.extend([DpiAction::Cycle, DpiAction::Plus, DpiAction::Minus].map(ButtonAction::Dpi));
    actions.extend([WheelAction::Up, WheelAction::Down].map(ButtonAction::Wheel));
    actions.push(ButtonAction::Shortcut);

    for action in actions {
        let bytes = action.encode().expect("editable action encodes");

        assert_eq!(ButtonAction::decode(bytes), Ok(action));
        assert!(action.is_editable());
    }
}

#[test]
fn unknown_class_decodes_to_other_and_does_not_encode() {
    let bytes = [6, 3, 4, check::sum(&[6, 3, 4])];

    let action = ButtonAction::decode(bytes).expect("valid check");

    assert_eq!(
        action,
        ButtonAction::Other {
            class: 6,
            value1: 3,
            value2: 4
        }
    );
    assert_eq!(action.encode(), None);
    assert!(!action.is_editable());
}

#[test]
fn class_zero_and_unknown_values_decode_to_other() {
    let off = [0, 0, 0, check::sum(&[0, 0, 0])];
    let odd_mouse = [1, 3, 0, check::sum(&[1, 3, 0])];

    assert!(matches!(
        ButtonAction::decode(off),
        Ok(ButtonAction::Other { class: 0, .. })
    ));
    assert!(matches!(
        ButtonAction::decode(odd_mouse),
        Ok(ButtonAction::Other {
            class: 1,
            value1: 3,
            ..
        })
    ));
}

#[test]
fn decode_rejects_bad_check() {
    assert_eq!(
        ButtonAction::decode([1, 1, 0, 0x00]),
        Err(ProtocolError::BadCheck { offset: 3 })
    );
}

#[test]
fn button_addresses() {
    assert_eq!(Button::Left.address(), 0x60);
    assert_eq!(Button::Side2.address(), 0x70);
    assert_eq!(Button::Left.shortcut_address(), 0x100);
    assert_eq!(Button::Side2.shortcut_address(), 0x180);
}
