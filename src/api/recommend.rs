use actix_web::{get, web, HttpResponse};
use serde::Serialize;
use std::fs; // 将 json 字符串解析为结构体

use crate::database::mysql::*;

// const IP_PORT: &str = "127.0.0.1:8082";

#[derive(Debug, Serialize)]
struct Number {
    number: u32,
}

// 推荐列表的总数
#[get("/recommend/totalnumbers")]
pub async fn get_recommend_post_total_numbers() -> actix_web::Result<HttpResponse> {
    log::debug!("Start get_recommend_post_total_numbers function");
    // 获取线程池，这个线程池为单例模式
    let my_pool = MysqlPool::instance();

    // 事务查询帖子总数量的数据
    let query = format!("SELECT COUNT(*) FROM post;");
    let numbers: Vec<(u32,)> = match my_pool.exec(&query, &my_pool.read_only_txopts) {
        Ok(result) => result,
        Err(err) => {
            log::error!(
                "Error get_recommend_post_total_numbers executing query: {:?}",
                err
            );
            // eprintln!("Error executing query: {:?}", err);
            return Err(actix_web::error::ErrorInternalServerError(
                "Internal Server Error",
            ));
        }
    };

    let number = numbers[0].0;

    let json_response = serde_json::to_string(&Number { number }).map_err(|err| {
        // eprintln!("Error serializing response: {:?}", err);
        log::error!(
            "Error get_recommend_post_total_numbers serializing response: {:?}",
            err,
        );
        actix_web::error::ErrorInternalServerError("Error serializing response")
    })?;

    log::debug!("End get_recommend_post_total_numbers function");
    Ok(HttpResponse::Ok()
        .content_type("application/json")
        .body(json_response))
}

// 获取推荐页中的第 page 页帖子列表
#[get("/recommend/postlist/{page}")]
pub async fn get_recommend_posts_list(page: web::Path<u32>) -> actix_web::Result<HttpResponse> {
    log::debug!("Start get_recommend_posts_list functionn");
    let page = *page;

    // 获取起时下标条数
    let start = (page - 1) * 10;
    let query = format!(
        "SELECT id, title, release_time, cover_url, content_url, user_id, user_name
        FROM post
        order by release_time desc
        LIMIT {}, 10",
        start
    );

    // 获取线程池，这个线程池为单例模式
    let my_pool = MysqlPool::instance();

    // 将查询的值映射到数结构体中
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
                .expect("get_follow_posts_list: Failed fs::read_to_string content_url");
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
            // eprintln!("Error executing query: {:?}", err);
            log::error!("Error get_recommend_posts_list executing query: {:?}", err);
            return Err(actix_web::error::ErrorInternalServerError(
                "Internal Server Error",
            ));
        }
    };

    let post_jsons = serde_json::to_string(&posts).map_err(|err| {
        // eprintln!("Error serializing response: {:?}", err);
        log::error!(
            "Error get_recommend_posts_list serializing response: {:?}",
            err
        );
        actix_web::error::ErrorInternalServerError("Error serializing response")
    })?;

    log::debug!("End get_recommend_posts_list function");
    Ok(HttpResponse::Ok().body(post_jsons))
}
