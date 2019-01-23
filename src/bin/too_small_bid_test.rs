extern crate ionosphere;

use ionosphere::IonosphereClient;
use std::path::Path;

fn main() {
    let mut client = IonosphereClient::new_blockstream_client(
        Path::new("/home/user/.lightning/lightning-rpc")
    );

    match client.place_bid("src/bin/lipsum.txt", 1000) {
        Ok(_) => unreachable!(),
        Err(e) => {
            println!("{:?}", e);
        }
    }
}