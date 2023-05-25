use crate::internal::client::conn::ConnTest;
use crate::internal::network::http::HttpClient;
use crate::internal::network::tcp::TcpClient;

use super::super::client::args;
use super::def::Config;

use std::fs;
use std::io::{BufRead, BufReader};
use std::net::{IpAddr, SocketAddr};
use std::time::Duration;

impl Config {
    fn check(mut self) -> Self {
        if self.conn.timeout > 60 {
            self.conn.timeout = args::DEFAULT_CONN_TIMEOUT;
        }

        if self.conn.http.resp_timeout > 60 {
            self.conn.http.resp_timeout = args::DEFAULT_CONN_TIMEOUT;
        };

        if self.download.timeout > 60 {
            self.download.timeout = args::DEFAULT_DOWNLOAD_TIMEOUT;
        };

        self
    }

    // 读配置
    fn new(path: &str) -> Self {
        println!("从 {path} 加载配置 ...");
        let file_content: String = fs::read_to_string(path).unwrap();
        let conf: Config = serde_yaml::from_str(&file_content).unwrap();
        conf.check()
    }

    pub fn init(conf_path: &str, ip_path: &str) -> (Self, Vec<IpAddr>) {
        // 加载配置
        let conf = Config::new(conf_path);
        // 读取 ip 文件
        let ips = load_ips(ip_path);

        (conf, ips)
    }

    pub fn create_conn_test_client(&self, ips: Vec<IpAddr>) -> Box<dyn ConnTest> {
        let mut socket_addrs = Vec::new();
        let timeout = Duration::from_secs(self.conn.timeout);
        for ip in ips {
            socket_addrs.push(SocketAddr::new(ip, self.port));
        }

        match self.conn.method.as_str() {
            "http" => {
                return Box::new(HttpClient::build(
                    self.url.as_str().parse().unwrap(),
                    socket_addrs,
                    timeout,
                    self.conn.top,
                ));
            }
            "tcp" => return Box::new(TcpClient::build(socket_addrs, timeout, self.conn.top)),

            others => panic!("invalid method: {others}"),
        }
    }
}

// 读 ip 文件
fn load_ips(ip_file_path: &str) -> Vec<IpAddr> {
    println!("从 {ip_file_path} 加载 ip 数据 ...");
    let f = fs::File::open(ip_file_path).unwrap();
    let lines = BufReader::new(f).lines();
    let mut ips = Vec::new();
    for line in lines {
        ips.push(line.unwrap().parse().unwrap());
    }
    ips
}
