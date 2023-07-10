use crate::geom::{Point, Segment, Vector};
use crate::io::{
    default_volumes_task, MUSICIAN_BLOCK_RADIUS, MUSICIAN_RADIUS, MUSICIAN_RADIUS_SQR,
};
use crate::score::{self, calc, calc_ex, calc_visibility, calc_visibility_fast};
use crate::{genetics, solution};
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

type Optimizer =
    fn(&Task, &Solution, &Visibility, &mut Xoshiro256PlusPlus) -> (Solution, Visibility);
type OptimizerSlice = [(Optimizer, &'static str)];

const ALL_OPTIMIZERS: &OptimizerSlice = &[
    (rap::rapier_optimizer, "Rigid body based"),
    (default_force_based_optimizer, "Force based"),
    (big_step_force_based_optimizer, "Force based with big steps"),
    (big_gap_force_based_optimizer, "Force based with big gaps"),
    (
        big_gap_big_step_force_based_optimizer,
        "Force based with big gaps and big steps",
    ),
    (force_random_walk_optimizer, "Force based random walk"),
    (
        silent_musicians_together_force_based_optimizer,
        "Force based silent musicians together",
    ),
    (
        all_musicians_together_force_based_optimizer,
        "Force based musicians together",
    ),
    (
        musicians_out_of_the_way_force_based_optimizer,
        "Force based out of the way",
    ),
    (
        zeros_away_from_audience_optimizer,
        "Wrong taste away from audience",
    ),
    (optimize_placements_greedy_opt, "Greedy placement"),
    (random_swap_positions, "Random swap positions"),
    (random_change_positions, "Random change positions"),
    //(optimize_border, "Optimize border"),
];

const SAFE_OPTIMIZERS: &OptimizerSlice = &[
    (rap::rapier_optimizer, "Rigid body based"),
    (default_force_based_optimizer, "Force based"),
    (big_step_force_based_optimizer, "Force based with big steps"),
    (big_gap_force_based_optimizer, "Force based with big gaps"),
    (
        big_gap_big_step_force_based_optimizer,
        "Force based with big gaps and big steps",
    ),
    (force_random_walk_optimizer, "Force based random walk"),
    (
        silent_musicians_together_force_based_optimizer,
        "Force based silent musicians together",
    ),
    (
        all_musicians_together_force_based_optimizer,
        "Force based musicians together",
    ),
    (
        musicians_out_of_the_way_force_based_optimizer,
        "Force based out of the way",
    ),
    (
        zeros_away_from_audience_optimizer,
        "Wrong taste away from audience",
    ),
    (optimize_placements_greedy_opt, "Greedy placement"),
    //(optimize_border, "Optimize border"),
];

const FINAL_OPTIMIZERS: &OptimizerSlice = &[
    (rap::rapier_optimizer, "Rigid body based"),
    // (force_random_walk_optimizer, "Force based random walk"),
    (
        silent_musicians_together_force_based_optimizer,
        "Force based silent musicians together",
    ),
    // (
    //     all_musicians_together_force_based_optimizer,
    //     "Force based musicians together",
    // ),
    // (
    //     musicians_out_of_the_way_force_based_optimizer,
    //     "Force based out of the way",
    // ),
    // (
    //     zeros_away_from_audience_optimizer,
    //     "Wrong taste away from audience",
    // ),
    (optimize_placements_greedy_opt, "Greedy placement"),
];

fn optimize_placements_greedy_opt(
    task: &Task,
    solution: &Solution,
    visibility: &Visibility,
    _rng: &mut Xoshiro256PlusPlus,
) -> (Solution, Visibility) {
    optimize_placements_greedy(task, solution, visibility)
}

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

    force_gap_size: f64,
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

            force_gap_size: 0.0,
        }
    }
}

fn is_position_possible(
    solution: &Solution,
    pos_index: usize,
    new_position: Point,
    dist_sqr: f64,
) -> bool {
    solution
        .placements
        .iter()
        .enumerate()
        .filter(|(i, _)| *i != pos_index)
        .all(|(_, other_new_pos)| new_position.dist_sqr(*other_new_pos) >= dist_sqr)
}

fn run_force_based_step(
    task: &Task,
    start_solution: &Solution,
    visibility: &Visibility,
    mut force_collector: impl FnMut(
        &Task,
        &Solution,
        &Visibility,
        usize,
        Point,
        &mut Xoshiro256PlusPlus,
    ) -> Vector,
    power_multiplier: f64,
    force_gap_size: f64,
    rng: &mut Xoshiro256PlusPlus,
) -> Solution {
    let musician_dist_gap_sqr: f64 =
        (MUSICIAN_RADIUS + force_gap_size) * (MUSICIAN_RADIUS + force_gap_size);

    let mut new_positions = start_solution.clone();

    use rand::prelude::SliceRandom;

    let mut forces = start_solution
        .placements
        .iter()
        .enumerate()
        .map(|(pos_index, old_position)| {
            (
                pos_index,
                force_collector(
                    task,
                    start_solution,
                    visibility,
                    pos_index,
                    *old_position,
                    rng,
                ),
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

            if !is_position_possible(
                &new_positions,
                pos_index,
                new_position,
                musician_dist_gap_sqr,
            ) {
                return;
            }

            new_positions.placements[pos_index] = new_position;
        });

    new_positions
}

pub fn attract_musicians_to_attendees_force_collector(
    task: &Task,
    visibility: &Visibility,
    pos_index: usize,
    old_position: Point,
) -> Vector {
    task.attendees
        .iter()
        .enumerate()
        // todo too slow w/o filter?
        .filter(|(index, _)| visibility.is_visible(*index, pos_index))
        .map(|(att_idx, attendee)| {
            let visible = visibility.is_visible(att_idx, pos_index);
            let visible_k = if visible { 1.0 } else { 0.1 };

            let instrument = task.musicians[pos_index];
            let force = attendee_score_without_q(attendee, instrument, old_position) as f64;
            (attendee.coord() - old_position) * force * visible_k
        })
        .sum::<Vector>()
}

pub fn force_based_optimizer(
    task: &Task,
    initial_solution: &Solution,
    visibility: &Visibility,
    params: ForceParams,
    rng: &mut Xoshiro256PlusPlus,
) -> (Solution, Visibility) {
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
                                   _old_position: Point,
                                   rng: &mut Xoshiro256PlusPlus| {
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
                params.force_gap_size,
                rng,
            );
        }

        // optimizing phase
        {
            let force_collector =
                |task: &Task,
                 start_solution: &Solution,
                 visibility: &Visibility,
                 pos_index: usize,
                 old_position: Point,
                 _rng: &mut Xoshiro256PlusPlus| {
                    attract_musicians_to_attendees_force_collector(
                        task,
                        visibility,
                        pos_index,
                        old_position,
                    )
                };

            result = run_force_based_step(
                task,
                &result,
                &visibility,
                force_collector,
                optimizing_force_sched_multiplier,
                params.force_gap_size,
                rng,
            );
        }

        // same instrument - together
        if false && task.pillars.len() > 0 {
            let force_collector = |task: &Task,
                                   start_solution: &Solution,
                                   visibility: &Visibility,
                                   pos_index: usize,
                                   old_position: Point,
                                   rng: &mut Xoshiro256PlusPlus| {
                task.musicians
                    .iter()
                    .zip(start_solution.placements.iter())
                    .enumerate()
                    .filter(|(other_mus_idx, _)| *other_mus_idx != pos_index)
                    .filter(|(_, (other_instr, _))| **other_instr == task.musicians[pos_index])
                    .map(|(other_mus_idx, (_, other_pos))| {
                        let v = *other_pos - old_position;
                        v * (1.0 / v.norm())
                    })
                    .sum()
            };

            result = run_force_based_step(
                task,
                &result,
                &visibility,
                force_collector,
                optimizing_force_sched_multiplier,
                params.force_gap_size,
                rng,
            );
        }

        // moving out of the way from crossing
        if false {
            let force_collector = |task: &Task,
                                   start_solution: &Solution,
                                   visibility: &Visibility,
                                   pos_index: usize,
                                   old_position: Point,
                                   rng: &mut Xoshiro256PlusPlus| {
                const MAX_ATTENDEES: usize = 20;
                const MAX_SOURCE_MUSICIANS: usize = 20;

                let att_indices = rand::seq::index::sample(
                    rng,
                    task.attendees.len(),
                    task.attendees.len().min(MAX_ATTENDEES),
                );
                let mus_indices = rand::seq::index::sample(
                    rng,
                    task.musicians.len(),
                    task.musicians.len().min(MAX_SOURCE_MUSICIANS),
                );

                att_indices
                    .into_iter()
                    .map(|att_idx| (att_idx, &task.attendees[att_idx]))
                    .map(|(att_idx, att)| {
                        mus_indices
                            .iter()
                            .map(|source_m_idx| {
                                (
                                    source_m_idx,
                                    (
                                        &task.musicians[source_m_idx],
                                        &start_solution.placements[source_m_idx],
                                    ),
                                )
                            })
                            .filter(|(source_m_idx, _)| {
                                !visibility.is_visible(att_idx, *source_m_idx)
                            })
                            .filter(|(_, (instr, _))| att.tastes[**instr] > 0.0)
                            .filter(|(_, (_, source_pos))| {
                                let seg_source_att = Segment {
                                    from: **source_pos,
                                    to: att.coord(),
                                };
                                seg_source_att.dist(old_position) < MUSICIAN_BLOCK_RADIUS
                            })
                            .map(|(_, (_, source_pos))| {
                                let v_source_att = att.coord() - *source_pos;
                                let a_source_att = v_source_att.atan2();
                                let v_source_me = old_position - *source_pos;
                                let a_source_me = v_source_me.atan2();
                                let a_force = if a_source_att > a_source_me {
                                    a_source_att - std::f64::consts::FRAC_PI_2
                                } else {
                                    a_source_att + std::f64::consts::FRAC_PI_2
                                };

                                Vector {
                                    x: a_force.cos(),
                                    y: a_force.sin(),
                                }
                            })
                            .sum()
                    })
                    .sum()
            };

            result = run_force_based_step(
                task,
                &result,
                &visibility,
                force_collector,
                optimizing_force_sched_multiplier,
                params.force_gap_size,
                rng,
            );
        }

        // relaxing phase
        {
            let force_collector = |_task: &Task,
                                   start_solution: &Solution,
                                   _visibility: &Visibility,
                                   pos_index: usize,
                                   old_position: Point,
                                   rng: &mut Xoshiro256PlusPlus| {
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
                params.force_gap_size,
                rng,
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

pub fn single_force_optimizer(
    task: &Task,
    initial_solution: &Solution,
    visibility: &Visibility,
    params: ForceParams,
    mut force_collector: impl FnMut(
        &Task,
        &Solution,
        &Visibility,
        usize,
        Point,
        &mut Xoshiro256PlusPlus,
    ) -> Vector,
    rng: &mut Xoshiro256PlusPlus,
) -> (Solution, Visibility) {
    let mut result = initial_solution.clone();
    recalc_volumes(task, &mut result, &visibility);

    let mut random_walk_sched_multiplier = params.random_walk_multiplier;
    let mut optimizing_force_sched_multiplier = params.optimizing_force_multiplier;
    let mut relaxing_force_sched_multiplier = params.relaxing_force_multiplier;

    let mut visibility = visibility.clone();
    for step in 0..params.steps {
        result = run_force_based_step(
            task,
            &result,
            &visibility,
            &mut force_collector,
            optimizing_force_sched_multiplier,
            params.force_gap_size,
            rng,
        );

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

pub fn force_musicians_out_of_the_way(
    task: &Task,
    initial_solution: &Solution,
    visibility: &Visibility,
    params: ForceParams,
    rng: &mut Xoshiro256PlusPlus,
) -> (Solution, Visibility) {
    // moving out of the way from crossing
    let force_collector = |task: &Task,
                           start_solution: &Solution,
                           visibility: &Visibility,
                           pos_index: usize,
                           old_position: Point,
                           rng: &mut Xoshiro256PlusPlus| {
        const MAX_ATTENDEES: usize = 20;
        const MAX_SOURCE_MUSICIANS: usize = 20;

        let att_indices = rand::seq::index::sample(
            rng,
            task.attendees.len(),
            task.attendees.len().min(MAX_ATTENDEES),
        );
        let mus_indices = rand::seq::index::sample(
            rng,
            task.musicians.len(),
            task.musicians.len().min(MAX_SOURCE_MUSICIANS),
        );

        att_indices
            .into_iter()
            .map(|att_idx| (att_idx, &task.attendees[att_idx]))
            .map(|(att_idx, att)| {
                mus_indices
                    .iter()
                    .map(|source_m_idx| {
                        (
                            source_m_idx,
                            (
                                &task.musicians[source_m_idx],
                                &start_solution.placements[source_m_idx],
                            ),
                        )
                    })
                    .filter(|(source_m_idx, _)| !visibility.is_visible(att_idx, *source_m_idx))
                    .filter(|(_, (instr, _))| att.tastes[**instr] > 0.0)
                    .filter(|(_, (_, source_pos))| {
                        let seg_source_att = Segment {
                            from: **source_pos,
                            to: att.coord(),
                        };
                        seg_source_att.dist(old_position) < MUSICIAN_BLOCK_RADIUS
                    })
                    .map(|(_, (_, source_pos))| {
                        let v_source_att = att.coord() - *source_pos;
                        let a_source_att = v_source_att.atan2();
                        let v_source_me = old_position - *source_pos;
                        let a_source_me = v_source_me.atan2();
                        let a_force = if a_source_att > a_source_me {
                            a_source_att - std::f64::consts::FRAC_PI_2
                        } else {
                            a_source_att + std::f64::consts::FRAC_PI_2
                        };

                        Vector {
                            x: a_force.cos(),
                            y: a_force.sin(),
                        }
                    })
                    .sum()
            })
            .sum()
    };

    single_force_optimizer(
        task,
        initial_solution,
        visibility,
        params,
        force_collector,
        rng,
    )
}

pub fn musicians_out_of_the_way_force_based_optimizer(
    task: &Task,
    initial_solution: &Solution,
    visibility: &Visibility,
    rng: &mut Xoshiro256PlusPlus,
) -> (Solution, Visibility) {
    force_musicians_out_of_the_way(task, initial_solution, visibility, Default::default(), rng)
}

pub fn force_musicians_together(
    task: &Task,
    initial_solution: &Solution,
    visibility: &Visibility,
    params: ForceParams,
    rng: &mut Xoshiro256PlusPlus,
    only_silent: bool,
) -> (Solution, Visibility) {
    if task.pillars.len() == 0 {
        // q is not used in score
        return (initial_solution.clone(), visibility.clone());
    }

    let mut solution_with_volume = initial_solution.clone();
    recalc_volumes(task, &mut solution_with_volume, visibility);

    // silent same instrument - together
    let force_collector = |task: &Task,
                           start_solution: &Solution,
                           visibility: &Visibility,
                           pos_index: usize,
                           old_position: Point,
                           rng: &mut Xoshiro256PlusPlus| {
        if only_silent && start_solution.volumes[pos_index] > 0.0 {
            return Vector { x: 0.0, y: 0.0 };
        }
        task.musicians
            .iter()
            .zip(start_solution.placements.iter())
            .enumerate()
            .filter(|(other_mus_idx, _)| *other_mus_idx != pos_index)
            .filter(|(other_mus_idx, _)| start_solution.volumes[*other_mus_idx] > 0.0)
            .filter(|(_, (other_instr, _))| **other_instr == task.musicians[pos_index])
            .map(|(other_mus_idx, (_, other_pos))| {
                let v = *other_pos - old_position;
                v * (1.0 / v.norm())
            })
            .sum()
    };

    single_force_optimizer(
        task,
        &solution_with_volume,
        visibility,
        params,
        force_collector,
        rng,
    )
}

pub fn silent_musicians_together_force_based_optimizer(
    task: &Task,
    initial_solution: &Solution,
    visibility: &Visibility,
    rng: &mut Xoshiro256PlusPlus,
) -> (Solution, Visibility) {
    force_musicians_together(
        task,
        initial_solution,
        visibility,
        Default::default(),
        rng,
        true,
    )
}

pub fn all_musicians_together_force_based_optimizer(
    task: &Task,
    initial_solution: &Solution,
    visibility: &Visibility,
    rng: &mut Xoshiro256PlusPlus,
) -> (Solution, Visibility) {
    force_musicians_together(
        task,
        initial_solution,
        visibility,
        Default::default(),
        rng,
        false,
    )
}

pub fn force_random_walk_optimizer(
    task: &Task,
    initial_solution: &Solution,
    visibility: &Visibility,
    rng: &mut Xoshiro256PlusPlus,
) -> (Solution, Visibility) {
    let angle_distr = Uniform::from(0.0..std::f64::consts::TAU);

    // random_walk
    let force_collector = |_task: &Task,
                           _start_solution: &Solution,
                           _visibility: &Visibility,
                           _pos_index: usize,
                           _old_position: Point,
                           rng: &mut Xoshiro256PlusPlus| {
        let angle = angle_distr.sample(rng);

        Vector {
            x: angle.cos(),
            y: angle.sin(),
        }
    };

    single_force_optimizer(
        task,
        initial_solution,
        visibility,
        ForceParams::default(),
        force_collector,
        rng,
    )
}

pub fn zeros_away_from_audience_optimizer(
    task: &Task,
    initial_solution: &Solution,
    visibility: &Visibility,
    rng: &mut Xoshiro256PlusPlus,
) -> (Solution, Visibility) {
    let angle_distr = Uniform::from(0.0..std::f64::consts::TAU);

    // musicians with 0 or less taste for _every_ attendee move away from every atendee
    let force_collector = |_task: &Task,
                           start_solution: &Solution,
                           _visibility: &Visibility,
                           pos_index: usize,
                           old_position: Point,
                           rng: &mut Xoshiro256PlusPlus| {
        if task
            .attendees
            .iter()
            .any(|att| att.tastes[task.musicians[pos_index]] > 0.0)
        {
            return Vector { x: 0.0, y: 0.0 };
        }

        task.attendees
            .iter()
            .map(|att| {
                let v = old_position - att.coord();
                let v = v * (1.0 / v.norm());
                v * att.tastes[task.musicians[pos_index]]
            })
            .sum()
    };

    single_force_optimizer(
        task,
        initial_solution,
        visibility,
        ForceParams::default(),
        force_collector,
        rng,
    )
}

pub fn default_force_based_optimizer(
    task: &Task,
    initial_solution: &Solution,
    visibility: &Visibility,
    rng: &mut Xoshiro256PlusPlus,
) -> (Solution, Visibility) {
    force_based_optimizer(task, initial_solution, visibility, Default::default(), rng)
}

pub fn big_step_force_based_optimizer(
    task: &Task,
    initial_solution: &Solution,
    visibility: &Visibility,
    rng: &mut Xoshiro256PlusPlus,
) -> (Solution, Visibility) {
    force_based_optimizer(
        task,
        initial_solution,
        visibility,
        ForceParams {
            steps: 100,
            refresh_visibility_rate: 10,

            random_walk_multiplier: 0.5,
            random_walk_decay: 0.9,
            optimizing_force_multiplier: 100.0,
            optimizing_force_decay: 0.99,
            relaxing_force_multiplier: 2.0,
            relaxing_force_decay: 0.95,

            force_gap_size: 0.0,
        },
        rng,
    )
}

pub fn big_gap_force_based_optimizer(
    task: &Task,
    initial_solution: &Solution,
    visibility: &Visibility,
    rng: &mut Xoshiro256PlusPlus,
) -> (Solution, Visibility) {
    force_based_optimizer(
        task,
        initial_solution,
        visibility,
        ForceParams {
            force_gap_size: 1.0 * MUSICIAN_RADIUS,
            ..Default::default()
        },
        rng,
    )
}

pub fn big_gap_big_step_force_based_optimizer(
    task: &Task,
    initial_solution: &Solution,
    visibility: &Visibility,
    rng: &mut Xoshiro256PlusPlus,
) -> (Solution, Visibility) {
    force_based_optimizer(
        task,
        initial_solution,
        visibility,
        ForceParams {
            optimizing_force_multiplier: 100.0,
            optimizing_force_decay: 0.99,
            force_gap_size: 1.0 * MUSICIAN_RADIUS,
            ..Default::default()
        },
        rng,
    )
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
                } * dist_distr.sample(&mut rng);
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
    rng: &mut Xoshiro256PlusPlus,
) -> (Solution, Visibility) {
    let mut solution = initial_solution.clone();

    let swap_count = task.musicians.len() / 20;
    let swap_count = swap_count.min(5).max(2);
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

pub fn random_point_on_stage(task: &Task, rng: &mut impl rand::Rng) -> Point {
    let x_distr = if task.stage_width > 2.0 * MUSICIAN_RADIUS {
        Some(Uniform::from(
            task.stage_bottom_left.0 + MUSICIAN_RADIUS
                ..task.stage_bottom_left.0 + task.stage_width - MUSICIAN_RADIUS,
        ))
    } else {
        None
    };
    let y_distr = if task.stage_height > 2.0 * MUSICIAN_RADIUS {
        Some(Uniform::from(
            task.stage_bottom_left.1 + MUSICIAN_RADIUS
                ..task.stage_bottom_left.1 + task.stage_height - MUSICIAN_RADIUS,
        ))
    } else {
        None
    };

    Point {
        x: x_distr
            .map(|x| x.sample(rng))
            .unwrap_or(task.stage_bottom_left.0 + MUSICIAN_RADIUS),
        y: y_distr
            .map(|y| y.sample(rng))
            .unwrap_or(task.stage_bottom_left.1 + MUSICIAN_RADIUS),
    }
}

pub fn random_change_positions(
    task: &Task,
    initial_solution: &Solution,
    _visibility: &Visibility,
    rng: &mut Xoshiro256PlusPlus,
) -> (Solution, Visibility) {
    let x_distr = if task.stage_width > 2.0 * MUSICIAN_RADIUS {
        Some(Uniform::from(
            task.stage_bottom_left.0 + MUSICIAN_RADIUS
                ..task.stage_bottom_left.0 + task.stage_width - MUSICIAN_RADIUS,
        ))
    } else {
        None
    };
    let y_distr = if task.stage_height > 2.0 * MUSICIAN_RADIUS {
        Some(Uniform::from(
            task.stage_bottom_left.1 + MUSICIAN_RADIUS
                ..task.stage_bottom_left.1 + task.stage_height - MUSICIAN_RADIUS,
        ))
    } else {
        None
    };

    let mut solution = initial_solution.clone();

    let changes_count = task.musicians.len() / 10;
    let changes_count = changes_count.max(1);
    for _ in 0..changes_count {
        let pos_idx = rng.gen_range(0..task.musicians.len());

        let new_pos = Point {
            x: x_distr
                .map(|x| x.sample(rng))
                .unwrap_or(task.stage_bottom_left.0 + MUSICIAN_RADIUS),
            y: y_distr
                .map(|y| y.sample(rng))
                .unwrap_or(task.stage_bottom_left.1 + MUSICIAN_RADIUS),
        };

        if is_position_possible(&solution, pos_idx, new_pos, MUSICIAN_RADIUS_SQR) {
            solution.placements[pos_idx] = new_pos;
        }
    }

    let visibility = calc_visibility_fast(task, &solution);
    (solution, visibility)
}

pub fn optimize_border(
    task: &Task,
    initial_solution: &Solution,
    initial_visibility: &Visibility,
    _rng: &mut Xoshiro256PlusPlus,
) -> (Solution, Visibility) {
    let mut valid_points = vec![];
    let mut x = task.stage_left() + MUSICIAN_RADIUS;
    let mut y = task.stage_bottom() + MUSICIAN_RADIUS;
    valid_points.push(Point { x, y });
    while y <= task.stage_top() - 2.0 * MUSICIAN_RADIUS {
        y += MUSICIAN_RADIUS;
        valid_points.push(Point { x, y });
    }
    while x <= task.stage_right() - 2.0 * MUSICIAN_RADIUS {
        x += MUSICIAN_RADIUS;
        valid_points.push(Point { x, y });
    }
    while y >= task.stage_bottom() + 2.0 * MUSICIAN_RADIUS {
        y -= MUSICIAN_RADIUS;
        valid_points.push(Point { x, y });
    }
    while x >= task.stage_left() + 2.0 * MUSICIAN_RADIUS {
        x -= MUSICIAN_RADIUS;
        valid_points.push(Point { x, y });
    }
    valid_points = valid_points
        .iter()
        .filter(|p| {
            initial_solution
                .placements
                .iter()
                .all(|p2| p.dist(*p2) >= MUSICIAN_RADIUS)
        })
        .copied()
        .collect();

    let mut visibility = initial_visibility.clone();
    if valid_points.is_empty() {
        return (initial_solution.clone(), visibility);
    }

    let mut valid_dist_points = vec![];
    valid_dist_points.push(valid_points[0]);
    for point in valid_points.iter() {
        if point.dist(*valid_dist_points.last().unwrap()) > MUSICIAN_RADIUS * 3.0 {
            valid_dist_points.push(*point);
        }
    }

    let mut min_score_by_instrument = vec![None; task.instruments_len()];
    let score = calc_ex(task, initial_solution, &visibility);
    for i in 0..score.musician.len() {
        let Point { x, y } = initial_solution.placements[i];
        let mindist = (x - task.stage_left())
            .min(task.stage_right() - x)
            .min(y - task.stage_bottom())
            .min(task.stage_top() - y);
        if mindist < 1.5 * MUSICIAN_RADIUS {
            continue;
        }
        let instrument = task.musicians[i];
        let cur_score = score.musician[i];
        let min_score = min_score_by_instrument[instrument]
            .map(|(score, _)| score)
            .unwrap_or(1_000_000_000_000i64);
        if cur_score < min_score {
            min_score_by_instrument[instrument] = Some((cur_score, i));
        }
    }

    let musicians: Vec<usize> = min_score_by_instrument
        .iter()
        .flat_map(|o| o.map(|(_, idx)| vec![idx]).unwrap_or(vec![]))
        .collect();

    let mut best_change = None;
    let mut solution = initial_solution.clone();

    if valid_dist_points.len() < 10 && musicians.len() > 30 {
        return (initial_solution.clone(), visibility);
    }

    for musician_idx in musicians {
        let org_point = solution.placements[musician_idx];
        for point in valid_dist_points.iter() {
            solution.placements[musician_idx] = *point;
            let visibility = calc_visibility_fast(task, &solution);
            let cur_score = calc(task, &solution, &visibility)
                .expect("Border optimizer generated incorrect solution");
            let best_score = best_change
                .map(|(score, _, _)| score)
                .unwrap_or(score.score);
            if cur_score > best_score {
                best_change = Some((cur_score, musician_idx, point));
            }
        }
        solution.placements[musician_idx] = org_point;
    }

    if let Some((_, musician_idx, point)) = best_change {
        solution.placements[musician_idx] = *point;
        visibility = calc_visibility(task, &solution);
    }
    (solution, visibility)
}

pub fn one_by_one_do_talogo(
    task: &Task,
    initial_solution: &Solution,
    visibility: &Visibility,
) -> (Solution, Visibility) {
    let initial_score = score::calc(&task, &initial_solution, &visibility)
        .map(|s| s.to_string())
        .unwrap_or("NA".to_string());

    let mut partial_task = task.clone();
    partial_task.musicians = vec![];
    let mut partial_solution = initial_solution.clone();
    partial_solution.placements = vec![];
    partial_solution.volumes = vec![];

    // TODO common rng
    // todo fixup for positions in shuffled task
    let mut rng = rand::thread_rng();
    let pos_indices =
        rand::seq::index::sample(&mut rng, task.musicians.len(), task.musicians.len());

    let mut positions_fixup = vec![];

    let chunk_size: usize = if task.musicians.len() > 50 {
        task.musicians.len() / 20
    } else {
        1
    };

    for pos_idxs in &pos_indices.iter().chunks(chunk_size) {
        for pos_idx in pos_idxs {
            println!(
                "=== One-by-one: point {}/{}",
                partial_solution.placements.len() + 1,
                task.musicians.len()
            );

            partial_task.musicians.push(task.musicians[pos_idx]);
            let mut pos = initial_solution.placements[pos_idx];
            // TODO this loop can stuck
            let mut tries = 0;
            loop {
                if is_position_possible(
                    &partial_solution,
                    /*next pos idx*/ partial_solution.placements.len(),
                    pos,
                    MUSICIAN_RADIUS_SQR,
                ) {
                    partial_solution.placements.push(pos);
                    break;
                }
                pos = random_point_on_stage(&partial_task, &mut rng);
                tries += 1;
                if tries == 1000 {
                    panic!("Could not find proper place for another point");
                }
            }
            partial_solution.volumes.push(1.0);
            positions_fixup.push(pos_idx);
        }

        if partial_solution.placements.len() == 1 {
            // rest of code works badly with single musician
            continue;
        }

        let partial_visibility = calc_visibility_fast(&partial_task, &partial_solution);

        let (new_solution, new_visibility) =
            optimize_do_talogo(&partial_task, &partial_solution, partial_visibility);
        partial_solution = new_solution;

        let new_score = score::calc(&partial_task, &partial_solution, &new_visibility)
            .map(|s| s.to_string())
            .unwrap_or("NA".to_string());
        println!("=== One-by-one: score {new_score}/{initial_score}");
    }

    let mut final_placements = vec![Point::default(); partial_solution.placements.len()];
    let mut final_volumes = vec![1.0; partial_solution.placements.len()];
    for i in 0..partial_solution.placements.len() {
        final_placements[positions_fixup[i]] = partial_solution.placements[i];
        final_volumes[positions_fixup[i]] = partial_solution.volumes[i];
    }

    let final_solution = Solution {
        placements: final_placements,
        volumes: final_volumes,
    };

    let final_visibility = calc_visibility_fast(&task, &final_solution);
    (final_solution, final_visibility)
}

pub fn optimize_do_talogo(
    task: &Task,
    initial_solution: &Solution,
    visibility: Visibility,
) -> (Solution, Visibility) {
    let mut rng = Xoshiro256PlusPlus::seed_from_u64(42);

    let mut best_solution = initial_solution.clone();
    let mut best_visibility = visibility.clone();
    let mut max_score = match calc(task, initial_solution, &visibility) {
        Ok(res) => res,
        _ => return (best_solution, visibility),
    };

    let mut score_changed = true;
    while score_changed {
        score_changed = false;

        const TRIES: usize = 5;

        use rand::prelude::SliceRandom;

        let mut run = |optimizers: &OptimizerSlice| {
            let chain_len: usize = rng.gen_range(1..=optimizers.len());

            for _ in 0..TRIES {
                let mut try_solution = best_solution.clone();
                let mut try_visibility = best_visibility.clone();

                let (_, mut prev_name) = optimizers.choose(&mut rng).unwrap();
                let mut chain_names = Vec::with_capacity(chain_len);
                for _ in 0..chain_len {
                    let (optimize, name) = loop {
                        let (optimize, name) = optimizers.choose(&mut rng).unwrap();
                        if prev_name != *name {
                            prev_name = name;
                            chain_names.push(name);
                            break (optimize, name);
                        }
                    };

                    // println!("Trying {name}");

                    try_solution.volumes = default_volumes_task(task);
                    let (mut solution, visibility) =
                        optimize(&task, &try_solution, &try_visibility, &mut rng);
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
        };

        // run(ALL_OPTIMIZERS);
        // run(SAFE_OPTIMIZERS);
        run(FINAL_OPTIMIZERS);
    }

    (best_solution, best_visibility)
}

mod rap {
    use rand_xoshiro::Xoshiro256PlusPlus;
    use rapier2d::prelude::*;

    use crate::io::{Solution, Task, MUSICIAN_RADIUS};
    use crate::score::{calc_visibility, calc_visibility_fast, Visibility};

    const MUSICIAN_RESTITUTION: f32 = 0.7;
    const MUSICIAN_BALL_RADIUS: f32 = (MUSICIAN_RADIUS / 2.0) as f32;
    const STEPS: usize = 200;
    const REFRESH_VISIBILITY_RATE: usize = 10;

    pub fn rapier_optimizer(
        task: &Task,
        solution: &Solution,
        visibility: &Visibility,
        _rng: &mut Xoshiro256PlusPlus,
    ) -> (Solution, Visibility) {
        let mut rigid_body_set = RigidBodySet::new();
        let mut collider_set = ColliderSet::new();

        /* Create the stage box. */
        {
            let sw = task.stage_width as f32;
            let sh = task.stage_height as f32;
            let sl = task.stage_left() as f32;
            let sr = task.stage_right() as f32;
            let sb = task.stage_bottom() as f32;
            let st = task.stage_top() as f32;
            let border = (MUSICIAN_RADIUS / 2.0) as f32;

            let collider = ColliderBuilder::cuboid(border / 2.0, sh / 2.0)
                .translation(vector![sl + border / 2.0, sb + sh / 2.0])
                .build();
            collider_set.insert(collider);

            let collider = ColliderBuilder::cuboid(border / 2.0, sh / 2.0)
                .translation(vector![sr - border / 2.0, sb + sh / 2.0])
                .build();
            collider_set.insert(collider);

            let collider = ColliderBuilder::cuboid(sw / 2.0, border / 2.0)
                .translation(vector![sl + sw / 2.0, sb + border / 2.0])
                .build();
            collider_set.insert(collider);

            let collider = ColliderBuilder::cuboid(sw / 2.0, border / 2.0)
                .translation(vector![sl + sw / 2.0, st - border / 2.0])
                .build();
            collider_set.insert(collider);
        }

        /* Create the musicians balls. */
        let musician_body_handles = solution
            .placements
            .iter()
            .map(|pos| {
                let rigid_body = RigidBodyBuilder::dynamic()
                    .translation(vector![pos.x as f32, pos.y as f32])
                    .build();
                let collider = ColliderBuilder::ball(MUSICIAN_BALL_RADIUS)
                    .restitution(MUSICIAN_RESTITUTION)
                    .build();
                let ball_body_handle = rigid_body_set.insert(rigid_body);
                collider_set.insert_with_parent(collider, ball_body_handle, &mut rigid_body_set);
                ball_body_handle
            })
            .collect::<Vec<_>>();

        /* Create other structures necessary for the simulation. */
        let gravity = vector![0.0, 0.0];
        let integration_parameters = IntegrationParameters::default();
        let mut physics_pipeline = PhysicsPipeline::new();
        let mut island_manager = IslandManager::new();
        let mut broad_phase = BroadPhase::new();
        let mut narrow_phase = NarrowPhase::new();
        let mut impulse_joint_set = ImpulseJointSet::new();
        let mut multibody_joint_set = MultibodyJointSet::new();
        let mut ccd_solver = CCDSolver::new();
        let physics_hooks = ();
        let event_handler = ();

        fn body_point(body: &RigidBody) -> crate::geom::Point {
            crate::geom::Point {
                x: body.translation().x as f64,
                y: body.translation().y as f64,
            }
        }

        fn collect_solution(
            rigid_body_set: &RigidBodySet,
            musician_body_handles: &Vec<RigidBodyHandle>,
            initial_solution: &Solution,
        ) -> Solution {
            Solution {
                placements: musician_body_handles
                    .iter()
                    .enumerate()
                    .map(|(pos_idx, body_handle)| {
                        let musician_body = &rigid_body_set[*body_handle];
                        crate::geom::Point {
                            x: musician_body.translation().x as f64,
                            y: musician_body.translation().y as f64,
                        }
                    })
                    .collect(),
                volumes: initial_solution.volumes.clone(),
            }
        };

        let mut visibility = visibility.clone();
        let mut result = solution.clone();

        /* Run the game loop, stepping the simulation once per frame. */
        for step in 0..STEPS {
            musician_body_handles
                .iter()
                .enumerate()
                .for_each(|(pos_idx, body_handle)| {
                    let body = &mut rigid_body_set[*body_handle];
                    body.reset_forces(true);
                    let force = super::attract_musicians_to_attendees_force_collector(
                        task,
                        &visibility,
                        pos_idx,
                        body_point(&body),
                    );
                    // todo proper mag
                    let force = force * (1.0 / 1e9);
                    dbg!((body_point(&body), force));

                    body.add_force(
                        vector![force.x as f32, force.y as f32],
                        false, /*already woken up*/
                    );
                });

            physics_pipeline.step(
                &gravity,
                &integration_parameters,
                &mut island_manager,
                &mut broad_phase,
                &mut narrow_phase,
                &mut rigid_body_set,
                &mut collider_set,
                &mut impulse_joint_set,
                &mut multibody_joint_set,
                &mut ccd_solver,
                None,
                &physics_hooks,
                &event_handler,
            );

            if (step + 1) % REFRESH_VISIBILITY_RATE == 0 {
                let result = collect_solution(&rigid_body_set, &musician_body_handles, solution);
                // visibility = calc_visibility_fast(&task, &result);
                visibility = calc_visibility(&task, &result);
            }
        }

        let result = collect_solution(&rigid_body_set, &musician_body_handles, solution);
        // let visibility = calc_visibility_fast(&task, &result);
        let visibility = calc_visibility(&task, &result);
        (result, visibility)
    }
}
