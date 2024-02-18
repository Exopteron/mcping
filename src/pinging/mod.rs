pub mod mc_legacy;
pub mod mc_modern;
pub mod mc_query;

use std::net::SocketAddr;

pub trait Pinger {

    /// The data returned from a ping.
    type Data;

    /// A reported error value.
    type Error: std::error::Error;

    fn ping(&self, addr: SocketAddr) -> std::result::Result<Self::Data, Self::Error>;
}