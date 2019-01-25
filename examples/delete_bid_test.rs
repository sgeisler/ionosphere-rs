extern crate ionosphere;

use ionosphere::IonosphereClient;
use std::path::Path;

fn main() {
    let mut client = IonosphereClient::new_blockstream_client(
        Path::new("/home/user/.lightning/lightning-rpc")
    );

    let order = client.place_bid("examples/lipsum.txt", 100000).unwrap();
    client.delete_bid(&order).unwrap();
}