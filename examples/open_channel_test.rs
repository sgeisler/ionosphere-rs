extern crate ionosphere;

use ionosphere::IonosphereClient;
use std::path::Path;

fn main() {
    let mut client = IonosphereClient::new_blockstream_client(
        Path::new("/home/user/.lightning/lightning-rpc")
    );
    println!("{:?}", client.open_channel(1000000).unwrap());
}