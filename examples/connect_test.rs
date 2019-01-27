extern crate ionosphere;

use ionosphere::IonosphereClient;

fn main() {
    let mut client = IonosphereClient::new_blockstream_client(
        &"/home/user/.lightning/lightning-rpc"
    );
    client.connect().unwrap();
}