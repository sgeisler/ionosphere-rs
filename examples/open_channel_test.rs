extern crate ionosphere;

use ionosphere::IonosphereClient;

fn main() {
    let mut client = IonosphereClient::new_blockstream_client(
        &"/home/user/.lightning/lightning-rpc"
    );
    println!("{:?}", client.open_channel(1000000).unwrap());
}