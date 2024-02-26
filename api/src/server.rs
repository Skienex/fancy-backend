use std::net::{SocketAddr, TcpListener, TcpStream};
use std::thread;
use secure_common::encryption::Encrypted;
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
        // Send length information
        self.stream.send(&data.len().to_be_bytes()).expect("Can not send length information");

        // Send data
        self.stream.send(data).expect("Can not send data");

        Ok(())
    }

    pub fn receive(&mut self) -> Result<Vec<u8>> {
        let size_buf = self.stream.receive(8).expect("Can not receive length information");
        let size = u64::from_be_bytes(size_buf.try_into().expect("Can not convert bytes in u64"));

        self.stream.receive(size as usize)
    }

    pub fn send_json(&mut self, data: Value) -> Result<()> {
        let string = data.to_string();
        let bytes = string.as_bytes();
        if let Err(why) = self.send_bytes(bytes) {
            println!("Error: {why}")
        }

        Ok(())
    }

    pub fn receive_json(&mut self) -> Result<Value> {
        let data = String::from_utf8(self.receive().expect("Can not receive data")).expect("Can not parse Vec<u8> in String");

        let value = serde_json::from_str::<Value>(&data).expect("Can not parse received data into value");
        Ok(value)
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