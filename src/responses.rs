use serde::Serialize;

#[derive(Serialize)]
pub struct Meta {
    pub current_page: i64,
    pub per_page: i64,
    pub from: i64,
    pub to: i64,
    pub total_pages: i64,
    pub total_docs: i64,
}

#[derive(Serialize)]
pub struct PaginatedResponse<T> {
    pub records: Vec<T>,
    pub meta: Meta,
}
