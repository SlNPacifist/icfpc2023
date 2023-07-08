use crate::geom::{Point, Vector};
use crate::io::MUSICIAN_RADIUS;
use crate::score::calc_visibility;
use crate::{
    io::{Solution, Task},
    score::{attendee_score, Visibility},
};
use std::collections::BinaryHeap;

pub fn optimize_placements_greedy(
    task: &Task,
    solution: &Solution,
    visibility: &Visibility,
) -> Solution {
    let mut moves = BinaryHeap::new();
    for instrument in 0..task.instruments_len() {
        for pos_index in 0..solution.placements.len() {
            let score = task
                .attendees
                .iter()
                .enumerate()
                .filter(|(index, _)| visibility.is_visible(*index, pos_index))
                .map(|(_, attendee)| {
                    attendee_score(attendee, instrument, solution.placements[pos_index])
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
    res
}

// TODO schedule
const OPTIMIZING_FORCE_MULTIPLIER: f64 = 1e-7;
const RELAXING_FORCE_MULTIPLIER: f64 = OPTIMIZING_FORCE_MULTIPLIER / 2.0;
// const RELAXING_FORCE_MULTIPLIER: f64 = 0.0;
const RELAXING_FORCE_BASE: f64 = 1e7;
// const STEPS: usize = 1000;
const STEPS: usize = 5;

const MUSICIAN_RADIUS_SQR: f64 = MUSICIAN_RADIUS * MUSICIAN_RADIUS;

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
) -> Solution {
    let mut result = initial_solution.clone();

    let mut optimizing_force_sched_multiplier = OPTIMIZING_FORCE_MULTIPLIER;
    let mut relaxing_force_sched_multiplier = RELAXING_FORCE_MULTIPLIER;

    for _ in 0..STEPS {
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
                        let force =
                            attendee_score(attendee, instrument, result.placements[pos_index])
                                as f64;
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

        // TODO proper schedule
        optimizing_force_sched_multiplier *= 0.999;
        relaxing_force_sched_multiplier *= 0.999;
    }

    result
}

pub fn force_greedy_combined(task: &Task, initial_solution: &Solution) -> (Solution, Visibility) {
    const STEPS: usize = 10;
    let mut result = initial_solution.clone();
    let mut visibility = calc_visibility(task, &result);
    for _ in 0..STEPS {
        result = force_based_optimizer(&task, &result, &visibility);
        visibility = calc_visibility(&task, &result);
        result = optimize_placements_greedy(&task, &result, &visibility);
        visibility = calc_visibility(&task, &result);
        // dbg!(crate::score::calc(&task, &result, &visibility));
    }

    (result, visibility)
}
