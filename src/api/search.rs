use actix_web::{get, web, HttpResponse};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::fs; // 将 json 字符串解析为结构体

use crate::database::mysql::*;

// const IP_PORT: &str = "127.0.0.1:8082";

#[derive(Debug, Serialize, Deserialize)]
struct Search {
    search_query: String,
    page: u32,
}

// 搜索函数
#[get("/search")]
pub async fn get_search(info: web::Query<Search>) -> actix_web::Result<HttpResponse> {
    log::info!("Satrt get_search function");

    let Search { search_query, page } = info.into_inner();
    let start = (page - 1) * 20;
    println!(
        "search_query: {}, page: {}, start: {}",
        search_query, page, start
    );

    // 获取线程池，这个线程池为单例模式
    let my_pool = MysqlPool::instance();

    let query = format!(
        "SELECT id, title, release_time, cover_url, content_url, user_id, user_name
        FROM post
        LIMIT {}, 20",
        0
    );

    let posts: Vec<Post> = match my_pool.query_map(
        query,
        |(id, title, release_time, cover_url, content_url, user_id, user_name): (
            u32,
            String,
            String,
            String,
            String,
            u32,
            String,
        )| {
            let content = fs::read_to_string(content_url)
                .expect("get_search: Failed fs::read_to_string content_url");
            Post {
                id,
                title,
                release_time,
                cover_url,
                content,
                user_id,
                user_name,
            }
        },
        &my_pool.read_only_txopts,
    ) {
        Ok(result) => result,
        Err(err) => {
            log::error!("get_search: Error query_map query: {:?}", err);
            // eprintln!("get_search: Error query_map query: {:?}", err);
            return Err(actix_web::error::ErrorInternalServerError(
                "Internal Server Error",
            ));
        }
    };

    let post_jsons = serde_json::to_string(&json!({
        "total": 55,
        "posts": posts,
    }))
    .map_err(|err| {
        log::error!("Error get_search serializing response: {:?}", err);
        // eprintln!("Error serializing response: {:?}", err);
        actix_web::error::ErrorInternalServerError("Error serializing response")
    })?;

    log::info!("End get_search function");
    Ok(HttpResponse::Ok().body(post_jsons))
}
