mod server_info;

use server_info::ServerInfo;
use std::{
    env,
    error::Error,
    net::SocketAddr,
};

const PORT: u16 = 55674;

fn main() -> Result<(), Box<dyn Error>> {
    let address = SocketAddr::from(([0, 0, 0, 0], PORT));

    if env::args().len() == 2 {
        // server
        let server = ServerInfo::new(address, env::args().nth(1).expect("infallible"));
        server.start_announce()?;
        server.serve_file()?;
    } else {
        // client
        let remote_server = ServerInfo::discover(address)?;
        remote_server.reveive_file()?;
    }

    Ok(())
}
