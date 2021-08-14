use actix_web::{web, HttpResponse, Responder};
use serde;

#[derive(serde::Deserialize)]
pub struct FormData {
    email : String,
    name : String
}

pub async fn subscriptions(form : web::Form<FormData>) -> impl Responder {
    HttpResponse::Ok()
}