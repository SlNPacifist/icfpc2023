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

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Output {
    placements: Vec<Coord>
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Coord {
    x: f64,
    y: f64,
}

pub fn read(path: &str) -> Task {
    let data = fs::read_to_string(path).expect(&format!("Unable to read file {path}"));
    serde_json::from_str(&data).expect("Could not parse data")
}

pub fn write(path: &str, data: &Output) {
    fs::write(path, serde_json::to_vec(data).expect("Could not serialize data")).expect("Got error when writing to file");
}