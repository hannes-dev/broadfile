use std::{net::{UdpSocket,TcpStream, TcpListener, SocketAddr}, time::Duration, fs::File, path::{Path, PathBuf}, io::{Write, self, Seek}, str::from_utf8, ffi::OsStr};
use std::error::Error;
use std::thread;
use std::env;

struct Server {
    address: SocketAddr,
    file_name: String,
}

const ADDRESS: &str = "0.0.0.0:12345";

fn main() -> Result<(), Box<dyn Error>> {
    let broadcast_s = UdpSocket::bind(ADDRESS)?;
    broadcast_s.set_broadcast(true)?;


    let args: Vec<String> = env::args().collect();
    if args.len() == 2 {
        // server
        let mut path = PathBuf::new();
        path.push(args[1].clone());
        let more_path = path.clone();
        thread::spawn(move || {
            announce(broadcast_s, more_path.file_name().unwrap())
        });
        let socket = TcpListener::bind(ADDRESS)?;
        
        let mut file = File::open(path)?;
        for client in socket.incoming() {
            // file.seek(0);
            io::copy(&mut file, &mut client?)?;
        }
    } else {
        // client
        let server = discover(broadcast_s);

        let mut socket = TcpStream::connect(server.address)?;
        let mut file = File::create(server.file_name)?;
        io::copy(&mut socket, &mut file)?;
    }

    Ok(())
}

fn announce(s: UdpSocket, file_name: &OsStr) {
    loop {
        s.send_to(file_name.as_encoded_bytes(), "255.255.255.255:55674").unwrap();
        thread::sleep(Duration::from_secs(5));
    }

}

fn discover(s: UdpSocket) -> Server {
    let mut buf = [0; 512];
    let (data_len, address) = s.recv_from(&mut buf).unwrap();
    let file_name = from_utf8(&buf[..data_len]).unwrap().to_owned();

    Server { address, file_name }
}
