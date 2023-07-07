use crate::geom::Point;
use serde::{Deserialize, Serialize};
use std::fs;

pub const MUSICIAN_RADIUS: f64 = 10.0;
pub const MUSICIAN_BLOCK_RADIUS: f64 = 5.0;
pub const SCORE_CONST: f64 = 1000000.0;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Task {
    pub room_width: f64,
    pub room_height: f64,
    pub stage_width: f64,
    pub stage_height: f64,
    pub stage_bottom_left: (f64, f64),
    pub musicians: Vec<usize>,
    pub attendees: Vec<Attendee>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Attendee {
    pub x: f64,
    pub y: f64,
    pub tastes: Vec<f64>,
}

impl Attendee {
    pub fn transpose(self) -> Self {
        Self {
            x: self.y,
            y: self.x,
            tastes: self.tastes,
        }
    }

    pub fn coord(&self) -> Point {
        Point {
            x: self.x,
            y: self.y,
        }
    }
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
        x >= self.stage_left() + MUSICIAN_RADIUS
            && x <= self.stage_right() - MUSICIAN_RADIUS
            && y >= self.stage_bottom() + MUSICIAN_RADIUS
            && y <= self.stage_top() - MUSICIAN_RADIUS
    }

    pub fn transpose(self) -> Self {
        Self {
            room_width: self.room_height,
            room_height: self.room_width,
            stage_width: self.stage_height,
            stage_height: self.room_width,
            stage_bottom_left: (self.stage_bottom_left.1, self.stage_bottom_left.0),
            musicians: self.musicians,
            attendees: self.attendees.into_iter().map(|a| a.transpose()).collect(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Solution {
    pub placements: Vec<Point>,
}

impl Solution {
    pub fn transpose(self) -> Self {
        Self {
            placements: self.placements.into_iter().map(|c| c.transpose()).collect(),
        }
    }
}

pub fn read(path: &str) -> Task {
    let data = fs::read_to_string(path).expect(&format!("Unable to read file {path}"));
    serde_json::from_str(&data).expect("Could not parse data")
}

pub fn write(path: &str, data: &Solution) {
    fs::write(
        path,
        serde_json::to_vec(data).expect("Could not serialize data"),
    )
    .expect("Got error when writing to file");
}
