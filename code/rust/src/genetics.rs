use genevo::{operator::prelude::*, population::*, prelude::*, types::fmt::Display};
use crate::geom::{Point};
use crate::io::{Solution, Task};
use crate::score::{calc_visibility, calc};

type Genome = Vec<f64>;

pub fn genome_from_solution(solution: &Solution) -> Genome {
    solution.placements.iter().copied().flat_map(|Point {x, y}| [x, y]).collect()
}

pub fn genome_to_solution(genome: &Genome) -> Solution {
    Solution {
        placements: genome.chunks(2).map(|c| Point {x: c[0], y: c[1]}).collect()
    }
}

const MIN_SCORE: i64 = -1_000_000_000;

/// The genotype is Solution
impl<'a> FitnessFunction<Genome, i64> for &'a Task {
    fn fitness_of(&self, genome: &Genome) -> i64 {
        let solution = genome_to_solution(genome);
        let visibility = calc_visibility(self, &solution);

        match calc(self, &solution, &visibility) {
            Ok(val) => val,
            _ => MIN_SCORE, 
        }
    }

    fn average(&self, values: &[i64]) -> i64 {
        (values.iter().sum::<i64>() as f32 / values.len() as f32 + 0.5).floor() as i64
    }

    fn highest_possible_fitness(&self) -> i64 {
        -MIN_SCORE
    }

    fn lowest_possible_fitness(&self) -> i64 {
        MIN_SCORE
    }
}

pub fn optimize_placements(task: &Task, solution: &Solution) -> Solution {
    let placement: Vec<f64> = genome_from_solution(solution);
    let initial_population = Population::with_individuals(vec![placement; 6]);

    let mut task_sim = simulate(
        genetic_algorithm()
            .with_evaluation(task)
            .with_selection(MaximizeSelector::new(0.85, 12))
            .with_crossover(SinglePointCrossBreeder::new())
            // TODO set min and max!
            .with_mutation(RandomValueMutator::new(0.2, task.stage_left(), task.stage_right()))
            .with_reinsertion(ElitistReinserter::new(task, false, 0.85))
            .with_initial_population(initial_population)
            .build(),
    )
    .until(GenerationLimit::new(20))
    .build();

    'sim: loop {
        let result = task_sim.step();

        match result {
            Ok(SimResult::Intermediate(step)) => {
                let evaluated_population = step.result.evaluated_population;
                let best_solution = step.result.best_solution;
                println!(
                    "step: generation: {}, average_fitness: {}, \
                     best fitness: {}, duration: {}, processing_time: {}",
                    step.iteration,
                    evaluated_population.average_fitness(),
                    best_solution.solution.fitness,
                    step.duration.fmt(),
                    step.processing_time.fmt(),
                );
                // let Task = best_solution
                //     .solution
                //     .genome
                //     .as_task(&problem.given_items);
                // println!(
                //     "      Task: number of items: {}, total value: {}, total weight: {}",
                //     Task.items.len(),
                //     Task.value,
                //     Task.weight
                // );
            },
            Ok(SimResult::Final(step, processing_time, duration, stop_reason)) => {
                let best_solution = step.result.best_solution;
                println!("{}", stop_reason);
                println!(
                    "Final result after {}: generation: {}, \
                     best genome with fitness {} found in generation {}, processing_time: {}",
                    duration.fmt(),
                    step.iteration,
                    best_solution.solution.fitness,
                    best_solution.generation,
                    processing_time.fmt(),
                );
                // let Task = best_solution
                //     .genome
                //     .genome
                //     .as_task(&problem.given_items);
                // println!(
                //     "      Task: number of items: {}, total value: {}, total weight: {}",
                //     Task.items.len(),
                //     Task.value,
                //     Task.weight
                // );
                return genome_to_solution(&best_solution.solution.genome);
            },
            Err(error) => {
                println!("{}", error);
                break 'sim;
            },
        }
    }

    solution.clone()
}