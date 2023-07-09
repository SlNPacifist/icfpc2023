use crate::score::{calc, calc_ex, calc_visibility_fast};
use crate::{read_task, write_optimal_solution};
use rouille;

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

            (POST) (/solution/score/{id: usize}) => {
                let text = rouille::input::plain_text_body_with_limit(request, 1000000000).expect("input expected");
                let solution = serde_json::from_str(&text).expect("Could not parse data");
                let task = read_task(id);
                let visibility = calc_visibility_fast(&task, &solution);
                rouille::Response::text(serde_json::to_string(&calc_ex(&task, &solution, &visibility)).expect("Could not format score_ex json"))
            },

            _ => rouille::Response::empty_404()
        )
    });
}
