use super::def::ServerAddress;
use async_trait::async_trait;
use futures::future::join_all;
use std::error::Error;
use std::fmt::{write, Display};
use std::{net::IpAddr, time::Duration};
use tokio::runtime::Runtime;

#[derive(Clone)]
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
        write!(f, "连接 {:<15} 耗时 {:?}", self.ip, self.cost)
    }
}

#[derive(Clone)]
pub struct ConnectTestResult {
    top: usize,
    list: Option<Vec<ConnectTestStats>>,
}

impl Display for ConnectTestResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.list {
            None => write!(f, "没有测速数据，可以尝试以下方法后再次重试：\n1. 更换网络\n2. 更新 Cloudflare 反代 IP 数据源\n3. 增大网络连接 timeout 数值"),
            Some(list) => {
                let mut content = String::new();
                content.push_str("测速结果：\n");
                content.push_str(format!("总有效测速结果：{} 条\n", list.len()).as_str());
                content.push_str(format!("下面是连接速度最快的 {} 条数据：\n", self.top).as_str());
                list.iter().enumerate().all(|(i, x)| {
                    if i >= self.top {
                        return false
                    }
                    
                    content.push_str(format!("{}\n", x).as_str());
                    true
                });

                write!(f, "{}", content)
            }
        }
    }
}

#[async_trait]
pub trait ConnTest {
    async fn connect(
        &self,
        dst: ServerAddress,
        via: Option<ServerAddress>,
        timeout: Duration,
    ) -> Result<ConnectTestStats, Box<dyn Error>>;

    fn connect_test(&self) -> ConnectTestResult {
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
            stats.into_iter().for_each(|x| {
                if let Ok(x) = x {
                    retain.push(x)
                }
            });

            retain.sort_by(|a, b| a.cost.cmp(&b.cost));
        });

        if retain.len() == 0 {
            return ConnectTestResult {
                list: None,
                top: self.get_top(),
            };
        }

        return ConnectTestResult {
            list: Some(retain),
            top: self.get_top(),
        };
    }

    fn get_address_conn(&self) -> Vec<ServerAddress>;
    fn get_address_remote(&self) -> Option<ServerAddress>;
    fn get_timeout(&self) -> Duration;
    fn get_top(&self) -> usize;
}
