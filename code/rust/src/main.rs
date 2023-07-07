use crate::io::MUSICIAN_RADIUS;

mod geom;
mod io;
mod optimizer;
mod score;
mod solution;

fn main() {
    for i in 1..=45 {
        let task = io::read(&format!("../../data/problem-{i}.json"));

        let solution = {
            let task = task.clone();
            // dummy solutions are row-based on width, and we have stages, narrow by height
            let transpose = task.stage_height < task.stage_width;
            let task = if transpose {
                println!("Transposing task {i}");
                task.transpose()
            } else {
                task
            };

            let task = task.simplify();

            let solver = if task.stage_width < 3.0 * MUSICIAN_RADIUS {
                println!("Using narrow solver for task {i}");
                solution::dummy_narrow
            } else {
                solution::dummy_hex
            };

            let solution = solver(&task);
            if transpose {
                solution.transpose()
            } else {
                solution
            }
        };

        let visibility = score::calc_visibility(&task, &solution);

        match score::calc(&task, &solution, &visibility) {
            Result::Ok(points) => {
                println!("Solution for task {i} got {points} points before optimization")
            }
            Result::Err(err) => {
                println!("Solution for task {i} is incorrect before optimization: {err}")
            }
        }

        let optimized_solution =
            optimizer::optimize_placements_greedy(&task, &solution, &visibility);
        let optimized_visibility = score::calc_visibility(&task, &optimized_solution);
        match score::calc(&task, &optimized_solution, &optimized_visibility) {
            Result::Ok(points) => {
                println!("Solution for task {i} got {points} points after optimization")
            }
            Result::Err(err) => {
                println!("Solution for task {i} is incorrect after optimization: {err}")
            }
        }

        io::write(&format!("../../solutions/problem-{i}.json"), &optimized_solution);
    }
}
