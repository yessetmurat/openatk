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
