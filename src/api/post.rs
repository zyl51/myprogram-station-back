use actix_web::{get, web, HttpResponse};
use std::fs; // 将 json 字符串解析为结构体

use crate::database::mysql::*;

// const IP_PORT: &str = "127.0.0.1:8082";

// 封面获取函数
#[get("/cover/{post_id}")]
pub async fn get_cover(post_id: web::Path<u32>) -> actix_web::Result<HttpResponse> {
    // println!("get_cover");
    let post_id = *post_id;
    let cover_path = format!("./static/cover/cover-{}.jpg", post_id);

    let cover_data = match fs::read(cover_path) {
        Ok(bytes) => bytes,
        Err(err) => return Err(actix_web::error::ErrorInternalServerError(err)),
    };

    // 返回包含图片数据的 HTTP 响应
    Ok(HttpResponse::Ok()
        .content_type("image/jpeg") // 指定图片的 MIME 类型
        .body(cover_data))
}

// 获取单个帖子
#[get("/post/{post_id}")]
pub async fn get_post(post_id: web::Path<u32>) -> actix_web::Result<HttpResponse> {
    let post_id = *post_id;

    // 获取线程池，这个线程池为单例模式
    let my_pool = MysqlPool::instance();
    // 获取起时下标条数
    let query = format!("SELECT id, title, release_time, 
        cover_url, content_url, user_id, user_name FROM post where id = {}", post_id);

    // 获取帖子列表
    let post: Vec<Post> = match my_pool.query_map(
        query,
        |(id, title, release_time, cover_url, content_url, user_id, user_name)
        : (u32, String, String, String, String, u32, String)| {
            let content = fs::read_to_string(content_url)
                .expect("get_post: Failed fs::read_to_string content_url");
            Post { id, title, release_time, cover_url, content, user_id, user_name }
        },
        &my_pool.read_only_txopts
    ) 
    {
        Ok(result) => result,
        Err(err) => {
            eprintln!("get_post: Error query_map query: {:?}", err);
            return Err(actix_web::error::ErrorInternalServerError("Internal Server Error"));
        }
    };

    // 解析错误
    let post_jsons = serde_json::to_string(&post[0])
        .map_err(|err| {
            eprintln!("get_post: Error serializing response: {:?}", err);
            actix_web::error::ErrorInternalServerError("Error serializing response")
        })?;

    Ok(HttpResponse::Ok().body(post_jsons))
}
