use super::*;

#[test]
fn check_single_wraps() {
    assert_eq!(single(0x60), 0xf5);
}

#[test]
fn check_sum_masks() {
    assert_eq!(sum(&[0xff, 0xff]), 0x57);
}
