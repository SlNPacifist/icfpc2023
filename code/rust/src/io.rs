use serde::{Deserialize, Serialize};
use std::fs;

pub const musician_radius: f64 = 10.0;

#[derive(Serialize, Deserialize, Debug)]
pub struct Task {
    pub room_width: f64,
    pub room_height: f64,
    pub stage_width: f64,
    pub stage_height: f64,
    pub stage_bottom_left: ( f64, f64 ),
    pub musicians: Vec<usize>,
    pub attendees: Vec<Attendee>
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Attendee {
    pub x: f64,
    pub y: f64,
    pub tastes: Vec<f64>,
}

impl Task {
    pub fn stage_left(&self) -> f64 {
        self.stage_bottom_left.0
    }

    pub fn stage_right(&self) -> f64 {
        self.stage_bottom_left.0 + self.stage_width
    }

    pub fn stage_bottom(&self) -> f64 {
        self.stage_bottom_left.1
    }

    pub fn stage_top(&self) -> f64 {
        self.stage_bottom_left.1 + self.stage_height
    }

    pub fn musician_in_stage(&self, x: f64, y: f64) -> bool {
        x >= self.stage_left() + musician_radius && x <= self.stage_right() - musician_radius && y >= self.stage_bottom() + musician_radius && y <= self.stage_top() - musician_radius
    }
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Solution {
    pub placements: Vec<Coord>
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Coord {
    pub x: f64,
    pub y: f64,
}

impl Coord {
    pub fn dist_sqr(&self, other: &Coord) -> f64 {
        (self.x - other.x) * (self.x - other.x) + (self.y - other.y) * (self.y - other.y)
    }
}

pub fn read(path: &str) -> Task {
    let data = fs::read_to_string(path).expect(&format!("Unable to read file {path}"));
    serde_json::from_str(&data).expect("Could not parse data")
}

pub fn write(path: &str, data: &Solution) {
    fs::write(path, serde_json::to_vec(data).expect("Could not serialize data")).expect("Got error when writing to file");
}