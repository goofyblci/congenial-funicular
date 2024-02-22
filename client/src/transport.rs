use anyhow::Result;
use http_body_util::{BodyExt, Empty};
use hyper::body::Bytes;
use hyper::http::uri::Scheme;
use hyper::{Request, Uri};
use hyper_util::rt::TokioIo;
use ipgeolocate::{Locator, Service};
use tokio::sync::mpsc::Sender;
use tokio::time::{self};

use arti_client::{DataStream, TorClient, TorClientConfig};

use crate::app::{ChannelTypes, CircuitInfo};

#[derive(Debug)]
pub struct OnionConnection {
    pub host: String,
    pub port: u16,
    pub is_https: bool,
    pub sender: Sender<ChannelTypes>,
}

impl OnionConnection {
    pub async fn new(with_host: &str, sender: Sender<ChannelTypes>) -> Self {
        let url: Uri = with_host.parse().expect("URL can not be parsed");
        let host = url.host().unwrap();
        let https = url.scheme() == Some(&Scheme::HTTPS);
        let port = match url.port_u16() {
            Some(port) => port,
            _ if https => 443,
            _ => 80,
        };
        Self {
            host: host.to_owned(),
            port,
            is_https: https,
            sender,
        }
    }

    pub async fn get_circuit_information(&self, stream: DataStream) -> DataStream {
        let circuit = stream.circuit().path_ref();
        let service = Service::IpApi;
        let mut circuit_infos: Vec<CircuitInfo> = Vec::new();
        for path_entry in circuit.iter() {
            let path_entry_string = path_entry.to_string();
            if !path_entry_string.contains('>') {
                let path_vec: Vec<&str> = path_entry_string.split_whitespace().collect();
                for path_element in path_vec {
                    if path_element.contains('.') {
                        let ip_addr_with_port: Vec<&str> = path_element.split(':').collect();
                        match Locator::get(&ip_addr_with_port[0].replace('[', ""), service).await {
                            Ok(ip) => {
                                circuit_infos.push(CircuitInfo {
                                    ip_address: ip.ip,
                                    city: ip.city,
                                    country: ip.country,
                                });
                            }
                            Err(_error) => {
                                circuit_infos.push(CircuitInfo {
                                    ip_address: "x.x.x.x".to_owned(),
                                    city: "UNKNOWN".to_owned(),
                                    country: "UNKNOWN".to_owned(),
                                });
                            }
                        };
                    }
                }
            } else {
                circuit_infos.push(CircuitInfo {
                    ip_address: "x.x.x.x".to_owned(),
                    city: "UNKNOWN".to_owned(),
                    country: "UNKNOWN".to_owned(),
                });
            }
        }
        self.sender
            .send(ChannelTypes::CircuitInformation(circuit_infos))
            .await
            .unwrap();
        stream
    }

    pub async fn make_request(&self) -> Result<()> {
        let mut config = TorClientConfig::builder();
        config
            .address_filter()
            .allow_onion_addrs(true)
            .build()
            .unwrap();
        // let stream_prefs = StreamPrefs::new();
        let final_config = config.build().unwrap();
        let client = TorClient::create_bootstrapped(final_config)
            .await
            .expect("tor client creation");
        let mut stream = client
            .connect((self.host.clone(), self.port))
            .await
            .unwrap();
        stream = self.get_circuit_information(stream).await;
        let (mut request_sender, connection) =
            hyper::client::conn::http1::handshake(TokioIo::new(stream)).await?;

        // spawn a task to poll the connection and drive the HTTP state
        tokio::spawn(async move {
            connection.await.unwrap();
        });

        let mut resp = request_sender
            .send_request(
                Request::builder()
                    .header("Host", self.host.clone())
                    .method("GET")
                    .body(Empty::<Bytes>::new())?,
            )
            .await?;

        println!("status: {}", resp.status());

        let now = time::Instant::now();

        while let Some(frame) = resp.body_mut().frame().await {
            let bytes = frame?.into_data().unwrap();
            println!("body: {}", std::str::from_utf8(&bytes)?);
            let vec_bytes = bytes.to_vec();
            if now.elapsed().as_secs() > 10 {
                break;
            }
        }

        Ok(())
    }

    pub async fn make_websocket_connection(&self) -> Result<()> {
        tokio_tungstenite::connect_async(&self.host)
            .await
            .expect("CONNECT_TO_WS");
        Ok(())
    }
}
