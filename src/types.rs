use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct Post {
    id: i32,
    score: i32,
    rating: String,
    file_ext: String,
    file_url: String,
    large_file_url: String,
}

