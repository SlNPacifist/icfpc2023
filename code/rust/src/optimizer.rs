use crate::genetics;
use crate::geom::{Point, Vector};
use crate::io::{default_volumes_task, MUSICIAN_RADIUS};
use crate::score::{self, calc, calc_visibility, calc_visibility_fast};
use crate::{
    io::{Solution, Task},
    score::{attendee_score_without_q, Visibility},
};
use itertools::Itertools;
use std::collections::BinaryHeap;

use crate::solution::recalc_volumes;
use rand::distributions::{Distribution, Uniform};
use rand::{Rng, SeedableRng};
use rand_xoshiro::Xoshiro256PlusPlus;

static OPTIMIZERS: [(
    fn(&Task, &Solution, &Visibility) -> (Solution, Visibility),
    &'static str,
); 4] = [
    (default_force_based_optimizer, "Force based"),
    (big_step_force_based_optimizer, "Force based with big steps"),
    (optimize_placements_greedy, "Greedy placement"),
    (random_swap_positions, "Random swap positions"),
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

pub struct ForceParams {
    steps: usize,
    refresh_visibility_rate: usize,

    random_walk_multiplier: f64,
    random_walk_decay: f64,
    optimizing_force_multiplier: f64,
    optimizing_force_decay: f64,
    relaxing_force_multiplier: f64,
    relaxing_force_decay: f64,
}

impl Default for ForceParams {
    fn default() -> Self {
        ForceParams {
            steps: 100,
            refresh_visibility_rate: 10,

            random_walk_multiplier: 0.5,
            random_walk_decay: 0.9,
            optimizing_force_multiplier: 1.0,
            optimizing_force_decay: 0.999,
            relaxing_force_multiplier: 0.5,
            relaxing_force_decay: 0.999,
        }
    }
}

const FORCE_GAP_SIZE: f64 = 0.0;

fn run_force_based_step(
    task: &Task,
    start_solution: &Solution,
    visibility: &Visibility,
    mut force_collector: impl FnMut(&Task, &Solution, &Visibility, usize, Point) -> Vector,
    power_multiplier: f64,
    rng: &mut impl rand::Rng,
) -> Solution {
    let musician_dist_gap_sqr: f64 = (MUSICIAN_RADIUS + FORCE_GAP_SIZE) * (MUSICIAN_RADIUS + FORCE_GAP_SIZE);

    let mut new_positions = start_solution.clone();

    use rand::prelude::SliceRandom;

    let mut forces = start_solution
        .placements
        .iter()
        .enumerate()
        .map(|(pos_index, old_position)| {
            (
                pos_index,
                force_collector(task, start_solution, visibility, pos_index, *old_position),
            )
        })
        .collect::<Vec<_>>();

    let max_norm = forces
        .iter()
        .map(|(_, f)| f.norm())
        .max_by(|a, b| a.partial_cmp(b).unwrap())
        .unwrap();

    forces.shuffle(rng);

    // println!("max force norm: {max_norm}");

    forces
        .into_iter()
        .map(|(i, f)| (i, f * (1.0 / max_norm)))
        .for_each(|(pos_index, force)| {
            let force = force * power_multiplier;
            let cur_position = new_positions.placements[pos_index];
            // println!("applying force: point #{pos_index} {cur_position:?}, force norm {}", force.norm());

            // TODO try multiple times with smaller steps in same directions
            let mut new_position = cur_position + force;
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
                .any(|(_, other_new_pos)| {
                    new_position.dist_sqr(*other_new_pos) < musician_dist_gap_sqr
                })
            {
                return;
            }

            new_positions.placements[pos_index] = new_position;
        });

    new_positions
}

pub fn force_based_optimizer(
    task: &Task,
    initial_solution: &Solution,
    visibility: &Visibility,
    params: ForceParams,
) -> (Solution, Visibility) {
    let mut rng = Xoshiro256PlusPlus::seed_from_u64(42);

    // to avoid double borrowing
    let mut rng_angles = Xoshiro256PlusPlus::seed_from_u64(42);
    let angle_distr = Uniform::from(0.0..std::f64::consts::TAU);

    let mut result = initial_solution.clone();

    let mut random_walk_sched_multiplier = params.random_walk_multiplier;
    let mut optimizing_force_sched_multiplier = params.optimizing_force_multiplier;
    let mut relaxing_force_sched_multiplier = params.relaxing_force_multiplier;

    let mut visibility = visibility.clone();
    for step in 0..params.steps {
        // random walk phase
        {
            let force_collector = |_task: &Task,
                                   _start_solution: &Solution,
                                   _visibility: &Visibility,
                                   _pos_index: usize,
                                   _old_position: Point| {
                let angle = angle_distr.sample(&mut rng_angles);

                Vector {
                    x: angle.cos(),
                    y: angle.sin(),
                }
            };

            result = run_force_based_step(
                task,
                &result,
                &visibility,
                force_collector,
                random_walk_sched_multiplier,
                &mut rng,
            );
        }

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
                &mut rng,
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
                        let force = -1.0 / old_position.dist_sqr(*other_musician_position);
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
                &mut rng,
            );
        }

        random_walk_sched_multiplier *= params.random_walk_decay;
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

pub fn big_step_force_based_optimizer(
    task: &Task,
    initial_solution: &Solution,
    visibility: &Visibility,
) -> (Solution, Visibility) {
    force_based_optimizer(task, initial_solution, visibility, ForceParams {
        steps: 100,
        refresh_visibility_rate: 10,

        random_walk_multiplier: 0.5,
        random_walk_decay: 0.9,
        optimizing_force_multiplier: 100.0,
        optimizing_force_decay: 0.99,
        relaxing_force_multiplier: 2.0,
        relaxing_force_decay: 0.95,
    })
}

pub fn optimize_single_musicians(
    task: &Task,
    initial_solution: &Solution,
    visibility: &Visibility,
) -> (Solution, Visibility) {
    let mut solution = initial_solution.clone();
    let mut best_score = calc(task, &solution, &visibility).unwrap_or(-1000000000000i64);
    let angle_distr = Uniform::from(0.0..std::f64::consts::TAU);
    let dist_distr = Uniform::from(0.0..MUSICIAN_RADIUS);
    let mut rng = Xoshiro256PlusPlus::seed_from_u64(42);

    let mut optimized = true;
    while optimized {
        optimized = false;
        let mut best_res = None;
        for i in 0..solution.placements.len() {
            let org = solution.placements[i];
            for i in 0..30 {
                let angle = angle_distr.sample(&mut rng);

                let v = Vector {
                    x: angle.cos(),
                    y: angle.sin(),
                } *  dist_distr.sample(&mut rng);
                solution.placements[i] = solution.placements[i] + v;
                let visibility = calc_visibility(task, &solution);
                let score = calc(task, &solution, &visibility).unwrap_or(-1000000000000i64);
                let cur_best_score = best_res
                    .map(|(score, _)| score)
                    .unwrap_or(-1000000000000i64);
                if score > best_score && score > cur_best_score {
                    best_res = Some((score, v));
                }
                solution.placements[i] = org;
            }

            if let Some((score, v)) = best_res {
                solution.placements[i] = solution.placements[i] + v;
                best_score = score;
                optimized = true;
            }
        }
    }

    let visibility = calc_visibility_fast(task, &solution);
    (solution, visibility)
}

pub fn random_swap_positions(
    task: &Task,
    initial_solution: &Solution,
    _visibility: &Visibility,
) -> (Solution, Visibility) {
    // TODO lift me to do_talogo
    let mut rng = Xoshiro256PlusPlus::seed_from_u64(42);

    let mut solution = initial_solution.clone();

    let swap_count = task.musicians.len() / 20;
    let swap_count = swap_count.max(1);
    for _ in 0..swap_count {
        let from = rng.gen_range(0..task.musicians.len());
        let to = rng.gen_range(0..task.musicians.len());
        if from == to {
            continue;
        }

        solution.placements.swap(from, to);
        solution.volumes.swap(from, to);
    }

    let visibility = calc_visibility_fast(task, &solution);
    (solution, visibility)
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

        const TRIES: usize = 3;
        const CHAIN_LEN: usize = 4;

        let mut rng = Xoshiro256PlusPlus::seed_from_u64(42);
        use rand::prelude::SliceRandom;

        for _ in 0..TRIES {
            let mut try_solution = best_solution.clone();
            let mut try_visibility = best_visibility.clone();

            let (_, mut prev_name) = OPTIMIZERS.choose(&mut rng).unwrap();
            let mut chain_names = Vec::with_capacity(CHAIN_LEN);
            for _ in 0..CHAIN_LEN {
                let (optimize, name) = loop {
                    let (optimize, name) = OPTIMIZERS.choose(&mut rng).unwrap();
                    if prev_name != *name {
                        prev_name = name;
                        chain_names.push(name);
                        break (optimize, name);
                    }
                };

                try_solution.volumes = default_volumes_task(task);
                let (mut solution, visibility) = optimize(&task, &try_solution, &try_visibility);
                recalc_volumes(task, &mut solution, &visibility);

                try_solution = solution;
                try_visibility = visibility;
            }

            match score::calc(&task, &try_solution, &try_visibility) {
                Ok(points) => {
                    println!(
                        "Chain {} got {points} points",
                        chain_names.iter().join(" -> ")
                    );
                    if points > max_score {
                        max_score = points;
                        best_solution = try_solution;
                        best_visibility = try_visibility;
                        score_changed = true;
                    }
                }
                Err(err) => {
                    println!("Chain solution is incorrect: {err}")
                }
            }
        }
    }

    (best_solution, best_visibility)
}
