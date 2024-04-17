use actix_web::{post, HttpRequest, HttpResponse};
use serde::{Deserialize, Serialize};

use crate::{database::mysql::*, common::token::*};

#[derive(Debug, Deserialize, Serialize, Clone)]
struct User {
    id: u32,
    name: String,
    avatar_url: String,
}

#[post("/token_get_userinfo")]
pub async fn token_get_userinfo(req: HttpRequest) -> actix_web::Result<HttpResponse> {
    log::debug!("Start token_get_userinfo function");
    println!("token_get_userinfo");
    if Token::verif_jwt(req.clone()).is_err() {
        return Ok(HttpResponse::BadRequest().body("Failed is verif token"));
    }

    // 通过 token 获得用户信息
    let claims = match Token::token_to_claims(req) {
        Ok(ok) => ok, 
        Err(err) => {
            log::error!("Error is Token verif_jwt");
            log::error!(
                "Error executing token_get_userinfo function token_to_claims: {:?}",
                err
            );
            return Err(actix_web::error::ErrorInternalServerError(
                "Internal Server Error",
            ));
        }
    };

    // 获取用户 id
    let id = claims.get_id();

    let my_pool = MysqlPool::instance();

    // 查询用户的个人信息
    let query = format!(
        "SELECT id, name, avatar_url FROM user WHERE id = {}",
        id
    );
    let user: Vec<User> = match my_pool.query_map(
        query,
        |(id, name, avatar_url): (u32, String, String)| User {
            id,
            name,
            avatar_url,
        },
        &my_pool.read_only_txopts,
    ) {
        Ok(result) => result,
        Err(err) => {
            log::error!("Error token_get_userinfo query_map query: {:?}", err);
            // eprintln!("get_user: Error query_map query: {:?}", err);
            return Err(actix_web::error::ErrorInternalServerError(
                "Internal Server Error",
            ));
        }
    };

    // 获取到用户数据
    let User {
        id,
        name,
        avatar_url,
    } = user[0].clone();

    // 获取用户的粉丝数量
    let query = format!(
        "select follower_count, fans from user_stats where user_id = {}",
        id
    );
    let user_stats: Vec<(u32, u32)> = match my_pool.exec(&query, &my_pool.read_only_txopts) {
        Ok(result) => result,
        Err(err) => {
            // eprintln!("Error executing query: {:?}", err);
            log::error!("Error executing token_get_userinfo function query {:?}", err);
            return Err(actix_web::error::ErrorInternalServerError(
                "Internal Server Error",
            ));
        }
    };

    let avatar_url = if avatar_url.len() != 0 {
        avatar_url
    } else {
        AVATAR_URL.to_string()
    };

    let token = Token::get_jwt(&claims);
    if token.is_err() {
        log::error!("Error token_get_userinfo Token get jwt: {:?}", token);
        return Ok(
            HttpResponse::InternalServerError().body(serde_json::to_string("服务器繁忙").unwrap())
        );
    }
    let token = token.unwrap();

    let login_info = UserInfomation {
        id: id,
        name: name,
        avatar_url: avatar_url, // 默认头像路径
        follower_count: user_stats[0].0,
        fans: user_stats[0].1,
        token: token,
    };

    let response = serde_json::to_string(&login_info);
    if response.is_err() {
        log::error!("Error verify_verification_code serde_json: {:?}", response);

        return Ok(
            HttpResponse::InternalServerError().body(serde_json::to_string("服务器繁忙").unwrap())
        );
    }

    let response = response.unwrap();

    log::debug!("End token_get_userinfo function");
    Ok(HttpResponse::Ok().body(response))
}