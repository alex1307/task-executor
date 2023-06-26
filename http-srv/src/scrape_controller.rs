use actix_web::{HttpResponse, post};
use actix_web::web::Json;


use crate::model::DownloadRequest;

#[post("/")]
pub async fn download(_words: Json<DownloadRequest>) -> HttpResponse {
    HttpResponse::Accepted().body("Hello, Mr. World")
}