use actix_web::{get, web, HttpResponse};
use serde::{Deserialize, Serialize};
use std::fs; // 将 json 字符串解析为结构体

use crate::database::mysql::*;

// const IP_PORT: &str = "127.0.0.1:8082";

// 封面获取函数
#[get("/avatar/{user_id}")]
pub async fn get_avatar(user_id: web::Path<u32>) -> actix_web::Result<HttpResponse> {
    // println!("get_avatar");
    let user_id = *user_id;
    let avatar_path = format!("./static/avatar/avatar-{}.jpg", user_id);

    let avatar_data = match fs::read(avatar_path) {
        Ok(bytes) => bytes,
        Err(err) => return Err(actix_web::error::ErrorInternalServerError(err)),
    };

    // 返回包含图片数据的 HTTP 响应
    Ok(HttpResponse::Ok()
        .content_type("image/jpeg") // 指定图片的 MIME 类型
        .body(avatar_data))
}

// 创建一个帖子的结构体，用来发送数据
#[derive(Debug, Deserialize, Serialize)]
struct User {
    id: u32,
    name: String,
    avatar_url: String,
}

// 获取单个用户
#[get("/user/{user_id}")]
pub async fn get_user(user_id: web::Path<u32>) -> actix_web::Result<HttpResponse> {
    let user_id = *user_id;

    // 获取线程池，这个线程池为单例模式
    let my_pool = MysqlPool::instance();

    let query = format!("SELECT id, name, avatar_url FROM user WHERE id = {}", user_id);
    let user: Vec<User> = match my_pool.query_map(
        query, 
        |(id, name, avatar_url): (u32, String, String)| {
            User {id, name, avatar_url}
        },
        &my_pool.read_only_txopts
    )
    {
        Ok(result) => result,
        Err(err) => {
            eprintln!("get_user: Error query_map query: {:?}", err);
            return Err(actix_web::error::ErrorInternalServerError("Internal Server Error"));
        }
    };

    println!("user_id: {}, {:?}", user_id, user);

    let post_jsons = serde_json::to_string(&user[0])
    .map_err(|err| {
        eprintln!("Error serializing response: {:?}", err);
        actix_web::error::ErrorInternalServerError("Error serializing response")
    })?;

    Ok(HttpResponse::Ok().body(post_jsons))
}
