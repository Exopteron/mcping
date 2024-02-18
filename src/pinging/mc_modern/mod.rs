use std::{io::{self, Write}, net::TcpStream, time::{Duration, Instant}};

use byteorder::{BigEndian, WriteBytesExt};
use thiserror::Error;

use self::{helpers::{McModernValue, ProtocolError, VarInt}, ping_json::PingResponse};

use super::Pinger;

pub mod helpers;
pub mod ping_json;

#[derive(Debug)]
pub struct ModernPingData {
    pub response: PingResponse,
    pub latency: Duration
}

pub struct ModernPinger {
    pub protocol_version: i32,
    pub hostname: String,
    pub read_timeout: Duration
}

impl Pinger for ModernPinger {
    type Data = ModernPingData;

    type Error = ModernPingError;

    fn ping(&self, addr: std::net::SocketAddr) -> std::result::Result<Self::Data, Self::Error> {
        
        let mut stream = TcpStream::connect(addr)?;
        stream.set_read_timeout(Some(self.read_timeout));

        {
            let mut handshake_packet = vec![];

            VarInt(0x00).write_to(&mut handshake_packet)?;
            VarInt(self.protocol_version).write_to(&mut handshake_packet)?;
            self.hostname.write_to(&mut handshake_packet)?;
            handshake_packet.write_u16::<BigEndian>(addr.port())?;
            VarInt(1).write_to(&mut handshake_packet)?;

            VarInt(handshake_packet.len() as i32).write_to(&mut stream)?;
            stream.write_all(&handshake_packet)?;

        }

        {
            stream.write_all(&[0x01, 0x00])?; // status request packet
        }

        let _response_length = VarInt::read_from(&mut stream)?.0;

        let packet_id = VarInt::read_from(&mut stream)?.0;

        if packet_id != 0x00 {
            return Err(ModernPingError::WrongId(packet_id, 0x00));
        }

        let string_data = String::read_from(&mut stream)?;

        let data: PingResponse = serde_json::from_str(&string_data)?;
        

        stream.write_all(&[0x09, 0x01, 0x00, 0x00 ,0x00 ,0x00, 0x00, 0x00, 0x00, 0x00])?; // status request packet
        let start = Instant::now();

        let _response = VarInt::read_from(&mut stream)?.0;
        let latency = start.elapsed();




        Ok(ModernPingData { response: data, latency })

    }
}

#[derive(Error, Debug)]
pub enum ModernPingError {
    #[error("received wrong packet id {0}, expected {1}")]
    WrongId(i32, i32),

    #[error("JSON parse error")]
    JsonError(#[from] serde_json::Error),

    #[error("Protocol error: {0}")]
    ProtocolError(#[from] ProtocolError),

    #[error("IO error during ping")]
    IoError(#[from] io::Error),
}
