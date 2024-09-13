use serde::Deserialize;

#[derive(Deserialize)]
pub struct Board([[Option<char>; 10]; 10]);

impl Board {
    // pub async fn new
}
