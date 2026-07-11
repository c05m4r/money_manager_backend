// Copyright (C) 2026 Marcos Gabriel Miller
use serde::Serialize;
use utoipa::ToSchema;

#[derive(Debug, Serialize, ToSchema)]
pub struct Meta {
    pub total_records: u64,
    pub current_page: u64,
    pub total_pages: u64,
    pub per_page: u64,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct Links {
    #[serde(rename = "self")]
    pub self_link: String,
    pub next: Option<String>,
    pub prev: Option<String>,
    pub first: String,
    pub last: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct PaginatedResponse<T: Serialize + ToSchema> {
    pub data: Vec<T>,
    pub meta: Meta,
    pub links: Links,
}

pub fn build_links(base_path: &str, page: u64, per_page: u64, total_pages: u64) -> Links {
    let mk = |p: u64| format!("{base_path}?page={p}&per_page={per_page}");
    Links {
        self_link: mk(page),
        next: (page < total_pages).then(|| mk(page + 1)),
        prev: (page > 1).then(|| mk(page - 1)),
        first: mk(1),
        last: mk(total_pages.max(1)),
    }
}
