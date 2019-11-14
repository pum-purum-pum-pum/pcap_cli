// buffer in order to save gap with 3 seconds for reordering quotes by accept time
use crate::quote::QuotePacket;
use chrono::{Duration, NaiveTime};
use std::cmp::Reverse;
use std::collections::{BTreeSet, BinaryHeap};

/// Wraps PriorityQueue and checks diameter of the set
pub struct Buffer {
    queue: BinaryHeap<Reverse<QuotePacket>>, // min heap
    timestamps: AcceptArrivedGap,
}

impl Buffer {
    pub fn new() -> Self {
        Buffer {
            queue: BinaryHeap::new(),
            timestamps: AcceptArrivedGap::new(),
        }
    }

    pub fn insert(&mut self, quote: QuotePacket) {
        self.timestamps.insert(quote.packet_time, quote.accept_time);
        self.queue.push(Reverse(quote));
    }

    pub fn pop(&mut self) -> Option<QuotePacket> {
        let quote = self.queue.pop();
        if let Some(Reverse(ref quote)) = quote {
            self.timestamps.remove(quote.packet_time, quote.accept_time);
        }
        quote.map(|quote| quote.0)
    }

    pub fn diameter(&self) -> Option<Duration> {
        if let (Some(packet_time), Some(accept_time)) =
            (self.timestamps.look_packet(), self.timestamps.look_accept())
        {
            return Some(*packet_time - *accept_time);
        }
        None
    }

    // current_packet_time(max value) - current_accet_time(min_value) > accept_gap(=3 seconds)
    pub fn check_invariant(&self, accept_gap: i64) -> bool {
        self.diameter()
            .map(|diameter| diameter.num_seconds())
            .map(|d| d > accept_gap || d < 0)
            .unwrap_or(false)
    }
}

pub struct AcceptArrivedGap {
    packet: BTreeSet<Reverse<NaiveTime>>,
    accept: BTreeSet<NaiveTime>,
}

impl AcceptArrivedGap {
    pub fn new() -> Self {
        AcceptArrivedGap {
            packet: BTreeSet::new(),
            accept: BTreeSet::new(),
        }
    }

    pub fn insert(&mut self, packet_value: NaiveTime, accept_value: NaiveTime) {
        self.packet.insert(Reverse(packet_value));
        self.accept.insert(accept_value);
    }

    pub fn remove(&mut self, packet_value: NaiveTime, accept_value: NaiveTime) {
        self.packet.remove(&Reverse(packet_value));
        self.accept.remove(&accept_value);
    }

    pub fn look_accept(&self) -> Option<&NaiveTime> {
        self.accept.iter().next()
    }

    pub fn look_packet(&self) -> Option<&NaiveTime> {
        if let Some(Reverse(value)) = self.packet.iter().next() {
            return Some(value);
        }
        None
    }
}
