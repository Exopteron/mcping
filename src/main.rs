use colored_json::prelude::*;
use env_logger::Builder;
use log::LevelFilter;
use rust_minecraft_networking::{PacketBuilder, PacketUtils};
use std::io::{Read, Write};
use std::net::TcpStream;
use std::process::exit;
use std::time::Instant;
use structopt::StructOpt;
use trust_dns_resolver::config::*;
use trust_dns_resolver::Resolver;
mod query_netutil;
const DEFAULT_PVN: i32 = -1;
#[derive(Debug, StructOpt)]
#[structopt(name = "mcping")]
struct Options {
    #[structopt(long)]
    pvn: Option<i32>,
    #[structopt(long)]
    modlist: bool,
    #[structopt(short, long)]
    verbose: bool,
    #[structopt(long)]
    query: bool,
    #[structopt(long = "ping")]
    only_ping: bool,
    addr: String,
}
#[derive(serde::Serialize, serde::Deserialize)]
pub struct Mod {
    modid: String,
    version: String,
}
#[derive(serde::Serialize, serde::Deserialize)]
pub struct ModInfo {
    r#type: String,
    modList: Vec<Mod>
}
#[derive(serde::Serialize, serde::Deserialize)]
pub struct Players {
    max: usize,
    online: usize,
    sample: Vec<Player>
}
#[derive(serde::Serialize, serde::Deserialize)]
pub struct Player {
    id: String,
    name: String,
}
#[derive(Debug)]
pub enum PingError {
    StdinReadError,
    ParseIntError,
    ConnectError,
    WriteError,
    ReadError,
    GenericPingError,
    CantBind,
    WrongPacket,
    NoAddress,
    NumFromStrErr,
}
pub type Result<T> = std::result::Result<T, PingError>;
fn main() -> Result<()> {
    init_logger();
    let options = Options::from_args();
    let ip_split = options.addr.split(":").collect::<Vec<&str>>();
    let mut port = "25565".to_owned();
    if ip_split.len() >= 2 {
        port = ip_split[1].to_string();
    }
    let resolver = Resolver::new(ResolverConfig::cloudflare_tls(), ResolverOpts::default()).unwrap();
    let mut ip = ip_split[0].to_owned();
    if let Some((srv_ip, srv_port)) = srv_lookup(&ip) {
        log::info!("got SRV record for {}:{}, using it instead", srv_ip, srv_port);
        ip = srv_ip;
        port = srv_port.to_string();
    }
    let response = resolver.lookup_ip(ip);
    let response = match response {
        Ok(r) => r,
        Err(e) => {
            log::error!("error: {}", e.to_string());
            exit(1);
        }
    };
    let mut addresses = vec![];
    for address in response.iter() {
        if address.is_ipv4() {
            addresses.push(address);
        }
    }
    if options.query {
        let address = addresses.iter().next().ok_or(PingError::NoAddress)?;
        let addr = format!("{}:{}", address.to_string(), port);
        log::info!("attempting query to {}", addr);
        match query_ping(&options, &addr) {
            Ok(_) => {

            }
            Err(_) => {

            }
        }
        return Ok(());
    }
    let addr_to_use;
    let len = addresses.len();
    let mut pvn = DEFAULT_PVN;
    if let Some(x) = options.pvn {
        pvn = x;
        log::info!("pinging with protocol version {}", pvn);
    } else {
        log::info!("no protocol version provided, using default ({})", pvn);
    }
    if len > 1 {
        for i in 0..len {
            println!("{}: {}", i + 1, addresses[i]);
        }
        let mut num: usize;
        log::info!("multiple addresses found. which do we use? (0 for all)");
        loop {
            loop {
                let line = get_line().or_else(|_| Err(PingError::StdinReadError))?;
                let x = usize::from_str_radix(&line, 10);
                if x.is_err() {
                    log::warn!("not a valid number.");
                } else {
                    num = x.unwrap();
                    break;
                }
            }
            if num == 0 {
                log::info!("testing all addresses...");
                use std::collections::HashMap;
                let mut hashmap: HashMap<u128, String> = HashMap::new();
                for addr in &addresses {
                    let addr = format!("{}:{}", addr, port);
                    match ping(&options, pvn, &port, &addr) {
                        Ok(x) => {
                            hashmap.insert(x, addr);
                        }
                        Err(_) => {
                            log::error!("an error occured pinging {}.", addr);
                        }
                    }
                }
                let mut bestping = u128::MAX;
                let mut bestaddr = "".to_owned();
                for (ping, addr) in hashmap {
                    if ping < bestping {
                        bestaddr = addr;
                        bestping = ping;
                    }
                }
                log::info!("best ping is {} with a time of {}ms", bestaddr, bestping);
                exit(0);
            }
            if num as usize > len {
                log::info!("out of range. try again");
            } else {
                break;
            }
        }
        addr_to_use = addresses[num as usize - 1].to_string();
    } else {
        addr_to_use = addresses[0].to_string();
    }
    let addr = format!("{}:{}", addr_to_use, port);
    log::info!("attempting to ping {}...", addr);
    match ping(&options, pvn, &port, &addr) {
        Ok(_) => {

        }
        Err(_) => {
        
        }
    }
    log::info!("attempting query to {}...", addr);
    match query_ping(&options, &addr) {
        Ok(_) => {

        }
        Err(_) => {
        
        }
    }
    Ok(())
}
fn query_ping(options: &Options, addr: &str) -> Result<()> {
    use std::net::UdpSocket;
    use rand::RngCore;
    let mut socket = UdpSocket::bind("0.0.0.0:62034");
    let mut set = false;
    if socket.is_err() {
        for i in (0..65535).rev() {
            let x = UdpSocket::bind(format!("0.0.0.0:{}", i));
            if x.is_ok() {
                socket = x;
                set = true;
                break;
            }
        }
    } else {
        set = true;
    }
    if !set {
        return Err(PingError::CantBind);
    }
    let socket = socket.unwrap();

    match socket.connect(addr) {
        Ok(_) => {

        }
        Err(_) => {
            log::info!("failed to connect. server probably doesn't support query/wrong port.");
            return Err(PingError::ConnectError);
        }
    }
    socket.set_read_timeout(Some(std::time::Duration::from_secs(5))).unwrap();
    let mut session_id = [0; 4];
    rand::thread_rng().fill_bytes(&mut session_id);
    let session_id = i32::from_be_bytes(session_id) & 0x0F0F0F0F;
    let packet = query_netutil::PacketBuilder::new();
    let packet = packet.build(09, session_id);
    socket.send(&packet).or(Err(PingError::WriteError))?;
    let mut vec = vec![0; 64];
    let amt = socket.recv(&mut vec);
    let amt = match amt {
        Ok(a) => a,
        Err(e) => {
            match e.kind() {
                std::io::ErrorKind::WouldBlock {} => {
                    log::info!("no response after 5 seconds. server probably doesn't support query.");
                    return Err(PingError::ReadError);
                }
                std::io::ErrorKind::TimedOut {} => {
                    log::info!("no response after 5 seconds. server probably doesn't support query.");
                    return Err(PingError::ReadError);
                }
                _ => {
                    return Err(PingError::ReadError);
                }
            }
            
        }
    };
    let vec = vec.drain(..amt);
    let mut vec = std::io::Cursor::new(vec);
    let p_type = query_netutil::PacketUtils::read_byte(&mut vec).ok_or(PingError::ReadError)?;
    if p_type != 0x09 {
        return Err(PingError::WrongPacket);
    }
    let recv_s_id = query_netutil::PacketUtils::read_int32(&mut vec).ok_or(PingError::ReadError)?;
    if recv_s_id != session_id {
        return Err(PingError::WrongPacket);
    }
    let challenge_token = query_netutil::PacketUtils::read_string(&mut vec).ok_or(PingError::ReadError)?;
    let challenge_token = i32::from_str_radix(&challenge_token, 10).or(Err(PingError::NumFromStrErr))?;
    let mut packet = query_netutil::PacketBuilder::new();
    packet.insert_int(challenge_token);
    packet.insert_bytearray(vec![0; 4]);
    let packet = packet.build(0x00, session_id);
    socket.send(&packet).or(Err(PingError::WriteError))?;
    let mut vec = vec![0; 2048];
    let amt = socket.recv(&mut vec).or(Err(PingError::ReadError))?;
    let vec = vec.drain(..amt);
    let mut vec = std::io::Cursor::new(vec);
    let p_type = query_netutil::PacketUtils::read_byte(&mut vec).ok_or(PingError::ReadError)?;
    if p_type != 0x00 {
        return Err(PingError::WrongPacket);
    }
    let recv_s_id = query_netutil::PacketUtils::read_int32(&mut vec).ok_or(PingError::ReadError)?;
    if recv_s_id != session_id {
        return Err(PingError::WrongPacket);
    }
    let mut x = [0; 11];
    vec.read_exact(&mut x).or(Err(PingError::ReadError))?;
    drop(x);
    use std::collections::HashMap;
    let mut hashmap: HashMap<String, String> = HashMap::new();
    loop {
        let key = query_netutil::PacketUtils::read_string(&mut vec).ok_or(PingError::ReadError)?;
        if key.len() == 0 {
            break;
        }
        let value = query_netutil::PacketUtils::read_string(&mut vec).ok_or(PingError::ReadError)?;
        hashmap.insert(key, value);
    }
    log::info!("query results:");
    for (k, v) in hashmap {
        println!("- {}: {}", k, v);
    }
    let mut x = [0; 10];
    vec.read_exact(&mut x).or(Err(PingError::ReadError))?;
    drop(x);
    let mut players = vec![];
    loop {
        let player = query_netutil::PacketUtils::read_string(&mut vec).ok_or(PingError::ReadError)?;
        if player.len() < 1 {
            break;
        }
        players.push(player);
    }
    if players.len() > 0 {
        log::info!("online players:");
        for player in players {
            println!("- {}", player);
        }
    }
    Ok(())
}
fn ping(options: &Options, pvn: i32, port: &str, addr: &str) -> Result<u128> {
    let mut builder = PacketBuilder::new();
    builder.insert_varint(pvn);
    builder.insert_string(&options.addr);
    builder.insert_unsigned_short(u16::from_str_radix(&port, 10).or_else(|_| Err(PingError::ParseIntError))?);
    builder.insert_varint(1);
    let packet = builder.build(0x00);
    let mut stream = TcpStream::connect(&addr).or_else(|_| Err(PingError::ConnectError))?;
    stream
        .write(&packet)
        .or_else(|_| Err(PingError::WriteError))?;
    let builder = PacketBuilder::new();
    let packet = builder.build(0x00);
    stream
        .write(&packet)
        .or_else(|_| Err(PingError::WriteError))?;
    let response = PacketUtils::read_packet(&mut stream);
    let response = match response {
        Ok(x) => x,
        Err(_) => {
            log::error!("incorrect response from server. attempting legacy ping...");
            match legacy_ping(&addr) {
                Ok(_) => {
                    return Ok(0);
                }
                Err(_) => {
                    return Err(PingError::GenericPingError);
                }
            }
        }
    };
    let mut builder = PacketBuilder::new();
    builder.insert_long(420);
    let packet = builder.build(0x01);
    if response.id != 0x00 {
        return Err(PingError::GenericPingError);
    }
    let mut packetdata = std::io::Cursor::new(response.contents);
    let json = read_string(&mut packetdata)?;
    stream
        .write(&packet)
        .or_else(|_| Err(PingError::ConnectError))?;
    if options.verbose {
        log::info!("[V] entire response: \n{}", json);
    }
    let deserialized: serde_json::Value = match serde_json::from_str::<serde_json::Value>(&json) {
        Ok(x) => x,
        Err(_) => {
            return Err(PingError::GenericPingError);
        }
    };
    if !options.only_ping {
        match deserialized["description"]
        .to_string()
        .to_colored_json_auto()
    {
        Ok(x) => {
            log::info!("server description:\n{}", x);
        }
        Err(e) => {
            if options.verbose {
                log::warn!("json pretty-ification failed. error: {:?}", e);
            }
            log::info!("server description:\n{:?}", deserialized["description"]);
        }
    }
    if !deserialized["modinfo"].is_null() {
        loop {
            let modinfo: ModInfo = match serde_json::from_str(&deserialized["modinfo"].to_string()) {
                Ok(m) => m,
                Err(_) => {
                    break;
                }
            };
            if modinfo.r#type == "FML" && options.modlist || options.verbose {
                log::info!("FML mod info:");
                for i in 0..modinfo.modList.len() {
                    log::info!("mod #{}", i + 1);
                    println!("   --- id: {}", modinfo.modList[i].modid);   
                    println!("   --- version: {}", modinfo.modList[i].version);   
                }
            } else if modinfo.r#type == "FML" {
                let len = modinfo.modList.len();
                match len {
                    1 => {
                        log::info!("server is an FML compatible server with {} mod.", len);
                    }
                    _ => {
                        log::info!("server is an FML compatible server with {} mods.", len);
                    }
                }
            }
            break;
        }
    }
    //log::info!("poo: {}", deserialized["ashh1"].is_null());
    log::info!(
        "server version:\n   --- {}\n   --- protocol version {}",
        deserialized["version"]["name"],
        deserialized["version"]["protocol"]
    );
    log::info!(
        "players:\n   --- {}/{}",
        deserialized["players"]["online"],
        deserialized["players"]["max"]
    );
    if !deserialized["players"]["sample"].is_null() {
        loop {
            let sample: Players = match serde_json::from_str(&deserialized["players"].to_string()) {
                Ok(m) => m,
                Err(_) => {
                    break;
                }
            };
            log::info!("sample:");
            for player in sample.sample {
                println!("   --- {}", player.name);
            }
            break;
        }
    }
    }
    let now = Instant::now();
    let response = PacketUtils::read_packet(&mut stream).or_else(|_| Err(PingError::ReadError))?;
    if response.id != 0x01 {
        return Err(PingError::GenericPingError);
    }
    let mut cursor = std::io::Cursor::new(response.contents);
    let mut bytes = [0; 8];
    cursor
        .read_exact(&mut bytes)
        .or_else(|_| Err(PingError::ReadError))?;
    let long = i64::from_be_bytes(bytes);
    if long != 420 {
        return Err(PingError::GenericPingError);
    }
    let elapsed = now.elapsed();
    log::info!("[{addr}] ping: {}ms", elapsed.as_millis(), addr = addr);
    Ok(elapsed.as_millis())
}
fn init_logger() {
    Builder::new()
        .format(move |buf, record| writeln!(buf, "mcping - {}", record.args()))
        .filter(None, LevelFilter::Info)
        .init();
}

fn get_line() -> std::io::Result<String> {
    let mut line = String::new();
    std::io::stdin().read_line(&mut line)?;
    return Ok(line.trim().to_string());
}

fn read_string(reader: &mut dyn std::io::Read) -> Result<String> {
    let array = PacketUtils::read_varint_prefixed_bytearray(reader)
        .or_else(|_| Err(PingError::ReadError))?;
    Ok(String::from_utf8_lossy(&array).to_string())
}


fn srv_lookup(domain: &str) -> Option<(String, u16)> {
    let resolver = Resolver::new(ResolverConfig::cloudflare_tls(), ResolverOpts::default()).unwrap();
    let srv = resolver.srv_lookup(format!("_minecraft._tcp.{}", domain.to_string()));
    let srv = match srv {
        Ok(r) => r,
        Err(_) => {
            return None;
        }
    };
    let address = match srv.as_lookup().iter().next() {
        Some(x) => x,
        None => {
            return None;
        }
    };
    let address = match address.as_srv() {
        Some(x) => x,
        None => {
            return None;
        }
    };
    let port = address.port();
    let target = address.target();
    return Some((target.to_string(), port));
}

fn legacy_ping(addr: &str) -> Result<()> {
    let mut stream = TcpStream::connect(addr).or_else(|_| Err(PingError::ConnectError))?;
    stream.write(&[0xFE, 0x01]).or_else(|_| Err(PingError::WriteError))?;
    let mut buf = vec![];
    stream.read_to_end(&mut buf).unwrap();
    let buf = &buf[2..];
    let mut newbuf = vec![];
    let mut flag = false;
    for i in 0..buf.len() - 1 {
        if flag == false {
            newbuf.push(buf[i]);
        } else {
            flag ^= true;
        }
    }
    println!("{:?}", String::from_utf8_lossy(&newbuf));
    Ok(())
}