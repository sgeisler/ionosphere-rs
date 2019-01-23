# Ionosphere

**Don't use this library in production yet, it's hacky and incomplete! PRs are always welcome :)**

This library can send data over [Blockstream satellite](https://blockstream.com/satellite-api/)
compatible APIs. By default it will connect to the API endpoint run by Blockstream, but users
can choose other providers should they exist at some point.

To use the API you need a running [c-lightning](https://github.com/ElementsProject/lightning/)
instance on the same machine. It has to be on the same bitcoin network as the API endpoint.
The API network version (currently testnet for Blockstream) can be queried using
`IonosphereClient::lightning_node(&self)`.

## Usage example

If you have just funded a new testnet c-lightning node you could run the following code to send some lorem ipsum to
earth:
```rust
let mut client = IonosphereClient::new_blockstream_client(
    Path::new("/home/user/.lightning/lightning-rpc")
);
// Open direct lightning channel to API node
client.open_channel(1000000).unwrap();
// Place bid and pay for it
client.place_bid("src/bin/lipsum.txt", 100000).unwrap();
```

You can find more usage examples in `src/bin`.

## Future development goals
* Stronger typed API, many fields that could be their own types are currently `String`s
* Better/complete error handling
* Automatic integration tests
* Support fetching and updating bids