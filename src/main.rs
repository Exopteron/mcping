use std::{net::{SocketAddr, TcpStream}, str::FromStr};

use crate::{input::selector, pinging::{mc_legacy::LegacyPinger, mc_modern::ModernPinger, Pinger}};

mod input;
mod pinging;


fn main() {
    let value = selector::select_one_of(["a", "b", "c", "d"].into_iter()).unwrap();



    let l = ModernPinger {
        protocol_version: -1,
        hostname: "play.cubecraft.net".to_string()
    };

    let data = l.ping(SocketAddr::from_str("139.99.83.6:25565").unwrap());
    println!("Hello, world! {:?}", data);
}
