use std::net::{SocketAddr, TcpStream};
use crypto_service::encryption::Encrypted;
use anyhow::Result;
use std::thread;
use std::time::Duration;
use serde_json::{json, Value};

pub struct Connection<'a> {
    pub stream: Encrypted<&'a mut TcpStream>,
}

impl<'a> Connection<'a> {
    pub fn establish(stream: &'a mut TcpStream) -> Result<Self> {
        let mut stream = Encrypted::request(stream)?;
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
    let mut stream = TcpStream::connect(addr).expect("Cant connect to stream");
    let mut conn = Connection::establish(&mut stream).expect("Cant establish connection");
    let json = json!({
        "name": "Timo"
    });

    loop {
        let bytes = conn.receive_json().expect("Cant receive something");
        if !bytes.to_string().is_empty() {
            println!("Message from server: {:?}", bytes.to_string());
            thread::sleep(Duration::from_millis(300));
            conn.send_json(json.clone()).expect("Cant send bytes");
        }
    }
}

fn main() {
    handle_server("localhost:12345");
}
