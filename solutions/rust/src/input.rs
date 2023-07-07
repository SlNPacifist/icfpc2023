use serde::{Deserialize, Serialize};
use std::fs;

#[derive(Serialize, Deserialize, Debug)]
pub struct Task {
    room_width: f64,
    room_height: f64,
    stage_width: f64,
    stage_height: f64,
    stage_bottom_left: ( f64, f64 ),
    musicians: Vec<usize>,
    attendees: Vec<Attendee>
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Attendee {
    x: f64,
    y: f64,
    tastes: Vec<f64>,
}

pub fn read(path: &str) -> Task {
    let data = fs::read_to_string(path).expect(&format!("Unable to read file {path}"));
    serde_json::from_str(&data).expect("Could not parse data")
}