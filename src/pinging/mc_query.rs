use std::{collections::HashMap, ffi::CStr, io::{self, ErrorKind}, net::{IpAddr, Ipv4Addr, SocketAddr, SocketAddrV4, UdpSocket}, time::Duration};

use thiserror::Error;

use super::Pinger;



pub struct QueryData {
    pub hostname: String,
    pub game_type: String,
    pub version: String,
    pub plugins: Option<String>,
    pub map: String,
    pub num_players: String,
    pub max_players: String,
    pub host_port: String,
    pub host_ip: String,
    pub unrecognised: HashMap<String, String>
}


pub struct QueryPinger {
    pub read_timeout: Duration
}


impl Pinger for QueryPinger {
    type Data = QueryData;

    type Error = QueryPingError;

    fn ping(&self, addr: std::net::SocketAddr) -> std::result::Result<Self::Data, Self::Error> {
        
            
        let mut socket = UdpSocket::bind(SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), 0))?;
        socket.connect(addr)?;
        socket.set_read_timeout(Some(self.read_timeout))?;

        socket.send(&[0xFE, 0xFD, 0x09, 0x00, 0x00, 0x00, 0x01])?; // handshake with session id 1

        
        

        let mut data = [0; 64];
        let challenge_token = match socket.recv(&mut data) {
            Ok(read) => {
                let data = &data[9..read];
                let data = CStr::from_bytes_with_nul(data).map_err(|_| QueryPingError::BadResponseString)?.to_string_lossy();
                data.parse::<i32>().map_err(|_| QueryPingError::BadResponseString)?
            }
            Err(e) if e.kind() == ErrorKind::TimedOut => return Err(QueryPingError::TimeoutReached),
            Err(e) => return Err(QueryPingError::IoError(e))
        };
        


        todo!()
    }
}




#[derive(Error, Debug)]
pub enum QueryPingError {
    #[error("Timeout reached.")]
    TimeoutReached,
    #[error("Bad string data received.")]
    BadResponseString,

    #[error("IO error during ping")]
    IoError(#[from] io::Error),
}
