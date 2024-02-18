use std::{net::{SocketAddr, TcpStream}, str::FromStr};

use crate::{input::selector, pinging::{mc_legacy::LegacyPinger, Pinger}};

mod input;
mod pinging;


fn main() {
    let value = selector::select_one_of(["a", "b", "c", "d"].into_iter()).unwrap();



    let l = LegacyPinger {
        protocol_version: 0,
        hostname: "betachy.eu".to_string()
    };

    let data = l.ping(SocketAddr::from_str("45.132.90.119:25565").unwrap());
    println!("Hello, world! {:?}", data);
}
