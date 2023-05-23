use crate::internal::config::def::Config;
use std::{path::PathBuf, time::Duration};

#[test]
fn test_read_config() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));

    // 读配置
    let mut conf_path = path.clone();
    conf_path.push("src/config/conf.yaml");
    // 读 ip 文件
    let mut ip_path = path.clone();
    ip_path.push("src/config/ip.txt");

    // let mut conf: Config = Config::new(conf_path.to_str().unwrap());
    let (conf, ips) = Config::init(conf_path.to_str().unwrap(), ip_path.to_str().unwrap());

    let timeout = Duration::from_secs(conf.conn.timeout);
}
