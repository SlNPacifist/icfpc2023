mod io;
mod score;
mod solution;

fn main() {
    for i in 1..45 {
        let task = io::read(&format!("../../data/problem-{i}.json"));
        let solution = solution::dummy(&task);
        match score::calc(&task, &solution) {
            Result::Ok(points) => println!("Solution for task {i} got {points} points"),
            Result::Err(err) => println!("Solution for task {i} is incorrect: {err}"),
        }

        io::write(&format!("../../solutions/problem-{i}.json"), &solution);
    }
}
