use actix_web::{get, web, HttpResponse};
use serde::Serialize;
use std::fs; // 将 json 字符串解析为结构体

use crate::database::mysql::*;

#[derive(Debug, Serialize)]
struct Number {
    number: u32,
}

#[get("/follow/totalnumbers/{user_id}")]
pub async fn get_follow_post_total_numbers(
    user_id: web::Path<u32>,
) -> actix_web::Result<HttpResponse> {
    let user_id = *user_id;
    // println!("{}", user_id);
    // 获取线程池，这个线程池为单例模式
    let my_pool = MysqlPool::instance();

    // 事务查询帖子总数量的数据
    let query = format!("SELECT COUNT(*) FROM post where user_id = {};", user_id);
    let numbers: Vec<(u32, )> = match my_pool.exec(&query, &my_pool.read_only_txopts) {
        Ok(result) => result,
        Err(err) => {
            eprintln!("Error executing query: {:?}", err);
            return Err(actix_web::error::ErrorInternalServerError("Internal Server Error"));
        }
    };
    
    let number = numbers[0].0;

    let json_response = serde_json::to_string(&Number { number })
        .map_err(|err| {
            eprintln!("Error serializing response: {:?}", err);
            actix_web::error::ErrorInternalServerError("Error serializing response")
        })?;

    Ok(HttpResponse::Ok().content_type("application/json").body(json_response))
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
    // print!("{}, {} ", user_id, page);

    // 获取线程池，这个线程池为单例模式
    let my_pool = MysqlPool::instance();

    let start = (page - 1) * 10;
    let query = format!("
            SELECT post.id, post.title, post.release_time, post.cover_url, 
                post.content_url, post.user_id, post.user_name FROM post
            JOIN follow ON post.user_id = follow.following_id 
            WHERE follow.follower_id = {}
            ORDER BY post.release_time DESC
            LIMIT {}, 10;
    ", user_id, start);

    // let posts: Vec<String> = transaction.exec(query, ()).unwrap();

    // 将查询的值映射到数结构体中
    let posts: Vec<Post> = match my_pool.query_map(
        query,
        |(id, title, release_time, cover_url, content_url, user_id, user_name)
        : (u32, String, String, String, String, u32, String)| {
            let content = fs::read_to_string(content_url)
                .expect("get_follow_posts_list: Failed fs::read_to_string content_url");
            Post { id, title, release_time, cover_url, content, user_id, user_name }
        },
        &my_pool.read_only_txopts,
    ) 
    {
        Ok(result) => result,
        Err(err) => {
            eprintln!("Error executing query: {:?}", err);
            return Err(actix_web::error::ErrorInternalServerError("Internal Server Error"));
        }
    };


    let post_jsons = serde_json::to_string(&posts)
        .map_err(|err| {
            eprintln!("Error serializing response: {:?}", err);
            actix_web::error::ErrorInternalServerError("Error serializing response")
        })?;

    Ok(HttpResponse::Ok().body(post_jsons))
}
