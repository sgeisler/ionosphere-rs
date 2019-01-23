extern crate ionosphere;

use ionosphere::IonosphereClient;
use std::path::Path;

fn main() {
    let mut client = IonosphereClient::new_blockstream_client(
        Path::new("/home/user/.lightning/lightning-rpc")
    );

    let data = include_str!("lipsum.txt");

    println!("reader mode: {:?}", client.place_bid_reader(data.as_bytes(), "lipsum.txt", 100000).unwrap());

    println!("file mode: {:?}", client.place_bid("src/bin/lipsum.txt", 100000).unwrap());
}