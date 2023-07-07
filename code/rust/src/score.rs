use crate::geom::Segment;
use crate::io::{Solution, Task, MUSICIAN_BLOCK_RADIUS, MUSICIAN_RADIUS, SCORE_CONST};
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
    pub fn is_visible(&self, attendee_index: usize, musician_index: usize) -> bool {
        self.visibility[attendee_index][musician_index]
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
            .enumerate()
            .map(|(attendee_index, a)| {
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
                            .all(|(_, c)| segment.dist(*c) > MUSICIAN_BLOCK_RADIUS)
                    })
                    .collect()
            })
            .collect(),
    }
}

pub fn calc(task: &Task, solution: &Solution, visibility: &Visibility) -> Result<f64> {
    validate(task, solution).map(|_| {
        task.attendees
            .par_iter()
            .enumerate()
            .map(|(attendee_index, a)| {
                visibility
                    .for_attendee(attendee_index)
                    .map(|index| {
                        let instrument = task.musicians[index];
                        let weight = a.tastes[instrument];
                        (weight * SCORE_CONST / solution.placements[index].dist_sqr(a.coord()))
                            .ceil()
                    })
                    .sum::<f64>()
            })
            .sum()
    })
}
