use actix_web::{post, web, HttpResponse};
use serde::{Deserialize, Serialize};
use tokio::time::{sleep, Duration};

use crate::{common::email::*, database::mysql::*};

struct 

#[post("verify/login")]
pub async fn user_login() -> actix_web::Result<HttpResponse> {
    
}