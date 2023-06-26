use std::iter::Map;

use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DownloadRequest {
    pub words: Vec<String>
}

pub struct Report {
    pub status: String,
    pub not_found: Map<String, Vec<String>>,
    pub found: Map<String, Vec<String>>
}