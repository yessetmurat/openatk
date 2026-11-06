use openatk_core::device::{Battery, MouseSnapshot};
use openatk_core::settings::{
    Button, ButtonAction, DpiStage, MouseAction, PollingRate, Sensor, Shortcut, ShortcutKey,
};

use super::*;

fn sample_snapshot() -> MouseSnapshot {
    let shortcut = Shortcut::from_keys(&[ShortcutKey::Modifier(1), ShortcutKey::General(6)])
        .expect("valid shortcut");

    MouseSnapshot {
        model_name: "Test Mouse".to_string(),
        cid: 1,
        mid: 2,
        profile: 3,
        battery: Battery {
            percent: 80,
            charging: true,
        },
        polling: PollingRate::Hz1000,
        polling_rates: vec![PollingRate::Hz125, PollingRate::Hz1000],
        sensor: Sensor::Paw3950Ultra,
        current_stage: 1,
        stages: vec![DpiStage { x: 800, y: 800 }, DpiStage { x: 1600, y: 1200 }],
        buttons: vec![
            (Button::Left, ButtonAction::Mouse(MouseAction::Left)),
            (Button::Side1, ButtonAction::Shortcut),
            (
                Button::Side2,
                ButtonAction::Other {
                    class: 9,
                    value1: 8,
                    value2: 7,
                },
            ),
        ],
        shortcuts: vec![(Button::Side1, shortcut)],
    }
}

#[test]
fn snapshot_converts_all_fields() {
    let value = SnapshotValue::from(&sample_snapshot());

    assert_eq!(value.model_name, "Test Mouse");
    assert_eq!(value.profile, 3);
    assert_eq!(value.percent, 80);
    assert!(value.charging);
    assert_eq!(value.polling_hz, 1000);
    assert_eq!(value.polling_options_hz, vec![125, 1000]);
    assert_eq!((value.dpi_min, value.dpi_max), (10, 42000));
    assert_eq!(value.sensor, SensorValue::Paw3950Ultra);
    assert_eq!(value.current_stage, 1);
    assert_eq!(
        value.stages,
        vec![
            StageValue { x: 800, y: 800 },
            StageValue { x: 1600, y: 1200 }
        ]
    );
    assert_eq!(value.buttons.len(), 3);
    assert_eq!(value.buttons[0].keys, None);
    assert_eq!(
        value.buttons[1].keys,
        Some(vec![
            ShortcutKeyValue::Modifier { mask: 1 },
            ShortcutKeyValue::General { usage: 6 },
        ])
    );
    assert_eq!(value.buttons[1].action, Action::Shortcut);
}

#[test]
fn nearest_valid_dpi_uses_the_grid() {
    let value = SnapshotValue::from(&sample_snapshot());

    assert_eq!(nearest_valid_dpi(&value, 10030), 10050);
}
