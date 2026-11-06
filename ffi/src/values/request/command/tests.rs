use super::*;

#[test]
fn unknown_hz_is_refused() {
    let command = Command::SetPolling { hz: 333 };

    assert!(command.into_core().is_err());
    assert_eq!(
        Command::SetPolling { hz: 8000 }.into_core(),
        Ok(MouseCommand::SetPolling(PollingRate::Hz8000))
    );
}

#[test]
fn bad_key_list_is_refused() {
    let command = Command::SetShortcut {
        button: MouseButton::Side1,
        keys: vec![
            ShortcutKeyValue::General { usage: 4 },
            ShortcutKeyValue::Modifier { mask: 1 },
        ],
    };

    assert!(command.into_core().is_err());
}

#[test]
fn sensor_commands_convert() {
    let pairs = [
        (
            Command::SetLiftOff { tenths: 12 },
            MouseCommand::SetLiftOff(12),
        ),
        (Command::SetDebounce { ms: 4 }, MouseCommand::SetDebounce(4)),
        (
            Command::SetMotionSync { enabled: true },
            MouseCommand::SetMotionSync(true),
        ),
        (
            Command::SetLinearCorrection { enabled: true },
            MouseCommand::SetLinearCorrection(true),
        ),
        (
            Command::SetRippleControl { enabled: false },
            MouseCommand::SetRippleControl(false),
        ),
        (
            Command::SetSleep { seconds: 1800 },
            MouseCommand::SetSleep(1800),
        ),
        (
            Command::SetAngle { degrees: -30 },
            MouseCommand::SetAngle(-30),
        ),
        (
            Command::SetAngleEnabled { enabled: true },
            MouseCommand::SetAngleEnabled(true),
        ),
        (
            Command::SetFarDistance { enabled: true },
            MouseCommand::SetFarDistance(true),
        ),
        (
            Command::SetProfile { index: 3 },
            MouseCommand::SetProfile(3),
        ),
    ];

    for (command, expected) in pairs {
        assert_eq!(command.into_core(), Ok(expected));
    }
}

#[test]
fn set_profile_passes_any_index_through_for_the_core_to_refuse() {
    let command = Command::SetProfile { index: 4 };

    assert_eq!(command.into_core(), Ok(MouseCommand::SetProfile(4)));
}

#[test]
fn profile_button_action_out_of_range_is_refused_by_index() {
    let command = Command::SetButton {
        button: MouseButton::Left,
        action: Action::Profile { index: 4 },
    };

    assert_eq!(command.into_core(), Err(CommandError::UnknownProfile(4)));
    assert!(CommandError::UnknownProfile(4).to_string().contains('4'));
}

#[test]
fn profile_button_action_in_range_converts() {
    let command = Command::SetButton {
        button: MouseButton::Left,
        action: Action::Profile { index: 3 },
    };

    assert!(command.into_core().is_ok());
}
