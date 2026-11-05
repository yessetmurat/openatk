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
