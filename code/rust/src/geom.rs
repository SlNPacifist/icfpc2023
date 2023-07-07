use serde::{Deserialize, Serialize};
use std::ops::Sub;

#[derive(Serialize, Deserialize, Debug, Default, Clone, Copy)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

impl Point {
    pub fn dist_sqr(self, other: Point) -> f64 {
        (self.x - other.x) * (self.x - other.x) + (self.y - other.y) * (self.y - other.y)
    }

    pub fn dist(self, other: Point) -> f64 {
        self.dist_sqr(other).sqrt()
    }

    pub fn transpose(self) -> Self {
        Self {
            x: self.y,
            y: self.x,
        }
    }
}

impl Sub for Point {
    type Output = Vector;

    fn sub(self, other: Self) -> Vector {
        Vector {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Vector {
    pub x: f64,
    pub y: f64,
}

impl Vector {
    pub fn dot(self, other: Self) -> f64 {
        self.x * other.x + self.y * other.y
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Segment {
    pub from: Point,
    pub to: Point,
}

impl Segment {
    pub fn dist(self, to: Point) -> f64 {
        // @see https://ru.stackoverflow.com/questions/721414/Евклидова-геометрия-Расстояние-от-точки-до-отрезка
        let v = self.to - self.from;
        let w0 = self.from - to;
        let w1 = self.to - to;

        if w0.dot(v) <= 0.0 {
            to.dist(self.from)
        } else if w1.dot(v) <= 0.0 {
            to.dist(self.to)
        } else {
            // ((y0-y1)*x + (x0-x1)*y + (x0*y1-x1*y0))/dist(P0, P1)
            ((self.from.y - self.to.y) * to.x + (self.from.x - self.to.x) * to.y)
                / self.to.dist(self.from)
        }
    }
}
