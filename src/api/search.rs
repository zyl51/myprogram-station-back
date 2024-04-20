use actix_web::{get, web, HttpResponse};
use serde::{Deserialize, Serialize};
use std::collections::{HashSet, HashMap};
use serde_json::json;
use std::fs; // 将 json 字符串解析为结构体

use crate::database::mysql::*;

// const IP_PORT: &str = "127.0.0.1:8082";

#[derive(Debug, Serialize, Deserialize)]
struct Search {
    search_query: String,
    page: u32,
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

    // 获取搜索的数量
    let query = format!("
        SELECT count(*)
        FROM post
        where title like '%{}%';
    ", search_query);

    // println!("{}", query);

    let numbers: Vec<u32> = match my_pool.exec(&query, &my_pool.read_only_txopts) {
        Ok(result) => result,
        Err(err) => {
            log::error!(
                "Error get_search executing query: {:?}",
                err
            );
            // eprintln!("Error executing query: {:?}", err);
            return Err(actix_web::error::ErrorInternalServerError(
                "Internal Server Error",
            ));
        }
    };

    // 文章的数量
    let number = numbers[0];

    let query = format!(
        "SELECT id, title, release_time, cover_url, content_url, user_id
        FROM post
        where title like '%{}%'
        LIMIT {}, 20",
        search_query, start
    );

    println!("{}", query);

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
                .expect("get_search: Failed fs::read_to_string content_url");
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
            log::error!("get_search: Error query_map query: {:?}", err);
            // eprintln!("get_search: Error query_map query: {:?}", err);
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
             log::error!("Error get_search executing query: {:?}", err);
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

    let post_jsons = serde_json::to_string(&json!({
        "total": number,
        "posts": result,
    }))
    .map_err(|err| {
        log::error!("Error get_search serializing response: {:?}", err);
        // eprintln!("Error serializing response: {:?}", err);
        actix_web::error::ErrorInternalServerError("Error serializing response")
    })?;

    log::info!("End get_search function");
    Ok(HttpResponse::Ok().body(post_jsons))
}
