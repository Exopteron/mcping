use std::{
    borrow::Cow,
    process::ExitCode,
    time::Duration,
};

use clap::Parser;
use env_logger::Builder;
use log::LevelFilter;

use crate::{
    input::selector::{self, Selected},
    pinging::{mc_legacy::LegacyPinger, mc_modern::ModernPinger, Pinger},
};

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
                return ExitCode::FAILURE;
            }
        } else {
            25565
        };
        (hostname, port)
    };

    let lookup = match resolution::resolve_minecraft_ips(name.clone()) {
        Ok(v) => v,
        Err(e) => {
            log::error!("IP resolution failure.");
            log::debug!("details: {:?}", e);
            return ExitCode::FAILURE;
        }
    };

    let addresses_to_ping = if lookup.len() > 1 {
        log::info!("multiple addresses found. which should we use? (-1 for all)");
        match selector::select_one_of(lookup.iter()).unwrap() {
            Selected::Value(v) => vec![*v],
            Selected::Special(_) => lookup.into_iter().collect(),
        }
    } else {
        vec![lookup.into_iter().next().unwrap()]
    };


    let len = addresses_to_ping.len();
    for (idx, address_to_ping) in addresses_to_ping.into_iter().enumerate() {
        log::info!("attempting to ping {}...", address_to_ping);

        let res = (|| {
            let l = ModernPinger {
                protocol_version: -1,
                hostname: name.0.clone(),
                read_timeout: Duration::from_secs(5),
            };
            let data = l.ping(address_to_ping);
    
            match data {
                Ok(data) => {
                    log::info!("server description:\n{}", data.response.description);
    
                    if let Some(mods) = data.response.mods {
                        log::info!(
                            "server uses the {:?} mod software and has {} installed mods.",
                            mods.ty,
                            mods.mod_list.len()
                        );
                        log::debug!("mods: {:?}", mods.mod_list);
                    }
    
                    log::info!(
                        "server version:\n   --- {:?}\n   --- protocol version {}",
                        data.response.version.name,
                        data.response.version.protocol
                    );
    
                    if let Some(players) = data.response.players {
                        log::info!("players:\n   --- {}/{}", players.online, players.max);
    
                        if !players.sample.is_empty() {
                            log::info!("sample:");
                            for v in players.sample {
                                print!("   --- {}", v.name);
                                if options.verbose {
                                    print!(" (uuid {})", v.id);
                                }
                                println!();
                            }
                        }
                    } else {
                        log::info!("server is not announcing player count.");
                    }
    
                    log::info!("[{}] ping: {}ms", address_to_ping, data.latency.as_millis());
                }
                Err(e) => {
                    log::info!("standard ping failed. attempting legacy ping...");
                    log::debug!("failure reason: {:?}", e);
                    let l = LegacyPinger {
                        protocol_version: 0,
                        hostname: name.0.clone(),
                    };
    
                    let data = match l.ping(address_to_ping) {
                        Ok(v) => v,
                        Err(e) => {
                            log::error!("legacy ping failed. {:?}", e);
                            return ExitCode::FAILURE;
                        }
                    };
    
                    log::info!("motd:");
                    println!("   --- {}", get_or(data.motd, "none"));
                    log::info!("players:");
                    println!(
                        "   --- {}/{}",
                        get_or(data.online_players, "none"),
                        get_or(data.max_players, "none")
                    );
    
                    if let Some(extra) = data.extra {
                        log::info!("protocol version:");
                        println!("   --- {}", get_or(extra.protocol_version, "none"));
                        log::info!("server version:");
                        println!("   --- {}", get_or(extra.server_version, "none"));
                    }
    
                    if !data.unrecognised.is_empty() {
                        log::debug!(
                            "unrecognised fields in ping response: {:?}",
                            data.unrecognised
                        );
                    }
                }
            }
            ExitCode::SUCCESS
        })();

        if idx == len {
            return res;
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
        .filter(
            None,
            if !verbose {
                LevelFilter::Info
            } else {
                LevelFilter::Debug
            },
        )
        .init();
}
