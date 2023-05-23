extern crate async_trait;

use async_trait::async_trait;
use futures::future::join_all;
use std::{
    error::Error,
    fmt::Display,
    net::{IpAddr, SocketAddr},
    time::Duration,
};
use tokio::runtime::Runtime;
use url::Url;

const SPEED_MULTIPLE: usize = 1 << 10;

#[derive(Clone, Debug)]
pub enum Speed {
    Byte(f64),
    KByte(f64),
    MByte(f64),
    GByte(f64),
}

fn scale_up(x: f64) -> f64 {
    x * SPEED_MULTIPLE as f64
}

fn scale_down(x: f64) -> f64 {
    x / SPEED_MULTIPLE as f64
}

impl Speed {
    pub fn byte_per_second(byte: usize, cost: Duration) -> Self {
        Speed::Byte(byte as f64 / cost.as_secs_f64())
    }

    fn upper(self) -> Self {
        match self {
            Speed::Byte(s) => Speed::KByte(scale_down(s)),
            Speed::KByte(s) => Speed::MByte(scale_down(s)),
            Speed::MByte(s) => Speed::GByte(scale_down(s)),
            Speed::GByte(_) => self.clone(),
        }
    }

    fn lower(self) -> Self {
        match self {
            Speed::Byte(_) => self.clone(),
            Speed::KByte(s) => Speed::Byte(scale_up(s)),
            Speed::MByte(s) => Speed::KByte(scale_up(s)),
            Speed::GByte(s) => Speed::MByte(scale_up(s)),
        }
    }

    #[allow(dead_code)]
    pub fn byte(self) -> Self {
        match self {
            Speed::Byte(_) => self,
            Speed::KByte(_) => self.lower(),
            Speed::MByte(_) => self.lower().lower(),
            Speed::GByte(_) => self.lower().lower().lower(),
        }
    }

    #[allow(dead_code)]
    pub fn kb(self) -> Self {
        match self {
            Speed::Byte(_) => self.upper(),
            Speed::KByte(_) => self,
            Speed::MByte(_) => self.lower(),
            Speed::GByte(_) => self.lower().lower(),
        }
    }

    pub fn mb(self) -> Self {
        match self {
            Speed::Byte(_) => self.upper().upper(),
            Speed::KByte(_) => self.upper(),
            Speed::MByte(_) => self,
            Speed::GByte(_) => self.lower(),
        }
    }

    #[allow(dead_code)]
    pub fn gb(self) -> Self {
        match self {
            Speed::Byte(_) => self.upper().upper().upper(),
            Speed::KByte(_) => self.upper().upper(),
            Speed::MByte(_) => self.upper(),
            Speed::GByte(_) => self,
        }
    }
}

impl Display for Speed {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Byte(b) => write!(f, "{:.4} Byte/s", b),
            Self::KByte(b) => write!(f, "{:.4} KB/s", b),
            Self::MByte(b) => write!(f, "{:.4} MB/s", b),
            Self::GByte(b) => write!(f, "{:.4} GB/s", b),
        }
    }
}

pub struct ConnectTestStats {
    pub ip: IpAddr,
    pub cost: Duration,
}

impl ConnectTestStats {
    pub fn new(ip: IpAddr, cost: Duration) -> Self {
        Self { ip, cost }
    }
}

impl Display for ConnectTestStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:>15} connect cost {:?}", self.ip, self.cost)
    }
}

pub struct DownloadTestStats {
    pub ip: IpAddr,
    pub speed: Speed,
}

impl Display for DownloadTestStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} download speed {}", self.ip, self.speed)
    }
}

impl DownloadTestStats {
    pub fn new(ip: IpAddr, speed: Speed) -> Self {
        Self { ip, speed }
    }
}

#[derive(Clone, Debug)]
pub enum ServerAddress {
    Socket(SocketAddr),
    URL(Url),
}

#[async_trait]
pub trait ProxyTest {
    async fn connect(
        &self,
        dst: ServerAddress,
        via: Option<ServerAddress>,
        timeout: Duration,
    ) -> Result<ConnectTestStats, Box<dyn Error>>;

    fn connect_test(&self) -> Vec<ConnectTestStats> {
        let addrs_conn = self.get_address_conn();
        let addr_remote = self.get_address_remote();
        let timeout = self.get_timeout();
        let mut futures = Vec::new();
        for addr in addrs_conn {
            let future = self.connect(addr, addr_remote.clone(), timeout);
            futures.push(future);
        }

        let mut retain = Vec::new();
        println!("开始测试连接速度，请等待 {:?}", timeout);
        Runtime::new().unwrap().block_on(async {
            let stats = join_all(futures).await;

            for s in stats {
                if let Ok(x) = s {
                    retain.push(x)
                }
            }
            retain.sort_by(|a, b| a.cost.cmp(&b.cost));
            println!("连接速度测试完毕，有效数据 {} 条", retain.len());
        });
        retain
    }

    fn get_address_conn(&self) -> Vec<ServerAddress>;
    fn get_address_remote(&self) -> Option<ServerAddress>;
    fn get_timeout(&self) -> Duration;

    fn download_test(&self, _ips: Vec<ServerAddress>) -> Vec<DownloadTestStats> {
        let v = Vec::new();
        v
    }
}
