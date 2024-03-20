use std::env;
use std::error::Error;
use std::path::Path;
use std::str::FromStr;
use std::thread;
use std::{
    ffi::OsStr,
    fs::File,
    io::{self, Seek},
    net::{SocketAddr, TcpListener, TcpStream, UdpSocket},
    path::PathBuf,
    str::from_utf8,
    time::Duration,
};

struct ServerInfo {
    address: SocketAddr,
    file_path: PathBuf,
}

const PORT: u16 = 55674;

fn main() -> Result<(), Box<dyn Error>> {
    let address = SocketAddr::from(([0, 0, 0, 0], PORT));
    let broadcast_s = UdpSocket::bind(address)?;
    broadcast_s.set_broadcast(true)?;

    if env::args().len() == 2 {
        // server
        let path_str = env::args().nth(1).unwrap();
        let path_str_move = path_str.clone();

        thread::spawn(move || {
            announce(broadcast_s, Path::new(&path_str_move).file_name().unwrap())
        });

        let path = PathBuf::from_str(&path_str).unwrap();
        serve_file(ServerInfo { address, file_path: path });
    } else {
        // client
        let server = discover(broadcast_s);
        receive_file(server);
    }

    Ok(())
}

fn announce(s: UdpSocket, file_name: &OsStr) {
    let addr = SocketAddr::from(([255, 255, 255, 255], PORT));
    loop {
        s.send_to(file_name.as_encoded_bytes(), addr).unwrap();
        thread::sleep(Duration::from_secs(5));
    }
}

fn discover(s: UdpSocket) -> ServerInfo {
    let mut buf = [0; 512];
    let (data_len, address) = s.recv_from(&mut buf).unwrap();
    let file_name = PathBuf::from(from_utf8(&buf[..data_len]).unwrap().to_owned());

    ServerInfo {
        address,
        file_path: file_name,
    }
}

fn serve_file(server: ServerInfo) {
    let socket = TcpListener::bind(server.address).unwrap();

    let mut file = File::open(server.file_path).unwrap();
    for client in socket.incoming() {
        let mut client = client.unwrap();
        file.seek(io::SeekFrom::Start(0)).unwrap();
        io::copy(&mut file, &mut client).unwrap();
    }
}

fn receive_file(server: ServerInfo) {
    let mut socket = TcpStream::connect(server.address).unwrap();
    let mut file = File::options()
        .read(true)
        .write(true)
        .create_new(true)
        .open(server.file_path)
        .unwrap();
    io::copy(&mut socket, &mut file).unwrap();
}
