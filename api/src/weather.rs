use std::fs;
use openweathermap::{CurrentWeather, init, update};
use serde_json::{json, Value};

pub fn get_weather_json() -> Option<Value> {
    println!("Weather fetching");
    let receiver = openweathermap::init("Rheine", "metric", "de", fs::read_to_string("openweathermap.api.key").expect("Can not find api key").as_str(), 0);

    let weather = loop {
        let option = update(&receiver);
        if let Some(res) = option {
            if let Ok(current) = res {
                break
                current
            }
        }
    };
    println!("Got weather");
    
    let temp = weather.main.temp;
    let pressure = weather.main.pressure;
    let max_temp = weather.main.temp_max;
    let min_temp = weather.main.temp_min;
    let sea_level = weather.main.sea_level.unwrap_or(0.0);

    Some(json!({
        "type": "weather report",
        "temp": temp,
        "pressure": pressure,
        "max_temp": max_temp,
        "min_temp": min_temp,
        "sea_level": sea_level
    }))
}