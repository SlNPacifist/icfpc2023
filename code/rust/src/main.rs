use crate::io::{Task, MUSICIAN_RADIUS};

mod geom;
mod io;
mod optimizer;
mod score;
mod solution;

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

        io::write(&format!("../../solutions/problem-{i}.json"), &solution);
    }
}
