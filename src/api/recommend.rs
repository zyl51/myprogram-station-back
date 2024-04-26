use actix_web::{get, web, HttpResponse};
use serde::Serialize;
use std::collections::{HashMap, HashSet};
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

#[derive(Debug)]
struct MyPost {
    pub id: u32,
    pub title: String,
    pub release_time: String,
    pub cover_url: String,
    pub content: String,
    pub user_id: u32,
}

#[derive(Debug)]
struct MyUser {
    pub user_id: u32,
    pub user_name: String,
}

// 获取推荐页中的第 page 页帖子列表
#[get("/recommend/postlist/{page}")]
pub async fn get_recommend_posts_list(page: web::Path<u32>) -> actix_web::Result<HttpResponse> {
    log::debug!("Start get_recommend_posts_list functionn");
    let page = *page;

    // 获取起时下标条数
    let start = (page - 1) * 10;
    let query = format!(
        "SELECT id, title, release_time, cover_url, content_url, user_id
        FROM post
        order by release_time desc
        LIMIT {}, 10",
        start
    );

    // 获取线程池，这个线程池为单例模式
    let my_pool = MysqlPool::instance();

    // 将查询的值映射到数结构体中
    let posts: Vec<MyPost> = match my_pool.query_map(
        query,
        |(id, title, release_time, cover_url, content_url, user_id): (
            u32,
            String,
            String,
            String,
            String,
            u32,
        )| {
            let content = fs::read_to_string(content_url)
                .expect("get_recommend_posts_list: Failed fs::read_to_string content_url");
            MyPost {
                id,
                title,
                release_time,
                cover_url,
                content,
                user_id,
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

    // 将用户的 id 提取出来并且去重
    let user_ids: HashSet<u32> = posts.iter().map(|post| post.user_id).collect();

    // 构建数据库的查询参数
    let params = user_ids
        .iter()
        .map(|user_id| user_id.to_string())
        .collect::<Vec<String>>()
        .join(",");

    // 构建查询语句
    let query = format!(
        "
        select id, name
        from user
        where id in ({});
    ",
        params
    );

    // 通过 user_id 和 user_id 和 user_name 查出来
    let users: Vec<MyUser> = match my_pool.query_map(
        query,
        |(user_id, user_name): (u32, String)| MyUser { user_id, user_name },
        &my_pool.read_only_txopts,
    ) {
        Ok(ok) => ok,
        Err(err) => {
            log::error!("Error get_recommend_posts_list executing query: {:?}", err);
            return Err(actix_web::error::ErrorInternalServerError(
                "Internal Server Error",
            ));
        }
    };

    // 将用户数据映射到 HashMap 中
    let user_map: HashMap<u32, String> = users
        .into_iter()
        .map(|user| (user.user_id, user.user_name))
        .collect();

    // 合并帖子和用户数据
    let result: Vec<Post> = posts
        .into_iter()
        .map(|post| Post {
            id: post.id,
            title: post.title,
            release_time: post.release_time,
            cover_url: post.cover_url,
            content: post.content,
            user_id: post.user_id,
            user_name: user_map
                .get(&post.user_id)
                .cloned()
                .unwrap_or_else(|| "编程驿站一份子".to_string()),
        })
        .collect();

    let post_jsons = serde_json::to_string(&result).map_err(|err| {
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
