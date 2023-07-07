use crate::io::{Task, Solution, Coord, musician_radius};

pub fn dummy(task: &Task) -> Solution {
    let mut res = Solution::default();
    let mut x = task.stage_left() + musician_radius;
    let mut y = task.stage_bottom() + musician_radius;
    for _m in &task.musicians {
        res.placements.push(Coord {x, y});
        x += 2.0 * musician_radius;
        if !task.musician_in_stage(x, y) {
            x = task.stage_left() + musician_radius;
            y += 2.0 * musician_radius;
        }
    }
    res
}