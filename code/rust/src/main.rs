use crate::solution::dummy;
use crate::{optimizer::force_greedy_combined, score::potential_score};
use num_format::{Locale, ToFormattedString};

mod genetics;
mod geom;
mod io;
mod optimizer;
mod score;
mod solution;

fn main() {
    let base_solutions_dir = "../../solutions-20230708-124428";

    let mut potential_scores: Vec<(i64, usize, i64)> = Vec::new();

    for i in 1..=90 {
        let task = io::read(&format!("../../data/problem-{i}.json"));

        let solution_path = format!("{base_solutions_dir}/problem-{i}.json");

        let base_solution = if std::fs::metadata(&solution_path).is_ok() {
            io::read_solution(&solution_path)
        } else {
            let sol = dummy(&task);
            io::write(&solution_path, &sol);
            sol
        };

        let visibility = score::calc_visibility(&task, &base_solution);

        let mut best_solution = base_solution.clone();

        let mut max_score = match score::calc(&task, &base_solution, &visibility) {
            Ok(points) => {
                println!(
                    "Base solution from {base_solutions_dir} for task {i} got {points} points"
                );
                points
            }
            Err(err) => {
                println!(
                    "Base solution from {base_solutions_dir} for task {i} is incorrect: {err}"
                );
                // panic!("bad base solution")
                0
            }
        };
        potential_scores.push((potential_score(&task), i, max_score));

        continue;

        {
            let (solution, visibility) = force_greedy_combined(&task, &base_solution);

            match score::calc(&task, &solution, &visibility) {
                Ok(points) => {
                    println!("Force based from greedy solution for task {i} got {points} points");
                    if points > max_score {
                        max_score = points;
                        io::write(&format!("../../solutions/problem-{i}.json"), &solution);
                        best_solution = solution;
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
                    score::calc(&task, &solution, &visibility)
                        .ok()
                        .map(|_| solution)
                })
                .next()
                .unwrap();
            let (solution, visibility) = force_greedy_combined(&task, &largest_spread_solution);

            match score::calc(&task, &solution, &visibility) {
                Result::Ok(points) => {
                    println!("Force based from dummy solution for task {i} got {points} points");
                    if points > max_score {
                        max_score = points;
                        io::write(&format!("../../solutions/problem-{i}.json"), &solution);
                        best_solution = solution;
                    }
                }
                Result::Err(err) => {
                    println!("Force based from dummy solution for task {i} is incorrect: {err}")
                }
            }
        }

        {
            if best_solution.placements.len() <= 0 {
                let solution = genetics::optimize_placements(&task, &best_solution);
                let visibility = score::calc_visibility(&task, &solution);
                match score::calc(&task, &solution, &visibility) {
                    Result::Ok(points) => {
                        println!("Genetic solution for task {i} got {points} points");
                        if points > max_score {
                            // max_score = points;
                            io::write(&format!("../../solutions/problem-{i}.json"), &solution);
                            // best_solution = solution;
                        }
                    }
                    Result::Err(err) => {
                        println!("Genetic solution for task {i} is incorrect: {err}")
                    }
                }
            }
        }

        // io::write(&format!("../../solutions/problem-{i}.json"), &solution);
    }

    potential_scores.sort();
    for (score, index, cur_score) in potential_scores {
        println!(
            "Potential score for task {index:2} is {:>15}, cur score is {:>13}",
            score.to_formatted_string(&Locale::en),
            cur_score.to_formatted_string(&Locale::en)
        );
    }
}
