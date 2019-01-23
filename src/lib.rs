extern crate clightningrpc;
extern crate reqwest;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate url;

use clightningrpc::LightningRPC;
use std::path::Path;
use url::Url;

pub const BLOCKSTREAM_ENDPOINT: &str = "https://satellite.blockstream.com/api/";

pub struct IonosphereClient {
    client: reqwest::Client,
    endpoint: Url,
    ligthningd: LightningRPC,
}

#[derive(Debug, Eq, PartialEq, Hash, Ord, PartialOrd, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum BitcoinNetwork {
    Mainnet,
    Testnet,
    Regtest,
}

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
#[derive(Debug, Eq, PartialEq, Hash, Ord, PartialOrd, Deserialize)]
pub struct NodeAddress {
    #[serde(rename = "type")]
    pub addr_type: String,
    pub address: String,
    pub port: u16
}

pub struct NewOrder {
    pub bid: u64,
}

impl IonosphereClient {
    pub fn new(api_endpoint: Url, lightning_rpc: &Path) -> IonosphereClient {
        IonosphereClient {
            client: reqwest::Client::new(),
            endpoint: api_endpoint,
            ligthningd: LightningRPC::new(lightning_rpc),
        }
    }

    pub fn new_blockstream_client(lightning_rpc: &Path) -> IonosphereClient {
        IonosphereClient::new(
            BLOCKSTREAM_ENDPOINT.parse().expect("hardcoded URL is valid"),
            lightning_rpc
        )
    }

    pub fn lightning_node(&self) -> Result<LightningNode, reqwest::Error> {
        let url = self.endpoint.join("info").expect("should always work if endpoint is valid");
        self.client.get(url).send()?.json()
    }

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

    pub fn open_channel(&mut self, amount_sat: u32) -> Result<clightningrpc::responses::FundChannel, Error> {
        let clightningrpc::responses::Connect {id} = self.connect()?;
        Ok(self.ligthningd.fundchannel(
            id,
            amount_sat as i64,
            None
        )?)
    }
}

#[derive(Debug)]
pub enum Error {
    ApiError(reqwest::Error),
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

#[cfg(test)]
mod tests {
    use ::IonosphereClient;

    #[test]
    fn test_lightning_node() {
        let client = IonosphereClient::new_blockstream_client("".parse().unwrap());
        client.lightning_node().unwrap();
        assert!(client.lightning_node().is_ok());
    }
}