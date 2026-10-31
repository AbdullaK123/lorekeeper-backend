use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct PageResponse<T> {
    pub data: T,
    pub next_cursor: Option<String>,
}

impl<T> From<(T, Option<String>)> for PageResponse<T> {
    fn from((data, next_cursor): (T, Option<String>)) -> Self {
        Self { data, next_cursor }
    }
}
