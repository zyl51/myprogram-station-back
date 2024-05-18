use actix_web::{get, web, HttpResponse};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use serde_json::json;

use crate::database::{mysql::*, elasticsearch::*};

// const IP_PORT: &str = "127.0.0.1:8082";

#[derive(Debug, Serialize, Deserialize)]
struct Search {
    search_query: String,
    page: u32,
}

#[derive(Debug)]
struct MyPost {
    pub release_time: String,
    pub cover_url: String,
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

    // 获取 ES 线程池，这个线程池为单例模式
    let elasticsearch_pool = ElasticsearchPool::instance();

    // 得到查询结果，为 json 格式
    let response_body: serde_json::Value = elasticsearch_pool.search_article(start, &search_query).await?;
    let result_array = response_body["hits"]["hits"].as_array().unwrap();

    // 得到查询结果的长度
    let number = result_array.len();

    if number == 0 {
        return Err(actix_web::error::ErrorBadRequest("无结果"));
    }

    // println!("{}", number);

    // 将文章 id 提取出来
    let post_ids: Vec<u32> = result_array.iter()
        .map(|hit| hit["_id"].as_str().unwrap().parse().unwrap())
        .collect();

    // 构建参数
    let params = post_ids
        .iter()
        .map(|post_id: &u32| post_id.to_string())
        .collect::<Vec<String>>()
        .join(",");

    // 获得数据库连接池
    let my_pool = MysqlPool::instance();

    // 查询文章的发布时间、封面路由、作者id
    let query = format!("
        SELECT release_time, cover_url, user_id
        FROM post
        where id in ({});
    ", params);

    // println!("{}", query);

    let posts: Vec<MyPost> = match my_pool.query_map(
        query,
        |(release_time, cover_url, user_id): (
            String,
            String,
            u32,
        )| {
            MyPost {
                release_time,
                cover_url,
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

    // println!("posts: {:?}", posts);

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
 
    // 通过 user_id 将 user_id 和 user_name 查出来
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
    let mut result: Vec<Post> = Vec::new();
    for i in 0..result_array.len() {
        let hit = result_array[i]["_source"].clone();
        result.push(Post {
            id: post_ids[i],
            title: hit["title"].as_str().unwrap().to_string(),
            release_time: posts[i].release_time.clone(),
            cover_url: posts[i].cover_url.clone(),
            content: hit["content"].as_str().unwrap().to_string(),
            user_id: posts[i].user_id,
            user_name: user_map
                .get(&posts[i].user_id)
                .cloned()
                .unwrap_or_else(|| "编程驿站一份子".to_string()),
        });
    }

    // println!("{:?}", result);


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
