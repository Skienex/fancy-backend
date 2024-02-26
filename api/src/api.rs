mod get_request_handler;
mod set_request_handler;

use serde_json::Value;

pub fn handle_request(request: Value) -> Option<Value> {

    println!("Handeling request");
    println!("{:?}", request["type"].to_string().as_str());

    match request["type"].to_string().as_str() {
        "\"get\"" => get_request_handler::handle_get_request(request),
        "set" => set_request_handler::handle_set_request(request),
        "handshake" => None,
        _ => None
    }
}