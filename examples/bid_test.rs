extern crate ionosphere;

use ionosphere::IonosphereClient;

fn main() {
    let mut client = IonosphereClient::new_blockstream_client(
        &"/home/user/.lightning/lightning-rpc"
    );

    let data = include_str!("../examples/lipsum.txt");

    println!("reader mode: {:?}", client.place_bid_reader(data.as_bytes(), "lipsum.txt", 100000).unwrap());

    println!("file mode: {:?}", client.place_bid("examples/lipsum.txt", 100000).unwrap());
}