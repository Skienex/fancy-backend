use std::net::{SocketAddr, TcpListener, TcpStream};
use std::thread;
use std::time::Duration;
use crypto_service::encryption::Encrypted;
use anyhow::Result;
use serde_json::{json, to_string, Value};
use crate::api::handle_request;

pub struct Connection<'a> {
    pub stream: Encrypted<&'a mut TcpStream>,
}

impl<'a> Connection<'a> {
    pub fn establish(stream: &'a mut TcpStream, adress: SocketAddr) -> Result<Self> {
        let mut stream = Encrypted::accept(stream)?;
        Ok(Self { stream })
    }

    pub fn send_bytes(&mut self, data: &[u8]) -> Result<()> {
        self.stream.send_bytes(data)
    }

    pub fn receive(&mut self) -> Result<Vec<u8>> {
        self.stream.receive_bytes()
    }

    pub fn send_json(&mut self, data: Value) -> Result<()> {
        self.stream.send_json(data)
    }

    pub fn receive_json(&mut self) -> Result<Value> {
        self.stream.receive_json()
    }
}

pub fn handle_server(addr: &str) {
    let listener = TcpListener::bind(addr).expect("Cant bind tcp listener");

    thread::scope(move |scope| loop {
        let (mut stream, address) = match listener.accept() {
            Ok(ok) => ok,
            Err(err) => {
                eprintln!("Error: {err}");
                break;
            }
        };
        let json = json!({
            "server": "handshake"
        });

        scope.spawn(move || {
            println!("Try to establish connection");
            let mut connection = Connection::establish(&mut stream, address).expect("Cant establish connection");
            println!("Established connection");
            connection.send_json(json.clone()).expect("Cant send bytes");
            println!("Sent handshake");
            loop {
                let bytes = connection.receive_json().expect("Cant receive bytes");
                if !bytes.to_string().is_empty() {
                    println!("Message from client: {:?}", bytes.to_string());
                    if let Some(value) = handle_request(bytes) {
                        connection.send_json(value).unwrap()
                    }
                }
            }
        });
    });
}