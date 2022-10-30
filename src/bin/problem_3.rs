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
    fn user_joined(&mut self, name: &str, mut stream: std::net::TcpStream) -> std::io::Result<()> {
        let addr = stream.peer_addr().unwrap();
        println!("User joined [{}] from {}", name, addr);

        //send the new user a message that lists all present users names. 
        let user_list:Vec<String> = self.connections.values().map(|c|c.name.clone()).collect();
        let user_list = user_list.join(", ");

        stream.write_all("* The room contains: ".as_bytes())?;
        stream.write_all(user_list.as_bytes())?;
        stream.write_all(".\n".as_bytes())?;

        match self.connections.entry(addr) {
            Occupied(_) => {
                todo!("Socket addresses can be reused, previous user has been disconnected")
            },
            Vacant(e) => {
                e.insert(Connection { name: name.to_owned(), connection: stream })
            },
        };

        Ok(())
    }

    fn user_disconnected(&mut self, peer_addr: SocketAddr) {
        println!("Socket disconnected {}", peer_addr);
        self.connections.remove(&peer_addr); 
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
    con_man.lock().unwrap().user_joined(name, writer)?;

    Ok(())
}

fn main() -> std::io::Result<()> {
    println!("Hello, world!");

    let listener = std::net::TcpListener::bind("0.0.0.0:20345")?;

    let con_man = ConMan::default();

    for stream in listener.incoming() {
        println!("got connection");
        let con_man = con_man.clone();
        let con_disconect = con_man.clone();
        if let Ok(stream) = stream {
            std::thread::spawn(move || {
                let addr = stream.peer_addr().unwrap();
                let _ = handle_thread(stream, con_man);
                println!("connection ded");
                con_disconect.lock().unwrap().user_disconnected(addr);
            });
        }
    }

    Ok(())
}
