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
#[derive(Debug, StructOpt)]
#[structopt(name = "mcping")]
struct Options {
    #[structopt(long)]
    pvn: Option<i32>,
    #[structopt(long)]
    modlist: bool,
    #[structopt(short, long)]
    verbose: bool,
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
    let addr_to_use;
    let len = addresses.len();
    let mut pvn = -1;
    if let Some(x) = options.pvn {
        pvn = x;
        log::info!("pinging with protocol version {}", pvn);
    } else {
        log::info!("no protocol version provided, using default {}", pvn);
    }
    if len > 1 {
        for i in 0..len {
            println!("{}: {}", i + 1, addresses[i]);
        }
        let mut num: usize;
        log::info!("multiple addresses found. which do we use? (0 for all)");
        loop {
            let line = get_line().or_else(|_| Err(PingError::StdinReadError))?;
            loop {
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
    ping(&options, pvn, &port, &addr)?;
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