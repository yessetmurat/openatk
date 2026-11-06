use super::*;

#[test]
fn polling_codes_match_spec() {
    let expected = [
        (8, PollingRate::Hz125, 125),
        (4, PollingRate::Hz250, 250),
        (2, PollingRate::Hz500, 500),
        (1, PollingRate::Hz1000, 1000),
        (16, PollingRate::Hz2000, 2000),
        (32, PollingRate::Hz4000, 4000),
        (64, PollingRate::Hz8000, 8000),
    ];

    for (code, rate, hz) in expected {
        assert_eq!(PollingRate::from_code(code), Some(rate));
        assert_eq!(rate.code(), code);
        assert_eq!(rate.hz(), hz);
        assert_eq!(PollingRate::from_hz(hz), Some(rate));
    }
    assert_eq!(PollingRate::from_code(3), None);
    assert_eq!(PollingRate::from_hz(333), None);
}
