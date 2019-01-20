extern crate reqwest;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate url;

use reqwest::Result as ReqwestResult;
use url::Url;

pub const BLOCKSTREAM_ENDPOINT: &str = "https://satellite.blockstream.com/api/";

pub struct IonosphereClient {
    client: reqwest::Client,
    endpoint: Url
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
    pub fn new(api_endpoint: &str) -> Result<IonosphereClient, url::ParseError> {
        Ok(IonosphereClient {
            client: reqwest::Client::new(),
            endpoint: api_endpoint.parse()?,
        })
    }

    pub fn new_blockstream_client() -> IonosphereClient {
        IonosphereClient::new(BLOCKSTREAM_ENDPOINT)
            .expect("hardcoded URL is valid")
    }

    pub fn lightning_node(&self) -> ReqwestResult<LightningNode> {
        let url = self.endpoint.join("info").expect("should always work if endpoint is valid");
        self.client.get(url).send()?.json()
    }
}

#[cfg(test)]
mod tests {
    use ::IonosphereClient;

    #[test]
    fn test_lightning_node() {
        let client = IonosphereClient::new_blockstream_client();
        client.lightning_node().unwrap();
        assert!(client.lightning_node().is_ok());
    }
}