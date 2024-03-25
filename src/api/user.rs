use actix_web::{get, web, HttpResponse};
use serde::{Deserialize, Serialize};
use std::fs; // 将 json 字符串解析为结构体

// const IP_PORT: &str = "127.0.0.1:8082";

// 封面获取函数
#[get("/avatar/{user_id}")]
pub async fn get_avatar(info: web::Path<(u32,)>) -> actix_web::Result<HttpResponse> {
    // println!("get_avatar");
    let avatar_path = format!("./static/avatar/avatar-{}.jpg", &info.0);

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
    id: usize,
    name: String,
    register_time: String,
    avatar_url: String,
}

// 获取单个用户
#[get("/user/{user_id}")]
pub async fn get_user(_: web::Path<u32>) -> actix_web::Result<HttpResponse> {
    // println!("get_recommend_posts_list, {:?}", env::current_dir());
    let user = User {
        id: 666,
        name: String::from("username666"),
        register_time: String::from("2024-03-04 18:36"),
        avatar_url: String::from("https://127.0.0.1:8082/api/avatar/0"),
    };

    let post_jsons = serde_json::to_string(&user).expect("Failed to serialize posts");

    Ok(HttpResponse::Ok().body(post_jsons))
}
