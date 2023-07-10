use crate::geom::{Point, Segment};
use crate::io::{
    Attendee, ScoreExtended, Solution, Task, MUSICIAN_BLOCK_RADIUS, MUSICIAN_RADIUS, SCORE_CONST,
};
use anyhow::{bail, Result};
use float_ord::FloatOrd;
use rayon::prelude::*;
use std::cmp::Ordering;
use std::collections::BTreeMap;
use std::sync::Mutex;

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

        let min_pos = solution.placements.iter().enumerate()
        .filter(|(index, _)| {
            i != *index
        })
        .min_by(|(_, &a), (_, &b)| {
            c.dist_sqr(a).partial_cmp(&c.dist_sqr(b)).unwrap()
        });

        if let Some((pos, min_dist_coord)) = min_pos {
            if min_dist_coord.dist_sqr(*c) < musician_radius_sqr {
                bail!("Musician {i} is too close to musician {pos}")
            }
        }

        Result::Ok(())
    }).collect()
}

#[derive(Clone, Debug)]
pub struct Visibility {
    pub visibility: Vec<Vec<bool>>,
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
                        let musicians_clear = solution
                            .placements
                            .iter()
                            .enumerate()
                            .filter(|(i, _)| *i != index)
                            .all(|(_, c)| segment.dist(*c) >= MUSICIAN_BLOCK_RADIUS);
                        let pillars_clear = task
                            .pillars
                            .iter()
                            .all(|p| segment.dist(p.point()) >= p.radius);
                        musicians_clear && pillars_clear
                    })
                    .collect()
            })
            .collect(),
    }
}

pub fn attendee_score_without_q(
    attendee: &Attendee,
    instrument: usize,
    musician_coord: Point,
) -> i64 {
    let weight = attendee.tastes[instrument];
    (weight * SCORE_CONST / musician_coord.dist_sqr(attendee.coord())).ceil() as i64
}

pub fn calc_musician2q(task: &Task, solution: &Solution) -> Vec<f64> {
    let mut result = vec![1.0; task.musicians.len()];

    // Aymeric Fromherz — Вчера, в 23:01
    // ...you can also see it as active if and only if pillars is not empty in the problem description.

    if task.pillars.is_empty() {
        return result;
    }

    for (mus_idx, mus_instr) in task.musicians.iter().enumerate() {
        let mus_pos = solution.placements[mus_idx];

        for other_mus_idx in (mus_idx + 1)..task.musicians.len() {
            let other_mus_instr = task.musicians[other_mus_idx];
            if *mus_instr != other_mus_instr {
                continue;
            }
            let other_mus_pos = solution.placements[other_mus_idx];
            let d = 1.0 / mus_pos.dist(other_mus_pos);
            result[mus_idx] += d;
            result[other_mus_idx] += d;
        }
    }
    result
}

pub fn calc(task: &Task, solution: &Solution, visibility: &Visibility) -> Result<i64> {
    let musician2q = calc_musician2q(task, solution);

    validate(task, solution).map(|_| {
        task.attendees
            .par_iter()
            .enumerate()
            .map(|(attendee_index, a)| {
                visibility
                    .for_attendee(attendee_index)
                    .map(|index| {
                        let score = attendee_score_without_q(
                            a,
                            task.musicians[index],
                            solution.placements[index],
                        );

                        // Volumes should be allowed even on lightning tasks
                        let mut score = (score as f64) * solution.volumes[index];

                        // Aymeric Fromherz — Вчера, в 23:01
                        // ...you can also see it as active if and only if pillars is not empty in the problem description.
                        if !task.pillars.is_empty() {
                            // Aymeric Fromherz — Вчера, в 18:55
                            // It is implemented as calling ceil when computing I_i(k), and ceil again after multiplying with q(k), as indicated in the spec.
                            score *= musician2q[index];
                        }

                        score.ceil() as i64
                    })
                    .sum::<i64>()
            })
            .sum()
    })
}

pub fn calc_ex(task: &Task, solution: &Solution, visibility: &Visibility) -> ScoreExtended {
    let musician2q = calc_musician2q(task, solution);
    let mut musician = vec![0; solution.placements.len()];
    let mut attendee = vec![0; task.attendees.len()];

    let score = validate(task, solution)
        .map(|_| {
            task.attendees
                .iter()
                .enumerate()
                .map(|(attendee_index, a)| {
                    visibility
                        .for_attendee(attendee_index)
                        .map(|index| {
                            let score = attendee_score_without_q(
                                a,
                                task.musicians[index],
                                solution.placements[index],
                            );

                            // Volumes should be allowed even on lightning tasks
                            let mut score = (score as f64) * solution.volumes[index];

                            // Aymeric Fromherz — Вчера, в 23:01
                            // ...you can also see it as active if and only if pillars is not empty in the problem description.

                            if !task.pillars.is_empty() {
                                // Aymeric Fromherz — Вчера, в 18:55
                                // It is implemented as calling ceil when computing I_i(k), and ceil again after multiplying with q(k), as indicated in the spec.
                                score *= musician2q[index];
                            }

                            let score = score.ceil() as i64;

                            attendee[attendee_index] += score;
                            musician[index] += score;
                            score
                        })
                        .sum::<i64>()
                })
                .sum()
        })
        .unwrap_or(-1_000_000_000_000);

    ScoreExtended {
        score,
        attendee,
        musician,
    }
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
                    attendee_score_without_q(
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

fn get_angle_comparator(p0: Point) -> impl FnMut(&Point, &Point) -> Ordering {
    move |p1, p2| {
        let v01 = *p1 - p0;
        let v02 = *p2 - p0;
        let a1 = v01.atan2();
        let a2 = v02.atan2();
        return a1.partial_cmp(&a2).unwrap();
    }
}

pub fn calc_visibility_fast(task: &Task, solution: &Solution) -> Visibility {
    let mut result = Mutex::new(vec![vec![true; task.musicians.len()]; task.attendees.len()]);

    solution
        .placements
        .par_iter()
        .enumerate()
        .for_each(|(pos_index, pos)| {
            // TODO pillars require more complex logic
            let mut obstacles = solution.placements[0..pos_index]
                .iter()
                .map(|p| (*p, MUSICIAN_BLOCK_RADIUS))
                .chain(
                    solution.placements[pos_index + 1..]
                        .iter()
                        .map(|p| (*p, MUSICIAN_BLOCK_RADIUS)),
                )
                .chain(task.pillars.iter().map(|p| (p.point(), p.radius)))
                .map(|(p, r)| (p - *pos, r))
                .collect::<Vec<_>>();

            let mut crossing_zero_distances = BTreeMap::new();

            let mut obstacles = obstacles
                .into_iter()
                .flat_map(|(p, r)| {
                    let d = p.norm();
                    let alpha = (r / d).asin();
                    let theta = p.atan2();
                    let mut a1 = theta - alpha;
                    if a1 < -std::f64::consts::PI {
                        a1 += std::f64::consts::TAU;
                    }
                    let mut a2 = theta + alpha;
                    if a2 > std::f64::consts::PI {
                        a2 -= std::f64::consts::TAU;
                    }

                    // if a1.is_nan() || a2.is_nan() {
                    //     dbg!((pos_index, pos, p, r, d, alpha, theta, a1, a2, a2 < a1));
                    // }

                    if a2 < a1 {
                        *crossing_zero_distances.entry(FloatOrd(d)).or_insert(0) += 1;
                    }
                    vec![(a1, true, d), (a2, false, d)].into_iter()
                })
                .collect::<Vec<_>>();
            obstacles.sort_by(|(a, _, _), (b, _, _)| a.partial_cmp(b).unwrap());

            let mut attendees = task
                .attendees
                .iter()
                .enumerate()
                .map(|(i, a)| {
                    let v_from_pos = a.coord() - *pos;
                    let angle = v_from_pos.atan2();
                    let d = v_from_pos.norm();
                    (i, angle, d)
                })
                .collect::<Vec<_>>();
            attendees.sort_by(|(_, a, _), (_, b, _)| a.partial_cmp(b).unwrap());

            // keep that previous angle
            let mut entered_distances = crossing_zero_distances;
            let mut obstacles = obstacles.into_iter().peekable();

            for (att_index, att_angle, att_dist) in attendees {
                while obstacles.peek().is_some() {
                    let (obstacle_angle, enter, obstacle_center_dist) = obstacles.peek().unwrap();
                    if obstacle_angle <= &att_angle {
                        let k = FloatOrd(*obstacle_center_dist);
                        if *enter {
                            *entered_distances.entry(k).or_insert(0) += 1;
                        } else {
                            let v = entered_distances.get_mut(&k).unwrap();
                            *v -= 1;
                            if *v == 0 {
                                entered_distances.remove(&k);
                            }
                        }
                        obstacles.next();
                    } else {
                        break;
                    }
                }
                let has_close_obstacle = entered_distances
                    .range(..=FloatOrd(att_dist))
                    .next()
                    .is_some();
                {
                    let mut result = result.lock().unwrap();
                    result[att_index][pos_index] = !has_close_obstacle;
                }
            }
        });

    Visibility {
        visibility: result.into_inner().unwrap(),
    }
}
