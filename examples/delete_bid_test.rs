extern crate ionosphere;

use ionosphere::IonosphereClient;

fn main() {
    let mut client = IonosphereClient::new_blockstream_client(
        &"/home/user/.lightning/lightning-rpc"
    );

    let order = client.place_bid("examples/lipsum.txt", 100000).unwrap();
    client.delete_bid(&order).unwrap();
}