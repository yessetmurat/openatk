use openatk_core::settings::{DpiAction, MouseAction, WheelAction};

use super::*;

#[test]
fn every_editable_action_round_trips() {
    let actions = [
        ButtonAction::Mouse(MouseAction::Disabled),
        ButtonAction::Mouse(MouseAction::Left),
        ButtonAction::Mouse(MouseAction::Right),
        ButtonAction::Mouse(MouseAction::Middle),
        ButtonAction::Mouse(MouseAction::Side1),
        ButtonAction::Mouse(MouseAction::Side2),
        ButtonAction::Dpi(DpiAction::Cycle),
        ButtonAction::Dpi(DpiAction::Plus),
        ButtonAction::Dpi(DpiAction::Minus),
        ButtonAction::Wheel(WheelAction::Up),
        ButtonAction::Wheel(WheelAction::Down),
        ButtonAction::Shortcut,
    ];

    for action in actions {
        let value = Action::from(action);

        assert_eq!(value.editable(), Some(action));
    }
}

#[test]
fn other_action_keeps_its_bytes() {
    let other = ButtonAction::Other {
        class: 9,
        value1: 8,
        value2: 7,
    };

    let value = Action::from(other);

    assert_eq!(value.editable(), None);
    assert_eq!(
        value,
        Action::Other {
            class: 9,
            value1: 8,
            value2: 7
        }
    );
}
