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
    ];

    for (command, expected) in pairs {
        assert_eq!(command.into_core(), Ok(expected));
    }
}
