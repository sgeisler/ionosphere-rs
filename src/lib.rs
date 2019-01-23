extern crate clightningrpc;
extern crate reqwest;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate url;

use clightningrpc::LightningRPC;
use std::io::Read;
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

#[derive(Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct Order {
    pub uuid: String,
    pub auth_token: String,
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

                self.ligthningd.pay(payreq, pay_options)?;

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
}

#[derive(Debug)]
pub enum Error {
    ApiError(reqwest::Error),
    ApiUsageError(String),
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
        let client = IonosphereClient::new_blockstream_client("".parse().unwrap());
        client.lightning_node().unwrap();
        assert!(client.lightning_node().is_ok());
    }
}