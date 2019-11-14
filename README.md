* You can start by reading main.rs and doc.
* `parser.rs` contains parser written with nom crate.
* `quote.rs` contains boilerplate and definition for quote
* `buffer.rs` contains data structure that helps to do sorting data "online"

CLI parser for .pcap files with specific format (see `parser.rs` file for details)
```
cargo run --release data/mdf-kospi200.20110216-0.pcap 
```