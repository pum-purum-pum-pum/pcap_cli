use chrono::NaiveTime;
use chrono::{DateTime, Duration, NaiveDateTime, Utc};
use pcap::{Capture, Offline, Error};
use std::{cmp::Ordering, fmt, io, path::Path, str};

use crate::buffer::Buffer;
use crate::parser::content_parser;

// max 3 seconds between accept time and packet time
const ACCEPT_GAP: i64 = 3;

/// Iterator over Quotes(could be generic by Capture\<T\>)
/// # Example
/// ```
/// use pcap_cli::{QuotesIterator, QuotePacket};
/// let input_file = "data/mdf-kospi200.20110216-0.pcap";
/// let mut quotes_iterator = QuotesIterator::new(input_file, true, false).unwrap();
/// let prev = quotes_iterator.next().unwrap();
/// for quote in quotes_iterator {
///     assert!(prev.accept_time <= quote.accept_time);
/// }
/// ```
pub struct QuotesIterator {
    cap: Capture<Offline>,
    reorder: bool,
    step_by_step: bool,
    buffer_gap: Buffer,
}

impl QuotesIterator {
    /// creates quotes iterator from file
    /// reoder flag performs sorting by accept time
    /// step_by_step flag is for debugging with stdin wait
    pub fn new(input_file: &str, reorder: bool, step_by_step: bool) -> Result<Self, Error> {
        let path = Path::new(input_file);
        let cap = Capture::from_file(&path).unwrap();
        let buffer_gap = Buffer::new();
        Ok(QuotesIterator {
            cap,
            reorder,
            step_by_step,
            buffer_gap,
        })
    }
}

impl Iterator for QuotesIterator {
    type Item = QuotePacket;

    // for reordering we also want to return quotes ASAP
    // so we return quotes while we can and then collect more and 
    // checking if we can return something already
    fn next(&mut self) -> Option<Self::Item> {
        let mut input = String::new();
        // return quotes that we are ready to return
        if self.reorder && self.buffer_gap.check_invariant(ACCEPT_GAP) {
            return self.buffer_gap.pop();
        }
        // collect more quotes (and return if possible)
        while let Ok(packet) = self.cap.next() {
            if self.step_by_step {
                io::stdin().read_line(&mut input).unwrap();
            }
            // we can have different data starts(according to specs), so we just parse our quote
            for start in 0..packet.data.len() {
                if let Ok((_in, quote)) =
                    content_parser(&packet.data[start..], extract_time(packet.header))
                {
                    if self.reorder {
                        self.buffer_gap.insert(quote);
                    } else {
                        return Some(quote);
                    }
                }
            }
            if self.reorder && self.buffer_gap.check_invariant(ACCEPT_GAP) {
                return self.buffer_gap.pop();
            }
        }
        if self.reorder {
            return self.buffer_gap.pop();
        }
        None
    }
}

/// Structure wich contains packet timestamp plus quote data
#[derive(Debug)]
pub struct QuotePacket {
    pub issue_code: String,
    pub bids: Bids,
    pub asks: Asks,
    pub accept_time: NaiveTime,
    pub packet_time: NaiveTime,
}

#[derive(Default, Debug)]
pub struct Bid {
    pub price: u32,
    pub volume: u32,
}

impl fmt::Display for Bid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}@{}", self.price, self.volume)
    }
}

#[derive(Debug)]
pub struct Bids(pub Vec<Bid>);

#[derive(Debug)]
pub struct Asks(pub Vec<Ask>);

#[derive(Default, Debug)]
pub struct Ask {
    pub price: u32,
    pub volume: u32,
}

impl fmt::Display for Ask {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}@{}", self.volume, self.price)
    }
}

impl fmt::Display for Bids {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut result = "".to_string();
        for (i, order) in self.0.iter().enumerate() {
            let space = if i == self.0.len() - 1 { "" } else { " " };
            result.push_str(&format!("{}{}", order, space));
        }
        write!(f, "{}", result)
    }
}

impl fmt::Display for Asks {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut result = "".to_string();
        for (i, order) in self.0.iter().enumerate() {
            let space = if i == self.0.len() { "" } else { " " };
            result.push_str(&format!("{}{}", order, space));
        }
        write!(f, "{}", result)
    }
}

impl fmt::Display for QuotePacket {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} {} {} {} {}",
            self.packet_time, self.accept_time, self.issue_code, self.bids, self.asks
        )
    }
}

impl PartialEq for QuotePacket {
    fn eq(&self, other: &Self) -> bool {
        self.accept_time == other.accept_time
    }
}

impl Eq for QuotePacket {}

impl Ord for QuotePacket {
    fn cmp(&self, other: &Self) -> Ordering {
        self.accept_time.cmp(&other.accept_time)
    }
}

impl PartialOrd for QuotePacket {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

// kludge, in real system will use time zones
const TOKYO_OFFSET: i64 = 9;

fn extract_time(header: &pcap::PacketHeader) -> NaiveTime {
    let date_time = DateTime::<Utc>::from_utc(
        NaiveDateTime::from_timestamp(header.ts.tv_sec, 1000 * header.ts.tv_usec as u32),
        Utc,
    );
    date_time.time() + Duration::hours(TOKYO_OFFSET)
}
