use flash_layout::*;

#[test]
fn ok_layout() {
    let layout = FlashLayout::new(&[Region {
        addr: 100,
        eb_bytes: 4,
        eb_count: 4,
    }]);

    assert_eq!(layout.addr_start(), 100);
    assert_eq!(layout.addr_end(), 100 + 4 * 4);
}
