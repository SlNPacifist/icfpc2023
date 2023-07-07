use crate::io::{Task, Solution, Coord, MUSICIAN_RADIUS};

pub fn dummy(task: &Task) -> Solution {
    let mut res = Solution::default();
    let mut x = task.stage_left() + MUSICIAN_RADIUS;
    let mut y = task.stage_bottom() + MUSICIAN_RADIUS;
    for _m in &task.musicians {
        res.placements.push(Coord {x, y});
        x += 2.0 * MUSICIAN_RADIUS;
        if !task.musician_in_stage(x, y) {
            x = task.stage_left() + MUSICIAN_RADIUS;
            y += 2.0 * MUSICIAN_RADIUS;
        }
    }
    res
}

pub fn dummy_hex(task: &Task) -> Solution {
    let mut res = Solution::default();
    let mut x = task.stage_left() + MUSICIAN_RADIUS;
    let mut y = task.stage_bottom() + MUSICIAN_RADIUS;
    let mut even = false;
    for _m in &task.musicians {
        res.placements.push(Coord {x, y});
        x += 2.0 * MUSICIAN_RADIUS;
        if !task.musician_in_stage(x, y) {
            even = !even;
            if even {
                x = task.stage_left() + 2.0 * MUSICIAN_RADIUS;
                y += 2.0 * MUSICIAN_RADIUS * 60.0f64.to_radians().sin();
            } else {
                x = task.stage_left() + MUSICIAN_RADIUS;
                y += 2.0 * MUSICIAN_RADIUS;
            }
        }
    }
    res
}
