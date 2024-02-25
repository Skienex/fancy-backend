mod get_request_handler;
mod set_request_handler;

use std::fs;
use openssl::sha::Sha256;
use serde_json::Value;

pub fn handle_request(request: Value) -> Option<Value> {
    if request["password"].to_string().is_empty() {
        return None;
    }
    let password = request["password"].to_string();
    let password_bytes = password.as_bytes();
    let mut sha256 = Sha256::new();
    sha256.update(password_bytes);

    if sha256.finish().to_vec() != fs::read("password.key").expect("Cannot read password bytes") {
        println!("Client sent request with wrong password");
        return None;
    }

    match request["type"].to_string().as_str() {
        "get" => get_request_handler::handle_get_request(request),
        "set" => set_request_handler::handle_set_request(request),
        "handshake" => None,
        _ => None
    }
}