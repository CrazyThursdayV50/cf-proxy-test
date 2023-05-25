use std::net::SocketAddr;

use url::Url;

#[derive(Clone, Debug)]
pub enum ServerAddress {
    Socket(SocketAddr),
    URL(Url),
}
