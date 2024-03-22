mod server_info;

use server_info::ServerInfo;
use std::{
    env,
    error::Error,
    io::{self, Write},
    net::SocketAddr,
};

const SERVER_PORT: u16 = 55674;
const CLIENT_PORT: u16 = 55675;

fn server(address: SocketAddr, target_port: u16) -> io::Result<()> {
    let server = ServerInfo::new(address, env::args().nth(1).expect("infallible"));
    server.start_announce(target_port)?;
    server.serve_file()?;

    Ok(())
}

fn client(address: SocketAddr) -> io::Result<()> {
    let remote_server = ServerInfo::discover(address)?;
    println!("File found:");
    println!("{remote_server}");

    if !ask("Do you want to download?") {
        return Ok(());
    }

    let remote_file = remote_server
        .file_path
        .file_name()
        .unwrap()
        .to_str()
        .unwrap()
        .to_string(); // blergh
    let mut target_file = remote_file.clone();
    if remote_server.file_path.exists() {
        let input = prompt("File exists, new name (empty for overwrite):");
        if !input.is_empty() {
            target_file = input;
        }
    }

    println!("Retrieving {remote_file}, storing as {target_file}");
    remote_server.reveive_file(target_file)?;

    Ok(())
}

fn ask(question: &str) -> bool {
    let input = prompt((question.to_owned() + " [Y/n]").as_str());
    input.is_empty() || input == "y"
}

fn prompt(prompt: &str) -> String {
    print!("{prompt} ");
    io::stdout().flush().unwrap();
    read_input()
}

fn read_input() -> String {
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    input.trim().to_lowercase()
}

fn main() -> Result<(), Box<dyn Error>> {
    if env::args().len() == 2 {
        server(SocketAddr::from(([0, 0, 0, 0], SERVER_PORT)), CLIENT_PORT)?;
    } else {
        client(SocketAddr::from(([0, 0, 0, 0], CLIENT_PORT)))?;
    }

    Ok(())
}
