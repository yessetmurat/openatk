use super::*;

#[test]
fn line_round_trips() {
    let line = format_line(0x60, &[0x01, 0x01, 0x00, 0x53]);

    assert_eq!(line, "0x0060 01 01 00 53");
    assert_eq!(parse_line(&line), Ok((0x60, vec![0x01, 0x01, 0x00, 0x53])));
}

#[test]
fn rejects_a_bad_address() {
    assert!(parse_line("0xZZ 01").is_err());
    assert!(parse_line("0060 01").is_err());
    assert!(parse_line("0x10000 01").is_err());
}

#[test]
fn rejects_odd_hex() {
    assert!(parse_line("0x0060 1").is_err());
    assert!(parse_line("0x0060 001").is_err());
    assert!(parse_line("0x0060 zz").is_err());
}

#[test]
fn rejects_empty_data() {
    assert!(parse_line("0x0060").is_err());
}

#[test]
fn rejects_more_than_ten_bytes() {
    let eleven = format_line(0x00, &[0; 11]);

    assert!(parse_line(&eleven).is_err());
    assert!(parse_line(&format_line(0x00, &[0; 10])).is_ok());
}

#[test]
fn parse_dump_rejects_unwritable_address() {
    let error = parse_dump("0x0000 01\n0x00b3 01\n").expect_err("0xb3 is in a gap");

    assert!(error.contains("line 2"));
}

#[test]
fn parse_dump_rejects_range_spilling_past_its_block() {
    let text = format_line(0x08, &[0; 4]);

    assert!(parse_dump(&text).is_err());
}

#[test]
fn parse_dump_accepts_a_full_dump() {
    let text: String = dump_ranges()
        .into_iter()
        .map(|(start, len)| format_line(start, &vec![0; usize::from(len)]) + "\n")
        .collect();

    let parsed = parse_dump(&text).expect("a full dump is valid");

    assert_eq!(parsed.len(), dump_ranges().len());
}

#[test]
fn every_dump_range_is_writable_and_round_trips() {
    let ranges = dump_ranges();

    for block in [(0x0a, 2), (0xa9, 10), (0xb5, 6), (0xbd, 4)] {
        assert!(ranges.contains(&block), "{block:?}");
    }

    for (start, len) in ranges {
        let line = format_line(start, &vec![0; usize::from(len)]);

        assert!(
            address::is_writable(start, usize::from(len)),
            "{start:#06x}"
        );
        assert!(parse_line(&line).is_ok(), "{start:#06x}");
    }
}
