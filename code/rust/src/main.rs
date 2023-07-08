use crate::io::{Task, MUSICIAN_RADIUS};
use crate::optimizer::{force_based_optimizer, force_greedy_combined};

mod geom;
mod io;
mod optimizer;
mod score;
mod solution;
mod genetics;

fn main() {
    let base_solutions_dir = "../../solutions-20230708-124428";

    for i in 2..=55 {
        let task = io::read(&format!("../../data/problem-{i}.json"));

        let base_solution = io::read_solution(&format!("{base_solutions_dir}/problem-{i}.json"));

        let visibility = score::calc_visibility(&task, &base_solution);

        let mut max_score = match score::calc(&task, &base_solution, &visibility) {
            Ok(points) => {
                println!("Base solution from {base_solutions_dir} for task {i} got {points} points");
                points
            }
            Err(err) => {
                println!("Base solution from {base_solutions_dir} for task {i} is incorrect: {err}");
                panic!("bad base solution")
            }
        };

        {
            let (solution, visibility) = force_greedy_combined(&task, &base_solution);

            match score::calc(&task, &solution, &visibility) {
                Ok(points) => {
                    println!("Force based from greedy solution for task {i} got {points} points");
                    if points > max_score {
                        max_score = points;
                        io::write(&format!("../../solutions/problem-{i}.json"), &solution);
                    }
                }
                Err(err) => {
                    println!("Force based from greedy solution for task {i} is incorrect: {err}")
                }
            }
        }

        {
            let largest_spread_solution = [5.0, 3.0, 2.0, 1.5, 1.1, 1.05, 1.01, 1.005, 1.001, 1.0]
                .into_iter()
                .map(|spread| solution::dummy_hex(&task, spread, true))
                .filter_map(|solution| {
                    let visibility = score::calc_visibility(&task, &solution);
                    score::calc(&task, &solution, &visibility).ok().map(|_| solution)
                })
                .next().unwrap();
            let (solution, visibility) = force_greedy_combined(&task, &largest_spread_solution);

            match score::calc(&task, &solution, &visibility) {
                Result::Ok(points) => {
                    println!("Force based from dummy solution for task {i} got {points} points");
                    if points > max_score {
                        max_score = points;
                        io::write(&format!("../../solutions/problem-{i}.json"), &solution);
                    }
                }
                Result::Err(err) => {
                    println!("Force based from dummy solution for task {i} is incorrect: {err}")
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

        // io::write(&format!("../../solutions/problem-{i}.json"), &solution);
    }
}
