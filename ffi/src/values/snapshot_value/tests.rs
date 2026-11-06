use openatk_core::device::{self, Battery, MouseSnapshot};
use openatk_core::settings::{
    Button, ButtonAction, DpiStage, FirmwareVersion, MouseAction, PollingRate, Sensor, Shortcut,
    ShortcutKey,
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
        firmware: Some(FirmwareVersion {
            major: 3,
            minor: 0x10,
        }),
        lift_off_tenths: Some(12),
        debounce_ms: Some(4),
        debounce_options_ms: vec![0, 4, 20],
        motion_sync: Some(true),
        linear_correction: Some(false),
        ripple_control: Some(true),
        sleep_seconds: Some(60),
        sleep_options_seconds: vec![30, 60, 1800],
        angle: Some(device::AngleValue {
            degrees: Some(-30),
            enabled: Some(true),
        }),
        far_distance: Some(false),
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
fn snapshot_converts_new_fields() {
    let value = SnapshotValue::from(&sample_snapshot());

    assert_eq!(value.firmware.as_deref(), Some("3.10"));
    assert_eq!(value.lift_off_tenths, Some(12));
    assert_eq!(
        (value.lift_off_min_tenths, value.lift_off_max_tenths),
        (7, 17)
    );
    assert_eq!(value.debounce_ms, Some(4));
    assert_eq!(value.debounce_options_ms, vec![0, 4, 20]);
    assert_eq!(value.motion_sync, Some(true));
    assert_eq!(value.linear_correction, Some(false));
    assert_eq!(value.ripple_control, Some(true));
    assert_eq!(value.sleep_seconds, Some(60));
    assert_eq!(value.sleep_options_seconds, vec![30, 60, 1800]);
    assert_eq!(
        value.angle,
        Some(AngleValue {
            degrees: Some(-30),
            enabled: Some(true),
        })
    );
    assert_eq!((value.angle_min, value.angle_max), (-30, 30));
    assert_eq!(value.far_distance, Some(false));
}

#[test]
fn off_list_values_pass_through() {
    let mut snapshot = sample_snapshot();
    snapshot.lift_off_tenths = Some(6);
    snapshot.debounce_ms = Some(3);
    snapshot.sleep_seconds = Some(10);

    let value = SnapshotValue::from(&snapshot);

    assert_eq!(value.lift_off_tenths, Some(6));
    assert_eq!(value.debounce_ms, Some(3));
    assert_eq!(value.sleep_seconds, Some(10));
    assert!(!value.sleep_options_seconds.contains(&10));
}

#[test]
fn absent_angle_stays_absent() {
    let mut snapshot = sample_snapshot();
    snapshot.angle = None;

    let value = SnapshotValue::from(&snapshot);

    assert_eq!(value.angle, None);
}

#[test]
fn unknown_angle_fields_stay_unknown_inside_a_present_angle() {
    let mut snapshot = sample_snapshot();
    snapshot.angle = Some(device::AngleValue {
        degrees: Some(5),
        enabled: None,
    });

    let value = SnapshotValue::from(&snapshot);

    assert_eq!(
        value.angle,
        Some(AngleValue {
            degrees: Some(5),
            enabled: None,
        })
    );
}

#[test]
fn unknown_fields_stay_unknown() {
    let mut snapshot = sample_snapshot();
    snapshot.firmware = None;
    snapshot.lift_off_tenths = None;
    snapshot.far_distance = None;

    let value = SnapshotValue::from(&snapshot);

    assert_eq!(value.firmware, None);
    assert_eq!(value.lift_off_tenths, None);
    assert_eq!(value.far_distance, None);
}

#[test]
fn nearest_valid_dpi_uses_the_grid() {
    let value = SnapshotValue::from(&sample_snapshot());

    assert_eq!(nearest_valid_dpi(&value, 10030), 10050);
}
