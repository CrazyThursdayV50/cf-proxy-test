extern crate clap;

use clap::{arg, Arg};

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

pub fn new_cmd() -> clap::Command {
    clap::Command::new("cf-proxy-test")
        .about("用于测试 Cloudflare 反代IP，仅供学习或者娱乐使用。")
        .args(register_args())
}

pub struct Command {
    pub conf_path: String,
    pub ip_src: String,
}

impl Command {
    pub fn init() -> Self {
        let cmd = new_cmd().get_matches();

        let mut conf_path = DEFAULT_CONF.to_string();
        if let Some(p) = cmd.get_one::<String>("config") {
            conf_path = p.to_owned();
        }

        let mut ip_src = DEFAULT_IP_FILE.to_string();
        if let Some(p) = cmd.get_one::<String>("src") {
            ip_src = p.to_owned();
        }

        Self { conf_path, ip_src }
    }
}
