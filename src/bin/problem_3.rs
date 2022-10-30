use std::{
    io::{Read, Write, BufRead, ErrorKind},
    net::{Incoming, SocketAddr, SocketAddrV4, Ipv4Addr}, sync::{Arc, Mutex}, collections::{HashMap, hash_map::Entry::{Occupied, Vacant}},
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

        let user_joined_message = format!("User joined the room [{}]", name);
        let is_server = true;

        self.send_to_all(None, Some(addr), &user_joined_message, is_server);

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
        let name = match self.connections.entry(peer_addr) {
            Occupied(e) => {
                let name = e.get().name.clone();
                e.get().connection.shutdown(std::net::Shutdown::Both).expect("Shutdown not to fail");
                e.remove();
                name
            },
            Vacant(_) => {
                // User already disconnected
                return;
            },
        };

        println!("Socket disconnected {} [{}]", peer_addr, name);

        let is_server = true;
        let message = format!("User left the room [{}]", name);

        self.send_to_all(None, None, &message, is_server);
    }

    fn send_to_all(&mut self, name: Option<&str>, addr: Option<SocketAddr>, message: &str, is_server: bool) {
        let mut dead_peers = vec![];

        println!("User {:?} send message: {}", name, message);

        for (connection_addr, connection) in self.connections.iter_mut() {
            if let Some(addr) = addr{
                if *connection_addr == addr {
                    continue;
                }

            }

            match Self::send_to_connection(&mut connection.connection, name, message, is_server) {
                Ok(_) => {},
                Err(_) => {
                    // Error writing to this connection, disconnect them
                    dead_peers.push(*connection_addr);
                }
            };
        }
        
        // Disconnect dead peers
        for peer in dead_peers {
            println!("Unable to write to {}, disconnecting", peer);
            self.user_disconnected(peer);
        }
    }
    
    fn send_to_connection(connection: &mut std::net::TcpStream, name: Option<&str>, message: &str, is_server: bool)-> std::io::Result<()> {
        if is_server {
            connection.write_all("* ".as_bytes())?;
        }

        if let Some(name) = name {
            connection.write_all("[".as_bytes())?;
            connection.write_all(name.as_bytes())?;
            connection.write_all("] ".as_bytes())?;
        }

        connection.write_all(message.as_bytes())?;
        connection.write_all("\n".as_bytes())?;

        Ok(())
    }
}

impl Default for Connections {
    fn default() -> Self {
        Self { connections: Default::default()  }
    }
}

type ConMan = Arc<Mutex<Connections>>;

fn handle_thread(mut stream: std::net::TcpStream, con_man: ConMan) -> std::io::Result<()> {
    let addr = stream.peer_addr().unwrap();

    let mut buf_reader = std::io::BufReader::new(stream.try_clone().unwrap());

    stream.write_all("[SERVER] Please enter a name:\n".as_bytes())?;
    let mut buffer = String::new();
    let name_length = buf_reader.read_line(&mut buffer)?;

    if name_length == 0 {
        return Err(ErrorKind::BrokenPipe.into());
    }

    let name = buffer.trim();

    if !name.chars().all(char::is_alphanumeric) {
        stream.write_all("Name must only contain alphanumeric characters\n".as_bytes())?;
        return Err(ErrorKind::BrokenPipe.into());
    }

    let writer = stream.try_clone().unwrap();
    con_man.lock().unwrap().user_joined(name, writer)?;

    let mut buffer = String::new();
    loop {
        buffer.clear();

        let message_len = buf_reader.read_line(&mut buffer)?;
        if message_len == 0 {
            return Err(ErrorKind::BrokenPipe.into());
        }

        let message = buffer.trim();

        con_man.lock().unwrap().send_to_all(Some(name), Some(addr), message, false);
    }
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
