use rouille;
use crate::{write_optimal_solution, read_task};
use crate::score::{calc_visibility_fast, calc};

pub fn start_server() {
    rouille::start_server("localhost:8000", move |request| {
        router!(request,
            (POST) (/solution/update/{id: usize}) => {
                let text = rouille::input::plain_text_body_with_limit(request, 1000000000).expect("input expected");
                let solution = serde_json::from_str(&text).expect("Could not parse data");
                let task = read_task(id);
                let visibility = calc_visibility_fast(&task, &solution);
                let points = calc(&task, &solution, &visibility).unwrap_or(-1000000000000);
                write_optimal_solution(&task, &solution, points, id);
                rouille::Response::text("OK")
            },

            _ => rouille::Response::empty_404()
        )
    });    
}