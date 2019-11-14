use crate::quote::{QuotePacket, QuotesIterator};

#[test]
fn check_reordering() {
    let input_file = "data/mdf-kospi200.20110216-0.pcap";
    let quotes_iterator_reordered = QuotesIterator::new(input_file, true, false);
    let quotes_reoredered: Vec<QuotePacket> = quotes_iterator_reordered.unwrap().collect();

    let quotes_iterator_regular = QuotesIterator::new(input_file, false, false);
    let mut quotes_regular: Vec<QuotePacket> = quotes_iterator_regular.unwrap().collect();
    quotes_regular.sort_by_key(|k| k.accept_time);

    assert_eq!(quotes_reoredered, quotes_regular);
}
