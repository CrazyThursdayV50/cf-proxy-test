extern crate clap;

use clap::{arg, Arg, Command};

pub const DEFAULT_CONF: &str = "./conf.yaml";
pub const DEFAULT_IP_FILE: &str = "./ip.txt";
pub const DEFAULT_CONN_TIMEOUT: u64 = 10;
pub const DEFAULT_DOWNLOAD_TIMEOUT: u64 = 10;

fn register_args() -> Vec<Arg> {
    vec![
        arg!(-c --config <CONFIG> "指定配置文件，默认为 ./conf.yaml"),
        arg!(-s --src <IP_FILE_SOURCE> "指定 ip 文件，默认为 ./ip.txt"),
    ]
}

pub fn new_cmd() -> Command {
    Command::new("cf-proxy-test")
        .about("用于测试 Cloudflare 反代IP，仅供学习或者娱乐使用。")
        .args(register_args())
}
