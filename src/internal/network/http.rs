extern crate hyper;
extern crate reqwest;

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

use crate::internal::client::conn::ConnTest;
use crate::internal::client::conn::ConnectTestStats;
use crate::internal::client::def::ServerAddress;
use crate::internal::client::download::DownloadTest;
use crate::internal::client::download::DownloadTestResult;
use crate::internal::client::download::DownloadTestStats;
use crate::internal::client::download::Speed;

pub struct HttpClient {
    vias: Vec<ServerAddress>,
    remote: ServerAddress,
    timeout: Duration,
    top: usize,
}

impl HttpClient {
    pub fn build(url: Url, addrs: Vec<SocketAddr>, timeout: Duration, top: usize) -> Self {
        let mut vias = Vec::new();
        for addr in addrs {
            vias.push(ServerAddress::Socket(addr))
        }

        let remote = ServerAddress::URL(url);
        Self {
            vias,
            remote,
            timeout,
            top,
        }
    }
}

#[async_trait]
impl ConnTest for HttpClient {
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
            Ok(resp) => match resp.status() {
                StatusCode::OK | StatusCode::MOVED_PERMANENTLY | StatusCode::FOUND => {
                    Ok(ConnectTestStats::new(proxy_host.ip(), cost))
                }
                _ => Err(Box::new(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!("status {}", resp.status()),
                ))),
            },
            Err(e) => Err(Box::new(e)),
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

    fn get_top(&self) -> usize {
        self.top
    }
}

impl DownloadTest for HttpClient {
    fn download_test(&self) -> DownloadTestResult {
        let duration = self.get_timeout();
        let ips = &self.vias;

        let remote = match self.get_address_remote() {
            Some(ServerAddress::URL(url)) => url,
            Some(ServerAddress::Socket(socket)) => {
                Url::from_str(format!("https://{}:{}", socket.ip(), socket.port()).as_str())
                    .unwrap()
            }
            None => {
                return DownloadTestResult {
                    top: self.top,
                    list: None,
                }
            }
        };

        println!(
            "开始测试下载速度。程序会测试直到有 {} 条有效的下载数据为止，请耐心等待。",
            self.top
        );
        let mut stats = Vec::new();

        let rt = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap();

        for via in ips {
            print!(
                "正在测试从 {:?} 到 {} 的下载速度 ... ",
                via,
                remote.to_string()
            );
            let proxy_host = match via {
                ServerAddress::Socket(socket) => socket,

                ServerAddress::URL(url) => {
                    println!("====> 无效(url: {})", url.to_string());
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
                client_builder =
                    client_builder.resolve(remote.clone().host_str().unwrap(), proxy_host.clone());
            }

            let client = client_builder.build().unwrap();
            let start_conn = SystemTime::now();

            let result = rt.block_on(async {
                let req = client.request(Method::GET, remote.clone());
                req.send().await
            });

            let conn_cost = start_conn.elapsed().unwrap();
            if let Ok(mut resp) = result {
                if resp.status() == StatusCode::OK {
                    let now = SystemTime::now();

                    let mut total_data = 0;
                    let mut total_cost = Duration::ZERO;
                    loop {
                        if total_cost >= duration {
                            break;
                        }

                        let result = rt.block_on(async { resp.chunk().await.unwrap() });
                        total_cost = now.elapsed().unwrap();

                        match result {
                            None => break,
                            Some(data) => {
                                total_data += data.len();
                            }
                        };
                    }

                    let mut speed = Speed::byte_per_second(total_data, total_cost);
                    speed.mb();
                    stats.push(DownloadTestStats::new(
                        resp.remote_addr().unwrap().ip(),
                        speed,
                    ));

                    println!(
                        "===> 有效({}:{:?}, {:?})",
                        stats.len(),
                        conn_cost,
                        total_cost
                    );
                    if stats.len() >= self.top {
                        println!("测试下载速度结束。");
                        break;
                    }

                    continue;
                }

                println!("===> 无效({:?})", conn_cost);
                continue;
            }
            println!("===> 无效({:?})", conn_cost);
        }

        stats.sort_by(|x, y| x.speed.partial_cmp(&y.speed).unwrap());
        stats.reverse();
        if stats.len() == 0 {
            return DownloadTestResult {
                top: self.top,
                list: None,
            };
        }

        DownloadTestResult {
            top: self.top,
            list: Some(stats),
        }
    }
}
