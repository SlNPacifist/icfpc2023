#[macro_use]
extern crate rouille;

use crate::score::potential_score;
use crate::solution::{dummy, recalc_volumes};
use clap::{self, arg, value_parser};
use io::{Solution, Task};
use num_format::{Locale, ToFormattedString};
use optimizer::optimize_do_talogo;
use crate::io::default_volumes_task;

mod genetics;
mod geom;
mod http_api;
mod io;
mod optimizer;
mod score;
mod solution;

const BASE_SOLUTIONS_DIR: &str = "../../solutions-20230708-124428";
const OPTIMAL_SOLUTIONS_DIR: &str = "../../solutions";
const ORTOOLS_DATA_DIR: &str = "../../ortools-data";
const TASKS_NUM: usize = 90;

fn get_solution(task: &Task, solution_path: &str) -> Solution {
    if std::fs::metadata(solution_path).is_ok() {
        io::read_solution(solution_path)
    } else {
        let sol = dummy(&task);
        io::write(solution_path, &sol);
        sol
    }
}

fn get_base_solution(task: &Task, i: usize) -> Solution {
    get_solution(task, &format!("{BASE_SOLUTIONS_DIR}/problem-{i}.json"))
}

fn get_optimal_solution(task: &Task, i: usize) -> Solution {
    get_solution(task, &format!("{OPTIMAL_SOLUTIONS_DIR}/problem-{i}.json"))
}

pub fn read_task(i: usize) -> Task {
    io::read(&format!("../../data/problem-{i}.json"))
}

pub fn write_optimal_solution(task: &Task, solution: &Solution, points: i64, i: usize) {
    let cur_solution = get_optimal_solution(task, i);
    let visibility = score::calc_visibility(&task, &cur_solution);

    match score::calc(&task, &cur_solution, &visibility) {
        Ok(cur_points) => {
            if cur_points > points {
                println!(
                    "Solution for task {i} was not improved (currently {cur_points}, updated {points})"
                );
                return;
            } else if cur_points == points {
                println!("Solution for task {i} did not change");
                return;
            } else {
                println!("+++Solution for task {i} was improved from {cur_points} to {points}");
            }
        }
        Err(_) => {
            println!("Solution for task {i} was incorrect, got {points} points");
        }
    };

    io::write(
        &format!("{OPTIMAL_SOLUTIONS_DIR}/problem-{i}.json"),
        &solution,
    );
}

fn get_spread_solution(task: &Task) -> Solution {
    [5.0, 3.0, 2.0, 1.5, 1.1, 1.05, 1.01, 1.005, 1.001, 1.0]
        .into_iter()
        .map(|spread| solution::dummy_hex(&task, spread, true))
        .filter_map(|solution| {
            let visibility = score::calc_visibility(&task, &solution);
            score::calc(&task, &solution, &visibility)
                .ok()
                .map(|_| solution)
        })
        .next()
        .unwrap()
}

fn main() {
    let cmd = clap::Command::new("rust")
        .bin_name("rust")
        .subcommand_required(true)
        .subcommand(
            clap::command!("optimize").arg(
                arg!([base])
                    .value_parser(value_parser!(String))
                    .default_value("dummy"),
            ),
        )
        .subcommand(clap::command!("potential"))
        .subcommand(clap::command!("server"))
        .subcommand(clap::command!("make-ortools-input"))
        .subcommand(clap::command!("apply-ortools-output"))
        .subcommand(clap::command!("recalc-volumes"));
    let matches = cmd.get_matches();
    match matches.subcommand() {
        Some(("potential", _matches)) => {
            let mut potential_scores = (1..=TASKS_NUM)
                .map(|i| {
                    let task = read_task(i);
                    let optimal_solution = get_optimal_solution(&task, i);
                    let visibility = score::calc_visibility(&task, &optimal_solution);
                    let score = score::calc(&task, &optimal_solution, &visibility).unwrap_or(0);
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

        Some(("optimize", matches)) => {
            for i in 1..=TASKS_NUM {
                println!("===================================");
                let task = read_task(i);
                let base_solution_name = matches
                    .get_one::<String>("base")
                    .expect("base should be specified");
                let base_solution = match base_solution_name.as_str() {
                    "dummy" => get_base_solution(&task, i),
                    "spread" => get_spread_solution(&task),
                    _ => panic!("Unknown base solution {base_solution_name}"),
                };
                let visibility = score::calc_visibility(&task, &base_solution);

                match score::calc(&task, &base_solution, &visibility) {
                    Ok(points) => {
                        println!("{base_solution_name} solution for task {i} got {points} points");

                        let (best_solution, visibility) =
                            optimize_do_talogo(&task, &base_solution, visibility);
                        match score::calc(&task, &best_solution, &visibility) {
                            Ok(points) => write_optimal_solution(&task, &best_solution, points, i),
                            Err(_) => {
                                println!("Could not find correct solution for task {i}");
                            }
                        }
                    }
                    Err(err) => {
                        println!(
                            "{base_solution_name} solution from {BASE_SOLUTIONS_DIR} for task {i} is incorrect: {err}"
                        );
                    }
                };
            }
        }

        Some(("make-ortools-input", matches)) => {
            #[derive(serde::Serialize)]
            struct OrToolsInput {
                positions: Vec<geom::Point>,
                musicians: Vec<usize>,
                matrix: Vec<Vec<i64>>,
            }

            for i in 1..=TASKS_NUM {
                let task = read_task(i);
                let solution = get_optimal_solution(&task, i);
                let visibility = score::calc_visibility_fast(&task, &solution);

                let mut m = vec![vec![0; task.musicians.len()]; task.musicians.len()];
                for inst_idx in 0..task.musicians.len() {
                    for pos_idx in 0..task.musicians.len() {
                        let inst = task.musicians[inst_idx];
                        for (att_idx, att) in task.attendees.iter().enumerate() {
                            if visibility.is_visible(att_idx, pos_idx) {
                                m[inst_idx][pos_idx] += score::attendee_score_without_q(
                                    att,
                                    inst,
                                    solution.placements[pos_idx],
                                );
                            }
                        }
                    }
                }

                let input = OrToolsInput {
                    positions: solution.placements.clone(),
                    musicians: task.musicians.clone(),
                    matrix: m,
                };

                std::fs::write(
                    format!("{ORTOOLS_DATA_DIR}/input-{i}.json"),
                    serde_json::to_vec(&input).expect("Could not serialize matrix"),
                )
                .expect("Got error when writing to file");
            }
        }

        Some(("apply-ortools-output", matches)) => {
            for i in 1..=TASKS_NUM {
                let task = read_task(i);

                let output: Vec<geom::Point> = {
                    let path = format!("{ORTOOLS_DATA_DIR}/output-{i}.json");
                    let data = std::fs::read_to_string(&path)
                        .expect(&format!("Unable to read file {path}"));
                    serde_json::from_str(&data).expect("Could not parse data")
                };

                let mut solution = Solution {
                    placements: output,
                    volumes: default_volumes_task(&task),
                };
                let visibility = score::calc_visibility_fast(&task, &solution);
                recalc_volumes(&task, &mut solution, &visibility);
                match score::calc(&task, &solution, &visibility) {
                    Ok(points) => {
                        println!("ortools solution for task {i} got {points} points");
                        write_optimal_solution(&task, &solution, points, i);
                    }
                    Err(err) => {
                        println!("ortools solution from for task {i} is incorrect: {err}");
                    }
                };
            }
        }

        Some(("recalc-volumes", matches)) => {
            for i in 1..=TASKS_NUM {
                let task = read_task(i);
                let mut solution = get_optimal_solution(&task, i);
                let visibility = score::calc_visibility_fast(&task, &solution);
                recalc_volumes(&task, &mut solution, &visibility);
                match score::calc(&task, &solution, &visibility) {
                    Ok(points) => {
                        println!("ortools solution for task {i} got {points} points");
                        write_optimal_solution(&task, &solution, points, i);
                    }
                    Err(err) => {
                        println!("ortools solution from for task {i} is incorrect: {err}");
                    }
                };
            }
        }

        // Some(("spread_optimize", _matches)) => {

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
        // }
        // }
        Some(("server", _matches)) => {
            println!("Starting server on port 8000");
            http_api::start_server();
        }
        _ => unreachable!("clap should ensure we don't get here"),
    };
}
