use std::{
    io::{self, Write},
    net::TcpStream,
    str::Split,
    string::FromUtf16Error,
};

use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use thiserror::Error;

use super::Pinger;
#[derive(Debug)]

pub struct ExtraLegacyPingData {
    pub server_version: Option<String>,
    pub protocol_version: Option<String>,
}
#[derive(Debug)]
pub struct LegacyPingData {
    pub motd: Option<String>,
    pub online_players: Option<String>,
    pub max_players: Option<String>,
    pub unrecognised: Vec<String>,
    pub extra: Option<ExtraLegacyPingData>,
}

pub struct LegacyPinger {
    pub protocol_version: u8,
    pub hostname: String
}

impl Pinger for LegacyPinger {
    type Data = LegacyPingData;

    type Error = LegacyPingError;

    fn ping(&self, addr: std::net::SocketAddr) -> std::result::Result<Self::Data, Self::Error> {
        let mut stream = TcpStream::connect(addr)?;

        stream.write_all(&[0xFE])?; // opening bytes
        stream.write_all(&[0x01])?;
        stream.write_all(&[0xFA])?;

        let plugin_message_header = "MC|PingHost".encode_utf16().collect::<Vec<_>>();
        stream.write_u16::<BigEndian>(plugin_message_header.len() as u16)?;
        for v in plugin_message_header {
            stream.write_u16::<BigEndian>(v)?;
        }
        
        let hostname = self.hostname.encode_utf16().collect::<Vec<_>>();
        stream.write_u16::<BigEndian>((hostname.len() as u16) + 7)?; // 7 for rest of the data
        
        stream.write_u8(self.protocol_version)?;

        stream.write_u16::<BigEndian>(hostname.len() as u16)?;

        for v in hostname {
            stream.write_u16::<BigEndian>(v)?;
        }

        stream.write_u32::<BigEndian>(addr.port() as u32)?;



        let packet_id = stream.read_u8()?;


        if packet_id != 0xFF {
            return Err(LegacyPingError::WrongId(packet_id));
        }

        let string_length_in_chars = stream.read_u16::<BigEndian>()? as usize;

        let mut utf16_str = vec![0u16; string_length_in_chars];

        stream.read_u16_into::<BigEndian>(&mut utf16_str)?;

        let read_data = String::from_utf16(&utf16_str)?;

        let get_field = |v: &mut Split<char>| {
            v.next()
                .map(|v| v.to_owned())
        };

        let data = if read_data.starts_with("§1\0") {
            // 1.6 ping

            let mut fields = read_data.trim_start_matches("§1\0").split('\0'); // fields are delimited by NUL-chars

            let protocol_version = get_field(&mut fields);

            let server_version = get_field(&mut fields);


            #[allow(clippy::unnecessary_unwrap)]
            if protocol_version.is_some() && server_version.is_none() {
                // only one field was delivered. something weird happened!
                return Err(LegacyPingError::UnexpectedReply(protocol_version.unwrap()))
            }

            let motd = get_field(&mut fields);

            let online_players = get_field(&mut fields);

            let max_players = get_field(&mut fields);

            let unrecognised = fields.map(|v| v.to_owned()).collect::<Vec<_>>();

            LegacyPingData {
                motd,
                online_players,
                max_players,
                unrecognised,
                extra: Some(ExtraLegacyPingData {
                    server_version,
                    protocol_version,
                }),
            }
        } else {
            // pre 1.6 ping

            let mut fields = read_data.split('§'); // fields are delimited by the section symbol


            let motd = get_field(&mut fields);

            let online_players = get_field(&mut fields);

            #[allow(clippy::unnecessary_unwrap)]
            if motd.is_some() && online_players.is_none() {
                // only one field was delivered. something weird happened!
                return Err(LegacyPingError::UnexpectedReply(motd.unwrap()))
            }

            let max_players = get_field(&mut fields);

            let unrecognised = fields.map(|v| v.to_owned()).collect::<Vec<_>>();

            LegacyPingData {
                motd,
                online_players,
                max_players,
                unrecognised,
                extra: None,
            }
        };

        Ok(data)
    }
}

#[derive(Error, Debug)]
pub enum LegacyPingError {
    #[error("Unexpected reply from server: {0}")]
    UnexpectedReply(String),

    #[error("received wrong packet id {0}, expected 0xFF")]
    WrongId(u8),

    #[error("Received invalid UTF string data")]
    InvalidStringData(#[from] FromUtf16Error),

    #[error("IO error during ping")]
    IoError(#[from] io::Error),
}
