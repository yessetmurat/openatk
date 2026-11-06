use super::*;

#[test]
fn writable_ranges_are_accepted() {
    assert!(is_writable(0x00, 10));
    assert!(is_writable(0x0c, 10));
    assert!(is_writable(0x60, 10));
    assert!(is_writable(0x100, 10));
}

#[test]
fn ranges_crossing_a_boundary_are_rejected() {
    assert!(!is_writable(0x05, 10));
    assert!(!is_writable(0x1b8, 10));
}

#[test]
fn sensor_ranges_are_writable_whole() {
    assert!(is_writable(LIFT_OFF, 2));
    assert!(is_writable(SENSOR_BLOCK, 10));
    assert!(is_writable(SLEEP_BLOCK, 6));
    assert!(is_writable(ANGLE_BLOCK, 4));
}

#[test]
fn sensor_ranges_end_where_the_neighbours_start() {
    assert!(!is_writable(0x0a, 3));
    assert!(!is_writable(SENSOR_BLOCK, 11));
    assert!(!is_writable(0xb3, 2));
    assert!(!is_writable(SLEEP_BLOCK, 7));
    assert!(!is_writable(0xbb, 2));
    assert!(!is_writable(ANGLE_BLOCK, 5));
}
