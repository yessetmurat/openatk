use super::*;

#[test]
fn finds_the_supported_model_only() {
    let model = Model::find(2, 50).expect("known model");

    assert_eq!(model.name, "ATK F1 EXTREME");
    assert!(Model::find(9, 50).is_none());
    assert!(Model::find(2, 51).is_none());
}

#[test]
fn dongle_offers_seven_rates_and_cable_four() {
    let model = Model::find(2, 50).expect("known model");

    assert_eq!(model.polling_rates(DevicePath::Dongle).len(), 7);
    assert_eq!(model.polling_rates(DevicePath::Cable).len(), 4);
    assert_eq!(model.buttons.len(), 5);
}

#[test]
fn offers_the_atk_hub_lists() {
    let model = Model::find(2, 50).expect("known model");

    assert_eq!(model.debounce_ms, [0, 1, 2, 4, 8, 15, 20]);
    assert_eq!(
        model.sleep_seconds,
        [30, 60, 120, 180, 300, 1200, 1500, 1800]
    );
}

#[test]
fn angle_needs_the_minimum_firmware() {
    let model = Model::find(2, 50).expect("known model");
    let minimum = FirmwareVersion { major: 3, minor: 0 };
    let below = FirmwareVersion {
        major: 2,
        minor: 0x99,
    };

    assert_eq!(model.angle_min_firmware, Some(minimum));
    assert!(model.supports_angle(Some(minimum)));
    assert!(!model.supports_angle(Some(below)));
    assert!(!model.supports_angle(None));
}
