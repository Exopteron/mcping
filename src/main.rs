use std::{borrow::Cow, net::{SocketAddr, TcpStream}, option, process::ExitCode, str::FromStr, time::Duration};

use clap::Parser;
use env_logger::Builder;
use hickory_resolver::proto::{op, rr::rdata::opt};
use log::LevelFilter;

use crate::{input::selector::{self, Selected}, pinging::{mc_legacy::LegacyPinger, mc_modern::ModernPinger, Pinger}};

mod input;
mod pinging;
mod resolution;


#[derive(Parser, Debug)]
#[clap(name = "mcping")]
struct Options {
    #[arg(short, long)]
    verbose: bool,
    #[arg(long)]
    query: bool,
    #[arg(long = "ping")]
    only_ping: bool,
    addr: String,
}


fn main() -> ExitCode {

    let options = Options::parse();
    init_logger(options.verbose);


    let name = {
        let opts = options.addr.split(':').collect::<Vec<_>>();
        if opts.len() > 2 {
            log::error!("Invalid address provided.");
            return ExitCode::FAILURE;
        }

        let hostname = opts[0].to_owned();
        let port = if let Some(port) = opts.get(1) {
            if let Ok(v) = port.parse::<u16>() {
                v 
            } else {
                log::error!("Bad port provided.");
                return ExitCode::FAILURE
            }

        } else {
            25565
        };
        (hostname, port)
    };


    let lookup = resolution::resolve_minecraft_ips(name.clone()).unwrap();
    
    
    let value = if lookup.len() > 1 {
        log::info!("multiple addresses found. which should we use?");
        if let Selected::Value(v) = selector::select_one_of(lookup.into_iter()).unwrap() {
            v
        } else {
            panic!();
        }
    } else {
        lookup.into_iter().next().unwrap()
    };
    

    log::info!("attempting to ping {}...", value);

    let l = ModernPinger {
        protocol_version: -1,
        hostname: name.0.clone(),
        read_timeout: Duration::from_secs(5)
    };
    let data = l.ping(value);

    match data {
        Ok(data) => {
            panic!();
        }
        Err(e) => {
            log::info!("standard ping failed. attempting legacy ping...");
            log::debug!("failure reason: {:?}", e);
            let l = LegacyPinger {
                protocol_version: 0,
                hostname: name.0,
            };
    
            let data = match l.ping(value) {
                Ok(v) => v,
                Err(e) => {
                    log::error!("legacy ping failed. {:?}", e);
                    return ExitCode::FAILURE;
                }
            };

            log::info!("motd:");
            println!("   --- {}", get_or(data.motd, "none"));
            log::info!("players:");
            println!("   --- {}/{}", get_or(data.online_players, "none"), get_or(data.max_players, "none"));

            if let Some(extra) = data.extra {
                log::info!("extra data present (>1.6)");
                log::info!("protocol version:");
                println!("   --- {}", get_or(extra.protocol_version, "none"));
                log::info!("server version:");
                println!("---   {}", get_or(extra.server_version, "none"));
            }

            if !data.unrecognised.is_empty() {
                log::debug!("unrecognised fields in ping response: {:?}", data.unrecognised);
            }
        }
    }



    ExitCode::SUCCESS
}

fn get_or(v: Option<String>, val: &str) -> Cow<str> {
    if let Some(v) = v {
        Cow::Owned(v)
    } else {
        Cow::Borrowed(val)
    }
}


fn init_logger(verbose: bool) {
    use std::io::Write;
    Builder::new()
        .format(move |buf, record| writeln!(buf, "mcping - {}", record.args()))
        .filter(None, if !verbose {
            LevelFilter::Info
        } else {
            LevelFilter::Debug
        })
        .init();
}