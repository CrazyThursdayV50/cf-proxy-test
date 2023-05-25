extern crate futures;
extern crate tokio;

use crate::internal::client::conn::{ConnTest, ConnectTestStats};
use crate::internal::client::def::ServerAddress;

use async_trait::async_trait;
use std::error::Error;
use std::{net::SocketAddr, time::Duration};
use tokio::{net::TcpStream, time};

pub struct TcpClient {
    addrs: Vec<ServerAddress>,
    timeout: Duration,
    top: usize,
}

impl TcpClient {
    pub fn build(src: Vec<SocketAddr>, timeout: Duration, top: usize) -> Self {
        let mut addrs = Vec::new();
        for addr in src {
            addrs.push(ServerAddress::Socket(addr));
        }
        Self {
            addrs,
            timeout,
            top,
        }
    }
}

#[async_trait]
impl ConnTest for TcpClient {
    async fn connect(
        &self,
        dst: ServerAddress,
        _: Option<ServerAddress>,
        timeout: Duration,
    ) -> Result<ConnectTestStats, Box<dyn Error>> {
        let socket_addr = match dst {
            ServerAddress::Socket(socket) => socket,
            ServerAddress::URL(url) => {
                return Err(Box::new(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!("invalid dst {url}"),
                )))
            }
        };

        let now = std::time::SystemTime::now();
        let conn_result = time::timeout(timeout, TcpStream::connect(socket_addr.clone())).await;
        let cost = now.elapsed().unwrap();

        match conn_result {
            Ok(result) => match result {
                Ok(stream) => {
                    drop(stream);
                    Ok(ConnectTestStats::new(socket_addr.ip(), cost))
                }

                Err(e) => Err(Box::new(e)),
            },

            Err(e) => Err(Box::new(e)),
        }
    }

    fn get_address_conn(&self) -> Vec<ServerAddress> {
        self.addrs.clone()
    }

    fn get_address_remote(&self) -> Option<ServerAddress> {
        None
    }

    fn get_timeout(&self) -> Duration {
        self.timeout
    }

    fn get_top(&self) -> usize {
        self.top
    }
}
