use crate::geom::{Point, Segment};
use crate::io::{Attendee, Solution, Task, MUSICIAN_BLOCK_RADIUS, MUSICIAN_RADIUS, SCORE_CONST};
use anyhow::{bail, Result};
use rayon::prelude::*;

pub fn validate(task: &Task, solution: &Solution) -> Result<()> {
    let solution_length = solution.placements.len();
    let task_length = task.musicians.len();
    if solution_length != task_length {
        bail!("Expected {task_length} musician positions, got {solution_length}");
    }

    let musician_radius_sqr = MUSICIAN_RADIUS * MUSICIAN_RADIUS;

    solution.placements.iter().enumerate().map(|(i, c)| {
        if !task.musician_in_stage(c.x, c.y) {
            bail!("Musician is not in stage. x={}, y={}, stage borders are left={}, right={}, bottom={}, top={}", c.x, c.y, task.stage_left(), task.stage_right(), task.stage_bottom(), task.stage_top());
        }

        let (pos, min_dist_coord) = solution.placements.iter().enumerate()
        .filter(|(index, _)| {
            i != *index
        })
        .min_by(|(_, &a), (_, &b)| {
            c.dist_sqr(a).partial_cmp(&c.dist_sqr(b)).unwrap()
        }).unwrap();

        if min_dist_coord.dist_sqr(*c) < musician_radius_sqr {
            bail!("Musician {i} is too close to musician {pos}")
        }

        Result::Ok(())
    }).collect()
}

pub struct Visibility {
    visibility: Vec<Vec<bool>>,
}

impl Visibility {
    pub fn is_visible(&self, attendee_index: usize, pos_index: usize) -> bool {
        self.visibility[attendee_index][pos_index]
    }

    pub fn for_attendee(&self, attendee_index: usize) -> impl Iterator<Item = usize> + '_ {
        self.visibility[attendee_index]
            .iter()
            .enumerate()
            .filter(|(_, v)| **v)
            .map(|(i, _)| i)
    }
}

pub fn calc_visibility(task: &Task, solution: &Solution) -> Visibility {
    Visibility {
        visibility: task
            .attendees
            .par_iter()
            .map(|a| {
                solution
                    .placements
                    .iter()
                    .enumerate()
                    .map(|(index, coord)| {
                        let segment = Segment {
                            from: a.coord(),
                            to: *coord,
                        };
                        solution
                            .placements
                            .iter()
                            .enumerate()
                            .filter(|(i, _)| *i != index)
                            .all(|(_, c)| segment.dist(*c) >= MUSICIAN_BLOCK_RADIUS)
                    })
                    .collect()
            })
            .collect(),
    }
}

pub fn attendee_score(attendee: &Attendee, instrument: usize, musician_coord: Point) -> i64 {
    let weight = attendee.tastes[instrument];
    (weight * SCORE_CONST / musician_coord.dist_sqr(attendee.coord())).ceil() as i64
}

pub fn calc(task: &Task, solution: &Solution, visibility: &Visibility) -> Result<i64> {
    validate(task, solution).map(|_| {
        task.attendees
            .par_iter()
            .enumerate()
            .map(|(attendee_index, a)| {
                visibility
                    .for_attendee(attendee_index)
                    .map(|index| {
                        attendee_score(a, task.musicians[index], solution.placements[index])
                    })
                    .sum::<i64>()
            })
            .sum()
    })
}

pub fn potential_score(task: &Task) -> i64 {
    let left_bottom = Point {
        x: task.stage_left(),
        y: task.stage_bottom(),
    };
    let left_top = Point {
        x: task.stage_left(),
        y: task.stage_top(),
    };
    let right_bottom = Point {
        x: task.stage_right(),
        y: task.stage_bottom(),
    };
    let right_top = Point {
        x: task.stage_right(),
        y: task.stage_top(),
    };
    let scene_segments = vec![
        Segment {
            from: left_bottom,
            to: left_top,
        },
        Segment {
            from: left_top,
            to: right_top,
        },
        Segment {
            from: right_top,
            to: right_bottom,
        },
        Segment {
            from: right_bottom,
            to: left_bottom,
        },
    ];
    task.attendees
        .iter()
        .map(|attendee| {
            task.musicians
                .iter()
                .map(|&instrument| {
                    let coord = attendee.coord();
                    let closest_dist = scene_segments
                        .iter()
                        .map(|segment| segment.dist(coord))
                        .min_by(|a, b| a.partial_cmp(b).unwrap())
                        .unwrap();
                    attendee_score(
                        attendee,
                        instrument,
                        Point {
                            x: coord.x + closest_dist + MUSICIAN_RADIUS,
                            y: coord.y,
                        },
                    )
                    .max(0)
                })
                .sum::<i64>()
        })
        .sum::<i64>()
}
