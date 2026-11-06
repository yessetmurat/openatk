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
