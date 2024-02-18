use std::{collections::HashSet, io, net::SocketAddr};

use hickory_resolver::Resolver;

pub fn resolve_minecraft_ips(provided_address: (String, u16)) -> io::Result<HashSet<SocketAddr>> {
    let resolver = Resolver::from_system_conf()?;

    let mut resolved_addresses: HashSet<SocketAddr> = HashSet::default();



    if let Ok(srv) = resolver.srv_lookup(format!("_minecraft._tcp.{}", provided_address.0))  {
        for v in srv {
            let target = resolver.lookup_ip(v.target().clone())?;

            for address in target {
                resolved_addresses.insert(SocketAddr::new(address, v.port()));
            }
        }
    } else {
        let target = resolver.lookup_ip(&provided_address.0)?;

        for address in target {
            resolved_addresses.insert(SocketAddr::new(address, provided_address.1));
        }
    }


    Ok(resolved_addresses)
}
