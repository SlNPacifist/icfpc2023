use crate::io::{Task, MUSICIAN_RADIUS};
use crate::optimizer::force_based_optimizer;

mod geom;
mod io;
mod optimizer;
mod score;
mod solution;
mod genetics;

fn main() {
    for i in 1..=55 {
        let task = io::read(&format!("../../data/problem-{i}.json"));

        let solution = {
            let task = task.clone().simplify();

            let solver = |task: &Task| {
                if task.stage_width < 3.0 * MUSICIAN_RADIUS {
                    println!("Using narrow solver for task {i}");
                    solution::dummy_narrow(task)
                } else {
                    solution::multi_dummy_solver(task)
                }
            };
            let solver = solution::transposer_solver(solver);

            solver(&task)
        };

        let visibility = score::calc_visibility(&task, &solution);

        match score::calc(&task, &solution, &visibility) {
            Result::Ok(points) => {
                println!("Solution for task {i} got {points} points")
            }
            Result::Err(err) => {
                println!("Solution for task {i} is incorrect: {err}")
            }
        }

        {
            let mut new_s = solution.clone();
            for _ in 0..3 {
                let f_s = force_based_optimizer(&task, &new_s);
                let f_v = score::calc_visibility(&task, &f_s);
                new_s = optimizer::optimize_placements_greedy(&task, &f_s, &f_v);
            }
            let new_v = score::calc_visibility(&task, &new_s);

            match score::calc(&task, &new_s, &new_v) {
                Result::Ok(points) => {
                    println!("Force based solution for task {i} got {points} points")
                }
                Result::Err(err) => {
                    println!("Force based solution for task {i} is incorrect: {err}")
                }
            }
        }

        println!("Trying genetics optimizer");
        let optimized_solution = genetics::optimize_placements(&task, &solution);
        let optimized_visibility = score::calc_visibility(&task, &solution);
        match score::calc(&task, &optimized_solution, &optimized_visibility) {
            Result::Ok(points) => {
                println!("Optimized solution for task {i} got {points} points")
            }
            Result::Err(err) => {
                println!("Optimized solution for task {i} is incorrect: {err}")
            }
        }

        io::write(&format!("../../solutions/problem-{i}.json"), &solution);
    }
}
