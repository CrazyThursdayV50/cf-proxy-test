extern crate serde;
extern crate serde_yaml;

use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
// 配置说明在 src/config/example.yaml 中
pub struct Config {
    // 测试 URL
    pub url: String,
    // 测试代理端口
    pub port: u16,
    // 连通性测试配置
    pub conn: ConnConfig,
    // 下载测试配置
    pub download: DownloadConfig,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct ConnConfig {
    // http, tcp
    pub method: String,
    pub timeout: u64,
    pub http: ConnHttpConfig,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct ConnHttpConfig {
    pub resp_timeout: u64,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct DownloadConfig {
    pub timeout: u64,
}
