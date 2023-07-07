mod io;
mod score;
mod solution;

fn main() {
    for i in 1..45 {
        let task = io::read(&format!("../../data/problem-{i}.json"));

        let solution = {
            // dummy solutions are row-based on width, and we have stages, narrow by height
            let transpose = task.stage_height < task.stage_width;
            let task = if transpose {
                println!("Transposing task {i}");
                task.clone().transpose()
            } else {
                task.clone()
            };

            // TODO use narrow solution if width is less than 3r

            let solution = solution::dummy_hex(&task);
            if transpose {
                solution.transpose()
            } else {
                solution
            }
        };

        match score::calc(&task, &solution) {
            Result::Ok(points) => println!("Solution for task {i} got {points} points"),
            Result::Err(err) => println!("Solution for task {i} is incorrect: {err}"),
        }

        io::write(&format!("../../solutions/problem-{i}.json"), &solution);
    }
}
