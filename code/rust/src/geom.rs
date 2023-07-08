use derive_more::{Add, Mul, Sum};
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

impl std::ops::Add<Vector> for Point {
    type Output = Point;

    fn add(self, other: Vector) -> Point {
        Point {
            x: self.x + other.x,
            y: self.y + other.y,
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

#[derive(Debug, Clone, Copy, Add, Sum, Mul)]
pub struct Vector {
    pub x: f64,
    pub y: f64,
}

impl Vector {
    pub fn dot(self, other: Self) -> f64 {
        self.x * other.x + self.y * other.y
    }
    pub fn atan2(self) -> f64 {
        f64::atan2(self.y, self.x)
    }
    pub fn norm(self) -> f64 {
        (self.x * self.x + self.y * self.y).sqrt()
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Segment {
    pub from: Point,
    pub to: Point,
}

impl Segment {
    pub fn dist(self, to: Point) -> f64 {
        // @see https://habr.com/ru/articles/148325/
        let v = self.to - self.from;
        let w0 = self.from - to;
        let w1 = self.to - to;

        if w0.dot(v) >= 0.0 {
            to.dist(self.from)
        } else if w1.dot(v) <= 0.0 {
            to.dist(self.to)
        } else {
            (((self.to.x - self.from.x) * (to.y - self.from.y)
                - (self.to.y - self.from.y) * (to.x - self.from.x))
                / self.to.dist(self.from))
            .abs()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Point, Segment};

    fn assert_f64_eq(a: f64, b: f64) {
        println!("{}, {}", a, b);
        assert!((a - b).abs() < 1e-6);
    }

    #[test]
    fn test_segment_dist() {
        let seg = Segment {
            from: Point { x: 0.0, y: 0.0 },
            to: Point { x: 5.0, y: 5.0 },
        };

        assert_f64_eq(seg.dist(Point { x: 2.0, y: 2.0 }), 0.0);
    }
}
