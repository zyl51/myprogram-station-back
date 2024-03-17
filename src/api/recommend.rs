use actix_web::{get, web, HttpResponse};
use serde::{Deserialize, Serialize};
// use std::env;
use std::fs; // 将 json 字符串解析为结构体

// const IP_PORT: &str = "127.0.0.1:8082";

#[derive(Debug, Serialize)]
struct Number {
    number: u32,
}

#[get("/recommend/totalnumbers")]
pub async fn get_recommend_post_total_numbers() -> actix_web::Result<HttpResponse> {
    let number = Number { number: 550 };
    let json_response = serde_json::to_string(&number)?;

    Ok(HttpResponse::Ok()
        .content_type("application/json")
        .body(json_response))
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

// 封面获取函数
#[get("/cover/{post_id}")]
pub async fn get_cover(info: web::Path<(String,)>) -> actix_web::Result<HttpResponse> {
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

// 获取推荐页中的第 page 页帖子列表
#[get("/recommend/postlist/{page}")]
pub async fn get_recommend_posts_list(_: web::Path<u32>) -> actix_web::Result<HttpResponse> {
    // println!("get_recommend_posts_list, {:?}", env::current_dir());
    let posts = vec![
        Post {
            id: 1,
            title: String::from("Post 1"),
            release_time: String::from("2024-03-04 18:36"),
            cover_url: String::from("https://127.0.0.1:8082/api/cover/0"),
            content: fs::read_to_string("./static/content/content-0.md")
                .expect("content reading failed"),
            user_id: 1,
            user_name: String::from("username1"),
        },
        Post {
            id: 2,
            title: String::from("Post 2"),
            release_time: String::from("2024-03-04  18:36"),
            cover_url: String::from("https://127.0.0.1:8082/api/cover/0"),
            content: fs::read_to_string("./static/content/content-0.md")
                .expect("content reading failed"),
            user_id: 2,
            user_name: String::from("username2"),
        },
        Post {
            id: 3,
            title: String::from("Post 3"),
            release_time: String::from("2024-03-04  18:36"),
            cover_url: String::from("https://127.0.0.1:8082/api/cover/0"),
            content: fs::read_to_string("./static/content/content-0.md")
                .expect("content reading failed"),
            user_id: 3,
            user_name: String::from("username3"),
        },
        Post {
            id: 4,
            title: String::from("Post 4"),
            release_time: String::from("2024-03-04  18:36"),
            cover_url: String::from("https://127.0.0.1:8082/api/cover/0"),
            content: fs::read_to_string("./static/content/content-0.md")
                .expect("content reading failed"),
            user_id: 4,
            user_name: String::from("username4"),
        },
        Post {
            id: 5,
            title: String::from("Post 5"),
            release_time: String::from("2024-03-04  18:36"),
            cover_url: String::from("https://127.0.0.1:8082/api/cover/0"),
            content: fs::read_to_string("./static/content/content-0.md")
                .expect("content reading failed"),
            user_id: 5,
            user_name: String::from("username5"),
        },
    ];

    let post_jsons = serde_json::to_string(&posts).expect("Failed to serialize posts");

    Ok(HttpResponse::Ok().body(post_jsons))
}
