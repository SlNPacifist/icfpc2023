use crate::geom::Point;
use crate::io::{Solution, Task, MUSICIAN_RADIUS};

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

pub fn dummy_hex(task: &Task) -> Solution {
    let mut res = Solution::default();
    let mut x = task.stage_left() + MUSICIAN_RADIUS;
    let mut y = task.stage_bottom() + MUSICIAN_RADIUS;
    let mut even = false;
    for _m in &task.musicians {
        res.placements.push(Point { x, y });
        x += 2.0 * MUSICIAN_RADIUS;
        if !task.musician_in_stage(x, y) {
            even = !even;
            if even {
                x = task.stage_left() + 2.0 * MUSICIAN_RADIUS;
            } else {
                x = task.stage_left() + MUSICIAN_RADIUS;
            }
            y += 2.0 * MUSICIAN_RADIUS * 60.0f64.to_radians().sin();
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
