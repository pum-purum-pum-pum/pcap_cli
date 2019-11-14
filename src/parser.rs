use chrono::NaiveTime;
use nom::{
    bytes::complete::{tag, take},
    combinator::map_res,
    count, named,
    sequence::tuple,
    IResult,
};
use std::str;

use crate::quote::{Ask, Asks, Bid, Bids, QuotePacket};

const BIDS_NUM: usize = 5;
const ASKS_NUM: usize = 5;

const ISSUE_CODE_SIZE: usize = 12;
const ADDITIONAL_INFO1_SIZE: usize = 5;
const PRICE_SIZE: usize = 5;
const VOLUME_SIZE: usize = 7;
const TOTAL_VOLUME_SIZE: usize = 7;
const TOTAL_NUM_BEST_QUOTE__SIZE: usize = 5;
const NUM_BEST_QUOTE_SIZE: usize = 4;
const TIME_CHUNK_SIZE: usize = 2;

// searching for B6034 quote
fn type_parser(input: &[u8]) -> IResult<&[u8], &str> {
    map_res(tag("B6034"), str::from_utf8)(input)
}

fn issue_parser(input: &[u8]) -> IResult<&[u8], &str> {
    map_res(take(ISSUE_CODE_SIZE), str::from_utf8)(input)
}

fn parse_u32(input: &[u8], length: usize) -> IResult<&[u8], u32> {
    map_res(map_res(take(length), str::from_utf8), str::parse::<u32>)(input)
}

fn price_parser(input: &[u8]) -> IResult<&[u8], u32> {
    parse_u32(input, PRICE_SIZE)
}

fn volume_parser(input: &[u8]) -> IResult<&[u8], u32> {
    parse_u32(input, VOLUME_SIZE)
}

/// parse only one bid
fn bid_parser(input: &[u8]) -> IResult<&[u8], Bid> {
    let (input, (bp, bv)) = tuple((price_parser, volume_parser))(input)?;
    Ok((
        input,
        Bid {
            price: bp,
            volume: bv,
        },
    ))
}

/// parse only one ask
fn ask_parser(input: &[u8]) -> IResult<&[u8], Ask> {
    let (input, (bp, bv)) = tuple((price_parser, volume_parser))(input)?;
    Ok((
        input,
        Ask {
            price: bp,
            volume: bv,
        },
    ))
}

fn number_of_best_parser(input: &[u8]) -> IResult<&[u8], u32> {
    parse_u32(input, NUM_BEST_QUOTE_SIZE)
}

// Parsers: sequence of bids/asks
named!(collect_bids<Vec<Bid>>, count!(bid_parser, BIDS_NUM));
named!(collect_asks<Vec<Ask>>, count!(ask_parser, BIDS_NUM));

// Parsers: sequence number of best ask/bid
named!(
    collect_num_bids<Vec<u32>>,
    count!(number_of_best_parser, BIDS_NUM)
);
named!(
    collect_num_asks<Vec<u32>>,
    count!(number_of_best_parser, ASKS_NUM)
);

fn parse_time_chunk(input: &[u8]) -> IResult<&[u8], u32> {
    map_res(
        map_res(take(TIME_CHUNK_SIZE), str::from_utf8),
        str::parse::<u32>,
    )(input)
}

fn time_parser(input: &[u8]) -> IResult<&[u8], NaiveTime> {
    let (input, (hours, minutes, seconds, milliseconds)) = tuple((
        parse_time_chunk,
        parse_time_chunk,
        parse_time_chunk,
        parse_time_chunk,
    ))(input)?;
    let time = NaiveTime::from_hms_milli(hours, minutes, seconds, milliseconds);
    Ok((input, time))
}

/// Parse quote from bytes
pub fn content_parser(input: &[u8], packet_time: NaiveTime) -> IResult<&[u8], QuotePacket> {
    let (
        input,
        (
            _head,
            issue,
            _,
            _total_bid_volume,
            bids,
            _total_ask_volume,
            asks,
            _total_bq,
            _num_bids,
            _total_aq,
            _num_asks,
            accept_time,
        ),
    ) = tuple((
        type_parser,
        issue_parser,
        take(ADDITIONAL_INFO1_SIZE),
        take(TOTAL_VOLUME_SIZE),
        collect_bids,
        take(TOTAL_VOLUME_SIZE),
        collect_asks,
        take(TOTAL_NUM_BEST_QUOTE__SIZE),
        collect_num_bids,
        take(TOTAL_NUM_BEST_QUOTE__SIZE),
        collect_num_asks,
        time_parser,
    ))(input)?;
    Ok((
        input,
        QuotePacket {
            issue_code: issue.to_string(),
            bids: Bids(bids),
            asks: Asks(asks),
            accept_time,
            packet_time,
        },
    ))
}
