mod io;

fn main() {
    println!("Hello, world!");
    // println!("{:?}", io::read("../../data/problem-1.json"));
    io::write("../../solutions/problem-1.json", &io::Output::default());
}
