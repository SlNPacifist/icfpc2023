use crate::score::potential_score;
use crate::solution::dummy;
use clap;
use io::{Solution, Task};
use num_format::{Locale, ToFormattedString};
use optimizer::optimize_do_talogo;
use score::Visibility;

mod genetics;
mod geom;
mod io;
mod optimizer;
mod score;
mod solution;

const BASE_SOLUTIONS_DIR: &str = "../../solutions-20230708-124428";
const OPTIMAL_SOLUTIONS_DIR: &str = "../../solutions";
const TASKS_NUM: usize = 90;

fn get_base_solution(task: &Task, i: usize) -> Solution {
    let solution_path = format!("{BASE_SOLUTIONS_DIR}/problem-{i}.json");

    if std::fs::metadata(&solution_path).is_ok() {
        io::read_solution(&solution_path)
    } else {
        let sol = dummy(&task);
        io::write(&solution_path, &sol);
        sol
    }
}

fn read_task(i: usize) -> Task {
    io::read(&format!("../../data/problem-{i}.json"))
}

fn write_optimal_solution(solution: &Solution, i: usize) {
    io::write(
        &format!("{OPTIMAL_SOLUTIONS_DIR}/problem-{i}.json"),
        &solution,
    );
}

fn main() {
    let cmd = clap::Command::new("rust")
        .bin_name("rust")
        .subcommand_required(true)
        .subcommand(clap::command!("score"))
        .subcommand(clap::command!("potential"));
    let matches = cmd.get_matches();
    match matches.subcommand() {
        Some(("potential", _matches)) => {
            let mut potential_scores = (1..=TASKS_NUM)
                .map(|i| {
                    let task = read_task(i);
                    let base_solution = get_base_solution(&task, i);
                    let visibility = score::calc_visibility(&task, &base_solution);
                    let score = score::calc(&task, &base_solution, &visibility).unwrap_or(0);
                    (potential_score(&task), i, score)
                })
                .collect::<Vec<_>>();

            potential_scores.sort();
            for (score, index, cur_score) in potential_scores {
                println!(
                    "Potential score for task {index:2} is {:>15}, cur score is {:>13}",
                    score.to_formatted_string(&Locale::en),
                    cur_score.to_formatted_string(&Locale::en)
                );
            }
        }

        Some(("score", _matches)) => {
            for i in 1..=TASKS_NUM {
                println!("===================================");
                let task = read_task(i);
                let mut best_solution = get_base_solution(&task, i);
                let visibility = score::calc_visibility(&task, &best_solution);

                let mut max_score = match score::calc(&task, &best_solution, &visibility) {
                    Ok(points) => {
                        println!(
                            "Base solution from {BASE_SOLUTIONS_DIR} for task {i} got {points} points"
                        );
                        points
                    }
                    Err(err) => {
                        println!(
                            "Base solution from {BASE_SOLUTIONS_DIR} for task {i} is incorrect: {err}"
                        );
                        -1_000_000_000_000
                    }
                };

                let (solution, visibility) = optimize_do_talogo(&task, &best_solution, visibility);

                // {
                //     let (solution, visibility) = force_greedy_combined(&task, &best_solution);

                //     match score::calc(&task, &solution, &visibility) {
                //         Ok(points) => {
                //             println!("Force based from greedy solution for task {i} got {points} points");
                //             if points > max_score {
                //                 max_score = points;
                //                 best_solution = solution;
                //             }
                //         }
                //         Err(err) => {
                //             println!("Force based from greedy solution for task {i} is incorrect: {err}")
                //         }
                //     }
                // }

                // {
                //     let largest_spread_solution = [5.0, 3.0, 2.0, 1.5, 1.1, 1.05, 1.01, 1.005, 1.001, 1.0]
                //         .into_iter()
                //         .map(|spread| solution::dummy_hex(&task, spread, true))
                //         .filter_map(|solution| {
                //             let visibility = score::calc_visibility(&task, &solution);
                //             score::calc(&task, &solution, &visibility)
                //                 .ok()
                //                 .map(|_| solution)
                //         })
                //         .next()
                //         .unwrap();
                //     let (solution, visibility) = force_greedy_combined(&task, &largest_spread_solution);

                //     match score::calc(&task, &solution, &visibility) {
                //         Result::Ok(points) => {
                //             println!("Force based from dummy solution for task {i} got {points} points");
                //             if points > max_score {
                //                 max_score = points;
                //                 io::write(&format!("../../solutions/problem-{i}.json"), &solution);
                //                 best_solution = solution;
                //             }
                //         }
                //         Result::Err(err) => {
                //             println!("Force based from dummy solution for task {i} is incorrect: {err}")
                //         }
                //     }
                // }

                // {
                //     if best_solution.placements.len() <= 0 {
                //         let solution = genetics::optimize_placements(&task, &best_solution);
                //         let visibility = score::calc_visibility(&task, &solution);
                //         match score::calc(&task, &solution, &visibility) {
                //             Result::Ok(points) => {
                //                 println!("Genetic solution for task {i} got {points} points");
                //                 if points > max_score {
                //                     max_score = points;
                //                     best_solution = solution;
                //                 }
                //             }
                //             Result::Err(err) => {
                //                 println!("Genetic solution for task {i} is incorrect: {err}")
                //             }
                //         }
                //     }
                // }
                write_optimal_solution(&best_solution, i);
            }
        }
        _ => unreachable!("clap should ensure we don't get here"),
    };
}
