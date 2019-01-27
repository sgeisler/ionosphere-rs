extern crate ionosphere;

use ionosphere::IonosphereClient;

fn main() {
    let mut client = IonosphereClient::new_blockstream_client(
        "/home/user/.lightning/lightning-rpc"
    );

    match client.place_bid("examples/lipsum.txt", 1000) {
        Ok(_) => unreachable!(),
        Err(e) => {
            println!("{:?}", e);
        }
    }
}