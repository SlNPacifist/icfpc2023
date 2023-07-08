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
    pub pillars: Vec<Pillar>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Pillar {
    pub center: (f64, f64),
    pub radius: f64,
}

impl Pillar {
    pub fn transpose(self) -> Self {
        Self {
            center: (self.center.1, self.center.0),
            radius: self.radius
        }
    }

    pub fn point(&self) -> Point {
        Point {
            x: self.center.0,
            y: self.center.1,
        }
    }
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
            stage_height: self.stage_width,
            stage_bottom_left: (self.stage_bottom_left.1, self.stage_bottom_left.0),
            musicians: self.musicians,
            attendees: self.attendees.into_iter().map(|a| a.transpose()).collect(),
            pillars: self.pillars.into_iter().map(|p| p.transpose()).collect(),
        }
    }

    pub fn simplify(self) -> Self {
        let total_instruments = self.attendees[0].tastes.len();
        for inst1 in 0..total_instruments {
            for inst2 in (inst1 + 1)..total_instruments {
                let same = self
                    .attendees
                    .iter()
                    .all(|a| a.tastes[inst1] == a.tastes[inst2]);
                if same {
                    println!("Every attendee has same taste for instrument {inst1} and {inst2}");
                    // TODO replace them with single instrument and single taste
                }
            }
        }

        self
    }

    pub fn musician_by_instrument(&self) -> Vec<Vec<usize>> {
        let mut res = vec![vec![]; self.instruments_len()];
        for (index, instrument) in self.musicians.iter().enumerate() {
            res[*instrument].push(index);
        }
        res
    }

    pub fn instruments_len(&self) -> usize {
        self.attendees[0].tastes.len()
    }
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
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

pub fn read_solution(path: &str) -> Solution {
    let data = fs::read_to_string(path).expect(&format!("Unable to read file {path}"));
    serde_json::from_str(&data).expect("Could not parse data")
}
