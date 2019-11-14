extern crate nom;
use clap::{App, Arg};
use quote::QuotesIterator;

// parser from &[u8] to QuotePacket
mod parser;
// QuotePacket definition and boilerplate
mod quote;
#[cfg(test)]
// integration test on provided test data
mod tests;
// data structures for processing data with time(3 seconds) gap
mod buffer;

fn main() {
    let matches = App::new("PCAP printer")
        .version("1.0")
        .author("Vlad Zhukov")
        .about("Parse quote packets")
        .arg(
            Arg::with_name("INPUT")
                .required(true)
                .help("Path to pcap file")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("r")
                .short("r")
                .help("Reorder by quote accept time"),
        )
        .arg(
            Arg::with_name("debug")
                .short("d")
                .help("Run step by step by pressing Enter key"),
        )
        .get_matches();
    let input_file = matches.value_of("INPUT").unwrap();
    let reorder = matches.is_present("r");
    let step_by_step = matches.is_present("debug");
    let quotes_iterator = QuotesIterator::new(input_file, reorder, step_by_step);
    for quote in quotes_iterator.expect("failed to create iterator") {
        println!("{}", quote);
    }
}
