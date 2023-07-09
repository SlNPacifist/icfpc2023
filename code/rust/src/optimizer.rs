use crate::genetics;
use crate::geom::{Point, Vector};
use crate::io::MUSICIAN_RADIUS;
use crate::score::{self, calc, calc_visibility};
use crate::{
    io::{Solution, Task},
    score::{attendee_score_without_q, Visibility},
};
use std::collections::BinaryHeap;

static OPTIMIZERS: [(
    fn(&Task, &Solution, &Visibility) -> (Solution, Visibility),
    &'static str,
); 3] = [
    (default_force_based_optimizer, "Force based"),
    (optimize_placements_greedy, "Greedy placement"),
    (genetics::optimize_placements, "Genetic"),
];

pub fn optimize_placements_greedy(
    task: &Task,
    solution: &Solution,
    visibility: &Visibility,
) -> (Solution, Visibility) {
    let mut moves = BinaryHeap::new();
    for instrument in 0..task.instruments_len() {
        for pos_index in 0..solution.placements.len() {
            let score = task
                .attendees
                .iter()
                .enumerate()
                .filter(|(index, _)| visibility.is_visible(*index, pos_index))
                .map(|(_, attendee)| {
                    attendee_score_without_q(attendee, instrument, solution.placements[pos_index])
                })
                .sum::<i64>();
            moves.push((score, instrument, pos_index));
        }
    }

    let mut position_is_picked = vec![false; solution.placements.len()];
    let mut musician_by_instrument = task.musician_by_instrument();
    let mut res = solution.clone();

    while let Some((_, instrument, pos_index)) = moves.pop() {
        if position_is_picked[pos_index] || musician_by_instrument[instrument].is_empty() {
            continue;
        }

        let musician_id = musician_by_instrument[instrument]
            .pop()
            .expect("No musicians left");

        res.placements[musician_id] = solution.placements[pos_index];
        position_is_picked[pos_index] = true;
    }
    let visibility = calc_visibility(task, &res);
    (res, visibility)
}

// TODO schedule
const RELAXING_FORCE_BASE: f64 = 1e7;

const MUSICIAN_RADIUS_SQR: f64 = MUSICIAN_RADIUS * MUSICIAN_RADIUS;

pub struct ForceParams {
    steps: usize,
    refresh_visibility_rate: usize,

    optimizing_force_multiplier: f64,
    optimizing_force_decay: f64,
    relaxing_force_multiplier: f64,
    relaxing_force_decay: f64,
}

impl Default for ForceParams {
    fn default() -> Self {
        ForceParams {
            steps: 5,
            refresh_visibility_rate: 5,

            optimizing_force_multiplier: 1e-7,
            optimizing_force_decay: 0.999,
            relaxing_force_multiplier: 1e-7 / 2.0,
            relaxing_force_decay: 0.999,
        }
    }
}

fn run_force_based_step(
    task: &Task,
    start_solution: &Solution,
    visibility: &Visibility,
    force_collector: impl Fn(&Task, &Solution, &Visibility, usize, Point) -> Vector,
    power_multiplier: f64,
) -> Solution {
    let mut new_positions = start_solution.clone();
    // TODO shuffle positions before iteration
    for (pos_index, old_position) in start_solution.placements.iter().enumerate() {
        let force = force_collector(task, start_solution, visibility, pos_index, *old_position);

        let force = force * power_multiplier;
        // println!("applying force: point #{pos_index} {old_position:?}, force mag {}", force.dot(force).sqrt());

        // TODO try multiple times with smaller steps in same directions
        let mut new_position = new_positions.placements[pos_index] + force;
        new_position.x = new_position.x.clamp(
            task.stage_left() + MUSICIAN_RADIUS,
            task.stage_right() - MUSICIAN_RADIUS,
        );
        new_position.y = new_position.y.clamp(
            task.stage_bottom() + MUSICIAN_RADIUS,
            task.stage_top() - MUSICIAN_RADIUS,
        );

        if new_positions
            .placements
            .iter()
            .enumerate()
            .filter(|(i, _)| *i != pos_index)
            .any(|(_, other_new_pos)| new_position.dist_sqr(*other_new_pos) < MUSICIAN_RADIUS_SQR)
        {
            continue;
        }

        new_positions.placements[pos_index] = new_position;
    }

    new_positions
}

pub fn force_based_optimizer(
    task: &Task,
    initial_solution: &Solution,
    visibility: &Visibility,
    params: ForceParams,
) -> (Solution, Visibility) {
    let mut result = initial_solution.clone();

    let mut optimizing_force_sched_multiplier = params.optimizing_force_multiplier;
    let mut relaxing_force_sched_multiplier = params.relaxing_force_multiplier;

    let mut visibility = visibility.clone();
    for step in 0..params.steps {
        // optimizing phase
        {
            let force_collector = |task: &Task,
                                   _start_solution: &Solution,
                                   visibility: &Visibility,
                                   pos_index: usize,
                                   old_position: Point| {
                task.attendees
                    .iter()
                    .enumerate()
                    .filter(|(index, _)| visibility.is_visible(*index, pos_index))
                    .map(|(_, attendee)| {
                        let instrument = task.musicians[pos_index];
                        let force = attendee_score_without_q(
                            attendee,
                            instrument,
                            result.placements[pos_index],
                        ) as f64;
                        (attendee.coord() - old_position) * force
                    })
                    .sum::<Vector>()
            };

            result = run_force_based_step(
                task,
                &result,
                &visibility,
                force_collector,
                optimizing_force_sched_multiplier,
            );
        }

        // relaxing phase
        {
            let force_collector = |_task: &Task,
                                   start_solution: &Solution,
                                   _visibility: &Visibility,
                                   pos_index: usize,
                                   old_position: Point| {
                start_solution
                    .placements
                    .iter()
                    .enumerate()
                    .filter(|(i, _)| *i != pos_index)
                    .map(|(_, other_musician_position)| {
                        let force = -1.0 * RELAXING_FORCE_BASE
                            / old_position.dist_sqr(*other_musician_position);
                        (*other_musician_position - old_position) * force
                    })
                    .sum::<Vector>()
            };

            result = run_force_based_step(
                task,
                &result,
                &visibility,
                force_collector,
                relaxing_force_sched_multiplier,
            );
        }

        optimizing_force_sched_multiplier *= params.optimizing_force_decay;
        relaxing_force_sched_multiplier *= params.relaxing_force_decay;

        if (step + 1) % params.refresh_visibility_rate == 0 {
            visibility = calc_visibility_fast(&task, &result);
        }
    }

    visibility = calc_visibility_fast(&task, &result);
    (result, visibility)
}

pub fn default_force_based_optimizer(
    task: &Task,
    initial_solution: &Solution,
    visibility: &Visibility,
) -> (Solution, Visibility) {
    force_based_optimizer(task, initial_solution, visibility, Default::default())
}

pub fn optimize_do_talogo(
    task: &Task,
    initial_solution: &Solution,
    visibility: Visibility,
) -> (Solution, Visibility) {
    let mut best_solution = initial_solution.clone();
    let mut best_visibility = visibility.clone();
    let mut max_score = match calc(task, initial_solution, &visibility) {
        Ok(res) => res,
        _ => return (best_solution, visibility),
    };

    let mut score_changed = true;
    while score_changed {
        score_changed = false;

        for (optimize, name) in OPTIMIZERS {
            let (solution, visibility) = optimize(&task, &best_solution, &best_visibility);

            match score::calc(&task, &solution, &visibility) {
                Ok(points) => {
                    println!("{name} solution got {points} points");
                    if points > max_score {
                        max_score = points;
                        best_solution = solution;
                        best_visibility = visibility;
                        score_changed = true;
                    }
                }
                Err(err) => {
                    println!("{name} solution is incorrect: {err}")
                }
            }
        }
    }

    (best_solution, best_visibility)
}
