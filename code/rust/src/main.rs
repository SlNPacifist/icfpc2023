mod io;
mod solution;

fn main() {
    for i in 1..45 {
        let task = io::read(&format!("../../data/problem-{i}.json"));
        io::write(&format!("../../solutions/problem-{i}.json"), &solution::dummy(&task));
    }
}
