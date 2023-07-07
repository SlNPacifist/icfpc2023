use crate::geom::Point;
use crate::io::{Solution, Task, MUSICIAN_RADIUS};
use crate::optimizer::optimize_placements_greedy;
use crate::score;

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

pub fn dummy_hex(task: &Task, radius_multiplier: f64) -> Solution {
    let r = MUSICIAN_RADIUS;
    let r = r * radius_multiplier;

    let mut res = Solution::default();
    let mut x = task.stage_left() + r;
    let mut y = task.stage_bottom() + r;
    let mut even = false;
    for _m in &task.musicians {
        res.placements.push(Point { x, y });
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

fn dummy_opti_solver(task: &Task, spread: f64) -> anyhow::Result<Solution> {
    let solution = dummy_hex(task, spread);

    let visibility = score::calc_visibility(&task, &solution);
    score::calc(&task, &solution, &visibility)?;

    let solution = optimize_placements_greedy(&task, &solution, &visibility);
    Ok(solution)
}

pub fn multi_dummy_solver(task: &Task) -> Solution {
    let spreads: &[f64] =
        if task.attendees.len() * task.musicians.len() * task.musicians.len() > 1863225000 {
            &[1.1, 1.01, 1.0]
        } else {
            &[1.5, 1.1, 1.05, 1.01, 1.005, 1.001, 1.0]
        };

    spreads
        .iter()
        .map(|spread| dummy_opti_solver(task, *spread))
        .filter_map(|solution| solution.ok())
        .max_by_key(|solution| {
            let visibility = score::calc_visibility(&task, &solution);
            score::calc(&task, &solution, &visibility).unwrap_or(0)
        })
        .expect("No solutions found")
}
