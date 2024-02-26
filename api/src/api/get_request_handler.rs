use serde_json::Value;

pub fn handle_get_request(request: Value) -> Option<Value> {
    println!("Get request detected");
    match request["info"].to_string().as_str() {
        "\"weather\"" => crate::weather::get_weather_json(),
        _ => None
    }
}