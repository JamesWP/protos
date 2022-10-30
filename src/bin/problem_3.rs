use std::{
    io::{Read, Write, BufRead},
    net::{Incoming, SocketAddr}, sync::{Arc, Mutex}, collections::{HashMap, hash_map::Entry::{Occupied, Vacant}},
};
struct Connection {
    name: String,
    connection: std::net::TcpStream,
}
struct Connections {
    connections: HashMap<SocketAddr,Connection>
}

impl Connections {
    fn user_joined(&mut self, name: &str, mut stream: std::net::TcpStream) -> () {
        let addr = stream.peer_addr().unwrap();
        println!("User joined [{}] from {}", name, addr);

        match self.connections.entry(addr) {
            Occupied(_) => todo!("WAT"),
            Vacant(e) => {
                e.insert(Connection { name: name.to_owned(), connection: stream })
            },
        };
    }
}

impl Default for Connections {
    fn default() -> Self {
        Self { connections: Default::default()  }
    }
}

type ConMan = Arc<Mutex<Connections>>;

fn handle_thread(mut stream: std::net::TcpStream, con_man: ConMan) -> std::io::Result<()> {
    let mut buf_reader = std::io::BufReader::new(stream.try_clone().unwrap());

    stream.write_all("[SERVER] Please enter a name:\n".as_bytes())?;
    let mut buffer = String::new();
    buf_reader.read_line(&mut buffer)?;

    let name = buffer.trim();

    stream.write_all("[SERVER] Welcome [".as_bytes())?;
    stream.write_all(name.as_bytes())?;
    stream.write_all("]\n".as_bytes())?;

    let writer = stream.try_clone().unwrap();
    con_man.lock().unwrap().user_joined(name, writer);

    Ok(())
}

fn main() -> std::io::Result<()> {
    println!("Hello, world!");

    let listener = std::net::TcpListener::bind("0.0.0.0:20345")?;

    let con_man = ConMan::default();

    for stream in listener.incoming() {
        println!("got connection");
        let con_man = con_man.clone();
        if let Ok(stream) = stream {
            std::thread::spawn(move || {
                let _ = handle_thread(stream, con_man);
                println!("connection ded");
            });
        }
    }

    Ok(())
}
