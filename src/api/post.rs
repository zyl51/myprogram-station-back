use actix_web::{get, web, HttpResponse};
use serde::{Deserialize, Serialize};
use std::fs; // 将 json 字符串解析为结构体

// const IP_PORT: &str = "127.0.0.1:8082";

// 封面获取函数
#[get("/cover/{post_id}")]
pub async fn get_cover(info: web::Path<(u32,)>) -> actix_web::Result<HttpResponse> {
    // println!("get_cover");
    let cover_path = format!("./static/cover/cover-{}.jpg", &info.0);

    let cover_data = match fs::read(cover_path) {
        Ok(bytes) => bytes,
        Err(err) => return Err(actix_web::error::ErrorInternalServerError(err)),
    };

    // 返回包含图片数据的 HTTP 响应
    Ok(HttpResponse::Ok()
        .content_type("image/jpeg") // 指定图片的 MIME 类型
        .body(cover_data))
}

// 创建一个帖子的结构体，用来发送数据
#[derive(Debug, Deserialize, Serialize)]
struct Post {
    id: usize,
    title: String,
    release_time: String,
    cover_url: String,
    content: String,
    user_id: usize,
    user_name: String,
}

// 获取单个帖子
#[get("/post/{post_id}")]
pub async fn get_post(_: web::Path<u32>) -> actix_web::Result<HttpResponse> {
    // println!("get_recommend_posts_list, {:?}", env::current_dir());
    let post = Post {
        id: 666,
        title: String::from("Post 666"),
        release_time: String::from("2024-03-04 18:36"),
        cover_url: String::from("https://127.0.0.1:8082/api/cover/0"),
        content: fs::read_to_string("./static/content/content-0.md")
            .expect("content reading failed"),
        user_id: 666,
        user_name: String::from("username666"),
    };

    let post_jsons = serde_json::to_string(&post).expect("Failed to serialize posts");

    Ok(HttpResponse::Ok().body(post_jsons))
}
