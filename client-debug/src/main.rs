use std::net::{SocketAddr, TcpStream};
use anyhow::Result;
use std::thread;
use std::time::Duration;
use secure_common::encryption::Encrypted;
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
    let mut stream = TcpStream::connect(addr).expect("Cant connect to stream");
    let mut conn = Connection::establish(&mut stream).expect("Cant establish connection");
    conn.receive().expect("Can not receive handshake -> Panic");

    let json = json!({
        "type": "get",
        "info": "weather"
    });

    conn.send_json(json).unwrap();
    if let Ok(result) = conn.receive_json() {
        println!("Result received: {:?}", result)
    };

}

fn main() {
    handle_server("localhost:12345");
}
