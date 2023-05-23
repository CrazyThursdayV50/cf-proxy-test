mod internal;

#[cfg(test)]
mod test;

use std::net::SocketAddr;
use std::time::Duration;

use internal::client;
use internal::config::def::Config;

use crate::internal::network::api::ServerAddress;

fn main() {
    let args = client::Command::init();
    let (conf, ips) = Config::init(&args.conf_path, &args.ip_src);
    let proxy_client = conf.create_proxy_test_client(ips);
    let mut result = proxy_client.connect_test();
    result.sort_by(|a, b| a.cost.cmp(&b.cost));

    let max_cost = Duration::from_secs(2);
    let max_download_test_count = 10;
    let mut count = 0;
    result.iter().all(|x| {
        if x.cost >= max_cost && count >= max_download_test_count {
            return false;
        }
        if count == 0 {
            println!("===============");
        }
        println!("{}", x);
        count += 1;
        if count == max_download_test_count {
            println!("^^^^^^^^^^^^^^^ THESE IP WILL BE TESTED ON DOWNLOAD METHOD.");
            println!("===============");
        }
        true
    });

    let mut addrs = Vec::new();

    result
        .iter()
        .for_each(|r| addrs.push(ServerAddress::Socket(SocketAddr::new(r.ip, conf.port))));

    let download_result = proxy_client.download_test(addrs);
    println!("DOWNLOAD TEST RESULT: {}", download_result.len());
    download_result.iter().for_each(|x| {
        println!("{x}");
    });
}
