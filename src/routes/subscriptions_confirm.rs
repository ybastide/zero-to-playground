use actix_web::{web, HttpResponse};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Parameters {
    subscription_token: String,
}

#[tracing::instrument(
    name = "Confirm a pending subscriber"
    // fields(
    // subscription_token
    // )
)
]
pub async fn confirm(
    parameters: web::Query<Parameters>,
    // base_url: web::Data<String>,
) -> HttpResponse {
    HttpResponse::Ok().finish()
}
