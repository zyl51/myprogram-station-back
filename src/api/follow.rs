use actix_web::{get, web, HttpResponse};
use serde::{Deserialize, Serialize};
use std::env;
use std::fs; // 将 json 字符串解析为结构体

#[derive(Debug, Serialize)]
struct Number {
    number: u32,
}

#[get("/follow/totalnumbers/{user_id}")]
pub async fn get_follow_post_total_numbers(_: web::Path<u32>) -> actix_web::Result<HttpResponse> {
    println!("get_follow_post_total_numbers");
    let number = Number { number: 120 };
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

// 用于存储客户端寻求的用户 id 和 page 页数
#[derive(serde::Deserialize)]
struct FollowPost {
    user_id: u32,
    page: u32,
}

// 获取自己关注的帖子列表
#[get("/follow/postlist")]
pub async fn get_follow_posts_list(
    info: web::Query<FollowPost>,
) -> actix_web::Result<HttpResponse> {
    let FollowPost { user_id, page } = info.into_inner();
    print!("{}, {} ", user_id, page);
    println!("get_follow_posts_list, {:?}", env::current_dir());
    let posts = vec![
        Post {
            id: 1,
            title: String::from("Follow Post 1"),
            release_time: String::from("2024-03-04 18:36"),
            cover_url: String::from("https://127.0.0.1:8082/api/cover/0"),
            content: fs::read_to_string("./static/content/content-0.md")
                .expect("content reading failed"),
            user_id: 1,
            user_name: String::from("username1"),
        },
        Post {
            id: 2,
            title: String::from("Follow Post 2"),
            release_time: String::from("2024-03-04  18:36"),
            cover_url: String::from("https://127.0.0.1:8082/api/cover/0"),
            content: fs::read_to_string("./static/content/content-0.md")
                .expect("content reading failed"),
            user_id: 2,
            user_name: String::from("username2"),
        },
    ];

    let post_jsons = serde_json::to_string(&posts).expect("Failed to serialize posts");

    Ok(HttpResponse::Ok().body(post_jsons))
}
