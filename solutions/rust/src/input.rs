use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct User {
    #[serde(rename="userId")]
    pub user_id: usize,
    pub id: usize,
    pub title: String,
    pub completed: bool,
}

pub fn read() -> Vec<User> {
    let data = r#"
    [
        {
          "userId": 1,
          "id": 1,
          "title": "delectus aut autem",
          "completed": false
        }
    ]
    "#;
    
    serde_json::from_str(data).expect("Could not parse data")
}