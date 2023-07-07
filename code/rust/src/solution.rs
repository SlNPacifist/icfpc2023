use crate::io::{Task, Output, Coord};

pub fn dummy(task: &Task) -> Output {
    let mut res = Output::default();
    let (stage_left,stage_bottom) = task.stage_bottom_left;
    let stage_right = stage_left + task.stage_width;
    let min_radius = 10.0;
    let mut x = stage_left + min_radius;
    let mut y = stage_bottom + min_radius;
    for _m in &task.musicians {
        res.placements.push(Coord {x, y});
        x += 2.0 * min_radius;
        if stage_right - x < min_radius {
            x = stage_left + min_radius;
            y += 2.0 * min_radius;
        }
    }
    res
}