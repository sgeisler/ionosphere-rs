//! This library can send data over [Blockstream satellite](https://blockstream.com/satellite-api/)
//! compatible APIs. By default it will connect to the API endpoint run by Blockstream, but users
//! can choose other providers should they exist at some point.
//!
//! To use the API you need a running [c-lightning](https://github.com/ElementsProject/lightning/)
//! instance on the same machine. It has to be on the same bitcoin network as the API endpoint.
//! The API network version (currently testnet for Blockstream) can be queried using
//! `IonosphereClient::lightning_node(&self)`.
//!
//! Usage example:
//!
//! ```
//! let mut client = IonosphereClient::new_blockstream_client(
//!     &"/home/user/.lightning/lightning-rpc"
//! );
//!
//! // Open direct lightning channel to API node
//! client.open_channel(1000000).unwrap();
//!
//! // Place bid and pay for it
//! client.place_bid("src/bin/lipsum.txt", 100000).unwrap();
//! ```
//!
//! **Don't use this library in production yet, it's hacky and incomplete! PRs welcome :)**


extern crate clightningrpc;
extern crate lightning_invoice as bolt11;
extern crate reqwest;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate url;

use clightningrpc::LightningRPC;
use std::io::Read;
use std::path::Path;
use url::Url;

/// Blockstream's satellite API endpoint
pub const BLOCKSTREAM_ENDPOINT: &str = "https://satellite.blockstream.com/api/";

pub struct IonosphereClient {
    client: reqwest::Client,
    endpoint: Url,
    ligthningd: LightningRPC,
}

/// Bitcoin network enum
#[derive(Debug, Eq, PartialEq, Hash, Ord, PartialOrd, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum BitcoinNetwork {
    Mainnet,
    Testnet,
    Regtest,
}

/// Descriptor for the API endpoint's lightning node
#[derive(Debug, Eq, PartialEq, Hash, Ord, PartialOrd, Deserialize)]
pub struct LightningNode {
    pub id: String,
    pub address: Vec<NodeAddress>,
    pub version: String,
    #[serde(rename = "blockheight")]
    pub block_height: u64,
    pub network: BitcoinNetwork
}

// TODO: make this an enum
/// One address of the API endpoint's lightning node
#[derive(Debug, Eq, PartialEq, Hash, Ord, PartialOrd, Deserialize)]
pub struct NodeAddress {
    #[serde(rename = "type")]
    /// ipv4/ipv6/TOR
    pub addr_type: String,
    /// IP/hidden service
    pub address: String,
    pub port: u16
}

#[derive(Debug, Eq, PartialEq, Hash, Ord, PartialOrd, Deserialize)]
#[serde(untagged)]
enum OrderResponse {
    Success {
        auth_token: String,
        uuid: String,
        lightning_invoice: Invoice,
    },
    Error {
        message: String,
        errors: Vec<String>
    }
}

#[derive(Debug, Eq, PartialEq, Hash, Ord, PartialOrd, Deserialize)]
struct Invoice {
    pub payreq: String,
}

/// Bid handle needed to manipulate bids after placing them
#[derive(Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct Order {
    pub uuid: String,
    pub auth_token: String,
}

impl IonosphereClient {
    /// Creates a new API client for an arbitrary endpoint
    pub fn new<P: AsRef<Path>>(api_endpoint: Url, lightning_rpc: P) -> IonosphereClient {
        IonosphereClient {
            client: reqwest::Client::new(),
            endpoint: api_endpoint,
            ligthningd: LightningRPC::new(lightning_rpc.as_ref()),
        }
    }

    /// Creates a new API client for the Blockstream API endpoint
    pub fn new_blockstream_client<P: AsRef<Path>>(lightning_rpc: P) -> IonosphereClient {
        IonosphereClient::new(
            BLOCKSTREAM_ENDPOINT.parse().expect("hardcoded URL is valid"),
            lightning_rpc
        )
    }

    /// Fetches the API's lightning node info
    pub fn lightning_node(&self) -> Result<LightningNode, reqwest::Error> {
        let url = self.endpoint.join("info").expect("should always work if endpoint is valid");
        self.client.get(url).send()?.json()
    }

    /// Connects our lightningd to the API's lightning node
    pub fn connect(&mut self) -> Result<clightningrpc::responses::Connect, Error> {
        let target = self.lightning_node()?;
        let addr = target.address.first().map(|addr|
            format!("{}:{}", addr.address, addr.port)
        );

        Ok(self.ligthningd.connect(
            target.id,
            addr
        )?)
    }

    /// Opens a direct lightning channel to the API's lightning node
    pub fn open_channel(&mut self, amount_sat: u32) -> Result<clightningrpc::responses::FundChannel, Error> {
        let clightningrpc::responses::Connect {id} = self.connect()?;
        Ok(self.ligthningd.fundchannel(
            id,
            amount_sat as i64,
            None
        )?)
    }

    /// Places a bid for an uploaded file
    pub fn place_bid<P: AsRef<Path>>(
        &mut self,
        file_path: P,
        bid_msat: u64
    ) -> Result<Order, Error> {
        let file_name = match file_path.as_ref().file_name().and_then(|name| name.to_str()) {
            Some(name) => name.to_owned(),
            None => return Err(Error::FileNameError),
        };
        let file = std::fs::File::open(file_path)?;

        self.place_bid_reader(
            file,
            &file_name,
            bid_msat
        )
    }

    /// Places a bid for arbitrary data supplied by a reader. If the payment fails we try to delete
    /// the bid.
    pub fn place_bid_reader<T: Read + Send + 'static>(
        &mut self,
        data: T,
        file_name: &str,
        bid_msat: u64
    ) -> Result<Order, Error> {
        let url = self.endpoint.join("order")
            .expect("should always work if endpoint is valid");

        let file = reqwest::multipart::Part::reader(data)
            .file_name(file_name.to_owned());

        let form = reqwest::multipart::Form::new()
            .text("bid", bid_msat.to_string())
            .part("file", file);

        let response: OrderResponse = self.client.post(url)
            .multipart(form)
            .send()?
            .json()?;

        match response {
            OrderResponse::Success {
                auth_token,
                uuid,
                lightning_invoice,
            } => {
                let Invoice { payreq } = lightning_invoice;

                let pay_options = clightningrpc::lightningrpc::PayOptions {
                    msatoshi: None,
                    description: None,
                    riskfactor: None,
                    maxfeepercent: None,
                    exemptfee: None,
                    retry_for: None,
                    maxdelay: None
                };

                let invoice: bolt11::Invoice = match payreq.parse() {
                    Ok(x) => x,
                    Err(_) => return Err(Error::ApiResponseError),
                };

                if invoice.amount_pico_btc() != Some(bid_msat * 10) {
                    return Err(Error::ApiResponseError);
                }

                match self.ligthningd.pay(payreq, pay_options) {
                    Ok(_) => {},
                    Err(e) => {
                        self.delete_bid(&Order {
                            uuid,
                            auth_token,
                        })?;
                        return Err(e.into());
                    }
                }

                Ok(Order {
                    uuid,
                    auth_token,
                })
            },
            OrderResponse::Error {
                message,
                errors,
            } => {
                Err(Error::ApiUsageError(format!("{} ({:?})", message, errors)))
            },
        }
    }

    /// Deletes a previously placed bid
    ///
    /// # Caution
    /// Currently you will loose the funds you already paid for the bid.
    pub fn delete_bid(&self, order: &Order) -> Result<(), Error> {
        let url = self.endpoint.join("order/")
            .expect("should always work if endpoint is valid")
            .join(&order.uuid)
            .expect("should always work if endpoint is valid");

        // TODO: check HTTP status code for all functions
        self.client.delete(url)
            .header("X-Auth-Token", order.auth_token.clone())
            .send()?;

        Ok(())
    }
}

#[derive(Debug)]
pub enum Error {
    ApiError(reqwest::Error),
    ApiUsageError(String),
    ApiResponseError,
    FileNameError,
    FileOpenError(std::io::Error),
    LightningError(clightningrpc::Error),
}

impl From<clightningrpc::Error> for Error {
    fn from(e: clightningrpc::Error) -> Self {
        Error::LightningError(e)
    }
}

impl From<reqwest::Error> for Error {
    fn from(e: reqwest::Error) -> Self {
        Error::ApiError(e)
    }
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Error::FileOpenError(e)
    }
}

#[cfg(test)]
mod tests {
    use ::IonosphereClient;

    #[test]
    fn test_lightning_node() {
        let client = IonosphereClient::new_blockstream_client(&"");
        client.lightning_node().unwrap();
        assert!(client.lightning_node().is_ok());
    }
}