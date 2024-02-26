use serde_json::Value;

mod server;
mod api;
mod weather;

fn main() {
    server::handle_server("localhost:12345");
}
