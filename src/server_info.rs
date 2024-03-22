use std::thread;
use std::{
    fs::File,
    io::{self, Seek},
    net::{SocketAddr, TcpListener, TcpStream, UdpSocket},
    path::PathBuf,
    str::from_utf8,
    time::Duration,
};

pub struct ServerInfo {
    pub address: SocketAddr,
    pub file_path: PathBuf,
}

impl ServerInfo {
    pub fn new(address: SocketAddr, file_path: String) -> ServerInfo {
        let file_path = PathBuf::from(&file_path);
        ServerInfo { address, file_path }
    }

    pub fn discover(address: SocketAddr) -> Result<ServerInfo, io::Error> {
        let mut buf = [0; 512];

        let socket = UdpSocket::bind(address)?;
        let (data_len, address) = socket.recv_from(&mut buf).unwrap();
        let file_path = PathBuf::from(from_utf8(&buf[..data_len]).unwrap().to_owned());

        Ok(ServerInfo { address, file_path })
    }

    pub fn serve_file(&self) -> Result<(), io::Error> {
        let socket = TcpListener::bind(self.address)?;

        let mut file = File::open(&self.file_path)?;
        for mut client in socket.incoming().filter_map(|x| x.ok()) {
            file.seek(io::SeekFrom::Start(0))?;
            io::copy(&mut file, &mut client)?;
        }

        Ok(())
    }

    pub fn reveive_file(&self) -> Result<(), io::Error> {
        let mut socket = TcpStream::connect(self.address)?;
        let mut file = File::options()
            .read(true)
            .write(true)
            .create_new(true)
            .open(self.file_path.file_name().unwrap())?;
        io::copy(&mut socket, &mut file)?;

        Ok(())
    }

    pub fn start_announce(&self) -> Result<(), io::Error> {
        let broadcast_s = UdpSocket::bind(self.address)?;
        broadcast_s.set_broadcast(true)?;

        let file_name = self.file_path.file_name().unwrap().to_owned();
        let addr = SocketAddr::from(([255, 255, 255, 255], self.address.port()));

        thread::spawn(move || loop {
            broadcast_s
                .send_to(file_name.as_encoded_bytes(), addr)
                .unwrap();
            thread::sleep(Duration::from_secs(5));
        });

        Ok(())
    }
}
