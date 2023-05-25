mod internal;

#[cfg(test)]
mod test;

use internal::client;
use internal::config::def::Config;

fn main() {
    let args = client::args::Command::init();
    let (conf, ips) = Config::init(&args.conf_path, &args.ip_src);
    let connector = conf.create_conn_test_client(ips);
    let result = connector.connect_test();
    println!("{result}");
    let conn_top = result.top_ips();
    let downloader = conf.create_download_test_client(conn_top);
    let download_result = downloader.download_test();
    println!("{download_result}")
}
