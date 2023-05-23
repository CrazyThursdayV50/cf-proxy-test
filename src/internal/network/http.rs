extern crate hyper;
extern crate reqwest;

use super::api::ConnectTestStats;
use super::api::DownloadTestStats;
use super::api::ProxyTest;
use super::api::ServerAddress;
use super::api::Speed;
use async_trait::async_trait;
use reqwest::redirect::Policy;
use reqwest::Method;
use reqwest::StatusCode;
use reqwest::Url;
use std::error::Error;
use std::io::ErrorKind;
use std::net::SocketAddr;
use std::str::FromStr;
use std::time::Duration;
use std::time::SystemTime;
use tokio::runtime::Runtime;

pub struct HttpClient {
    vias: Vec<ServerAddress>,
    remote: ServerAddress,
    timeout: Duration,
}

impl HttpClient {
    pub fn build(url: Url, addrs: Vec<SocketAddr>, timeout: Duration) -> Self {
        let mut vias = Vec::new();
        for addr in addrs {
            vias.push(ServerAddress::Socket(addr))
        }

        let remote = ServerAddress::URL(url);
        Self {
            vias,
            remote,
            timeout,
        }
    }
}

#[async_trait]
impl ProxyTest for HttpClient {
    async fn connect(
        &self,
        remote: ServerAddress,
        via: Option<ServerAddress>,
        duration: Duration,
    ) -> Result<ConnectTestStats, Box<dyn Error>> {
        let proxy_host = match remote {
            ServerAddress::Socket(socket) => socket,
            ServerAddress::URL(url) => {
                return Err(Box::new(std::io::Error::new(
                    ErrorKind::Other,
                    format!("invalid via address {url}"),
                )))
            }
        };

        let mut client_builder = reqwest::Client::builder()
            .connect_timeout(duration)
            .timeout(duration)
            .connection_verbose(true)
            .user_agent("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_12_6) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/98.0.4758.80 Safari/537.36");

        let remote = match via {
            Some(ServerAddress::URL(url)) => url,
            Some(ServerAddress::Socket(socket)) => {
                Url::from_str(format!("https://{}:{}", socket.ip(), socket.port()).as_str())
                    .unwrap()
            }
            None => {
                return Err(Box::new(std::io::Error::new(
                    ErrorKind::Other,
                    format!("remote address not found"),
                )))
            }
        };

        if !proxy_host.ip().is_loopback() {
            client_builder =
                client_builder.resolve(remote.clone().host_str().unwrap(), proxy_host.clone());
        }

        let client = client_builder.build().unwrap();
        let now = SystemTime::now();
        let req = client.request(Method::HEAD, remote.clone());
        let result = req.send().await;
        let cost = now.elapsed().unwrap();

        match result {
            Ok(resp) => {
                // println!(
                //     "connect to {} via {} repsonse: {:?} cost {:?}",
                //     self.url, proxy_host, resp, cost
                // );

                match resp.status() {
                    StatusCode::OK | StatusCode::MOVED_PERMANENTLY | StatusCode::FOUND => {
                        println!("connect via {:?} response: {:?}", proxy_host, resp);
                        Ok(ConnectTestStats::new(proxy_host.ip(), cost))
                    }
                    _ => Err(Box::new(std::io::Error::new(
                        std::io::ErrorKind::Other,
                        format!("status {}", resp.status()),
                    ))),
                }
            }
            Err(e) => {
                println!("connect to {} via {} failed: {}", remote, proxy_host, e);
                Err(Box::new(e))
            }
        }
    }

    fn get_address_conn(&self) -> Vec<ServerAddress> {
        self.vias.clone()
    }

    fn get_address_remote(&self) -> Option<ServerAddress> {
        Some(self.remote.clone())
    }

    fn get_timeout(&self) -> Duration {
        self.timeout
    }

    fn download_test(&self, ips: Vec<ServerAddress>) -> Vec<DownloadTestStats> {
        let duration = self.get_timeout();
        let mut stats = Vec::new();
        let remote = match self.get_address_remote() {
            Some(ServerAddress::URL(url)) => url,
            Some(ServerAddress::Socket(socket)) => {
                Url::from_str(format!("https://{}:{}", socket.ip(), socket.port()).as_str())
                    .unwrap()
            }
            None => return stats,
        };

        let future = async {
            for via in ips {
                println!("download from {} via {:?}", remote.as_str(), via);
                if stats.len() > 9 {
                    break;
                }
                let proxy_host = match via {
                    ServerAddress::Socket(socket) => socket,

                    ServerAddress::URL(url) => {
                        println!("invalid via address {url}");
                        continue;
                    }
                };

                let mut client_builder = reqwest::Client::builder()
                    .connect_timeout(duration)
                    .timeout(duration)
                    .user_agent("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_12_6) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/98.0.4758.80 Safari/537.36")
                .redirect(Policy::custom(|attempt| {
                    if attempt.previous().len() > 10 {
                        return attempt.error("too many redirects");
                    }
                    attempt.follow()
                }));

                if !proxy_host.ip().is_loopback() {
                    client_builder = client_builder
                        .resolve(remote.clone().host_str().unwrap(), proxy_host.clone());
                }

                let client = client_builder.build().unwrap();
                let req = client.request(Method::GET, remote.clone());
                let result = req.send().await;
                if let Ok(mut resp) = result {
                    if resp.status() == StatusCode::OK {
                        let now = SystemTime::now();

                        let mut total_data = 0;
                        let mut total_cost = Duration::ZERO;
                        loop {
                            // println!("{} download loop", proxy_host);
                            if total_cost >= duration {
                                break;
                            }

                            let result = resp.chunk().await.unwrap();
                            total_cost = now.elapsed().unwrap();

                            match result {
                                None => break,
                                Some(data) => {
                                    total_data += data.len();
                                }
                            };
                        }

                        stats.push(DownloadTestStats::new(
                            resp.remote_addr().unwrap().ip(),
                            Speed::byte_per_second(total_data, total_cost).mb(),
                        ));
                    };
                }
            }
        };

        Runtime::new().unwrap().block_on(async { future.await });
        stats
    }
}
