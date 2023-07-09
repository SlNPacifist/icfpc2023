use crate::geom::Point;
use crate::io::{Solution, Task, MUSICIAN_RADIUS};
use crate::optimizer::optimize_placements_greedy;
use crate::score;
use crate::score::Visibility;

pub fn dummy(task: &Task) -> Solution {
    let mut res = Solution::default();
    let mut x = task.stage_left() + MUSICIAN_RADIUS;
    let mut y = task.stage_bottom() + MUSICIAN_RADIUS;
    for _m in &task.musicians {
        res.placements.push(Point { x, y });
        x += 2.0 * MUSICIAN_RADIUS;
        if !task.musician_in_stage(x, y) {
            x = task.stage_left() + MUSICIAN_RADIUS;
            y += 2.0 * MUSICIAN_RADIUS;
        }
    }
    res
}

pub fn dummy_hex(task: &Task, radius_multiplier: f64, scale_to_stage: bool) -> Solution {
    let r = MUSICIAN_RADIUS;
    let r = r * radius_multiplier;

    let mut res = Solution::default();
    let mut x = task.stage_left() + r;
    let mut y = task.stage_bottom() + r;
    let mut last_y = y;
    let mut even = false;
    for _m in &task.musicians {
        res.placements.push(Point { x, y });
        last_y = y;
        x += 2.0 * r;
        if !task.musician_in_stage(x, y) {
            even = !even;
            if even {
                x = task.stage_left() + 2.0 * r;
            } else {
                x = task.stage_left() + r;
            }
            y += 2.0 * r * 60.0f64.to_radians().sin();
        }
    }
    let max_y = last_y + r;
    if scale_to_stage {
        let scaling = (task.stage_top() - task.stage_bottom()) / (max_y - task.stage_bottom());
        for p in &mut res.placements {
            p.y = (p.y - task.stage_bottom()) * scaling + task.stage_bottom();
        }
    }
    res
}

pub fn dummy_narrow(task: &Task) -> Solution {
    let height_step = {
        // Solving the right triangle with hypo 2r and height w-2r
        let hypo = 2.0 * MUSICIAN_RADIUS;
        let width = task.stage_width - 2.0 * MUSICIAN_RADIUS;
        (hypo * hypo - width * width).sqrt()
    };

    let mut res = Solution::default();
    let mut x = task.stage_left() + MUSICIAN_RADIUS;
    let mut y = task.stage_bottom() + MUSICIAN_RADIUS;
    let mut even = false;
    for _m in &task.musicians {
        res.placements.push(Point { x, y });
        even = !even;
        if even {
            x = task.stage_right() - MUSICIAN_RADIUS;
        } else {
            x = task.stage_left() + MUSICIAN_RADIUS;
        }
        y += height_step;
    }
    res
}

pub fn transposer_solver(s: impl Fn(&Task) -> Solution) -> impl Fn(&Task) -> Solution {
    move |task| {
        let transpose = task.stage_height < task.stage_width;
        let task = task.clone();
        let task = if transpose { task.transpose() } else { task };
        let solution = s(&task);

        if transpose {
            solution.transpose()
        } else {
            solution
        }
    }
}

fn dummy_opti_solver(task: &Task, spread: f64, scale_to_stage: bool) -> anyhow::Result<Solution> {
    let solution = dummy_hex(task, spread, scale_to_stage);

    let visibility = score::calc_visibility(&task, &solution);
    score::calc(&task, &solution, &visibility)?;

    let (solution, _) = optimize_placements_greedy(&task, &solution, &visibility);
    Ok(solution)
}

pub fn multi_dummy_solver(task: &Task) -> Solution {
    const SPHERE_PACKING_CONST: f64 = 0.9069;
    let mut spreads =
        if task.attendees.len() * task.musicians.len() * task.musicians.len() > 1863225000 {
            vec![1.1, 1.01, 1.0]
        } else {
            vec![5.0, 3.0, 2.0, 1.5, 1.1, 1.05, 1.01, 1.005, 1.001, 1.0]
        };

    let single_musician_area = std::f64::consts::PI * MUSICIAN_RADIUS * MUSICIAN_RADIUS;
    let total_musicians_area =
        task.musicians.len() as f64 * single_musician_area * SPHERE_PACKING_CONST;
    let starting_spread = (task.stage_width * task.stage_height) as f64 / total_musicians_area;

    // just for safety
    let mut area_based_spread = 0.99 * starting_spread;
    let area_spreads_count = 3;
    for _ in 0..area_spreads_count {
        spreads.push(area_based_spread);
        area_based_spread *= 0.5;
    }

    let scale_stage = [false, true];

    use itertools::Itertools;

    spreads
        .into_iter()
        .cartesian_product(scale_stage.into_iter())
        .map(|(spread, scale)| dummy_opti_solver(task, spread, scale))
        .filter_map(|solution| solution.ok())
        .max_by_key(|solution| {
            let visibility = score::calc_visibility(&task, &solution);
            score::calc(&task, &solution, &visibility).unwrap_or(0)
        })
        .expect("No solutions found")
}

const MIN_VOLUME: f64 = 0.0;
const MAX_VOLUME: f64 = 10.0;

pub fn recalc_volumes(task: &Task, solution: &mut Solution, visibility: &Visibility) {
    solution.volumes = solution.placements.iter().enumerate()
        .map(|(mus_idx, mus_pos)| {
            let attendee_sum: i64 = task.attendees.iter().enumerate()
                .filter(|(att_idx, _)| visibility.is_visible(*att_idx, mus_idx))
                .map(|(_, att)| {
                    score::attendee_score_without_q(
                        att,
                        task.musicians[mus_idx],
                        *mus_pos,
                    )
                })
                .sum();

            if attendee_sum > 0 {
                MAX_VOLUME
            } else {
                MIN_VOLUME
            }
        })
        .collect();
}