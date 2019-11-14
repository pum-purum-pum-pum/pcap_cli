//! # PCAP CLI
//!
//! `pcap_cli` is a cli and crate for reading quotes data(with provided packet spec.) from pcap files made for
//!
//! Example of usage:
//! `cargo run --release data/mdf-kospi200.20110216-0.pcap  -r`

pub use parser::content_parser;
pub use quote::QuotesIterator;
pub use quote::{Ask, Asks, Bid, Bids, QuotePacket};
pub use buffer::Buffer;

// parser from &[u8] to QuotePacket
mod parser;
// QuotePacket definition and boilerplate
mod quote;
#[cfg(test)]
// integration test on provided test data
mod tests;
// data structures for processing data with time(3 seconds) gap
mod buffer;
